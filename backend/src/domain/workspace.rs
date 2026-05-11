use super::Direction;

pub(crate) struct Workspace {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) root_path: Option<String>,
}

pub(crate) struct WorkspaceElement {
    pub(crate) id: String,
    pub(crate) assigned_session_id: Option<String>,
    pub(crate) kind: String,
    pub(crate) label: String,
    pub(crate) position: Point,
    pub(crate) facing: Direction,
}

pub(crate) struct Point {
    pub(crate) x: i32,
    pub(crate) y: i32,
}
