use crate::{
    db::SqlitePool,
    schema::{minion_elements, minions},
    AnyError,
};
use actix_web::{error, get, web, HttpResponse, Result};
use diesel::{prelude::*, sqlite::SqliteConnection};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Queryable)]
struct MinionRow {
    id: String,
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
struct MinionElementRow {
    id: String,
    minion_id: String,
    kind: String,
    label: String,
    position_x: i32,
    position_y: i32,
    facing: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MinionWithElements {
    id: String,
    workspace_id: String,
    name: String,
    kind: String,
    status: String,
    spawn: PointWithFacing,
    current: PointWithFacing,
    elements: Vec<MinionElement>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MinionElement {
    id: String,
    minion_id: String,
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

#[get("/api/minions")]
pub(crate) async fn get_minions(pool: web::Data<SqlitePool>) -> Result<HttpResponse> {
    let pool = pool.get_ref().clone();
    let minions = web::block(move || {
        let mut connection = pool.get()?;
        load_minions_with_elements(&mut connection)
    })
    .await
    .map_err(error::ErrorInternalServerError)?
    .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(minions))
}

pub(crate) fn load_minions_with_elements(
    connection: &mut SqliteConnection,
) -> Result<Vec<MinionWithElements>, AnyError> {
    let minion_rows = minions::table
        .select((
            minions::id,
            minions::workspace_id,
            minions::name,
            minions::kind,
            minions::status,
            minions::spawn_x,
            minions::spawn_y,
            minions::spawn_facing,
            minions::current_x,
            minions::current_y,
            minions::current_facing,
        ))
        .filter(minions::archived_at.is_null())
        .order((minions::name.asc(), minions::id.asc()))
        .load::<MinionRow>(connection)?;

    let element_rows = minion_elements::table
        .select((
            minion_elements::id,
            minion_elements::minion_id,
            minion_elements::kind,
            minion_elements::label,
            minion_elements::position_x,
            minion_elements::position_y,
            minion_elements::facing,
        ))
        .order((
            minion_elements::minion_id.asc(),
            minion_elements::kind.asc(),
            minion_elements::label.asc(),
            minion_elements::id.asc(),
        ))
        .load::<MinionElementRow>(connection)?;

    let mut elements_by_minion_id = HashMap::<String, Vec<MinionElement>>::new();

    for element in element_rows {
        elements_by_minion_id
            .entry(element.minion_id.clone())
            .or_default()
            .push(MinionElement {
                id: element.id,
                minion_id: element.minion_id,
                kind: element.kind,
                label: element.label,
                position: Point {
                    x: element.position_x,
                    y: element.position_y,
                },
                facing: element.facing,
            });
    }

    Ok(minion_rows
        .into_iter()
        .map(|minion| MinionWithElements {
            id: minion.id.clone(),
            workspace_id: minion.workspace_id,
            name: minion.name,
            kind: minion.kind,
            status: minion.status,
            spawn: PointWithFacing {
                x: minion.spawn_x,
                y: minion.spawn_y,
                facing: minion.spawn_facing,
            },
            current: PointWithFacing {
                x: minion.current_x,
                y: minion.current_y,
                facing: minion.current_facing,
            },
            elements: elements_by_minion_id.remove(&minion.id).unwrap_or_default(),
        })
        .collect())
}
