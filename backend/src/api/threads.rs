use crate::{
    schema::{messages, threads},
    AnyError,
};
use diesel::{prelude::*, sqlite::SqliteConnection};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Queryable)]
struct ThreadRow {
    id: String,
    user_id: i32,
    workspace_id: String,
    session_id: Option<String>,
    title: String,
    codex_thread_id: Option<String>,
    cwd: Option<String>,
    status: String,
}

#[derive(Queryable)]
struct MessageRow {
    id: String,
    thread_id: String,
    role: String,
    text: String,
    status: String,
    codex_turn_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ThreadWithMessages {
    id: String,
    user_id: i32,
    workspace_id: String,
    #[serde(rename = "session_id")]
    session_id: Option<String>,
    title: String,
    codex_thread_id: Option<String>,
    cwd: Option<String>,
    status: String,
    messages: Vec<Message>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Message {
    id: String,
    #[serde(rename = "thread_id")]
    thread_id: String,
    role: String,
    text: String,
    status: String,
    codex_turn_id: Option<String>,
}

pub(crate) fn load_threads_with_messages(
    connection: &mut SqliteConnection,
) -> Result<Vec<ThreadWithMessages>, AnyError> {
    let thread_rows = threads::table
        .select((
            threads::id,
            threads::user_id,
            threads::workspace_id,
            threads::session_id,
            threads::title,
            threads::codex_thread_id,
            threads::cwd,
            threads::status,
        ))
        .filter(threads::archived_at.is_null())
        .order((threads::updated_at.desc(), threads::id.asc()))
        .load::<ThreadRow>(connection)?;

    let message_rows = messages::table
        .select((
            messages::id,
            messages::thread_id,
            messages::role,
            messages::text,
            messages::status,
            messages::codex_turn_id,
        ))
        .order((
            messages::thread_id.asc(),
            messages::created_at.asc(),
            messages::id.asc(),
        ))
        .load::<MessageRow>(connection)?;

    let mut messages_by_thread_id = HashMap::<String, Vec<Message>>::new();

    for message in message_rows {
        messages_by_thread_id
            .entry(message.thread_id.clone())
            .or_default()
            .push(Message {
                id: message.id,
                thread_id: message.thread_id,
                role: message.role,
                text: message.text,
                status: message.status,
                codex_turn_id: message.codex_turn_id,
            });
    }

    Ok(thread_rows
        .into_iter()
        .map(|thread| ThreadWithMessages {
            id: thread.id.clone(),
            user_id: thread.user_id,
            workspace_id: thread.workspace_id,
            session_id: thread.session_id,
            title: thread.title,
            codex_thread_id: thread.codex_thread_id,
            cwd: thread.cwd,
            status: thread.status,
            messages: messages_by_thread_id.remove(&thread.id).unwrap_or_default(),
        })
        .collect())
}
