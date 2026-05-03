use crate::{
    protocol::{send_error, send_event, ServerEvent, SessionCommand},
    AnyError,
};
use serde_json::{json, Value};
use std::{io, path::PathBuf, process::Stdio};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{Child, ChildStdin, ChildStdout, Command},
    sync::mpsc,
};

struct CodexThread {
    session_id: String,
    thread_id: String,
    child: Child,
    stdin: ChildStdin,
    lines: Lines<BufReader<ChildStdout>>,
    next_request_id: u64,
    pending_approval_id: Option<Value>,
}

impl CodexThread {
    async fn start(
        session_id: String,
        cwd: PathBuf,
        outbox: &mpsc::Sender<ServerEvent>,
    ) -> Result<Self, AnyError> {
        let mut child = Command::new("codex")
            .arg("app-server")
            .current_dir(&cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("codex stdin unavailable"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("codex stdout unavailable"))?;

        let mut codex = Self {
            session_id,
            thread_id: String::new(),
            child,
            stdin,
            lines: BufReader::new(stdout).lines(),
            next_request_id: 0,
            pending_approval_id: None,
        };

        codex
            .request(
                "initialize",
                json!({
                    "clientInfo": {
                        "name": "minions_backend",
                        "title": "Minions Backend",
                        "version": "0.1.0"
                    },
                    "capabilities": { "experimentalApi": true }
                }),
                outbox,
            )
            .await?;
        codex.notify("initialized", json!({})).await?;

        let response = codex
            .request(
                "thread/start",
                json!({
                    "cwd": cwd.to_string_lossy(),
                    "approvalPolicy": "untrusted",
                    "sandbox": "read-only"
                }),
                outbox,
            )
            .await?;

        codex.thread_id = response["thread"]["id"]
            .as_str()
            .ok_or_else(|| io::Error::other("thread/start missing thread id"))?
            .to_owned();

        Ok(codex)
    }

    async fn start_turn(&mut self, prompt: String) -> Result<u64, AnyError> {
        self.send_request(
            "turn/start",
            json!({
                "threadId": self.thread_id,
                "input": [{ "type": "text", "text": prompt }],
                "approvalPolicy": "untrusted",
                "sandboxPolicy": { "type": "readOnly" }
            }),
        )
        .await
    }

    async fn request(
        &mut self,
        method: &str,
        params: Value,
        outbox: &mpsc::Sender<ServerEvent>,
    ) -> Result<Value, AnyError> {
        let request_id = self.send_request(method, params).await?;

        loop {
            let message = self.read_message().await?;
            if self.handle_app_server_request(&message, outbox).await? {
                continue;
            }

            if message["id"].as_u64() == Some(request_id) {
                return match message.get("error") {
                    Some(error) => {
                        Err(io::Error::other(format!("codex request failed: {error}")).into())
                    }
                    None => Ok(message["result"].clone()),
                };
            }
        }
    }

    async fn send_request(&mut self, method: &str, params: Value) -> Result<u64, AnyError> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        self.write(json!({
            "id": request_id,
            "method": method,
            "params": params
        }))
        .await?;
        Ok(request_id)
    }

    async fn notify(&mut self, method: &str, params: Value) -> Result<(), AnyError> {
        self.write(json!({ "method": method, "params": params }))
            .await
    }

    async fn read_message(&mut self) -> Result<Value, AnyError> {
        loop {
            let line = self
                .lines
                .next_line()
                .await?
                .ok_or_else(|| io::Error::other("codex app-server stdout ended"))?;

            if !line.trim().is_empty() {
                return Ok(serde_json::from_str(&line)?);
            }
        }
    }

    async fn write(&mut self, message: Value) -> Result<(), AnyError> {
        let serialized = serde_json::to_string(&message)?;
        self.stdin.write_all(serialized.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        Ok(())
    }

    async fn handle_app_server_request(
        &mut self,
        message: &Value,
        outbox: &mpsc::Sender<ServerEvent>,
    ) -> Result<bool, AnyError> {
        let Some(id) = message.get("id") else {
            return Ok(false);
        };
        let Some(method) = message["method"].as_str() else {
            return Ok(false);
        };

        match method {
            "item/commandExecution/requestApproval" | "item/fileChange/requestApproval" => {
                self.pending_approval_id = Some(id.clone());
                send_event(
                    outbox,
                    ServerEvent::ApprovalRequest {
                        session_id: self.session_id.clone(),
                        request_id: id.clone(),
                        method: method.to_owned(),
                        params: message["params"].clone(),
                        question: "Choose an approval decision.".to_owned(),
                        answers: vec![
                            "accept".to_owned(),
                            "acceptForSession".to_owned(),
                            "decline".to_owned(),
                            "cancel".to_owned(),
                        ],
                    },
                )
                .await?;
            }
            _ => {
                self.write(json!({
                    "id": id,
                    "error": {
                        "code": -32601,
                        "message": "Method not found"
                    }
                }))
                .await?;
            }
        }

        Ok(true)
    }

    async fn respond_to_approval(&mut self, answer: String) -> Result<(), AnyError> {
        let Some(id) = self.pending_approval_id.take() else {
            return Ok(());
        };

        let decision = approval_decision_for_answer(&answer);

        self.write(json!({
            "id": id,
            "result": { "decision": decision }
        }))
        .await
    }

    async fn shutdown(mut self) {
        let _ = self.child.start_kill();
        let _ = self.child.wait().await;
    }
}

fn approval_decision_for_answer(answer: &str) -> &'static str {
    match answer.trim() {
        "accept" => "accept",
        "acceptForSession" => "acceptForSession",
        "cancel" => "cancel",
        _ => "decline",
    }
}

