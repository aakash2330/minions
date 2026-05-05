use crate::{
    api::{get_data, get_minions},
    db::{create_pool, database_url},
    websocket::{health, websocket as websocket_route},
};
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use std::{error::Error, io};

pub(crate) mod api;
#[allow(dead_code)]
pub(crate) mod db;
pub(crate) mod protocol;
pub(crate) mod router;
pub(crate) mod schema;
pub(crate) mod session;
pub(crate) mod websocket;

#[cfg(test)]
mod main_tests;

pub(crate) const CHANNEL_BUFFER: usize = 32;
pub(crate) type AnyError = Box<dyn Error + Send + Sync>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_url = database_url();
    let db_pool = create_pool(&database_url).map_err(io::Error::other)?;

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173")
            .allowed_origin("http://127.0.0.1:5173");

        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .wrap(cors)
            .service(health)
            .service(get_data)
            .service(get_minions)
            .service(websocket_route)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
