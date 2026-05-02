use crate::{
    protocol::{ClientMessage, ServerEvent, SessionCommand},
    router::{ConnectionRouter, SessionHandle},
    CHANNEL_BUFFER,
};
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::sync::mpsc;

fn router_with_outbox() -> (ConnectionRouter, mpsc::Receiver<ServerEvent>) {
    let (outbox_tx, outbox_rx) = mpsc::channel(CHANNEL_BUFFER);
    (ConnectionRouter::new(outbox_tx), outbox_rx)
}

fn active_session_mailbox() -> (mpsc::Sender<SessionCommand>, mpsc::Receiver<SessionCommand>) {
    mpsc::channel(CHANNEL_BUFFER)
}

fn session_handle(inbox: mpsc::Sender<SessionCommand>) -> SessionHandle {
    SessionHandle {
        inbox: Some(inbox),
        task: None,
    }
}

fn closed_session_mailbox() -> mpsc::Sender<SessionCommand> {
    let (session_tx, session_rx) = mpsc::channel(CHANNEL_BUFFER);
    drop(session_rx);
    session_tx
}

async fn recv_event(outbox_rx: &mut mpsc::Receiver<ServerEvent>) -> Value {
    let event = tokio::time::timeout(std::time::Duration::from_millis(100), outbox_rx.recv())
        .await
        .expect("timed out waiting for websocket event")
        .expect("websocket outbox closed");
    serde_json::to_value(event).expect("server event should serialize")
}

async fn expect_no_event(outbox_rx: &mut mpsc::Receiver<ServerEvent>) {
    let event = tokio::time::timeout(std::time::Duration::from_millis(20), outbox_rx.recv()).await;
    assert!(event.is_err(), "unexpected websocket event: {event:?}");
}

#[test]
fn session_start_deserializes_with_omitted_session_id() {
    let message: ClientMessage = serde_json::from_value(json!({
        "type": "session.start"
    }))
    .expect("session.start without session_id should deserialize");

    match message {
        ClientMessage::SessionStart { session_id } => assert_eq!(session_id, None),
        other => panic!("unexpected message: {other:?}"),
    }
}

#[test]
fn session_start_deserializes_with_null_session_id() {
    let message: ClientMessage = serde_json::from_value(json!({
        "type": "session.start",
        "session_id": null
    }))
    .expect("session.start with null session_id should deserialize");

    match message {
        ClientMessage::SessionStart { session_id } => assert_eq!(session_id, None),
        other => panic!("unexpected message: {other:?}"),
    }
}

#[test]
fn session_start_uses_provided_session_id_as_is() {
    let message: ClientMessage = serde_json::from_value(json!({
        "type": "session.start",
        "session_id": ""
    }))
    .expect("session.start with string session_id should deserialize");

    match message {
        ClientMessage::SessionStart { session_id } => {
            assert_eq!(session_id, Some(String::new()));
        }
        other => panic!("unexpected message: {other:?}"),
    }
}

#[test]
fn turn_start_accepts_provided_session_id_and_prompt() {
    let message: ClientMessage = serde_json::from_value(json!({
        "type": "turn.start",
        "session_id": "session-1",
        "prompt": "hello"
    }))
    .expect("turn.start should deserialize");

    match message {
        ClientMessage::TurnStart {
            session_id,
            cwd,
            prompt,
        } => {
            assert_eq!(session_id, Some("session-1".to_owned()));
            assert_eq!(cwd, None);
            assert_eq!(prompt, "hello");
        }
        other => panic!("unexpected message: {other:?}"),
    }
}

#[test]
fn turn_start_deserializes_with_omitted_session_id() {
    let message: ClientMessage = serde_json::from_value(json!({
        "type": "turn.start",
        "prompt": "hello"
    }))
    .expect("turn.start without session_id should deserialize");

    match message {
        ClientMessage::TurnStart {
            session_id,
            cwd,
            prompt,
        } => {
            assert_eq!(session_id, None);
            assert_eq!(cwd, None);
            assert_eq!(prompt, "hello");
        }
        other => panic!("unexpected message: {other:?}"),
    }
}

