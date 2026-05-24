use crate::{
    domain::{MessageRole, MessageStatus, WorkspaceChatMessage},
    infrastructure::db::{executor, shared_pool, DbError, SqlitePool},
    schema::workspace_chat_messages,
};
use diesel::{
    prelude::*,
    sql_types::{BigInt, Nullable, Text, Timestamp},
    sqlite::SqliteConnection,
};
use std::io;

#[derive(Clone)]
pub(crate) struct WorkspaceChatRepository {
    pool: SqlitePool,
}

impl WorkspaceChatRepository {
    pub(crate) fn new() -> Result<Self, DbError> {
        Ok(Self {
            pool: shared_pool()?,
        })
    }

    pub(crate) async fn load_messages(
        &self,
        workspace_id: &str,
    ) -> Result<Vec<WorkspaceChatMessage>, DbError> {
        let workspace_id = workspace_id.to_owned();

        self.run(move |connection| load_messages(connection, workspace_id.as_str()))
            .await
    }

    pub(crate) async fn load_recent_messages(
        &self,
        workspace_id: &str,
        limit: i64,
    ) -> Result<Vec<WorkspaceChatMessage>, DbError> {
        let workspace_id = workspace_id.to_owned();

        self.run(move |connection| load_recent_messages(connection, workspace_id.as_str(), limit))
            .await
    }

    pub(crate) async fn create_user_message(
        &self,
        workspace_id: &str,
        text: &str,
    ) -> Result<WorkspaceChatMessage, DbError> {
        self.create_message(
            workspace_id,
            None,
            None,
            None,
            MessageRole::User,
            text,
            MessageStatus::Complete,
        )
        .await
    }

    pub(crate) async fn create_assistant_message(
        &self,
        workspace_id: &str,
        session_id: &str,
        parent_message_id: &str,
    ) -> Result<WorkspaceChatMessage, DbError> {
        self.create_message(
            workspace_id,
            Some(session_id),
            None,
            Some(parent_message_id),
            MessageRole::Assistant,
            "",
            MessageStatus::Pending,
        )
        .await
    }

    pub(crate) async fn create_system_message(
        &self,
        workspace_id: &str,
        text: &str,
        parent_message_id: Option<&str>,
    ) -> Result<WorkspaceChatMessage, DbError> {
        self.create_message(
            workspace_id,
            None,
            None,
            parent_message_id,
            MessageRole::System,
            text,
            MessageStatus::Complete,
        )
        .await
    }

