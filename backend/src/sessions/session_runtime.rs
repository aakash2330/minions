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
    workspace_id: String,
    cwd: PathBuf,
    session_service: SessionService,
    workspace_service: WorkspaceService,
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

        let workspace_id = session.workspace_id.clone();
        let cwd = workspace_root_path(&workspace_service, workspace_id.as_str())
            .await
            .map_err(|error| error.to_string())?;

        Ok(Self {
            session_id,
            workspace_id,
            cwd,
            session_service,
            workspace_service,
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
        let mut codex = CodexAppServer::start(self.cwd.clone(), self.session_id.as_str()).await?;
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
                            let codex_prompt = self.session_tool_context(prompt.as_str()).await?;
                            codex.start_turn(codex_prompt).await?;
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
                    handle_codex_message(
                        &message,
                        &self.session_id,
                        &self.session_service,
                        &mut codex,
                        &mut self.pending_approval_id,
                        &outbox,
                    )
                    .await?;
                }
            }
        }

        codex.shutdown().await;
        Ok(())
    }

    async fn session_tool_context(&self, prompt: &str) -> Result<String, AnyError> {
        let element_lines = self
            .workspace_service
            .load_workspace_element_summaries_by_session(
                self.workspace_id.as_str(),
                self.session_id.as_str(),
            )
            .await?
            .into_iter()
            .map(|element| format!("- label: {}, kind: {}", element.label, element.kind))
            .collect::<Vec<_>>();

        Ok(with_session_tool_context(
            prompt,
            self.session_id.as_str(),
            &element_lines,
        ))
    }
}

async fn handle_codex_message(
    message: &Value,
    session_id: &str,
    session_service: &SessionService,
    codex: &mut CodexAppServer,
    pending_approval_id: &mut Option<Value>,
    outbox: &mpsc::Sender<SessionEvent>,
) -> Result<(), AnyError> {
    match message["method"].as_str() {
        Some(
            method @ ("item/commandExecution/requestApproval" | "item/fileChange/requestApproval"),
        ) => {
            let Some(id) = message.get("id").cloned() else {
                return Ok(());
            };
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
        Some("serverRequest/resolved") => {
            emit_session_event(
                outbox,
                SessionEvent::ApprovalResolved {
                    session_id: session_id.to_owned(),
                },
            )
            .await?;
        }
        Some("item/agentMessage/delta") => {
            if let Some(delta) = message["params"]["delta"].as_str() {
                let assistant_message_id = session_service
                    .append_assistant_delta(session_id, delta)
                    .await?;

                emit_session_event(
                    outbox,
                    SessionEvent::AssistantDelta {
                        session_id: session_id.to_owned(),
                        message_id: assistant_message_id,
                        text: delta.to_owned(),
                    },
                )
                .await?;
            }
        }
        Some("turn/completed") => {
            session_service
                .complete_assistant_message(session_id)
                .await?;
            session_service.complete_session(session_id).await?;

            emit_session_event(
                outbox,
                SessionEvent::TurnCompleted {
                    session_id: session_id.to_owned(),
                },
            )
            .await?;
        }
        Some(_) => {
            if let Some(id) = message.get("id").cloned() {
                codex.respond_method_not_found(id).await?;
            }
        }
        None => {}
    }

    Ok(())
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

fn with_session_tool_context(prompt: &str, session_id: &str, element_lines: &[String]) -> String {
    let elements = if element_lines.is_empty() {
        "No assigned workspace elements are currently available.".to_owned()
    } else {
        element_lines.join("\n")
    };

    format!(
        "Current controllable session_id: {session_id}\n\
Assigned workspace elements for this session:\n{elements}\n\
Supported interaction_type values: move-to-personal-table, move-to-meeting-table, turn-on-computer.\n\
If the user asks this character to do one of those interactions, call the perform_session_interaction MCP tool with the matching interaction_type.\n\n\
User prompt:\n{prompt}"
    )
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
