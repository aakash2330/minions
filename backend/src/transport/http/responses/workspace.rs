use crate::domain;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkspaceResponse {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) root_path: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkspaceElementResponse {
    pub(crate) id: String,
    pub(crate) assigned_session_id: Option<String>,
    pub(crate) kind: String,
    pub(crate) label: String,
    pub(crate) position: PointResponse,
    pub(crate) facing: String,
}

#[derive(Serialize)]
pub(crate) struct PointResponse {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl From<domain::Workspace> for WorkspaceResponse {
    fn from(workspace: domain::Workspace) -> Self {
        Self {
            id: workspace.id,
            name: workspace.name,
            root_path: workspace.root_path,
        }
    }
}

impl From<domain::WorkspaceElement> for WorkspaceElementResponse {
    fn from(element: domain::WorkspaceElement) -> Self {
        Self {
            id: element.id,
            assigned_session_id: element.assigned_session_id,
            kind: element.kind,
            label: element.label,
            position: PointResponse::from(element.position),
            facing: element.facing.as_str().to_owned(),
        }
    }
}

impl From<domain::Point> for PointResponse {
    fn from(point: domain::Point) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }
}
