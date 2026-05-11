use crate::domain;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionResponse {
    #[serde(rename = "session_id")]
    pub(crate) session_id: String,
    pub(crate) workspace_id: String,
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) status: String,
    pub(crate) spawn: PointWithFacingResponse,
    pub(crate) current: PointWithFacingResponse,
    pub(crate) messages: Vec<MessageResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MessageResponse {
    pub(crate) id: String,
    #[serde(rename = "session_id")]
    pub(crate) session_id: String,
    pub(crate) role: String,
    pub(crate) text: String,
    pub(crate) status: String,
}

#[derive(Serialize)]
pub(crate) struct PointWithFacingResponse {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) facing: String,
}

impl From<domain::Session> for SessionResponse {
    fn from(session: domain::Session) -> Self {
        Self {
            session_id: session.session_id,
            workspace_id: session.workspace_id,
            name: session.name,
            kind: session.kind.as_str().to_owned(),
            status: session.status.as_str().to_owned(),
            spawn: PointWithFacingResponse::from(session.spawn),
            current: PointWithFacingResponse::from(session.current),
            messages: session
                .messages
                .into_iter()
                .map(MessageResponse::from)
                .collect(),
        }
    }
}

impl From<domain::Message> for MessageResponse {
    fn from(message: domain::Message) -> Self {
        Self {
            id: message.id,
            session_id: message.session_id,
            role: message.role.as_str().to_owned(),
            text: message.text,
            status: message.status.as_str().to_owned(),
        }
    }
}

impl From<domain::PointWithFacing> for PointWithFacingResponse {
    fn from(point: domain::PointWithFacing) -> Self {
        Self {
            x: point.x,
            y: point.y,
            facing: point.facing.as_str().to_owned(),
        }
    }
}
