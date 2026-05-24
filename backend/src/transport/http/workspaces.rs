use super::helpers::PointResponse;
use crate::{
    domain,
    services::{workspace_service::CreateWorkspaceInput, WorkspaceService},
};
use actix_web::{error, get, post, put, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetWorkspacesWorkspaceResponse {
    id: String,
    name: String,
    root_path: Option<String>,
}

impl From<domain::Workspace> for GetWorkspacesWorkspaceResponse {
    fn from(workspace: domain::Workspace) -> Self {
        Self {
            id: workspace.id,
            name: workspace.name,
            root_path: workspace.root_path,
        }
    }
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
        .map(GetWorkspacesWorkspaceResponse::from)
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetWorkspaceResponse {
    id: String,
    name: String,
    root_path: Option<String>,
}

impl From<domain::Workspace> for GetWorkspaceResponse {
    fn from(workspace: domain::Workspace) -> Self {
        Self {
            id: workspace.id,
            name: workspace.name,
            root_path: workspace.root_path,
        }
    }
}

#[get("/api/workspaces/{workspace_id}")]
pub(crate) async fn get_workspace(workspace_id: web::Path<String>) -> Result<HttpResponse> {
    let workspace_service = WorkspaceService::new().map_err(error::ErrorInternalServerError)?;
    let workspace = workspace_service
        .load_workspace(workspace_id.as_str())
        .await
        .map_err(error::ErrorInternalServerError)?;

    match workspace {
        Some(workspace) => Ok(HttpResponse::Ok().json(GetWorkspaceResponse::from(workspace))),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateWorkspaceRequest {
    name: String,
    root_path: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateWorkspaceResponse {
    id: String,
    name: String,
    root_path: Option<String>,
}

impl From<domain::Workspace> for CreateWorkspaceResponse {
    fn from(workspace: domain::Workspace) -> Self {
        Self {
            id: workspace.id,
            name: workspace.name,
            root_path: workspace.root_path,
        }
    }
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

    Ok(HttpResponse::Created().json(CreateWorkspaceResponse::from(workspace)))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetWorkspaceElementsWorkspaceElementResponse {
    id: String,
    assigned_session_id: Option<String>,
    kind: domain::WorkspaceElementKind,
    label: String,
    position: PointResponse,
    facing: domain::Direction,
}

impl From<domain::WorkspaceElement> for GetWorkspaceElementsWorkspaceElementResponse {
    fn from(element: domain::WorkspaceElement) -> Self {
        Self {
            id: element.id,
            assigned_session_id: element.assigned_session_id,
            kind: element.kind,
            label: element.label,
            position: PointResponse::from(element.position),
            facing: element.facing,
        }
    }
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
        .map(GetWorkspaceElementsWorkspaceElementResponse::from)
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(response))
}

#[get("/api/workspaces/{workspace_id}/map-config")]
pub(crate) async fn get_workspace_map_config(
    workspace_id: web::Path<String>,
) -> Result<HttpResponse> {
    let workspace_service = WorkspaceService::new().map_err(error::ErrorInternalServerError)?;

    let Some(config_json) = workspace_service
        .load_workspace_map_config_json(workspace_id.as_str())
        .await
        .map_err(error::ErrorInternalServerError)?
    else {
        return Ok(HttpResponse::NotFound().finish());
    };
    let config = serde_json::from_str::<Value>(config_json.as_str())
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(config))
}

#[put("/api/workspaces/{workspace_id}/map-config")]
pub(crate) async fn update_workspace_map_config(
    workspace_id: web::Path<String>,
    request: web::Json<Value>,
) -> Result<HttpResponse> {
    let workspace_service = WorkspaceService::new().map_err(error::ErrorInternalServerError)?;
    let config = request.into_inner();
    let config_json = serde_json::to_string(&config).map_err(error::ErrorInternalServerError)?;

    workspace_service
        .save_workspace_map_config_json(workspace_id.as_str(), config_json.as_str())
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(config))
}