#[test]
fn turn_start_deserializes_with_null_session_id() {
    let message: ClientMessage = serde_json::from_value(json!({
        "type": "turn.start",
        "session_id": null,
        "prompt": "hello"
    }))
    .expect("turn.start with null session_id should deserialize");

    match message {
        ClientMessage::TurnStart {
            session_id,
            cwd,
            prompt,
        } => {
            assert_eq!(session_id, None);
            assert_eq!(cwd, None);
            assert_eq!(prompt, "hello");
        }
        other => panic!("unexpected message: {other:?}"),
    }
}

#[test]
fn turn_start_deserializes_first_turn_cwd() {
    let message: ClientMessage = serde_json::from_value(json!({
        "type": "turn.start",
        "cwd": "/tmp/project",
        "prompt": "hello"
    }))
    .expect("turn.start with cwd should deserialize");

    match message {
        ClientMessage::TurnStart {
            session_id,
            cwd,
            prompt,
        } => {
            assert_eq!(session_id, None);
            assert_eq!(cwd, Some(PathBuf::from("/tmp/project")));
            assert_eq!(prompt, "hello");
        }
        other => panic!("unexpected message: {other:?}"),
    }
}

#[test]
fn approval_respond_requires_session_id_and_answer() {
    let message: ClientMessage = serde_json::from_value(json!({
        "type": "approval.respond",
        "session_id": "session-1",
        "answer": "Y"
    }))
    .expect("approval.respond should deserialize");

    match message {
        ClientMessage::ApprovalRespond { session_id, answer } => {
            assert_eq!(session_id, "session-1");
            assert_eq!(answer, "Y");
        }
        other => panic!("unexpected message: {other:?}"),
    }
}

#[test]
fn generated_session_ids_increment_per_router() {
    let (mut router, _outbox_rx) = router_with_outbox();

    assert_eq!(router.next_session_id(), "session-1");
    assert_eq!(router.next_session_id(), "session-2");
    assert_eq!(router.next_session_id(), "session-3");
}

#[test]
fn generated_session_id_skips_existing_active_session_id() {
    let (mut router, _outbox_rx) = router_with_outbox();
    let (session_tx, _session_rx) = active_session_mailbox();
    router
        .sessions
        .insert("session-1".to_owned(), session_handle(session_tx));

    assert_eq!(router.next_session_id(), "session-2");
}

#[tokio::test]
async fn first_turn_without_cwd_emits_required_cwd_error() {
    let (mut router, mut outbox_rx) = router_with_outbox();

    router
        .start_turn(None, None, "hello".to_owned())
        .await
        .expect("missing first-turn cwd should be reported as websocket event");

    assert!(!router.sessions.contains_key("session-1"));
    assert_eq!(
        recv_event(&mut outbox_rx).await,
        json!({
            "type": "error",
            "session_id": "session-1",
            "message": "cwd is required"
        })
    );
}

#[tokio::test]
async fn first_turn_with_relative_cwd_emits_absolute_cwd_error() {
    let (mut router, mut outbox_rx) = router_with_outbox();

    router
        .start_turn(None, Some(PathBuf::from("frontend")), "hello".to_owned())
        .await
        .expect("relative first-turn cwd should be reported as websocket event");

    assert!(!router.sessions.contains_key("session-1"));
    assert_eq!(
        recv_event(&mut outbox_rx).await,
        json!({
            "type": "error",
            "session_id": "session-1",
            "message": "cwd must be absolute"
        })
    );
}

#[tokio::test]
async fn missing_turn_start_session_emits_session_not_found() {
    let (mut router, mut outbox_rx) = router_with_outbox();

    router
        .send_to_session(
            "missing-session".to_owned(),
            SessionCommand::StartTurn {
                prompt: "hello".to_owned(),
            },
        )
        .await
        .expect("missing session should be reported as websocket event");

    assert_eq!(
        recv_event(&mut outbox_rx).await,
        json!({
            "type": "error",
            "session_id": "missing-session",
            "message": "session not found"
        })
    );
}

#[tokio::test]
async fn missing_approval_session_emits_session_not_found() {
    let (mut router, mut outbox_rx) = router_with_outbox();

    router
        .send_to_session(
            "missing-session".to_owned(),
            SessionCommand::RespondToApproval {
                answer: "Y".to_owned(),
            },
        )
        .await
        .expect("missing session should be reported as websocket event");

    assert_eq!(
        recv_event(&mut outbox_rx).await,
        json!({
            "type": "error",
            "session_id": "missing-session",
            "message": "session not found"
        })
    );
}

