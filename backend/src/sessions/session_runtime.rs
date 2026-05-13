use crate::{
    infrastructure::codex::app_server::CodexAppServer,
    services::{SessionService, WorkspaceService},
    sessions::messages::{emit_session_event, ApprovalAnswer, SessionEvent, SessionTaskCommand},
    AnyError,
};
use serde_json::{json, Value};
use std::{io, path::PathBuf};
use tokio::sync::mpsc;

pub(crate) struct SessionRuntime {
    session_id: String,
    cwd: PathBuf,
    session_service: SessionService,
    pending_approval_id: Option<Value>,
}

impl SessionRuntime {
    pub(crate) async fn new(session_id: String) -> Result<Self, String> {
        let session_service = SessionService::new().map_err(|error| error.to_string())?;
        let workspace_service = WorkspaceService::new().map_err(|error| error.to_string())?;

        let Some(session_id) = clean_text(session_id) else {
            return Err("session_id is required".to_owned());
        };
        let session = session_service
            .load_session(session_id.as_str())
            .await
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("session not found: {session_id}"))?;

        let cwd = workspace_root_path(&workspace_service, session.workspace_id.as_str())
            .await
            .map_err(|error| error.to_string())?;

        Ok(Self {
            session_id,
            cwd,
            session_service,
            pending_approval_id: None,
        })
    }

    pub(crate) fn session_id(&self) -> &str {
        self.session_id.as_str()
    }

    pub(crate) async fn run(
        mut self,
        mut inbox: mpsc::Receiver<SessionTaskCommand>,
        outbox: mpsc::Sender<SessionEvent>,
    ) -> Result<(), AnyError> {
        let mut codex = CodexAppServer::start(self.cwd.clone()).await?;
        self.session_service
            .attach_codex_thread(self.session_id.as_str(), codex.thread_id())
            .await?;

        loop {
            tokio::select! {
                command = inbox.recv() => {
                    let Some(command) = command else {
                        break;
                    };

                    match command {
                        SessionTaskCommand::StartTurn { prompt } => {
                            codex.start_turn(prompt.clone()).await?;
                            self.session_service
                                .record_user_message(&self.session_id, prompt.as_str())
                                .await?;
                            let _assistant_message_id = self
                                .session_service
                                .start_assistant_message(&self.session_id)
                                .await?;

                            emit_session_event(
                                &outbox,
                                SessionEvent::TurnStarted {
                                    session_id: self.session_id.clone(),
                                },
                            )
                            .await?;
                        }
                        SessionTaskCommand::RespondToApproval { answer } => {
                            if let Some(id) = self.pending_approval_id.take() {
                                codex
                                    .respond_to_request(
                                        id,
                                        json!({ "decision": approval_decision_for_answer(answer) }),
                                    )
                                    .await?;
                            }
                        }
                    }
                }
                message = codex.read_message() => {
                    let message = message?;

                    if handle_codex_request(
                        &message,
                        &self.session_id,
                        &mut codex,
                        &mut self.pending_approval_id,
                        &outbox,
                    )
                    .await?
                    {
                        continue;
                    }

                    if message["method"] == "item/agentMessage/delta" {
                        if let Some(delta) = message["params"]["delta"].as_str() {
                            let assistant_message_id = self
                                .session_service
                                .append_assistant_delta(&self.session_id, delta)
                                .await?;

                            emit_session_event(
                                &outbox,
                                SessionEvent::AssistantDelta {
                                    session_id: self.session_id.clone(),
                                    message_id: assistant_message_id,
                                    text: delta.to_owned(),
                                },
                            )
                            .await?;
                        }
                    }

                    if message["method"] == "turn/completed" {
                        self.session_service
                            .complete_assistant_message(&self.session_id)
                            .await?;
                        self.session_service
                            .complete_session(&self.session_id)
                            .await?;

                        emit_session_event(
                            &outbox,
                            SessionEvent::TurnCompleted {
                                session_id: self.session_id.clone(),
                            },
                        )
                        .await?;
                    }
                }
            }
        }

        codex.shutdown().await;
        Ok(())
    }
}

async fn handle_codex_request(
    message: &Value,
    session_id: &str,
    codex: &mut CodexAppServer,
    pending_approval_id: &mut Option<Value>,
    outbox: &mpsc::Sender<SessionEvent>,
) -> Result<bool, AnyError> {
    let Some(id) = message.get("id").cloned() else {
        return Ok(false);
    };
    let Some(method) = message["method"].as_str() else {
        return Ok(false);
    };

    match method {
        "item/commandExecution/requestApproval" | "item/fileChange/requestApproval" => {
            *pending_approval_id = Some(id);
            emit_session_event(
                outbox,
                SessionEvent::ApprovalRequest {
                    session_id: session_id.to_owned(),
                    method: method.to_owned(),
                    params: message["params"].clone(),
                    question: "Choose an approval decision.".to_owned(),
                    answers: vec![
                        ApprovalAnswer::Accept,
                        ApprovalAnswer::AcceptForSession,
                        ApprovalAnswer::Decline,
                        ApprovalAnswer::Cancel,
                    ],
                },
            )
            .await?;
        }
        _ => {
            codex.respond_method_not_found(id).await?;
        }
    }

    Ok(true)
}

fn approval_decision_for_answer(answer: ApprovalAnswer) -> &'static str {
    match answer {
        ApprovalAnswer::Accept => "accept",
        ApprovalAnswer::AcceptForSession => "acceptForSession",
        ApprovalAnswer::Cancel => "cancel",
        ApprovalAnswer::Decline => "decline",
    }
}

fn clean_text(value: String) -> Option<String> {
    let value = value.trim().to_owned();
    (!value.is_empty()).then_some(value)
}

async fn workspace_root_path(
    workspace_service: &WorkspaceService,
    workspace_id: &str,
) -> Result<PathBuf, AnyError> {
    let cwd = PathBuf::from(workspace_service.root_path_for_id(workspace_id).await?);

    if !cwd.is_absolute() {
        return Err(io::Error::other("workspace root_path must be absolute").into());
    }

    Ok(cwd)
}
