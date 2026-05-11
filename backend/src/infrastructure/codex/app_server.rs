use crate::AnyError;
use serde_json::{json, Value};
use std::{io, path::PathBuf, process::Stdio};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{Child, ChildStdin, ChildStdout, Command},
};

pub(crate) struct CodexAppServer {
    thread_id: String,
    child: Child,
    stdin: ChildStdin,
    lines: Lines<BufReader<ChildStdout>>,
    next_request_id: u64,
}

impl CodexAppServer {
    pub(crate) async fn start(cwd: PathBuf) -> Result<Self, AnyError> {
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
            thread_id: String::new(),
            child,
            stdin,
            lines: BufReader::new(stdout).lines(),
            next_request_id: 0,
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
            )
            .await?;

        codex.thread_id = response["thread"]["id"]
            .as_str()
            .ok_or_else(|| io::Error::other("thread/start missing thread id"))?
            .to_owned();

        Ok(codex)
    }

    pub(crate) fn thread_id(&self) -> &str {
        self.thread_id.as_str()
    }

    pub(crate) async fn start_turn(&mut self, prompt: String) -> Result<u64, AnyError> {
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

    pub(crate) async fn read_message(&mut self) -> Result<Value, AnyError> {
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

    pub(crate) async fn respond_to_request(
        &mut self,
        id: Value,
        result: Value,
    ) -> Result<(), AnyError> {
        self.write(json!({
            "id": id,
            "result": result
        }))
        .await
    }

    pub(crate) async fn respond_method_not_found(&mut self, id: Value) -> Result<(), AnyError> {
        self.write(json!({
            "id": id,
            "error": {
                "code": -32601,
                "message": "Method not found"
            }
        }))
        .await
    }

    pub(crate) async fn shutdown(mut self) {
        let _ = self.child.start_kill();
        let _ = self.child.wait().await;
    }

    async fn request(&mut self, method: &str, params: Value) -> Result<Value, AnyError> {
        let request_id = self.send_request(method, params).await?;

        loop {
            let message = self.read_message().await?;

            if let (Some(id), Some(_method)) = (message.get("id").cloned(), message.get("method")) {
                self.respond_method_not_found(id).await?;
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

    async fn write(&mut self, message: Value) -> Result<(), AnyError> {
        let serialized = serde_json::to_string(&message)?;
        self.stdin.write_all(serialized.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        Ok(())
    }
}
