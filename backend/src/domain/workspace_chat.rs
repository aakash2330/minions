use super::{MessageRole, MessageStatus};

#[derive(Clone)]
pub(crate) struct WorkspaceChatMessage {
    pub(crate) id: String,
    pub(crate) workspace_id: String,
    pub(crate) session_id: Option<String>,
    pub(crate) session_message_id: Option<String>,
    pub(crate) parent_message_id: Option<String>,
    pub(crate) role: MessageRole,
    pub(crate) text: String,
    pub(crate) status: MessageStatus,
}
