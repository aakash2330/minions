use crate::{
    domain::SessionKind,
    infrastructure::codex::app_server::CodexAppServer,
    services::{SessionService, WorkspaceChatService, WorkspaceService},
    sessions::messages::{emit_session_event, ApprovalAnswer, SessionEvent, SessionTaskCommand},
    sessions::workspace_chat::{build_workspace_chat_turn_prompt, WorkspaceChatTurnOrigin},
    AnyError,
};
use serde_json::{json, Value};
use std::{
    io,
    path::PathBuf,
    time::{Duration, Instant},
};
use tokio::sync::mpsc;

const CODEX_TURN_STALL_TIMEOUT: Duration = Duration::from_secs(45);
const CODEX_TURN_STALL_CHECK_INTERVAL: Duration = Duration::from_secs(5);

pub(crate) struct SessionRuntime {
    session_id: String,
    session_name: String,
    session_kind: SessionKind,
    workspace_id: String,
    cwd: PathBuf,
    session_service: SessionService,
    workspace_chat_service: WorkspaceChatService,
    workspace_service: WorkspaceService,
    pending_approval_id: Option<Value>,
    current_turn_origin: Option<WorkspaceChatTurnOrigin>,
    last_turn_activity_at: Option<Instant>,
}

impl SessionRuntime {
    pub(crate) async fn new(session_id: String) -> Result<Self, String> {
        let session_service = SessionService::new().map_err(|error| error.to_string())?;
        let workspace_chat_service =
            WorkspaceChatService::new().map_err(|error| error.to_string())?;
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
        let session_name = session.name.clone();
        let session_kind = session.kind;
        let cwd = workspace_root_path(&workspace_service, workspace_id.as_str())
            .await
            .map_err(|error| error.to_string())?;

        Ok(Self {
            session_id,
            session_name,
            session_kind,
            workspace_id,
            cwd,
            session_service,
            workspace_chat_service,
            workspace_service,
            pending_approval_id: None,
            current_turn_origin: None,
            last_turn_activity_at: None,
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
        let mut codex = CodexAppServer::start(
            self.cwd.clone(),
            self.session_id.as_str(),
            self.session_name.as_str(),
            self.session_kind,
        )
        .await?;
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
                        SessionTaskCommand::StartTurn { prompt, origin } => {
                            self.current_turn_origin = origin.clone();

                            if let Err(error) = self
                                .start_turn(&mut codex, prompt, origin, &outbox)
                                .await
                            {
                                self.fail_active_turn(error.to_string(), &outbox).await?;
                                return Err(error);
                            }
                            self.last_turn_activity_at = Some(Instant::now());
                        }
                        SessionTaskCommand::RespondToApproval { answer } => {
                            if let Some(id) = self.pending_approval_id.take() {
                                if let Err(error) = codex
                                    .respond_to_request(
                                        id,
                                        json!({ "decision": approval_decision_for_answer(answer) }),
                                    )
                                    .await
                                {
                                    self.fail_active_turn(error.to_string(), &outbox).await?;
                                    return Err(error);
                                }
                            }
                        }
                    }
                }
                message = codex.read_message() => {
                    let message = match message {
                        Ok(message) => message,
                        Err(error) => {
                            self.fail_active_turn(error.to_string(), &outbox).await?;
                            return Err(error);
                        }
                    };
                    self.last_turn_activity_at = Some(Instant::now());
                    if let Err(error) = handle_codex_message(
                        &message,
                        &self.session_id,
                        &self.workspace_id,
                        &self.session_service,
                        &self.workspace_chat_service,
                        &mut codex,
                        &mut self.pending_approval_id,
                        &mut self.current_turn_origin,
                        &outbox,
                    )
                    .await
                    {
                        self.fail_active_turn(error.to_string(), &outbox).await?;
                        return Err(error);
                    }
                    if message["method"].as_str() == Some("turn/completed") {
                        self.last_turn_activity_at = None;
                    }
                }
                _ = tokio::time::sleep(CODEX_TURN_STALL_CHECK_INTERVAL) => {
                    if self.has_stalled_turn() {
                        let message = format!(
                            "Codex app-server produced no turn events for {} seconds.",
                            CODEX_TURN_STALL_TIMEOUT.as_secs()
                        );
                        self.fail_active_turn(message.clone(), &outbox).await?;
                        return Err(io::Error::other(message).into());
                    }
                }
            }
        }

        codex.shutdown().await;
        Ok(())
    }

    fn has_stalled_turn(&self) -> bool {
        self.last_turn_activity_at.is_some_and(|last_activity| {
            last_activity.elapsed() >= CODEX_TURN_STALL_TIMEOUT
                && self.current_turn_origin.is_some()
        })
    }

    async fn start_turn(
        &mut self,
        codex: &mut CodexAppServer,
        prompt: String,
        origin: Option<WorkspaceChatTurnOrigin>,
        outbox: &mpsc::Sender<SessionEvent>,
    ) -> Result<(), AnyError> {
        let codex_prompt = self
            .session_tool_context(prompt.as_str(), origin.as_ref())
            .await?;
        codex.start_turn(codex_prompt).await?;
        self.session_service
            .record_user_message(&self.session_id, prompt.as_str())
            .await?;
        let assistant_message_id = self
            .session_service
            .start_assistant_message(&self.session_id)
            .await?;

        if let Some(origin) = origin.as_ref() {
            self.workspace_chat_service
                .attach_session_message(
                    origin.response_message_id.as_str(),
                    assistant_message_id.as_str(),
                )
                .await?;
        }

        emit_session_event(
            outbox,
            SessionEvent::TurnStarted {
                session_id: self.session_id.clone(),
            },
        )
        .await
    }

    async fn fail_active_turn(
        &mut self,
        error: String,
        outbox: &mpsc::Sender<SessionEvent>,
    ) -> Result<(), AnyError> {
        let error_text = format!("Turn failed: {error}");

        self.session_service
            .fail_active_turn(self.session_id.as_str(), error_text.as_str())
            .await?;

        if let Some(origin) = self.current_turn_origin.take() {
            self.workspace_chat_service
                .fail_assistant_message(origin.response_message_id.as_str(), error_text.as_str())
                .await?;
            emit_session_event(
                outbox,
                SessionEvent::WorkspaceChatMessageCompleted {
                    workspace_id: origin.workspace_id,
                    message_id: origin.response_message_id,
                    session_id: Some(self.session_id.clone()),
                    status: "error".to_owned(),
                    text: Some(error_text),
                },
            )
            .await?;
        }

        self.last_turn_activity_at = None;

        Ok(())
    }

    async fn session_tool_context(
        &self,
        prompt: &str,
        origin: Option<&WorkspaceChatTurnOrigin>,
    ) -> Result<String, AnyError> {
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
        let prompt = if let Some(origin) = origin {
            let history = self
                .workspace_chat_service
                .load_context_messages(origin.workspace_id.as_str())
                .await?;

            format!(
                "{}\n\nGlobal chat user_message_id: {}",
                build_workspace_chat_turn_prompt(prompt, origin.role, &history),
                origin.user_message_id
            )
        } else {
            prompt.to_owned()
        };

        Ok(with_session_tool_context(
            prompt.as_str(),
            self.session_id.as_str(),
            &element_lines,
        ))
    }
}

