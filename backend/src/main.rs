use crate::transport::{
    http::{
        create_session, create_workspace, get_session, get_sessions, get_workspace,
        get_workspace_elements, get_workspace_sessions, get_workspaces,
        perform_session_interaction,
    },
    ws::{health, websocket as websocket_route},
};
use actix_cors::Cors;
use actix_web::{App, HttpServer};
use std::error::Error;

pub(crate) mod domain;
pub(crate) mod infrastructure;
pub(crate) mod schema;
pub(crate) mod services;
pub(crate) mod sessions;
pub(crate) mod transport;

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
            .service(create_session)
            .service(create_workspace)
            .service(get_session)
            .service(get_sessions)
            .service(get_workspace)
            .service(get_workspace_sessions)
            .service(get_workspaces)
            .service(get_workspace_elements)
            .service(perform_session_interaction)
            .service(websocket_route)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
