use crate::domain;
use serde::Serialize;

use super::{SessionResponse, WorkspaceResponse};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppDataResponse {
    pub(crate) workspaces: Vec<WorkspaceResponse>,
    pub(crate) sessions: Vec<SessionResponse>,
}

impl AppDataResponse {
    pub(crate) fn new(workspaces: Vec<domain::Workspace>, sessions: Vec<domain::Session>) -> Self {
        Self {
            workspaces: workspaces
                .into_iter()
                .map(WorkspaceResponse::from)
                .collect(),
            sessions: sessions.into_iter().map(SessionResponse::from).collect(),
        }
    }
}
