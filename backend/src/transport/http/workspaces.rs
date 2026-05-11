use crate::{
    services::{workspace_service::CreateWorkspaceInput, WorkspaceService},
    transport::http::responses::{WorkspaceElementResponse, WorkspaceResponse},
};
use actix_web::{error, get, post, web, HttpResponse, Result};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateWorkspaceRequest {
    name: String,
    root_path: Option<String>,
}

#[get("/api/workspaces")]
pub(crate) async fn get_workspaces() -> Result<HttpResponse> {
    let workspace_service = WorkspaceService::new().map_err(error::ErrorInternalServerError)?;
    let workspaces = workspace_service
        .load_workspaces()
        .await
        .map_err(error::ErrorInternalServerError)?;
    let response = workspaces
        .into_iter()
        .map(WorkspaceResponse::from)
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(response))
}

#[post("/api/workspaces")]
pub(crate) async fn create_workspace(
    request: web::Json<CreateWorkspaceRequest>,
) -> Result<HttpResponse> {
    let workspace_service = WorkspaceService::new().map_err(error::ErrorInternalServerError)?;
    let request = request.into_inner();
    let workspace = workspace_service
        .create_workspace(CreateWorkspaceInput {
            name: request.name,
            root_path: request.root_path,
        })
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Created().json(WorkspaceResponse::from(workspace)))
}

#[get("/api/workspaces/{workspace_id}/elements")]
pub(crate) async fn get_workspace_elements(
    workspace_id: web::Path<String>,
) -> Result<HttpResponse> {
    let workspace_service = WorkspaceService::new().map_err(error::ErrorInternalServerError)?;
    let elements = workspace_service
        .load_workspace_elements(workspace_id.as_str())
        .await
        .map_err(error::ErrorInternalServerError)?;
    let response = elements
        .into_iter()
        .map(WorkspaceElementResponse::from)
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(response))
}
