use crate::{
    protocol::{send_error, send_event, ClientMessage, ServerEvent, SessionCommand},
    session::run_session_task,
    AnyError, CHANNEL_BUFFER,
};
use futures_util::future::join_all;
use std::{collections::HashMap, path::PathBuf, time::Duration};
use tokio::{sync::mpsc, task::JoinHandle};

const APP_SERVER_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(2);

pub(crate) struct SessionHandle {
    pub(crate) inbox: Option<mpsc::Sender<SessionCommand>>,
    pub(crate) task: Option<JoinHandle<()>>,
}

impl SessionHandle {
    fn new(inbox: mpsc::Sender<SessionCommand>, task: JoinHandle<()>) -> Self {
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

pub(crate) struct ConnectionRouter {
    outbox: mpsc::Sender<ServerEvent>,
    pub(crate) sessions: HashMap<String, SessionHandle>,
}

impl ConnectionRouter {
    pub(crate) fn new(outbox: mpsc::Sender<ServerEvent>) -> Self {
        Self {
            outbox,
            sessions: HashMap::new(),
        }
    }

    pub(crate) async fn run(
        &mut self,
        mut inbox: mpsc::Receiver<ClientMessage>,
    ) -> Result<(), AnyError> {
        while let Some(message) = inbox.recv().await {
            let result = match message {
                ClientMessage::SessionStart { session_id, cwd } => {
                    self.start_session_from_client(session_id, cwd).await
                }
                ClientMessage::TurnStart { session_id, prompt } => {
                    self.start_turn(session_id, prompt).await
                }
                ClientMessage::ApprovalRespond { session_id, answer } => {
                    self.send_to_session(session_id, SessionCommand::RespondToApproval { answer })
                        .await
                }
            };

            if let Err(error) = result {
                self.shutdown_sessions().await;
                return Err(error);
            }
        }

        self.shutdown_sessions().await;
        Ok(())
    }

    pub(crate) async fn start_session_from_client(
        &mut self,
        session_id: Option<String>,
        cwd: Option<PathBuf>,
    ) -> Result<(), AnyError> {
        let session_id = match normalize_session_id(session_id) {
            Ok(session_id) => session_id,
            Err(message) => {
                send_error(&self.outbox, None, message).await?;
                return Ok(());
            }
        };
        let cwd = match cwd {
            Some(cwd) if cwd.is_absolute() => cwd,
            Some(_) => {
                send_error(&self.outbox, Some(&session_id), "cwd must be absolute").await?;
                return Ok(());
            }
            None => {
                send_error(&self.outbox, Some(&session_id), "cwd is required").await?;
                return Ok(());
            }
        };

        self.start_session(session_id, cwd).await
    }

    pub(crate) async fn start_turn(
        &mut self,
        session_id: Option<String>,
        prompt: String,
    ) -> Result<(), AnyError> {
        let Some(session_id) = session_id else {
            send_error(&self.outbox, None, "session_id is required").await?;
            return Ok(());
        };

        self.send_to_session(session_id, SessionCommand::StartTurn { prompt })
            .await
    }

    pub(crate) async fn start_session(
        &mut self,
        session_id: String,
        cwd: PathBuf,
    ) -> Result<(), AnyError> {
        if let Some(existing_session) = self.sessions.get(&session_id) {
            if !existing_session.is_closed() {
                send_event(&self.outbox, ServerEvent::SessionReady { session_id }).await?;
                return Ok(());
            }

            if let Some(session) = self.sessions.remove(&session_id) {
                session.shutdown().await;
            }
        }

        let (session_tx, session_rx) = mpsc::channel(CHANNEL_BUFFER);
        let task_session_id = session_id.clone();
        let task_cwd = cwd;
        let task_outbox = self.outbox.clone();

        let task = tokio::spawn(async move {
            if let Err(error) = run_session_task(
                task_session_id.clone(),
                task_cwd,
                session_rx,
                task_outbox.clone(),
            )
            .await
            {
                let _ = send_event(
                    &task_outbox,
                    ServerEvent::Error {
                        session_id: Some(task_session_id),
                        message: error.to_string(),
                    },
                )
                .await;
            }
        });

        self.sessions
            .insert(session_id, SessionHandle::new(session_tx, task));
        Ok(())
    }

    pub(crate) async fn send_to_session(
        &mut self,
        session_id: String,
        command: SessionCommand,
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

fn normalize_session_id(session_id: Option<String>) -> Result<String, &'static str> {
    let Some(session_id) = session_id else {
        return Err("session_id is required");
    };

    let session_id = session_id.trim().to_owned();
    if session_id.is_empty() {
        Err("session_id must not be empty")
    } else {
        Ok(session_id)
    }
}
