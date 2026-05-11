use super::{Direction, MessageRole, MessageStatus, SessionKind, SessionStatus};

pub(crate) struct Session {
    pub(crate) session_id: String,
    pub(crate) workspace_id: String,
    pub(crate) name: String,
    pub(crate) kind: SessionKind,
    pub(crate) status: SessionStatus,
    pub(crate) spawn: PointWithFacing,
    pub(crate) current: PointWithFacing,
    pub(crate) messages: Vec<Message>,
}

pub(crate) struct Message {
    pub(crate) id: String,
    pub(crate) session_id: String,
    pub(crate) role: MessageRole,
    pub(crate) text: String,
    pub(crate) status: MessageStatus,
}

pub(crate) struct PointWithFacing {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) facing: Direction,
}