#[cfg(test)]
mod tests {
    use super::approval_decision_for_answer;

    #[test]
    fn approval_decision_accepts_only_codex_decisions() {
        assert_eq!(approval_decision_for_answer("accept"), "accept");
        assert_eq!(
            approval_decision_for_answer("acceptForSession"),
            "acceptForSession"
        );
        assert_eq!(approval_decision_for_answer("decline"), "decline");
        assert_eq!(approval_decision_for_answer("cancel"), "cancel");
    }

    #[test]
    fn approval_decision_maps_unknown_answers_to_decline() {
        assert_eq!(approval_decision_for_answer("other"), "decline");
        assert_eq!(approval_decision_for_answer(""), "decline");
    }
}

pub(crate) async fn run_session_task(
    session_id: String,
    cwd: PathBuf,
    mut inbox: mpsc::Receiver<SessionCommand>,
    outbox: mpsc::Sender<ServerEvent>,
) -> Result<(), AnyError> {
    let mut codex = CodexThread::start(session_id.clone(), cwd, &outbox).await?;
    send_event(
        &outbox,
        ServerEvent::SessionReady {
            session_id: session_id.clone(),
        },
    )
    .await?;

    let mut pending_turn_start_id = None::<u64>;
    let mut active_turn_id = None::<String>;

    loop {
        tokio::select! {
            command = inbox.recv() => {
                let Some(command) = command else {
                    break;
                };

                match command {
                    SessionCommand::StartTurn { prompt } => {
                        if pending_turn_start_id.is_some() || active_turn_id.is_some() {
                            send_error(
                                &outbox,
                                Some(&session_id),
                                "wait for turn.completed before starting another turn",
                            )
                            .await?;
                            continue;
                        }

                        pending_turn_start_id = Some(codex.start_turn(prompt).await?);
                    }
                    SessionCommand::RespondToApproval { answer } => {
                        codex.respond_to_approval(answer).await?;
                    }
                }
            }
            message = codex.read_message() => {
                let message = message?;

                if codex.handle_app_server_request(&message, &outbox).await? {
                    continue;
                }

                if let Some(request_id) = pending_turn_start_id {
                    if message["id"].as_u64() == Some(request_id) {
                        pending_turn_start_id = None;

                        if let Some(error) = message.get("error") {
                            return Err(io::Error::other(format!("codex request failed: {error}")).into());
                        }

                        let turn_id = message["result"]["turn"]["id"]
                            .as_str()
                            .ok_or_else(|| io::Error::other("turn/start missing turn id"))?
                            .to_owned();
                        active_turn_id = Some(turn_id.clone());

                        send_event(
                            &outbox,
                            ServerEvent::TurnStarted {
                                session_id: session_id.clone(),
                                turn_id,
                            },
                        )
                        .await?;
                        continue;
                    }
                }

                if message["method"] == "item/agentMessage/delta" {
                    if let Some(delta) = message["params"]["delta"].as_str() {
                        send_event(
                            &outbox,
                            ServerEvent::AssistantDelta {
                                session_id: session_id.clone(),
                                text: delta.to_owned(),
                            },
                        )
                        .await?;
                    }
                }

                if let Some(turn_id) = active_turn_id.clone() {
                    if message["method"] == "turn/completed"
                        && message["params"]["turn"]["id"].as_str() == Some(turn_id.as_str())
                    {
                        send_event(
                            &outbox,
                            ServerEvent::TurnCompleted {
                                session_id: session_id.clone(),
                                turn_id,
                            },
                        )
                        .await?;
                        active_turn_id = None;
                    }
                }
            }
        }
    }

    codex.shutdown().await;
    Ok(())
}
