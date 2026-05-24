use crate::{domain, services::WorkspaceChatService};
use actix_web::{error, get, web, HttpResponse, Result};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkspaceChatMessageResponse {
    id: String,
    #[serde(rename = "workspace_id")]
    workspace_id: String,
    #[serde(rename = "session_id")]
    session_id: Option<String>,
    #[serde(rename = "session_message_id")]
    session_message_id: Option<String>,
    #[serde(rename = "parent_message_id")]
    parent_message_id: Option<String>,
    role: String,
    text: String,
    status: String,
}

impl From<domain::WorkspaceChatMessage> for WorkspaceChatMessageResponse {
    fn from(message: domain::WorkspaceChatMessage) -> Self {
        Self {
            id: message.id,
            workspace_id: message.workspace_id,
            session_id: message.session_id,
            session_message_id: message.session_message_id,
            parent_message_id: message.parent_message_id,
            role: message.role.as_str().to_owned(),
            text: message.text,
            status: message.status.as_str().to_owned(),
        }
    }
}

#[get("/api/workspaces/{workspace_id}/chat/messages")]
pub(crate) async fn get_workspace_chat_messages(
    workspace_id: web::Path<String>,
) -> Result<HttpResponse> {
    let workspace_chat_service =
        WorkspaceChatService::new().map_err(error::ErrorInternalServerError)?;
    let messages = workspace_chat_service
        .load_messages(workspace_id.as_str())
        .await
        .map_err(error::ErrorInternalServerError)?;
    let response = messages
        .into_iter()
        .map(WorkspaceChatMessageResponse::from)
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(response))
}
