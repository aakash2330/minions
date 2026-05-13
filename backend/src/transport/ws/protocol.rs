use crate::{
    sessions::messages::{ApprovalAnswer, SessionEvent},
    AnyError,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io;
use tokio::sync::mpsc;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub(crate) enum ClientMessage {
    #[serde(rename = "turn.start")]
    TurnStart { session_id: String, prompt: String },
    #[serde(rename = "approval.respond")]
    ApprovalRespond {
        session_id: String,
        answer: ApprovalAnswer,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub(crate) enum ServerEvent {
    #[serde(rename = "turn.started")]
    TurnStarted { session_id: String },
    #[serde(rename = "assistant.delta")]
    AssistantDelta {
        session_id: String,
        message_id: String,
        text: String,
    },
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

impl From<SessionEvent> for ServerEvent {
    fn from(event: SessionEvent) -> Self {
        match event {
            SessionEvent::TurnStarted { session_id } => Self::TurnStarted { session_id },
            SessionEvent::AssistantDelta {
                session_id,
                message_id,
                text,
            } => Self::AssistantDelta {
                session_id,
                message_id,
                text,
            },
            SessionEvent::TurnCompleted { session_id } => Self::TurnCompleted { session_id },
            SessionEvent::ApprovalRequest {
                session_id,
                method,
                params,
                question,
                answers,
            } => Self::ApprovalRequest {
                session_id,
                method,
                params,
                question,
                answers,
            },
            SessionEvent::Error {
                session_id,
                message,
            } => Self::Error {
                session_id,
                message,
            },
        }
    }
}
