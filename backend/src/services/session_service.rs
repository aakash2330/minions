use crate::{
    domain::{Direction, PointWithFacing, Session, SessionKind, SessionStatus},
    infrastructure::db::{DbError, SessionRepository},
};
use std::{
    io,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub(crate) struct SessionService {
    repository: SessionRepository,
}

impl SessionService {
    pub(crate) fn new() -> Result<Self, DbError> {
        Ok(Self {
            repository: SessionRepository::new()?,
        })
    }

    pub(crate) async fn load_sessions(&self) -> Result<Vec<Session>, DbError> {
        self.repository.load_sessions().await
    }

    pub(crate) async fn load_sessions_by_workspace_id(
        &self,
        workspace_id: &str,
    ) -> Result<Vec<Session>, DbError> {
        self.repository
            .load_sessions_by_workspace_id(workspace_id)
            .await
    }

    pub(crate) async fn load_session(&self, session_id: &str) -> Result<Option<Session>, DbError> {
        self.repository.session_by_id(session_id).await
    }

    pub(crate) async fn create_session(
        &self,
        input: CreateSessionInput,
    ) -> Result<String, DbError> {
        let workspace_id = clean_text(input.workspace_id)
            .ok_or_else(|| io::Error::other("workspace_id is required"))?;
        let session_id = input
            .session_id
            .and_then(clean_text)
            .unwrap_or_else(new_session_id);
        let name = input
            .name
            .and_then(clean_text)
            .unwrap_or_else(|| session_id.clone());
        let kind = input
            .kind
            .and_then(clean_text)
            .unwrap_or_else(|| SessionKind::Coder.as_str().to_owned())
            .parse::<SessionKind>()
            .map_err(io::Error::other)?;
        let spawn = input
            .spawn
            .map(PointWithFacing::try_from)
            .transpose()?
            .unwrap_or(PointWithFacing {
                x: 0,
                y: 0,
                facing: Direction::Down,
            });
        let current = input
            .current
            .map(PointWithFacing::try_from)
            .transpose()?
            .unwrap_or(PointWithFacing {
                x: spawn.x,
                y: spawn.y,
                facing: spawn.facing,
            });

        self.repository
            .create_session(
                session_id.as_str(),
                workspace_id.as_str(),
                name.as_str(),
                kind,
                spawn,
                current,
            )
            .await
    }

    pub(crate) async fn attach_codex_thread(
        &self,
        session_id: &str,
        codex_thread_id: &str,
    ) -> Result<(), DbError> {
        self.repository
            .update_session_codex_thread_id(session_id, codex_thread_id)
            .await
    }

    pub(crate) async fn record_user_message(
        &self,
        session_id: &str,
        text: &str,
    ) -> Result<(), DbError> {
        self.repository.record_user_message(session_id, text).await
    }

    pub(crate) async fn start_assistant_message(
        &self,
        session_id: &str,
    ) -> Result<String, DbError> {
        self.repository.start_assistant_message(session_id).await
    }

    pub(crate) async fn append_assistant_delta(
        &self,
        session_id: &str,
        delta: &str,
    ) -> Result<String, DbError> {
        self.repository
            .append_assistant_delta(session_id, delta)
            .await
    }

    pub(crate) async fn complete_assistant_message(&self, session_id: &str) -> Result<(), DbError> {
        self.repository.complete_assistant_message(session_id).await
    }

    pub(crate) async fn complete_session(&self, session_id: &str) -> Result<(), DbError> {
        self.repository
            .update_session_status(session_id, SessionStatus::Idle)
            .await
    }
}

pub(crate) struct CreateSessionInput {
    pub(crate) session_id: Option<String>,
    pub(crate) workspace_id: String,
    pub(crate) name: Option<String>,
    pub(crate) kind: Option<String>,
    pub(crate) spawn: Option<CreateSessionPointInput>,
    pub(crate) current: Option<CreateSessionPointInput>,
}

pub(crate) struct CreateSessionPointInput {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) facing: Option<Direction>,
}

impl TryFrom<CreateSessionPointInput> for PointWithFacing {
    type Error = io::Error;

    fn try_from(point: CreateSessionPointInput) -> Result<Self, Self::Error> {
        Ok(Self {
            x: point.x,
            y: point.y,
            facing: point.facing.unwrap_or(Direction::Down),
        })
    }
}

fn clean_text(value: String) -> Option<String> {
    let value = value.trim().to_owned();
    (!value.is_empty()).then_some(value)
}

fn new_session_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let sequence = NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed);

    format!("session-{timestamp:020}-{sequence:020}")
}
