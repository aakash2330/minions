use crate::{
    domain::WorkspaceChatMessage,
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
    #[serde(rename = "workspace_chat.turn.start")]
    WorkspaceChatTurnStart {
        workspace_id: String,
        prompt: String,
    },
    #[serde(rename = "approval.respond")]
    ApprovalRespond {
        session_id: String,
        answer: ApprovalAnswer,
    },
}

#[derive(Clone, Debug, Serialize)]
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
        workspace_id: String,
        method: String,
        params: Value,
        question: String,
        answers: Vec<ApprovalAnswer>,
    },
    #[serde(rename = "approval.resolved")]
    ApprovalResolved {
        session_id: String,
        workspace_id: String,
    },
    #[serde(rename = "workspace_chat.message.created")]
    WorkspaceChatMessageCreated {
        workspace_id: String,
        message_id: String,
        session_id: Option<String>,
        role: String,
        text: String,
        status: String,
    },
    #[serde(rename = "workspace_chat.message.delta")]
    WorkspaceChatMessageDelta {
        workspace_id: String,
        message_id: String,
        session_id: Option<String>,
        text: String,
    },
    #[serde(rename = "workspace_chat.message.completed")]
    WorkspaceChatMessageCompleted {
        workspace_id: String,
        message_id: String,
        session_id: Option<String>,
        status: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<String>,
    },
    #[serde(rename = "session.interaction")]
    SessionInteraction {
        session_id: String,
        interaction_type: String,
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
                workspace_id,
                method,
                params,
                question,
                answers,
            } => Self::ApprovalRequest {
                session_id,
                workspace_id,
                method,
                params,
                question,
                answers,
            },
            SessionEvent::ApprovalResolved {
                session_id,
                workspace_id,
            } => Self::ApprovalResolved {
                session_id,
                workspace_id,
            },
            SessionEvent::WorkspaceChatMessageDelta {
                workspace_id,
                message_id,
                session_id,
                text,
            } => Self::WorkspaceChatMessageDelta {
                workspace_id,
                message_id,
                session_id,
                text,
            },
            SessionEvent::WorkspaceChatMessageCompleted {
                workspace_id,
                message_id,
                session_id,
                status,
                text,
            } => Self::WorkspaceChatMessageCompleted {
                workspace_id,
                message_id,
                session_id,
                status,
                text,
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

pub(crate) fn workspace_chat_message_created_event(message: WorkspaceChatMessage) -> ServerEvent {
    ServerEvent::WorkspaceChatMessageCreated {
        workspace_id: message.workspace_id,
        message_id: message.id,
        session_id: message.session_id,
        role: message.role.as_str().to_owned(),
        text: message.text,
        status: message.status.as_str().to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserializes_workspace_chat_turn_start() {
        let message = serde_json::from_value::<ClientMessage>(json!({
            "type": "workspace_chat.turn.start",
            "workspace_id": "default",
            "prompt": "hello"
        }))
        .expect("workspace chat turn should deserialize");

        match message {
            ClientMessage::WorkspaceChatTurnStart {
                workspace_id,
                prompt,
            } => {
                assert_eq!(workspace_id, "default");
                assert_eq!(prompt, "hello");
            }
            _ => panic!("unexpected message type"),
        }
    }

    #[test]
    fn serializes_workspace_chat_created_event() {
        let event = ServerEvent::WorkspaceChatMessageCreated {
            workspace_id: "default".to_owned(),
            message_id: "message-1".to_owned(),
            session_id: Some("kevin".to_owned()),
            role: "assistant".to_owned(),
            text: "hi".to_owned(),
            status: "streaming".to_owned(),
        };
        let serialized = serde_json::to_value(event).expect("event should serialize");

        assert_eq!(
            serialized,
            json!({
                "type": "workspace_chat.message.created",
                "workspace_id": "default",
                "message_id": "message-1",
                "session_id": "kevin",
                "role": "assistant",
                "text": "hi",
                "status": "streaming"
            })
        );
    }
}