    pub(crate) async fn attach_session_message(
        &self,
        message_id: &str,
        session_message_id: &str,
    ) -> Result<(), DbError> {
        let message_id = message_id.to_owned();
        let session_message_id = session_message_id.to_owned();

        self.run(move |connection| {
            let updated_id = attach_session_message(
                connection,
                message_id.as_str(),
                session_message_id.as_str(),
            )?;

            expect_returned(updated_id, || {
                format!(
                    "workspace chat message not found while attaching session message {message_id}"
                )
            })?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn append_assistant_delta(
        &self,
        message_id: &str,
        delta: &str,
    ) -> Result<(), DbError> {
        let message_id = message_id.to_owned();
        let delta = delta.to_owned();

        self.run(move |connection| {
            let updated_id =
                append_assistant_delta(connection, message_id.as_str(), delta.as_str())?;

            expect_returned(updated_id, || {
                format!("active workspace chat assistant message not found for {message_id}")
            })?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn complete_assistant_message(&self, message_id: &str) -> Result<(), DbError> {
        let message_id = message_id.to_owned();

        self.run(move |connection| {
            let updated_id = complete_assistant_message(connection, message_id.as_str())?;

            expect_returned(updated_id, || {
                format!("active workspace chat assistant message not found for {message_id}")
            })?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn fail_assistant_message(
        &self,
        message_id: &str,
        error_text: &str,
    ) -> Result<(), DbError> {
        let message_id = message_id.to_owned();
        let error_text = error_text.to_owned();

        self.run(move |connection| {
            let updated_id =
                fail_assistant_message(connection, message_id.as_str(), error_text.as_str())?;

            expect_returned(updated_id, || {
                format!("active workspace chat assistant message not found for {message_id}")
            })?;
            Ok(())
        })
        .await
    }

    async fn create_message(
        &self,
        workspace_id: &str,
        session_id: Option<&str>,
        session_message_id: Option<&str>,
        parent_message_id: Option<&str>,
        role: MessageRole,
        text: &str,
        status: MessageStatus,
    ) -> Result<WorkspaceChatMessage, DbError> {
        let workspace_id = workspace_id.to_owned();
        let session_id = session_id.map(str::to_owned);
        let session_message_id = session_message_id.map(str::to_owned);
        let parent_message_id = parent_message_id.map(str::to_owned);
        let text = text.to_owned();

        self.run(move |connection| {
            Ok(insert_workspace_chat_message(
                connection,
                workspace_id.as_str(),
                session_id.as_deref(),
                session_message_id.as_deref(),
                parent_message_id.as_deref(),
                role,
                text.as_str(),
                status,
            )?)
        })
        .await
    }

    async fn run<T, F>(&self, operation: F) -> Result<T, DbError>
    where
        T: Send + 'static,
        F: FnOnce(&mut SqliteConnection) -> Result<T, DbError> + Send + 'static,
    {
        executor::run(self.pool.clone(), operation).await
    }
}

fn expect_returned<T, F>(returned: Option<T>, message: F) -> Result<T, DbError>
where
    F: FnOnce() -> String,
{
    returned.ok_or_else(|| io::Error::other(message()).into())
}

#[derive(Queryable)]
struct WorkspaceChatMessageRow {
    id: String,
    workspace_id: String,
    session_id: Option<String>,
    session_message_id: Option<String>,
    parent_message_id: Option<String>,
    role: MessageRole,
    text: String,
    status: MessageStatus,
}

impl From<WorkspaceChatMessageRow> for WorkspaceChatMessage {
    fn from(message: WorkspaceChatMessageRow) -> Self {
        Self {
            id: message.id,
            workspace_id: message.workspace_id,
            session_id: message.session_id,
            session_message_id: message.session_message_id,
            parent_message_id: message.parent_message_id,
            role: message.role,
            text: message.text,
            status: message.status,
        }
    }
}

fn workspace_chat_message_select() -> (
    workspace_chat_messages::id,
    workspace_chat_messages::workspace_id,
    workspace_chat_messages::session_id,
    workspace_chat_messages::session_message_id,
    workspace_chat_messages::parent_message_id,
    workspace_chat_messages::role,
    workspace_chat_messages::text,
    workspace_chat_messages::status,
) {
    (
        workspace_chat_messages::id,
        workspace_chat_messages::workspace_id,
        workspace_chat_messages::session_id,
        workspace_chat_messages::session_message_id,
        workspace_chat_messages::parent_message_id,
        workspace_chat_messages::role,
        workspace_chat_messages::text,
        workspace_chat_messages::status,
    )
}

fn load_messages(
    connection: &mut SqliteConnection,
    workspace_id: &str,
) -> Result<Vec<WorkspaceChatMessage>, DbError> {
    let messages = workspace_chat_messages::table
        .select(workspace_chat_message_select())
        .filter(workspace_chat_messages::workspace_id.eq(workspace_id))
        .order((
            workspace_chat_messages::created_at.asc(),
            diesel::dsl::sql::<BigInt>("workspace_chat_messages.rowid").asc(),
        ))
        .load::<WorkspaceChatMessageRow>(connection)?
        .into_iter()
        .map(WorkspaceChatMessage::from)
        .collect();

    Ok(messages)
}

fn load_recent_messages(
    connection: &mut SqliteConnection,
    workspace_id: &str,
    limit: i64,
) -> Result<Vec<WorkspaceChatMessage>, DbError> {
    let mut messages = workspace_chat_messages::table
        .select(workspace_chat_message_select())
        .filter(workspace_chat_messages::workspace_id.eq(workspace_id))
        .order((
            workspace_chat_messages::created_at.desc(),
            diesel::dsl::sql::<BigInt>("workspace_chat_messages.rowid").desc(),
        ))
        .limit(limit.max(0))
        .load::<WorkspaceChatMessageRow>(connection)?
        .into_iter()
        .map(WorkspaceChatMessage::from)
        .collect::<Vec<_>>();

    messages.reverse();
    Ok(messages)
}

fn insert_workspace_chat_message(
    connection: &mut SqliteConnection,
    workspace_id: &str,
    session_id: Option<&str>,
    session_message_id: Option<&str>,
    parent_message_id: Option<&str>,
    role: MessageRole,
    text: &str,
    status: MessageStatus,
) -> QueryResult<WorkspaceChatMessage> {
    let completed_at = if status == MessageStatus::Complete {
        diesel::dsl::sql::<Nullable<Timestamp>>("CURRENT_TIMESTAMP")
    } else {
        diesel::dsl::sql::<Nullable<Timestamp>>("NULL")
    };

    diesel::insert_into(workspace_chat_messages::table)
        .values((
            workspace_chat_messages::workspace_id.eq(workspace_id),
            workspace_chat_messages::session_id.eq(session_id),
            workspace_chat_messages::session_message_id.eq(session_message_id),
            workspace_chat_messages::parent_message_id.eq(parent_message_id),
            workspace_chat_messages::role.eq(role),
            workspace_chat_messages::text.eq(text),
            workspace_chat_messages::status.eq(status),
            workspace_chat_messages::completed_at.eq(completed_at),
        ))
        .returning(workspace_chat_message_select())
        .get_result::<WorkspaceChatMessageRow>(connection)
        .map(WorkspaceChatMessage::from)
}

fn attach_session_message(
    connection: &mut SqliteConnection,
    message_id: &str,
    session_message_id: &str,
) -> QueryResult<Option<String>> {
    diesel::update(
        workspace_chat_messages::table.filter(workspace_chat_messages::id.eq(message_id)),
    )
    .set((
        workspace_chat_messages::session_message_id.eq(Some(session_message_id)),
        workspace_chat_messages::updated_at.eq(diesel::dsl::sql::<Timestamp>("CURRENT_TIMESTAMP")),
    ))
    .returning(workspace_chat_messages::id)
    .get_result::<String>(connection)
    .optional()
}

fn append_assistant_delta(
    connection: &mut SqliteConnection,
    message_id: &str,
    delta: &str,
) -> QueryResult<Option<String>> {
    diesel::update(
        workspace_chat_messages::table
            .filter(workspace_chat_messages::id.eq(message_id))
            .filter(workspace_chat_messages::role.eq(MessageRole::Assistant))
            .filter(
                workspace_chat_messages::status
                    .eq(MessageStatus::Pending)
                    .or(workspace_chat_messages::status.eq(MessageStatus::Streaming)),
            ),
    )
    .set((
        workspace_chat_messages::text
            .eq(diesel::dsl::sql::<Text>("text || ").bind::<Text, _>(delta)),
        workspace_chat_messages::status.eq(MessageStatus::Streaming),
        workspace_chat_messages::updated_at.eq(diesel::dsl::sql::<Timestamp>("CURRENT_TIMESTAMP")),
    ))
    .returning(workspace_chat_messages::id)
    .get_result::<String>(connection)
    .optional()
}

fn complete_assistant_message(
    connection: &mut SqliteConnection,
    message_id: &str,
) -> QueryResult<Option<String>> {
    diesel::update(
        workspace_chat_messages::table
            .filter(workspace_chat_messages::id.eq(message_id))
            .filter(workspace_chat_messages::role.eq(MessageRole::Assistant))
            .filter(
                workspace_chat_messages::status
                    .eq(MessageStatus::Pending)
                    .or(workspace_chat_messages::status.eq(MessageStatus::Streaming)),
            ),
    )
    .set((
        workspace_chat_messages::status.eq(MessageStatus::Complete),
        workspace_chat_messages::completed_at
            .eq(diesel::dsl::sql::<Nullable<Timestamp>>("CURRENT_TIMESTAMP")),
        workspace_chat_messages::updated_at.eq(diesel::dsl::sql::<Timestamp>("CURRENT_TIMESTAMP")),
    ))
    .returning(workspace_chat_messages::id)
    .get_result::<String>(connection)
    .optional()
}

fn fail_assistant_message(
    connection: &mut SqliteConnection,
    message_id: &str,
    error_text: &str,
) -> QueryResult<Option<String>> {
    diesel::update(
        workspace_chat_messages::table
            .filter(workspace_chat_messages::id.eq(message_id))
            .filter(workspace_chat_messages::role.eq(MessageRole::Assistant))
            .filter(
                workspace_chat_messages::status
                    .eq(MessageStatus::Pending)
                    .or(workspace_chat_messages::status.eq(MessageStatus::Streaming)),
            ),
    )
    .set((
        workspace_chat_messages::text.eq(error_text),
        workspace_chat_messages::status.eq(MessageStatus::Error),
        workspace_chat_messages::completed_at
            .eq(diesel::dsl::sql::<Nullable<Timestamp>>("CURRENT_TIMESTAMP")),
        workspace_chat_messages::updated_at.eq(diesel::dsl::sql::<Timestamp>("CURRENT_TIMESTAMP")),
    ))
    .returning(workspace_chat_messages::id)
    .get_result::<String>(connection)
    .optional()
}
