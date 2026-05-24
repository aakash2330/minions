use crate::{
    domain::{
        Direction, Point, Workspace, WorkspaceElement, WorkspaceElementKind,
        WorkspaceElementSummary,
    },
    infrastructure::db::{executor, shared_pool, DbError, SqlitePool},
    schema::{workspace_elements, workspace_map_configs, workspaces},
};
use diesel::{prelude::*, sql_types::Timestamp, sqlite::SqliteConnection};

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

    pub(crate) async fn load_workspace_element_summaries_by_session(
        &self,
        workspace_id: &str,
        session_id: &str,
    ) -> Result<Vec<WorkspaceElementSummary>, DbError> {
        let workspace_id = workspace_id.to_owned();
        let session_id = session_id.to_owned();

        executor::run(self.pool.clone(), move |connection| {
            load_workspace_element_summaries_by_session(
                connection,
                workspace_id.as_str(),
                session_id.as_str(),
            )
        })
        .await
    }

    pub(crate) async fn load_workspace_map_config_json(
        &self,
        workspace_id: &str,
    ) -> Result<Option<String>, DbError> {
        let workspace_id = workspace_id.to_owned();

        executor::run(self.pool.clone(), move |connection| {
            load_workspace_map_config_json(connection, workspace_id.as_str())
        })
        .await
    }

    pub(crate) async fn save_workspace_map_config_json(
        &self,
        workspace_id: &str,
        config_json: &str,
    ) -> Result<(), DbError> {
        let workspace_id = workspace_id.to_owned();
        let config_json = config_json.to_owned();

        executor::run(self.pool.clone(), move |connection| {
            save_workspace_map_config_json(connection, workspace_id.as_str(), config_json.as_str())
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
    kind: WorkspaceElementKind,
    label: String,
    position_x: i32,
    position_y: i32,
    facing: Direction,
}

#[derive(Queryable)]
struct WorkspaceElementSummaryRow {
    kind: WorkspaceElementKind,
    label: String,
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

impl From<WorkspaceElementSummaryRow> for WorkspaceElementSummary {
    fn from(element: WorkspaceElementSummaryRow) -> Self {
        Self {
            kind: element.kind,
            label: element.label,
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
        })
        .collect())
}

fn load_workspace_element_summaries_by_session(
    connection: &mut SqliteConnection,
    workspace_id: &str,
    session_id: &str,
) -> Result<Vec<WorkspaceElementSummary>, DbError> {
    Ok(workspace_elements::table
        .select((workspace_elements::kind, workspace_elements::label))
        .filter(workspace_elements::workspace_id.eq(workspace_id))
        .filter(workspace_elements::assigned_session_id.eq(Some(session_id)))
        .order((
            workspace_elements::kind.asc(),
            workspace_elements::label.asc(),
            workspace_elements::id.asc(),
        ))
        .load::<WorkspaceElementSummaryRow>(connection)?
        .into_iter()
        .map(WorkspaceElementSummary::from)
        .collect())
}

fn load_workspace_map_config_json(
    connection: &mut SqliteConnection,
    workspace_id: &str,
) -> Result<Option<String>, DbError> {
    Ok(workspace_map_configs::table
        .select(workspace_map_configs::config_json)
        .filter(workspace_map_configs::workspace_id.eq(workspace_id))
        .first::<String>(connection)
        .optional()?)
}

fn save_workspace_map_config_json(
    connection: &mut SqliteConnection,
    workspace_id: &str,
    config_json: &str,
) -> Result<(), DbError> {
    diesel::insert_into(workspace_map_configs::table)
        .values((
            workspace_map_configs::workspace_id.eq(workspace_id),
            workspace_map_configs::config_json.eq(config_json),
        ))
        .on_conflict(workspace_map_configs::workspace_id)
        .do_update()
        .set((
            workspace_map_configs::config_json.eq(config_json),
            workspace_map_configs::updated_at
                .eq(diesel::dsl::sql::<Timestamp>("CURRENT_TIMESTAMP")),
        ))
        .execute(connection)?;

    Ok(())
}
