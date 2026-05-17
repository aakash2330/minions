use crate::{
    domain::{Direction, Point, Workspace, WorkspaceElement},
    infrastructure::db::{executor, shared_pool, DbError, SqlitePool},
    schema::{workspace_elements, workspaces},
};
use diesel::{prelude::*, sqlite::SqliteConnection};

#[derive(Clone)]
pub(crate) struct WorkspaceRepository {
    pool: SqlitePool,
}

impl WorkspaceRepository {
    pub(crate) fn new() -> Result<Self, DbError> {
        Ok(Self {
            pool: shared_pool()?,
        })
    }

    pub(crate) async fn create_workspace(
        &self,
        workspace_id: &str,
        name: &str,
        root_path: Option<&str>,
    ) -> Result<Workspace, DbError> {
        let workspace_id = workspace_id.to_owned();
        let name = name.to_owned();
        let root_path = root_path.map(str::to_owned);

        executor::run(self.pool.clone(), move |connection| {
            create_workspace(
                connection,
                workspace_id.as_str(),
                name.as_str(),
                root_path.as_deref(),
            )
        })
        .await
    }

    pub(crate) async fn load_workspaces(&self) -> Result<Vec<Workspace>, DbError> {
        executor::run(self.pool.clone(), load_workspaces).await
    }

    pub(crate) async fn workspace_by_id(
        &self,
        workspace_id: &str,
    ) -> Result<Option<Workspace>, DbError> {
        let workspace_id = workspace_id.to_owned();

        executor::run(self.pool.clone(), move |connection| {
            workspace_by_id(connection, workspace_id.as_str())
        })
        .await
    }

    pub(crate) async fn load_workspace_elements(
        &self,
        workspace_id: &str,
    ) -> Result<Vec<WorkspaceElement>, DbError> {
        let workspace_id = workspace_id.to_owned();

        executor::run(self.pool.clone(), move |connection| {
            load_workspace_elements(connection, workspace_id.as_str())
        })
        .await
    }
}

#[derive(Queryable)]
struct WorkspaceRow {
    id: String,
    name: String,
    root_path: Option<String>,
}

#[derive(Queryable)]
struct WorkspaceElementRow {
    id: String,
    assigned_session_id: Option<String>,
    kind: String,
    label: String,
    position_x: i32,
    position_y: i32,
    facing: Direction,
    asset_id: Option<String>,
    width: Option<i32>,
    height: Option<i32>,
}

impl From<WorkspaceRow> for Workspace {
    fn from(workspace: WorkspaceRow) -> Self {
        Self {
            id: workspace.id,
            name: workspace.name,
            root_path: workspace.root_path,
        }
    }
}

fn create_workspace(
    connection: &mut SqliteConnection,
    workspace_id: &str,
    name: &str,
    root_path: Option<&str>,
) -> Result<Workspace, DbError> {
    diesel::insert_into(workspaces::table)
        .values((
            workspaces::id.eq(workspace_id),
            workspaces::name.eq(name),
            workspaces::root_path.eq(root_path),
        ))
        .execute(connection)?;

    Ok(Workspace {
        id: workspace_id.to_owned(),
        name: name.to_owned(),
        root_path: root_path.map(str::to_owned),
    })
}

fn load_workspaces(connection: &mut SqliteConnection) -> Result<Vec<Workspace>, DbError> {
    Ok(workspaces::table
        .select((workspaces::id, workspaces::name, workspaces::root_path))
        .order((workspaces::name.asc(), workspaces::id.asc()))
        .load::<WorkspaceRow>(connection)?
        .into_iter()
        .map(Workspace::from)
        .collect())
}

fn workspace_by_id(
    connection: &mut SqliteConnection,
    workspace_id: &str,
) -> Result<Option<Workspace>, DbError> {
    Ok(workspaces::table
        .select((workspaces::id, workspaces::name, workspaces::root_path))
        .filter(workspaces::id.eq(workspace_id))
        .first::<WorkspaceRow>(connection)
        .optional()?
        .map(Workspace::from))
}

fn load_workspace_elements(
    connection: &mut SqliteConnection,
    workspace_id: &str,
) -> Result<Vec<WorkspaceElement>, DbError> {
    Ok(workspace_elements::table
        .select((
            workspace_elements::id,
            workspace_elements::assigned_session_id,
            workspace_elements::kind,
            workspace_elements::label,
            workspace_elements::position_x,
            workspace_elements::position_y,
            workspace_elements::facing,
            workspace_elements::asset_id,
            workspace_elements::width,
            workspace_elements::height,
        ))
        .filter(workspace_elements::workspace_id.eq(workspace_id))
        .order((
            workspace_elements::kind.asc(),
            workspace_elements::label.asc(),
            workspace_elements::id.asc(),
        ))
        .load::<WorkspaceElementRow>(connection)?
        .into_iter()
        .map(|element| WorkspaceElement {
            id: element.id,
            assigned_session_id: element.assigned_session_id,
            kind: element.kind,
            label: element.label,
            position: Point {
                x: element.position_x,
                y: element.position_y,
            },
            facing: element.facing,
            asset_id: element.asset_id,
            width: element.width,
            height: element.height,
        })
        .collect())
}
