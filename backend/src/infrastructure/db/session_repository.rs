use crate::{
    domain::{
        Direction, Message, MessageRole, MessageStatus, PointWithFacing, Session, SessionKind,
        SessionStatus,
    },
    infrastructure::db::{executor, shared_pool, DbError, SqlitePool},
    schema::{messages, sessions},
};
use diesel::{
    prelude::*,
    sql_types::{BigInt, Nullable, Text, Timestamp},
    sqlite::SqliteConnection,
    Connection,
};
use std::{collections::HashMap, io};

#[derive(Clone)]
pub(crate) struct SessionRepository {
    pool: SqlitePool,
}

impl SessionRepository {
    pub(crate) fn new() -> Result<Self, DbError> {
        Ok(Self {
            pool: shared_pool()?,
        })
    }

    pub(crate) async fn load_sessions(&self) -> Result<Vec<Session>, DbError> {
        self.run(load_sessions).await
    }

    pub(crate) async fn load_sessions_by_workspace_id(
        &self,
        workspace_id: &str,
    ) -> Result<Vec<Session>, DbError> {
        let workspace_id = workspace_id.to_owned();

        self.run(move |connection| load_sessions_by_workspace_id(connection, workspace_id.as_str()))
            .await
    }

    pub(crate) async fn session_by_id(&self, session_id: &str) -> Result<Option<Session>, DbError> {
        let session_id = session_id.to_owned();

        self.run(move |connection| session_by_id(connection, session_id.as_str()))
            .await
    }

    pub(crate) async fn create_session(
        &self,
        session_id: &str,
        workspace_id: &str,
        session_name: &str,
        kind: SessionKind,
        spawn: PointWithFacing,
        current: PointWithFacing,
    ) -> Result<String, DbError> {
        let session_id = session_id.to_owned();
        let workspace_id = workspace_id.to_owned();
        let session_name = session_name.to_owned();

        self.run(move |connection| {
            create_session(
                connection,
                session_id.as_str(),
                workspace_id.as_str(),
                session_name.as_str(),
                kind,
                spawn,
                current,
            )
        })
        .await
    }

