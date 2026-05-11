use crate::{
    services::{SessionService, WorkspaceService},
    transport::http::responses::AppDataResponse,
};
use actix_web::{error, get, HttpResponse, Result};

#[get("/api/data")]
pub(crate) async fn get_app_data() -> Result<HttpResponse> {
    let workspace_service = WorkspaceService::new().map_err(error::ErrorInternalServerError)?;
    let session_service = SessionService::new().map_err(error::ErrorInternalServerError)?;
    let workspaces = workspace_service
        .load_workspaces()
        .await
        .map_err(error::ErrorInternalServerError)?;
    let sessions = session_service
        .load_sessions()
        .await
        .map_err(error::ErrorInternalServerError)?;
    let response = AppDataResponse::new(workspaces, sessions);

    Ok(HttpResponse::Ok().json(response))
}
