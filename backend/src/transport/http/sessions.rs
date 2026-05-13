use super::helpers::{MessageResponse, PointWithFacingResponse, PointWithOptionalFacingRequest};
use crate::{
    domain,
    services::{session_service::CreateSessionInput, SessionService},
};
use actix_web::{error, get, post, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetSessionsSessionResponse {
    #[serde(rename = "session_id")]
    session_id: String,
    workspace_id: String,
    name: String,
    kind: String,
    status: String,
    spawn: PointWithFacingResponse,
    current: PointWithFacingResponse,
    messages: Vec<MessageResponse>,
}

impl From<domain::Session> for GetSessionsSessionResponse {
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

#[get("/api/sessions")]
pub(crate) async fn get_sessions() -> Result<HttpResponse> {
    let session_service = SessionService::new().map_err(error::ErrorInternalServerError)?;
    let sessions = session_service
        .load_sessions()
        .await
        .map_err(error::ErrorInternalServerError)?;
    let response = sessions
        .into_iter()
        .map(GetSessionsSessionResponse::from)
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetSessionResponse {
    #[serde(rename = "session_id")]
    session_id: String,
    workspace_id: String,
    name: String,
    kind: String,
    status: String,
    spawn: PointWithFacingResponse,
    current: PointWithFacingResponse,
    messages: Vec<MessageResponse>,
}

impl From<domain::Session> for GetSessionResponse {
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

#[get("/api/sessions/{session_id}")]
pub(crate) async fn get_session(session_id: web::Path<String>) -> Result<HttpResponse> {
    let session_service = SessionService::new().map_err(error::ErrorInternalServerError)?;
    let session = session_service
        .load_session(session_id.as_str())
        .await
        .map_err(error::ErrorInternalServerError)?;

    match session {
        Some(session) => Ok(HttpResponse::Ok().json(GetSessionResponse::from(session))),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetWorkspaceSessionsSessionResponse {
    #[serde(rename = "session_id")]
    session_id: String,
    workspace_id: String,
    name: String,
    kind: String,
    status: String,
    spawn: PointWithFacingResponse,
    current: PointWithFacingResponse,
    messages: Vec<MessageResponse>,
}

impl From<domain::Session> for GetWorkspaceSessionsSessionResponse {
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

#[get("/api/workspaces/{workspace_id}/sessions")]
pub(crate) async fn get_workspace_sessions(
    workspace_id: web::Path<String>,
) -> Result<HttpResponse> {
    let session_service = SessionService::new().map_err(error::ErrorInternalServerError)?;
    let sessions = session_service
        .load_sessions_by_workspace_id(workspace_id.as_str())
        .await
        .map_err(error::ErrorInternalServerError)?;
    let response = sessions
        .into_iter()
        .map(GetWorkspaceSessionsSessionResponse::from)
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateSessionRequest {
    #[serde(alias = "session_id")]
    session_id: Option<String>,
    #[serde(alias = "workspace_id")]
    workspace_id: String,
    name: Option<String>,
    kind: Option<String>,
    spawn: Option<PointWithOptionalFacingRequest>,
    current: Option<PointWithOptionalFacingRequest>,
}

#[derive(Serialize)]
pub(crate) struct CreateSessionResponse {
    #[serde(rename = "session_id")]
    session_id: String,
}

#[post("/api/sessions")]
pub(crate) async fn create_session(
    request: web::Json<CreateSessionRequest>,
) -> Result<HttpResponse> {
    let session_service = SessionService::new().map_err(error::ErrorInternalServerError)?;
    let request = request.into_inner();
    let session_id = session_service
        .create_session(CreateSessionInput {
            session_id: request.session_id,
            workspace_id: request.workspace_id,
            name: request.name,
            kind: request.kind,
            spawn: request.spawn.map(Into::into),
            current: request.current.map(Into::into),
        })
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Created().json(CreateSessionResponse { session_id }))
}