async fn handle_codex_message(
    message: &Value,
    session_id: &str,
    workspace_id: &str,
    session_service: &SessionService,
    workspace_chat_service: &WorkspaceChatService,
    codex: &mut CodexAppServer,
    pending_approval_id: &mut Option<Value>,
    current_turn_origin: &mut Option<WorkspaceChatTurnOrigin>,
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
                    workspace_id: workspace_id.to_owned(),
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
                    workspace_id: workspace_id.to_owned(),
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

                if let Some(origin) = current_turn_origin.as_ref() {
                    workspace_chat_service
                        .append_assistant_delta(origin.response_message_id.as_str(), delta)
                        .await?;
                    emit_session_event(
                        outbox,
                        SessionEvent::WorkspaceChatMessageDelta {
                            workspace_id: origin.workspace_id.clone(),
                            message_id: origin.response_message_id.clone(),
                            session_id: Some(session_id.to_owned()),
                            text: delta.to_owned(),
                        },
                    )
                    .await?;
                }
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

            if let Some(origin) = current_turn_origin.take() {
                workspace_chat_service
                    .complete_assistant_message(origin.response_message_id.as_str())
                    .await?;
                emit_session_event(
                    outbox,
                    SessionEvent::WorkspaceChatMessageCompleted {
                        workspace_id: origin.workspace_id,
                        message_id: origin.response_message_id,
                        session_id: Some(session_id.to_owned()),
                        status: "complete".to_owned(),
                        text: None,
                    },
                )
                .await?;
            }
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
