use crate::AnyError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{io, path::PathBuf};
use tokio::sync::mpsc;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) enum ApprovalAnswer {
    #[serde(rename = "accept")]
    Accept,
    #[serde(rename = "acceptForSession")]
    AcceptForSession,
    #[serde(rename = "cancel")]
    Cancel,
    #[serde(rename = "decline")]
    Decline,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub(crate) enum ClientMessage {
    #[serde(rename = "session.start")]
    SessionStart {
        session_id: Option<String>,
        cwd: Option<PathBuf>,
    },
    #[serde(rename = "turn.start")]
    TurnStart {
        session_id: Option<String>,
        prompt: String,
    },
    #[serde(rename = "approval.respond")]
    ApprovalRespond {
        session_id: String,
        answer: ApprovalAnswer,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub(crate) enum ServerEvent {
    #[serde(rename = "session.ready")]
    SessionReady { session_id: String },
    #[serde(rename = "turn.started")]
    TurnStarted { session_id: String },
    #[serde(rename = "assistant.delta")]
    AssistantDelta { session_id: String, text: String },
    #[serde(rename = "turn.completed")]
    TurnCompleted { session_id: String },
    #[serde(rename = "approval.request")]
    ApprovalRequest {
        session_id: String,
        method: String,
        params: Value,
        question: String,
        answers: Vec<ApprovalAnswer>,
    },
    #[serde(rename = "error")]
    Error {
        #[serde(skip_serializing_if = "Option::is_none")]
        session_id: Option<String>,
        message: String,
    },
}

#[derive(Debug)]
pub(crate) enum SessionCommand {
    StartTurn { prompt: String },
    RespondToApproval { answer: ApprovalAnswer },
}

pub(crate) async fn send_error(
    outbox: &mpsc::Sender<ServerEvent>,
    session_id: Option<&str>,
    message: &str,
) -> Result<(), AnyError> {
    send_event(
        outbox,
        ServerEvent::Error {
            session_id: session_id.map(str::to_owned),
            message: message.to_owned(),
        },
    )
    .await
}

pub(crate) async fn send_event(
    outbox: &mpsc::Sender<ServerEvent>,
    event: ServerEvent,
) -> Result<(), AnyError> {
    outbox
        .send(event)
        .await
        .map_err(|_| io::Error::other("websocket writer closed"))?;
    Ok(())
}
