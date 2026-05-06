use crate::{
    db::SqlitePool,
    schema::{session_elements, sessions},
    AnyError,
};
use actix_web::{error, get, web, HttpResponse, Result};
use diesel::{prelude::*, sqlite::SqliteConnection};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Queryable)]
struct SessionRow {
    session_id: String,
    workspace_id: String,
    name: String,
    kind: String,
    status: String,
    spawn_x: i32,
    spawn_y: i32,
    spawn_facing: String,
    current_x: i32,
    current_y: i32,
    current_facing: String,
}

#[derive(Queryable)]
struct SessionElementRow {
    id: String,
    session_id: String,
    kind: String,
    label: String,
    position_x: i32,
    position_y: i32,
    facing: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionWithElements {
    #[serde(rename = "session_id")]
    session_id: String,
    workspace_id: String,
    name: String,
    kind: String,
    status: String,
    spawn: PointWithFacing,
    current: PointWithFacing,
    elements: Vec<SessionElement>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionElement {
    id: String,
    #[serde(rename = "session_id")]
    session_id: String,
    kind: String,
    label: String,
    position: Point,
    facing: String,
}

#[derive(Serialize)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Serialize)]
struct PointWithFacing {
    x: i32,
    y: i32,
    facing: String,
}

#[get("/api/sessions")]
pub(crate) async fn get_sessions(pool: web::Data<SqlitePool>) -> Result<HttpResponse> {
    let pool = pool.get_ref().clone();
    let sessions = web::block(move || {
        let mut connection = pool.get()?;
        load_sessions_with_elements(&mut connection)
    })
    .await
    .map_err(error::ErrorInternalServerError)?
    .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(sessions))
}

pub(crate) fn load_sessions_with_elements(
    connection: &mut SqliteConnection,
) -> Result<Vec<SessionWithElements>, AnyError> {
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

    let element_rows = session_elements::table
        .select((
            session_elements::id,
            session_elements::session_id,
            session_elements::kind,
            session_elements::label,
            session_elements::position_x,
            session_elements::position_y,
            session_elements::facing,
        ))
        .order((
            session_elements::session_id.asc(),
            session_elements::kind.asc(),
            session_elements::label.asc(),
            session_elements::id.asc(),
        ))
        .load::<SessionElementRow>(connection)?;

    let mut elements_by_session_id = HashMap::<String, Vec<SessionElement>>::new();

    for element in element_rows {
        elements_by_session_id
            .entry(element.session_id.clone())
            .or_default()
            .push(SessionElement {
                id: element.id,
                session_id: element.session_id,
                kind: element.kind,
                label: element.label,
                position: Point {
                    x: element.position_x,
                    y: element.position_y,
                },
                facing: element.facing,
            });
    }

    Ok(session_rows
        .into_iter()
        .map(|session| SessionWithElements {
            session_id: session.session_id.clone(),
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
            elements: elements_by_session_id
                .remove(&session.session_id)
                .unwrap_or_default(),
        })
        .collect())
}
