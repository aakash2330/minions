use super::{Direction, WorkspaceElementKind};

pub(crate) struct Workspace {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) root_path: Option<String>,
}

pub(crate) struct WorkspaceElement {
    pub(crate) id: String,
    pub(crate) assigned_session_id: Option<String>,
    pub(crate) kind: WorkspaceElementKind,
    pub(crate) label: String,
    pub(crate) position: Point,
    pub(crate) facing: Direction,
    pub(crate) asset_id: Option<String>,
    pub(crate) width: Option<i32>,
    pub(crate) height: Option<i32>,
}

pub(crate) struct WorkspaceElementSummary {
    pub(crate) kind: WorkspaceElementKind,
    pub(crate) label: String,
}

pub(crate) struct Point {
    pub(crate) x: i32,
    pub(crate) y: i32,
}
