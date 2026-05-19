use super::protocol::ServerEvent;
use std::sync::OnceLock;
use tokio::sync::broadcast;

const SERVER_EVENT_BUFFER: usize = 32;

static SERVER_EVENTS: OnceLock<broadcast::Sender<ServerEvent>> = OnceLock::new();

fn server_events() -> &'static broadcast::Sender<ServerEvent> {
    SERVER_EVENTS.get_or_init(|| {
        let (sender, _) = broadcast::channel(SERVER_EVENT_BUFFER);
        sender
    })
}

pub(crate) fn subscribe_server_events() -> broadcast::Receiver<ServerEvent> {
    server_events().subscribe()
}

pub(crate) fn broadcast_server_event(event: ServerEvent) {
    let _ = server_events().send(event);
}
