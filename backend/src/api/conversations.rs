use crate::{
    schema::{conversations, messages},
    AnyError,
};
use diesel::{prelude::*, sqlite::SqliteConnection};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Queryable)]
struct ConversationRow {
    id: String,
    user_id: i32,
    workspace_id: String,
    minion_id: Option<String>,
    title: String,
    codex_thread_id: Option<String>,
    current_session_id: Option<String>,
    cwd: Option<String>,
    status: String,
}

#[derive(Queryable)]
struct MessageRow {
    id: String,
    conversation_id: String,
    role: String,
    text: String,
    status: String,
    codex_turn_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConversationWithMessages {
    id: String,
    user_id: i32,
    workspace_id: String,
    minion_id: Option<String>,
    title: String,
    codex_thread_id: Option<String>,
    current_session_id: Option<String>,
    cwd: Option<String>,
    status: String,
    messages: Vec<Message>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Message {
    id: String,
    conversation_id: String,
    role: String,
    text: String,
    status: String,
    codex_turn_id: Option<String>,
}

pub(crate) fn load_conversations_with_messages(
    connection: &mut SqliteConnection,
) -> Result<Vec<ConversationWithMessages>, AnyError> {
    let conversation_rows = conversations::table
        .select((
            conversations::id,
            conversations::user_id,
            conversations::workspace_id,
            conversations::minion_id,
            conversations::title,
            conversations::codex_thread_id,
            conversations::current_session_id,
            conversations::cwd,
            conversations::status,
        ))
        .filter(conversations::archived_at.is_null())
        .order((conversations::updated_at.desc(), conversations::id.asc()))
        .load::<ConversationRow>(connection)?;

    let message_rows = messages::table
        .select((
            messages::id,
            messages::conversation_id,
            messages::role,
            messages::text,
            messages::status,
            messages::codex_turn_id,
        ))
        .order((
            messages::conversation_id.asc(),
            messages::created_at.asc(),
            messages::id.asc(),
        ))
        .load::<MessageRow>(connection)?;

    let mut messages_by_conversation_id = HashMap::<String, Vec<Message>>::new();

    for message in message_rows {
        messages_by_conversation_id
            .entry(message.conversation_id.clone())
            .or_default()
            .push(Message {
                id: message.id,
                conversation_id: message.conversation_id,
                role: message.role,
                text: message.text,
                status: message.status,
                codex_turn_id: message.codex_turn_id,
            });
    }

    Ok(conversation_rows
        .into_iter()
        .map(|conversation| ConversationWithMessages {
            id: conversation.id.clone(),
            user_id: conversation.user_id,
            workspace_id: conversation.workspace_id,
            minion_id: conversation.minion_id,
            title: conversation.title,
            codex_thread_id: conversation.codex_thread_id,
            current_session_id: conversation.current_session_id,
            cwd: conversation.cwd,
            status: conversation.status,
            messages: messages_by_conversation_id
                .remove(&conversation.id)
                .unwrap_or_default(),
        })
        .collect())
}