#[tokio::test]
async fn start_turn_routes_to_existing_session_mailbox() {
    let (mut router, mut outbox_rx) = router_with_outbox();
    let (session_tx, mut session_rx) = active_session_mailbox();
    router
        .sessions
        .insert("session-1".to_owned(), session_handle(session_tx));

    router
        .send_to_session(
            "session-1".to_owned(),
            SessionCommand::StartTurn {
                prompt: "hello".to_owned(),
            },
        )
        .await
        .expect("send_to_session should succeed");

    let command = session_rx
        .recv()
        .await
        .expect("session command should be routed");
    match command {
        SessionCommand::StartTurn { prompt } => assert_eq!(prompt, "hello"),
        other => panic!("unexpected session command: {other:?}"),
    }
    expect_no_event(&mut outbox_rx).await;
}

#[tokio::test]
async fn approval_response_routes_to_existing_session_mailbox() {
    let (mut router, mut outbox_rx) = router_with_outbox();
    let (session_tx, mut session_rx) = active_session_mailbox();
    router
        .sessions
        .insert("session-1".to_owned(), session_handle(session_tx));

    router
        .send_to_session(
            "session-1".to_owned(),
            SessionCommand::RespondToApproval {
                answer: "N".to_owned(),
            },
        )
        .await
        .expect("send_to_session should succeed");

    let command = session_rx
        .recv()
        .await
        .expect("session command should be routed");
    match command {
        SessionCommand::RespondToApproval { answer } => assert_eq!(answer, "N"),
        other => panic!("unexpected session command: {other:?}"),
    }
    expect_no_event(&mut outbox_rx).await;
}

#[tokio::test]
async fn closed_session_mailbox_is_removed_and_reported() {
    let (mut router, mut outbox_rx) = router_with_outbox();
    router.sessions.insert(
        "session-1".to_owned(),
        session_handle(closed_session_mailbox()),
    );

    router
        .send_to_session(
            "session-1".to_owned(),
            SessionCommand::StartTurn {
                prompt: "hello".to_owned(),
            },
        )
        .await
        .expect("closed session should be reported as websocket event");

    assert!(!router.sessions.contains_key("session-1"));
    assert_eq!(
        recv_event(&mut outbox_rx).await,
        json!({
            "type": "error",
            "session_id": "session-1",
            "message": "session is not running"
        })
    );
}

#[tokio::test]
async fn duplicate_active_session_start_reports_ready_without_replacing_mailbox() {
    let (mut router, mut outbox_rx) = router_with_outbox();
    let (session_tx, mut session_rx) = active_session_mailbox();
    router
        .sessions
        .insert("session-1".to_owned(), session_handle(session_tx));

    router
        .start_session("session-1".to_owned(), PathBuf::from("/tmp/minions-test"))
        .await
        .expect("duplicate active session should report ready");

    assert_eq!(
        recv_event(&mut outbox_rx).await,
        json!({
            "type": "session.ready",
            "session_id": "session-1"
        })
    );
    assert!(router.sessions.contains_key("session-1"));
    assert!(session_rx.try_recv().is_err());
}

#[tokio::test]
async fn router_run_shuts_down_all_sessions_when_client_inbox_closes() {
    let (mut router, _outbox_rx) = router_with_outbox();
    let (session_1_tx, mut session_1_rx) = active_session_mailbox();
    let (session_2_tx, mut session_2_rx) = active_session_mailbox();

    router
        .sessions
        .insert("session-1".to_owned(), session_handle(session_1_tx));
    router
        .sessions
        .insert("session-2".to_owned(), session_handle(session_2_tx));

    let (client_tx, client_rx) = mpsc::channel(CHANNEL_BUFFER);
    drop(client_tx);

    router
        .run(client_rx)
        .await
        .expect("closed client inbox should stop the router cleanly");

    assert!(router.sessions.is_empty());
    assert!(session_1_rx.recv().await.is_none());
    assert!(session_2_rx.recv().await.is_none());
}