    pub(crate) async fn update_session_codex_thread_id(
        &self,
        session_id: &str,
        codex_thread_id: &str,
    ) -> Result<(), DbError> {
        let session_id = session_id.to_owned();
        let codex_thread_id = codex_thread_id.to_owned();

        self.run(move |connection| {
            let updated_session_id = update_session_codex_thread_id(
                connection,
                session_id.as_str(),
                codex_thread_id.as_str(),
            )?;

            expect_returned(updated_session_id, || {
                format!("session not found while attaching codex thread for {session_id}")
            })?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn update_session_status(
        &self,
        session_id: &str,
        status: SessionStatus,
    ) -> Result<(), DbError> {
        let session_id = session_id.to_owned();
        self.run(move |connection| {
            let updated_session_id =
                update_session_status(connection, session_id.as_str(), status)?;

            expect_returned(updated_session_id, || {
                format!("session not found while updating status for {session_id}")
            })?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn record_user_message(
        &self,
        session_id: &str,
        text: &str,
    ) -> Result<(), DbError> {
        let session_id = session_id.to_owned();
        let text = text.to_owned();

        self.run(move |connection| {
            connection.transaction::<_, DbError, _>(|connection| {
                insert_user_message(connection, session_id.as_str(), text.as_str())?;
                mark_session_working(connection, session_id.as_str())
            })
        })
        .await
    }

    pub(crate) async fn start_assistant_message(
        &self,
        session_id: &str,
    ) -> Result<String, DbError> {
        let session_id = session_id.to_owned();

        self.run(move |connection| {
            connection.transaction::<_, DbError, _>(|connection| {
                let latest_message = latest_message_for_session(connection, session_id.as_str())?;

                let assistant_message_id = match latest_message.as_ref() {
                    Some(message) if message.role == MessageRole::User => {
                        insert_assistant_message(
                            connection,
                            session_id.as_str(),
                            "",
                            MessageStatus::Pending,
                        )?
                    }
                    Some(message) => {
                        return Err(invalid_latest_message_error(
                            session_id.as_str(),
                            "start assistant message",
                            message,
                        )
                        .into());
                    }
                    None => {
                        return Err(io::Error::other(format!(
                            "cannot start assistant message for {session_id}: no latest user message"
                        ))
                        .into());
                    }
                };

                mark_session_working(connection, session_id.as_str())?;
                Ok(assistant_message_id)
            })
        })
        .await
    }

    pub(crate) async fn append_assistant_delta(
        &self,
        session_id: &str,
        delta: &str,
    ) -> Result<String, DbError> {
        let session_id = session_id.to_owned();
        let delta = delta.to_owned();

        self.run(move |connection| {
            connection.transaction::<_, DbError, _>(|connection| {
                let latest_message = latest_message_for_session(connection, session_id.as_str())?;

                let assistant_message_id = match latest_message.as_ref() {
                    Some(message)
                        if message.role == MessageRole::Assistant
                            && (message.status == MessageStatus::Pending
                                || message.status == MessageStatus::Streaming) =>
                    {
                        let updated_message = append_to_assistant_message(
                            connection,
                            message.id.as_str(),
                            delta.as_str(),
                        )?;

                        expect_returned(updated_message, || {
                            format!("active assistant message not found for {session_id}")
                        })?
                    }
                    Some(message) if message.role == MessageRole::User => insert_assistant_message(
                        connection,
                        session_id.as_str(),
                        delta.as_str(),
                        MessageStatus::Streaming,
                    )?,
                    Some(message) => {
                        return Err(invalid_latest_message_error(
                            session_id.as_str(),
                            "append assistant delta",
                            message,
                        )
                        .into());
                    }
                    None => {
                        return Err(io::Error::other(format!(
                            "cannot append assistant delta for {session_id}: no latest message"
                        ))
                        .into());
                    }
                };

                mark_session_working(connection, session_id.as_str())?;
                Ok(assistant_message_id)
            })
        })
        .await
    }

    pub(crate) async fn complete_assistant_message(&self, session_id: &str) -> Result<(), DbError> {
        let session_id = session_id.to_owned();

        self.run(move |connection| {
            connection.transaction::<_, DbError, _>(|connection| {
                let latest_message = latest_message_for_session(connection, session_id.as_str())?;

                let Some(message) = latest_message.as_ref() else {
                    return Err(io::Error::other(format!(
                        "cannot complete assistant message for {session_id}: no latest message"
                    ))
                    .into());
                };

                if message.role != MessageRole::Assistant
                    || (message.status != MessageStatus::Pending
                        && message.status != MessageStatus::Streaming)
                {
                    return Err(invalid_latest_message_error(
                        session_id.as_str(),
                        "complete assistant message",
                        message,
                    )
                    .into());
                }

                let completed_message_id =
                    complete_assistant_message_by_id(connection, message.id.as_str())?;

                expect_returned(completed_message_id, || {
                    format!("active assistant message not found for {session_id}")
                })?;
                Ok(())
            })
        })
        .await
    }

    pub(crate) async fn fail_active_turn(
        &self,
        session_id: &str,
        error_text: &str,
    ) -> Result<(), DbError> {
        let session_id = session_id.to_owned();
        let error_text = error_text.to_owned();

        self.run(move |connection| {
            connection.transaction::<_, DbError, _>(|connection| {
                let latest_message = latest_message_for_session(connection, session_id.as_str())?;

                if let Some(message) = latest_message.as_ref() {
                    if message.role == MessageRole::Assistant
                        && (message.status == MessageStatus::Pending
                            || message.status == MessageStatus::Streaming)
                    {
                        let failed_message_id = fail_assistant_message_by_id(
                            connection,
                            message.id.as_str(),
                            error_text.as_str(),
                        )?;

                        expect_returned(failed_message_id, || {
                            format!("active assistant message not found for {session_id}")
                        })?;
                    }
                }

                let updated_session_id =
                    update_session_status(connection, session_id.as_str(), SessionStatus::Idle)?;
                expect_returned(updated_session_id, || {
                    format!("session not found while releasing failed turn for {session_id}")
                })?;
                Ok(())
            })
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

fn mark_session_working(
    connection: &mut SqliteConnection,
    session_id: &str,
) -> Result<(), DbError> {
    let updated_session_id = update_session_status(connection, session_id, SessionStatus::Working)?;

    expect_returned(updated_session_id, || {
        format!("session not found while marking {session_id} as working")
    })?;
    Ok(())
}

fn expect_returned<T, F>(returned: Option<T>, message: F) -> Result<T, DbError>
where
    F: FnOnce() -> String,
{
    returned.ok_or_else(|| io::Error::other(message()).into())
}

fn invalid_latest_message_error(session_id: &str, action: &str, message: &Message) -> io::Error {
    io::Error::other(format!(
        "cannot {action} for {session_id}: latest message {} is {} with status {}",
        message.id, message.role, message.status
    ))
}

#[derive(Queryable)]
struct SessionRow {
    session_id: String,
    workspace_id: String,
    name: String,
    kind: SessionKind,
    status: SessionStatus,
    spawn_x: i32,
    spawn_y: i32,
    spawn_facing: Direction,
    current_x: i32,
    current_y: i32,
    current_facing: Direction,
}

#[derive(Queryable)]
struct MessageRow {
    id: String,
    session_id: String,
    role: MessageRole,
    text: String,
    status: MessageStatus,
}

impl From<MessageRow> for Message {
    fn from(message: MessageRow) -> Self {
        Self {
            id: message.id,
            session_id: message.session_id,
            role: message.role,
            text: message.text,
            status: message.status,
        }
    }
}

impl From<SessionRow> for Session {
    fn from(session: SessionRow) -> Self {
        Self {
            session_id: session.session_id,
            workspace_id: session.workspace_id,
            name: session.name,
            kind: session.kind,
            status: session.status,
            spawn: PointWithFacing {
                x: session.spawn_x,
                y: session.spawn_y,
                facing: session.spawn_facing,
            },
            current: PointWithFacing {
                x: session.current_x,
                y: session.current_y,
                facing: session.current_facing,
            },
            messages: Vec::new(),
        }
    }
}

fn load_sessions(connection: &mut SqliteConnection) -> Result<Vec<Session>, DbError> {
    let session_rows = sessions::table
        .select((
            sessions::session_id,
            sessions::workspace_id,
            sessions::name,
            sessions::kind,
            sessions::status,
            sessions::spawn_x,
            sessions::spawn_y,
            sessions::spawn_facing,
            sessions::current_x,
            sessions::current_y,
            sessions::current_facing,
        ))
        .filter(sessions::archived_at.is_null())
        .order((sessions::name.asc(), sessions::session_id.asc()))
        .load::<SessionRow>(connection)?;

    hydrate_sessions(connection, session_rows)
}

fn load_sessions_by_workspace_id(
    connection: &mut SqliteConnection,
    workspace_id: &str,
) -> Result<Vec<Session>, DbError> {
    let session_rows = sessions::table
        .select((
            sessions::session_id,
            sessions::workspace_id,
            sessions::name,
            sessions::kind,
            sessions::status,
            sessions::spawn_x,
            sessions::spawn_y,
            sessions::spawn_facing,
            sessions::current_x,
            sessions::current_y,
            sessions::current_facing,
        ))
        .filter(sessions::workspace_id.eq(workspace_id))
        .filter(sessions::archived_at.is_null())
        .order((sessions::name.asc(), sessions::session_id.asc()))
        .load::<SessionRow>(connection)?;

    hydrate_sessions(connection, session_rows)
}

fn hydrate_sessions(
    connection: &mut SqliteConnection,
    session_rows: Vec<SessionRow>,
) -> Result<Vec<Session>, DbError> {
    let session_ids = session_rows
        .iter()
        .map(|session| session.session_id.as_str())
        .collect::<Vec<_>>();

    if session_ids.is_empty() {
        return Ok(Vec::new());
    }

    let message_rows = messages::table
        .select((
            messages::id,
            messages::session_id,
            messages::role,
            messages::text,
            messages::status,
        ))
        .filter(messages::session_id.eq_any(session_ids))
        .order((
            messages::session_id.asc(),
            messages::created_at.asc(),
            diesel::dsl::sql::<BigInt>("messages.rowid").asc(),
        ))
        .load::<MessageRow>(connection)?;

    let mut messages_by_session_id = HashMap::<String, Vec<Message>>::new();

    for message in message_rows {
        messages_by_session_id
            .entry(message.session_id.clone())
            .or_default()
            .push(Message::from(message));
    }

    let mut sessions = Vec::with_capacity(session_rows.len());

    for session_row in session_rows {
        let session_id = session_row.session_id.clone();
        let mut session = Session::from(session_row);
        session.messages = messages_by_session_id
            .remove(&session_id)
            .unwrap_or_default();
        sessions.push(session);
    }

    Ok(sessions)
}

fn session_by_id(
    connection: &mut SqliteConnection,
    session_id: &str,
) -> Result<Option<Session>, DbError> {
    let Some(session_row) = sessions::table
        .select((
            sessions::session_id,
            sessions::workspace_id,
            sessions::name,
            sessions::kind,
            sessions::status,
            sessions::spawn_x,
            sessions::spawn_y,
            sessions::spawn_facing,
            sessions::current_x,
            sessions::current_y,
            sessions::current_facing,
        ))
        .filter(sessions::session_id.eq(session_id))
        .filter(sessions::archived_at.is_null())
        .first::<SessionRow>(connection)
        .optional()?
    else {
        return Ok(None);
    };

    Ok(hydrate_sessions(connection, vec![session_row])?.pop())
}

fn create_session(
    connection: &mut SqliteConnection,
    session_id: &str,
    workspace_id: &str,
    session_name: &str,
    kind: SessionKind,
    spawn: PointWithFacing,
    current: PointWithFacing,
) -> Result<String, DbError> {
    let inserted_session_id = diesel::insert_into(sessions::table)
        .values((
            sessions::session_id.eq(session_id),
            sessions::workspace_id.eq(workspace_id),
            sessions::name.eq(session_name),
            sessions::kind.eq(kind),
            sessions::spawn_x.eq(spawn.x),
            sessions::spawn_y.eq(spawn.y),
            sessions::spawn_facing.eq(spawn.facing),
            sessions::current_x.eq(current.x),
            sessions::current_y.eq(current.y),
            sessions::current_facing.eq(current.facing),
            sessions::status.eq(SessionStatus::Idle),
        ))
        .returning(sessions::session_id)
        .get_result(connection)?;

    Ok(inserted_session_id)
}

fn update_session_codex_thread_id(
    connection: &mut SqliteConnection,
    session_id: &str,
    codex_thread_id: &str,
) -> QueryResult<Option<String>> {
    diesel::update(
        sessions::table
            .filter(sessions::session_id.eq(session_id))
            .filter(sessions::archived_at.is_null()),
    )
    .set((
        sessions::codex_thread_id.eq(Some(codex_thread_id)),
        sessions::updated_at.eq(diesel::dsl::sql::<Timestamp>("CURRENT_TIMESTAMP")),
    ))
    .returning(sessions::session_id)
    .get_result::<String>(connection)
    .optional()
}

fn insert_user_message(
    connection: &mut SqliteConnection,
    session_id: &str,
    text: &str,
) -> QueryResult<String> {
    diesel::insert_into(messages::table)
        .values((
            messages::session_id.eq(session_id),
            messages::role.eq(MessageRole::User),
            messages::text.eq(text),
            messages::status.eq(MessageStatus::Complete),
            messages::completed_at.eq(diesel::dsl::sql::<Nullable<Timestamp>>("CURRENT_TIMESTAMP")),
        ))
        .returning(messages::id)
        .get_result(connection)
}

fn insert_assistant_message(
    connection: &mut SqliteConnection,
    session_id: &str,
    text: &str,
    status: MessageStatus,
) -> QueryResult<String> {
    diesel::insert_into(messages::table)
        .values((
            messages::session_id.eq(session_id),
            messages::role.eq(MessageRole::Assistant),
            messages::text.eq(text),
            messages::status.eq(status),
        ))
        .returning(messages::id)
        .get_result(connection)
}

fn latest_message_for_session(
    connection: &mut SqliteConnection,
    session_id: &str,
) -> QueryResult<Option<Message>> {
    let message = messages::table
        .select((
            messages::id,
            messages::session_id,
            messages::role,
            messages::text,
            messages::status,
        ))
        .filter(messages::session_id.eq(session_id))
        .order((
            messages::created_at.desc(),
            diesel::dsl::sql::<BigInt>("messages.rowid").desc(),
        ))
        .first::<MessageRow>(connection)
        .optional()?;

    Ok(message.map(Message::from))
}

fn append_to_assistant_message(
    connection: &mut SqliteConnection,
    message_id: &str,
    delta: &str,
) -> QueryResult<Option<String>> {
    diesel::update(
        messages::table
            .filter(messages::id.eq(message_id))
            .filter(messages::role.eq(MessageRole::Assistant))
            .filter(
                messages::status
                    .eq(MessageStatus::Pending)
                    .or(messages::status.eq(MessageStatus::Streaming)),
            ),
    )
    .set((
        messages::text.eq(diesel::dsl::sql::<Text>("text || ").bind::<Text, _>(delta)),
        messages::status.eq(MessageStatus::Streaming),
        messages::updated_at.eq(diesel::dsl::sql::<Timestamp>("CURRENT_TIMESTAMP")),
    ))
    .returning(messages::id)
    .get_result(connection)
    .optional()
}

fn complete_assistant_message_by_id(
    connection: &mut SqliteConnection,
    message_id: &str,
) -> QueryResult<Option<String>> {
    diesel::update(
        messages::table
            .filter(messages::id.eq(message_id))
            .filter(messages::role.eq(MessageRole::Assistant))
            .filter(
                messages::status
                    .eq(MessageStatus::Pending)
                    .or(messages::status.eq(MessageStatus::Streaming)),
            ),
    )
    .set((
        messages::status.eq(MessageStatus::Complete),
        messages::completed_at.eq(diesel::dsl::sql::<Nullable<Timestamp>>("CURRENT_TIMESTAMP")),
        messages::updated_at.eq(diesel::dsl::sql::<Timestamp>("CURRENT_TIMESTAMP")),
    ))
    .returning(messages::id)
    .get_result(connection)
    .optional()
}

fn fail_assistant_message_by_id(
    connection: &mut SqliteConnection,
    message_id: &str,
    error_text: &str,
) -> QueryResult<Option<String>> {
    diesel::update(
        messages::table
            .filter(messages::id.eq(message_id))
            .filter(messages::role.eq(MessageRole::Assistant))
            .filter(
                messages::status
                    .eq(MessageStatus::Pending)
                    .or(messages::status.eq(MessageStatus::Streaming)),
            ),
    )
    .set((
        messages::text.eq(error_text),
        messages::status.eq(MessageStatus::Error),
        messages::completed_at.eq(diesel::dsl::sql::<Nullable<Timestamp>>("CURRENT_TIMESTAMP")),
        messages::updated_at.eq(diesel::dsl::sql::<Timestamp>("CURRENT_TIMESTAMP")),
    ))
    .returning(messages::id)
    .get_result(connection)
    .optional()
}

fn update_session_status(
    connection: &mut SqliteConnection,
    session_id: &str,
    status: SessionStatus,
) -> QueryResult<Option<String>> {
    diesel::update(sessions::table.filter(sessions::session_id.eq(session_id)))
        .set((
            sessions::status.eq(status),
            sessions::updated_at.eq(diesel::dsl::sql::<Timestamp>("CURRENT_TIMESTAMP")),
        ))
        .returning(sessions::session_id)
        .get_result::<String>(connection)
        .optional()
}
