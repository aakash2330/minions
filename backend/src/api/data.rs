use crate::{
    api::{
        conversations::{load_conversations_with_messages, ConversationWithMessages},
        minions::{load_minions_with_elements, MinionWithElements},
        workspaces::{load_workspaces, Workspace},
    },
    db::SqlitePool,
    AnyError,
};
use actix_web::{error, get, web, HttpResponse, Result};
use diesel::sqlite::SqliteConnection;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LocalAppData {
    workspaces: Vec<Workspace>,
    minions: Vec<MinionWithElements>,
    conversations: Vec<ConversationWithMessages>,
}

#[get("/api/data")]
pub(crate) async fn get_data(pool: web::Data<SqlitePool>) -> Result<HttpResponse> {
    let pool = pool.get_ref().clone();
    let data = web::block(move || {
        let mut connection = pool.get()?;
        load_local_app_data(&mut connection)
    })
    .await
    .map_err(error::ErrorInternalServerError)?
    .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(data))
}

fn load_local_app_data(connection: &mut SqliteConnection) -> Result<LocalAppData, AnyError> {
    Ok(LocalAppData {
        workspaces: load_workspaces(connection)?,
        minions: load_minions_with_elements(connection)?,
        conversations: load_conversations_with_messages(connection)?,
    })
}
