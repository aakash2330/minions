use crate::{
    sessions::{
        messages::{SessionEvent, SessionTaskCommand},
        SessionRuntime,
    },
    transport::ws::protocol::{send_error, send_event, ClientMessage, ServerEvent},
    AnyError, CHANNEL_BUFFER,
};
use futures_util::future::join_all;
use std::{collections::HashMap, time::Duration};
use tokio::{sync::mpsc, task::JoinHandle};

const APP_SERVER_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(2);

pub(crate) struct SessionHandle {
    pub(crate) inbox: Option<mpsc::Sender<SessionTaskCommand>>,
    pub(crate) task: Option<JoinHandle<()>>,
}

impl SessionHandle {
    fn new(inbox: mpsc::Sender<SessionTaskCommand>, task: JoinHandle<()>) -> Self {
        Self {
            inbox: Some(inbox),
            task: Some(task),
        }
    }

    fn is_closed(&self) -> bool {
        self.inbox.as_ref().is_none_or(|inbox| inbox.is_closed())
            || self.task.as_ref().is_some_and(|task| task.is_finished())
    }

    async fn shutdown(mut self) {
        drop(self.inbox.take());

        let Some(mut task) = self.task.take() else {
            return;
        };

        tokio::select! {
            _ = &mut task => {}
            _ = tokio::time::sleep(APP_SERVER_SHUTDOWN_TIMEOUT) => {
                task.abort();
                let _ = task.await;
            }
        }
    }
}

impl Drop for SessionHandle {
    fn drop(&mut self) {
        drop(self.inbox.take());

        if let Some(task) = &self.task {
            task.abort();
        }
    }
}

pub(crate) struct SessionManager {
    inbox: mpsc::Receiver<ClientMessage>,
    session_events: mpsc::Receiver<SessionEvent>,
    session_event_tx: mpsc::Sender<SessionEvent>,
    outbox: mpsc::Sender<ServerEvent>,
    pub(crate) sessions: HashMap<String, SessionHandle>,
}

impl SessionManager {
    pub(crate) fn new(
        inbox: mpsc::Receiver<ClientMessage>,
        outbox: mpsc::Sender<ServerEvent>,
    ) -> Self {
        let (session_event_tx, session_events) = mpsc::channel(CHANNEL_BUFFER);

        Self {
            inbox,
            session_events,
            session_event_tx,
            outbox,
            sessions: HashMap::new(),
        }
    }

    pub(crate) async fn run(&mut self) -> Result<(), AnyError> {
        loop {
            tokio::select! {
                message = self.inbox.recv() => {
                    let Some(message) = message else {
                        break;
                    };

                    if let Err(error) = self.handle_message(message).await {
                        self.shutdown_sessions().await;
                        return Err(error);
                    }
                }
                event = self.session_events.recv() => {
                    if let Some(event) = event {
                        self.forward_session_event(event).await?;
                    }
                }
            }
        }

        self.shutdown_sessions().await;
        Ok(())
    }

    async fn handle_message(&mut self, message: ClientMessage) -> Result<(), AnyError> {
        match message {
            ClientMessage::TurnStart { session_id, prompt } => {
                self.handle_turn_start(session_id, prompt).await
            }
            ClientMessage::ApprovalRespond { session_id, answer } => {
                self.route_to_session(session_id, SessionTaskCommand::RespondToApproval { answer })
                    .await
            }
        }
    }

    async fn forward_session_event(&self, event: SessionEvent) -> Result<(), AnyError> {
        send_event(&self.outbox, ServerEvent::from(event)).await
    }

    async fn handle_turn_start(
        &mut self,
        session_id: String,
        prompt: String,
    ) -> Result<(), AnyError> {
        let session_id = session_id.trim().to_owned();

        if session_id.is_empty() {
            send_error(&self.outbox, None, "session_id is required").await?;
            return Ok(());
        }

        if !self.ensure_session_task_for_id(session_id.as_str()).await? {
            return Ok(());
        }

        self.route_to_session(session_id, SessionTaskCommand::StartTurn { prompt })
            .await
    }

    async fn ensure_session_task_for_id(&mut self, session_id: &str) -> Result<bool, AnyError> {
        if let Some(existing_session) = self.sessions.get(session_id) {
            if !existing_session.is_closed() {
                return Ok(true);
            }

            if let Some(session) = self.sessions.remove(session_id) {
                session.shutdown().await;
            }
        }

        let runtime = match SessionRuntime::new(session_id.to_owned()).await {
            Ok(runtime) => runtime,
            Err(message) => {
                send_error(&self.outbox, Some(session_id), message.as_str()).await?;
                return Ok(false);
            }
        };

        self.spawn_session_task(runtime).await;
        Ok(true)
    }

    async fn spawn_session_task(&mut self, runtime: SessionRuntime) {
        let session_id = runtime.session_id().to_owned();

        let (session_tx, session_rx) = mpsc::channel(CHANNEL_BUFFER);
        let task_session_id = session_id.clone();
        let task_outbox = self.session_event_tx.clone();

        let task = tokio::spawn(async move {
            if let Err(error) = runtime.run(session_rx, task_outbox.clone()).await {
                let _ = task_outbox
                    .send(SessionEvent::Error {
                        session_id: Some(task_session_id),
                        message: error.to_string(),
                    })
                    .await;
            }
        });

        self.sessions
            .insert(session_id, SessionHandle::new(session_tx, task));
    }

    async fn route_to_session(
        &mut self,
        session_id: String,
        command: SessionTaskCommand,
    ) -> Result<(), AnyError> {
        let Some(session) = self.sessions.get(&session_id) else {
            send_error(&self.outbox, Some(&session_id), "session not found").await?;
            return Ok(());
        };
        let Some(inbox) = session.inbox.clone() else {
            send_error(&self.outbox, Some(&session_id), "session not found").await?;
            return Ok(());
        };

        if inbox.send(command).await.is_err() {
            if let Some(session) = self.sessions.remove(&session_id) {
                session.shutdown().await;
            }
            send_error(&self.outbox, Some(&session_id), "session is not running").await?;
        }

        Ok(())
    }

    pub(crate) async fn shutdown_sessions(&mut self) {
        let sessions = std::mem::take(&mut self.sessions);
        join_all(sessions.into_values().map(SessionHandle::shutdown)).await;
    }
}
