use crate::{domain, services::session_service::CreateSessionPointInput};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MessageResponse {
    id: String,
    #[serde(rename = "session_id")]
    session_id: String,
    role: String,
    text: String,
    status: String,
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

#[derive(Serialize)]
pub(crate) struct PointResponse {
    x: i32,
    y: i32,
}

impl From<domain::Point> for PointResponse {
    fn from(point: domain::Point) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct PointWithFacingResponse {
    x: i32,
    y: i32,
    facing: String,
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

#[derive(Deserialize)]
pub(crate) struct PointWithOptionalFacingRequest {
    x: i32,
    y: i32,
    facing: Option<String>,
}

impl From<PointWithOptionalFacingRequest> for CreateSessionPointInput {
    fn from(point: PointWithOptionalFacingRequest) -> Self {
        Self {
            x: point.x,
            y: point.y,
            facing: point.facing,
        }
    }
}
