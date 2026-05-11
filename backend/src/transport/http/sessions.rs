use crate::{
    services::{
        session_service::{CreateSessionInput, CreateSessionPointInput},
        SessionService,
    },
    transport::http::responses::SessionResponse,
};
use actix_web::{error, get, post, web, HttpResponse, Result};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateSessionRequest {
    #[serde(alias = "session_id")]
    session_id: Option<String>,
    #[serde(alias = "workspace_id")]
    workspace_id: String,
    name: Option<String>,
    kind: Option<String>,
    spawn: Option<CreateSessionPointRequest>,
    current: Option<CreateSessionPointRequest>,
}

#[derive(Deserialize)]
pub(crate) struct CreateSessionPointRequest {
    x: i32,
    y: i32,
    facing: Option<String>,
}

#[get("/api/sessions")]
pub(crate) async fn get_sessions() -> Result<HttpResponse> {
    let session_service = SessionService::new().map_err(error::ErrorInternalServerError)?;
    let sessions = session_service
        .load_sessions()
        .await
        .map_err(error::ErrorInternalServerError)?;
    let response = sessions
        .into_iter()
        .map(SessionResponse::from)
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(response))
}

#[post("/api/sessions")]
pub(crate) async fn create_session(
    request: web::Json<CreateSessionRequest>,
) -> Result<HttpResponse> {
    let session_service = SessionService::new().map_err(error::ErrorInternalServerError)?;
    let request = request.into_inner();
    let session = session_service
        .create_session(CreateSessionInput {
            session_id: request.session_id,
            workspace_id: request.workspace_id,
            name: request.name,
            kind: request.kind,
            spawn: request.spawn.map(CreateSessionPointInput::from),
            current: request.current.map(CreateSessionPointInput::from),
        })
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Created().json(SessionResponse::from(session)))
}

impl From<CreateSessionPointRequest> for CreateSessionPointInput {
    fn from(point: CreateSessionPointRequest) -> Self {
        Self {
            x: point.x,
            y: point.y,
            facing: point.facing,
        }
    }
}
