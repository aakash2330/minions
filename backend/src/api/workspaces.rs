use crate::{schema::workspaces, AnyError};
use diesel::{prelude::*, sqlite::SqliteConnection};
use serde::Serialize;

#[derive(Queryable)]
struct WorkspaceRow {
    id: String,
    user_id: i32,
    name: String,
    root_path: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Workspace {
    id: String,
    user_id: i32,
    name: String,
    root_path: Option<String>,
}

pub(crate) fn load_workspaces(
    connection: &mut SqliteConnection,
) -> Result<Vec<Workspace>, AnyError> {
    let workspace_rows = workspaces::table
        .select((
            workspaces::id,
            workspaces::user_id,
            workspaces::name,
            workspaces::root_path,
        ))
        .order((workspaces::name.asc(), workspaces::id.asc()))
        .load::<WorkspaceRow>(connection)?;

    Ok(workspace_rows
        .into_iter()
        .map(|workspace| Workspace {
            id: workspace.id,
            user_id: workspace.user_id,
            name: workspace.name,
            root_path: workspace.root_path,
        })
        .collect())
}
