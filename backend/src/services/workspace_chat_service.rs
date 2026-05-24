use crate::{
    domain::WorkspaceChatMessage,
    infrastructure::db::{DbError, WorkspaceChatRepository},
};

const GLOBAL_CHAT_CONTEXT_MESSAGE_LIMIT: i64 = 12;

#[derive(Clone)]
pub(crate) struct WorkspaceChatService {
    repository: WorkspaceChatRepository,
}

impl WorkspaceChatService {
    pub(crate) fn new() -> Result<Self, DbError> {
        Ok(Self {
            repository: WorkspaceChatRepository::new()?,
        })
    }

    pub(crate) async fn load_messages(
        &self,
        workspace_id: &str,
    ) -> Result<Vec<WorkspaceChatMessage>, DbError> {
        self.repository.load_messages(workspace_id).await
    }

    pub(crate) async fn load_context_messages(
        &self,
        workspace_id: &str,
    ) -> Result<Vec<WorkspaceChatMessage>, DbError> {
        self.repository
            .load_recent_messages(workspace_id, GLOBAL_CHAT_CONTEXT_MESSAGE_LIMIT)
            .await
    }

    pub(crate) async fn create_user_message(
        &self,
        workspace_id: &str,
        text: &str,
    ) -> Result<WorkspaceChatMessage, DbError> {
        self.repository
            .create_user_message(workspace_id, text)
            .await
    }

    pub(crate) async fn create_assistant_message(
        &self,
        workspace_id: &str,
        session_id: &str,
        parent_message_id: &str,
    ) -> Result<WorkspaceChatMessage, DbError> {
        self.repository
            .create_assistant_message(workspace_id, session_id, parent_message_id)
            .await
    }

    pub(crate) async fn create_system_message(
        &self,
        workspace_id: &str,
        text: &str,
        parent_message_id: Option<&str>,
    ) -> Result<WorkspaceChatMessage, DbError> {
        self.repository
            .create_system_message(workspace_id, text, parent_message_id)
            .await
    }

    pub(crate) async fn attach_session_message(
        &self,
        message_id: &str,
        session_message_id: &str,
    ) -> Result<(), DbError> {
        self.repository
            .attach_session_message(message_id, session_message_id)
            .await
    }

    pub(crate) async fn append_assistant_delta(
        &self,
        message_id: &str,
        delta: &str,
    ) -> Result<(), DbError> {
        self.repository
            .append_assistant_delta(message_id, delta)
            .await
    }

    pub(crate) async fn complete_assistant_message(&self, message_id: &str) -> Result<(), DbError> {
        self.repository.complete_assistant_message(message_id).await
    }

    pub(crate) async fn fail_assistant_message(
        &self,
        message_id: &str,
        error_text: &str,
    ) -> Result<(), DbError> {
        self.repository
            .fail_assistant_message(message_id, error_text)
            .await
    }
}
