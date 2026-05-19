use crate::{
    sessions::session_manager::SessionManager,
    transport::ws::{
        events::subscribe_server_events,
        protocol::{ClientMessage, ServerEvent},
    },
    AnyError, CHANNEL_BUFFER,
};
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use actix_ws::{Message, MessageStream, Session};
use futures_util::StreamExt;
use serde_json::json;
use tokio::sync::{broadcast, mpsc};

#[get("/health")]
pub(crate) async fn health() -> impl Responder {
    HttpResponse::Ok().json(json!({ "status": "ok" }))
}

#[get("/ws")]
pub(crate) async fn websocket(
    req: HttpRequest,
    body: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, socket, client) = actix_ws::handle(&req, body)?;
    actix_web::rt::spawn(run_websocket_connection(socket, client));
    Ok(response)
}

async fn run_websocket_connection(socket: Session, client: MessageStream) {
    let (manager_tx, manager_rx) = mpsc::channel(CHANNEL_BUFFER);
    let (outbox_tx, outbox_rx) = mpsc::channel(CHANNEL_BUFFER);
    let broadcast_rx = subscribe_server_events();

    let reader = actix_web::rt::spawn(read_client_messages(client, manager_tx));
    let broadcast_forwarder =
        actix_web::rt::spawn(forward_server_events(broadcast_rx, outbox_tx.clone()));
    let mut writer = actix_web::rt::spawn(write_client_messages(socket, outbox_rx));

    let mut session_manager = SessionManager::new(manager_rx, outbox_tx);
    let mut writer_finished = false;

    tokio::select! {
        _ = session_manager.run() => {}
        _ = &mut writer => {
            writer_finished = true;
        }
    }

    reader.abort();
    broadcast_forwarder.abort();
    let _ = reader.await;
    let _ = broadcast_forwarder.await;
    session_manager.shutdown_sessions().await;
    drop(session_manager);

    if !writer_finished {
        let _ = writer.await;
    }
}

async fn forward_server_events(
    mut events: broadcast::Receiver<ServerEvent>,
    outbox: mpsc::Sender<ServerEvent>,
) {
    loop {
        match events.recv().await {
            Ok(event) => {
                if outbox.send(event).await.is_err() {
                    break;
                }
            }
            Err(broadcast::error::RecvError::Lagged(_)) => {}
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
}

async fn read_client_messages(mut client: MessageStream, inbox: mpsc::Sender<ClientMessage>) {
    while let Some(Ok(message)) = client.next().await {
        match message {
            Message::Text(text) => {
                let Ok(client_message) = serde_json::from_str::<ClientMessage>(text.trim()) else {
                    continue;
                };

                if inbox.send(client_message).await.is_err() {
                    break;
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}

async fn write_client_messages(
    mut socket: Session,
    mut outbox: mpsc::Receiver<ServerEvent>,
) -> Result<(), AnyError> {
    while let Some(message) = outbox.recv().await {
        socket.text(serde_json::to_string(&message)?).await?;
    }

    let _ = socket.close(None).await;
    Ok(())
}
