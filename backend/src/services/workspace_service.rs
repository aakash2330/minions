use crate::{
    domain::{Workspace, WorkspaceElement, WorkspaceElementSummary},
    infrastructure::db::{DbError, WorkspaceRepository},
};
use std::{
    io,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_WORKSPACE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub(crate) struct WorkspaceService {
    repository: WorkspaceRepository,
}

impl WorkspaceService {
    pub(crate) fn new() -> Result<Self, DbError> {
        Ok(Self {
            repository: WorkspaceRepository::new()?,
        })
    }

    pub(crate) async fn create_workspace(
        &self,
        input: CreateWorkspaceInput,
    ) -> Result<Workspace, DbError> {
        let name = clean_text(input.name).unwrap_or_else(|| "Untitled Workspace".to_owned());
        let root_path = input.root_path.and_then(clean_text);
        let workspace_id = new_workspace_id();

        self.repository
            .create_workspace(workspace_id.as_str(), name.as_str(), root_path.as_deref())
            .await
    }

    pub(crate) async fn load_workspaces(&self) -> Result<Vec<Workspace>, DbError> {
        self.repository.load_workspaces().await
    }

    pub(crate) async fn load_workspace(
        &self,
        workspace_id: &str,
    ) -> Result<Option<Workspace>, DbError> {
        self.repository.workspace_by_id(workspace_id).await
    }

    pub(crate) async fn load_workspace_elements(
        &self,
        workspace_id: &str,
    ) -> Result<Vec<WorkspaceElement>, DbError> {
        self.repository.load_workspace_elements(workspace_id).await
    }

    pub(crate) async fn load_workspace_element_summaries_by_session(
        &self,
        workspace_id: &str,
        session_id: &str,
    ) -> Result<Vec<WorkspaceElementSummary>, DbError> {
        self.repository
            .load_workspace_element_summaries_by_session(workspace_id, session_id)
            .await
    }

    pub(crate) async fn load_workspace_map_config_json(
        &self,
        workspace_id: &str,
    ) -> Result<Option<String>, DbError> {
        self.repository
            .load_workspace_map_config_json(workspace_id)
            .await
    }

    pub(crate) async fn save_workspace_map_config_json(
        &self,
        workspace_id: &str,
        config_json: &str,
    ) -> Result<(), DbError> {
        self.repository
            .save_workspace_map_config_json(workspace_id, config_json)
            .await
    }

    pub(crate) async fn root_path_for_id(&self, workspace_id: &str) -> Result<String, DbError> {
        let workspace = self
            .repository
            .workspace_by_id(workspace_id)
            .await?
            .ok_or_else(|| io::Error::other(format!("workspace not found: {workspace_id}")))?;

        let root_path = workspace
            .root_path
            .as_deref()
            .map(str::trim)
            .filter(|root_path| !root_path.is_empty())
            .ok_or_else(|| {
                io::Error::other(format!(
                    "workspace {workspace_id} does not have a root_path"
                ))
            })?;

        Ok(root_path.to_owned())
    }
}

pub(crate) struct CreateWorkspaceInput {
    pub(crate) name: String,
    pub(crate) root_path: Option<String>,
}

fn clean_text(value: String) -> Option<String> {
    let value = value.trim().to_owned();
    (!value.is_empty()).then_some(value)
}

fn new_workspace_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let sequence = NEXT_WORKSPACE_ID.fetch_add(1, Ordering::Relaxed);

    format!("workspace-{timestamp:020}-{sequence:020}")
}
