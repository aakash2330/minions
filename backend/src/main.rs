use crate::websocket::{health, websocket as websocket_route};
use actix_cors::Cors;
use actix_web::{App, HttpServer};
use std::error::Error;

pub(crate) mod protocol;
pub(crate) mod router;
pub(crate) mod session;
pub(crate) mod websocket;

#[cfg(test)]
mod main_tests;

pub(crate) const CHANNEL_BUFFER: usize = 32;
pub(crate) type AnyError = Box<dyn Error + Send + Sync>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173")
            .allowed_origin("http://127.0.0.1:5173");

        App::new()
            .wrap(cors)
            .service(health)
            .service(websocket_route)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
