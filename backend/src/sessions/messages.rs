use crate::AnyError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io;
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

#[derive(Debug)]
pub(crate) enum SessionTaskCommand {
    StartTurn { prompt: String },
    RespondToApproval { answer: ApprovalAnswer },
}

#[derive(Debug)]
pub(crate) enum SessionEvent {
    TurnStarted {
        session_id: String,
    },
    AssistantDelta {
        session_id: String,
        message_id: String,
        text: String,
    },
    TurnCompleted {
        session_id: String,
    },
    ApprovalRequest {
        session_id: String,
        method: String,
        params: Value,
        question: String,
        answers: Vec<ApprovalAnswer>,
    },
    ApprovalResolved {
        session_id: String,
    },
    Error {
        session_id: Option<String>,
        message: String,
    },
}

pub(crate) async fn emit_session_event(
    outbox: &mpsc::Sender<SessionEvent>,
    event: SessionEvent,
) -> Result<(), AnyError> {
    outbox
        .send(event)
        .await
        .map_err(|_| io::Error::other("session manager closed"))?;
    Ok(())
}
