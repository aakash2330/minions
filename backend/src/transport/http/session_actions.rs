use crate::{
    services::SessionService,
    transport::ws::{events::broadcast_server_event, protocol::ServerEvent},
};
use actix_web::{error, post, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Serialize)]
enum SessionInteractionType {
    #[serde(rename = "move-to-personal-table")]
    MoveToPersonalTable,
    #[serde(rename = "move-to-meeting-table")]
    MoveToMeetingTable,
    #[serde(rename = "turn-on-computer")]
    TurnOnComputer,
}

impl SessionInteractionType {
    fn as_str(self) -> &'static str {
        match self {
            Self::MoveToPersonalTable => "move-to-personal-table",
            Self::MoveToMeetingTable => "move-to-meeting-table",
            Self::TurnOnComputer => "turn-on-computer",
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PerformSessionInteractionRequest {
    #[serde(alias = "interaction_type")]
    interaction_type: SessionInteractionType,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PerformSessionInteractionResponse {
    #[serde(rename = "session_id")]
    session_id: String,
    #[serde(rename = "interaction_type")]
    interaction_type: SessionInteractionType,
}

#[post("/api/sessions/{session_id}/interaction")]
pub(crate) async fn perform_session_interaction(
    session_id: web::Path<String>,
    request: web::Json<PerformSessionInteractionRequest>,
) -> Result<HttpResponse> {
    let session_id = session_id.into_inner();
    let request = request.into_inner();
    let session_service = SessionService::new().map_err(error::ErrorInternalServerError)?;

    if session_service
        .load_session(session_id.as_str())
        .await
        .map_err(error::ErrorInternalServerError)?
        .is_none()
    {
        return Ok(HttpResponse::NotFound().finish());
    }

    broadcast_server_event(ServerEvent::SessionInteraction {
        session_id: session_id.clone(),
        interaction_type: request.interaction_type.as_str().to_owned(),
    });

    Ok(HttpResponse::Ok().json(PerformSessionInteractionResponse {
        session_id,
        interaction_type: request.interaction_type,
    }))
}
