use crate::{domain::SessionKind, AnyError};
use serde_json::{json, Value};
use std::{
    io,
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{Child, ChildStdin, ChildStdout, Command},
};

const CODEX_MODEL: &str = "gpt-5.5";
const CODEX_MODEL_REASONING_EFFORT: &str = "medium";
const CODEX_SERVICE_TIER: &str = "fast";
const DISCIPLINED_CODING_SKILL_PATH: &str =
    "/Users/aakashsingh/.codex/skills/disciplined-coding/SKILL.md";

pub(crate) struct CodexAppServer {
    thread_id: String,
    session_kind: SessionKind,
    child: Child,
    stdin: ChildStdin,
    lines: Lines<BufReader<ChildStdout>>,
    next_request_id: u64,
}

impl CodexAppServer {
    pub(crate) async fn start(
        cwd: PathBuf,
        session_id: &str,
        session_name: &str,
        session_kind: SessionKind,
    ) -> Result<Self, AnyError> {
        let mcp_server_path = repo_root().join("mcp/minions-server.mjs");
        let mcp_server_path = mcp_server_path.to_string_lossy();

        let mut child = Command::new("codex")
            .arg("app-server")
            .arg("-c")
            .arg(format!("model={}", toml_string(CODEX_MODEL)))
            .arg("-c")
            .arg(format!(
                "model_reasoning_effort={}",
                toml_string(CODEX_MODEL_REASONING_EFFORT)
            ))
            .arg("-c")
            .arg(format!("service_tier={}", toml_string(CODEX_SERVICE_TIER)))
            .arg("-c")
            .arg("mcp_servers.minions.command=\"node\"")
            .arg("-c")
            .arg(format!(
                "mcp_servers.minions.args=[{}]",
                toml_string(mcp_server_path.as_ref())
            ))
            .arg("-c")
            .arg("mcp_servers.minions.enabled_tools=[\"perform_session_interaction\"]")
            .arg("-c")
            .arg("mcp_servers.minions.default_tools_approval_mode=\"approve\"")
            .arg("-c")
            .arg(format!(
                "mcp_servers.minions.env={{ MINIONS_BACKEND_URL = \"http://127.0.0.1:8080\", MINIONS_SESSION_ID = {} }}",
                toml_string(session_id)
            ))
            .current_dir(&cwd)
            .env("MINIONS_BACKEND_URL", "http://127.0.0.1:8080")
            .env("MINIONS_SESSION_ID", session_id)
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
            session_kind,
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
                        "name": "sessions_backend",
                        "title": "Sessions Backend",
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
                thread_start_params(cwd.as_path(), session_id, session_name, session_kind),
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
            turn_start_params(self.thread_id.as_str(), prompt, self.session_kind),
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

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")))
}

fn thread_start_params(
    cwd: &Path,
    session_id: &str,
    session_name: &str,
    session_kind: SessionKind,
) -> Value {
    let mut params = json!({
        "cwd": cwd.to_string_lossy(),
        "approvalPolicy": "untrusted",
        "sandbox": "read-only"
    });

    if let Some(developer_instructions) =
        developer_instructions_for_session_kind(session_id, session_name, session_kind)
    {
        params["developerInstructions"] = json!(developer_instructions);
    }

    if let Some(personality) = personality_for_session_kind(session_kind) {
        params["personality"] = json!(personality);
    }

    params
}

fn turn_start_params(thread_id: &str, prompt: String, session_kind: SessionKind) -> Value {
    let mut input = skill_inputs_for_session_kind(session_kind);
    input.push(json!({ "type": "text", "text": prompt, "text_elements": [] }));

    json!({
        "threadId": thread_id,
        "input": input,
        "approvalPolicy": "untrusted",
        "sandboxPolicy": { "type": "readOnly" }
    })
}

fn developer_instructions_for_session_kind(
    session_id: &str,
    session_name: &str,
    session_kind: SessionKind,
) -> Option<String> {
    match session_kind {
        SessionKind::Coder => Some(format!(
            "You are {session_name}, a helpful coder minion in the Minions app. Your controllable session_id is {session_id}. \
Follow the loaded coder and disciplined-coding skills. Inspect the workspace before editing, keep changes scoped, verify your work, and use the Minions interaction tool for supported world interactions."
        )),
        SessionKind::Researcher => Some(format!(
            "You are {session_name}, a helpful researcher minion in the Minions app. Your controllable session_id is {session_id}. \
Follow the loaded researcher skill. Investigate with evidence, compare options, synthesize findings, avoid code edits unless explicitly requested, and use the Minions interaction tool for supported world interactions."
        )),
        _ => None,
    }
}

fn personality_for_session_kind(session_kind: SessionKind) -> Option<&'static str> {
    match session_kind {
        SessionKind::Coder | SessionKind::Researcher => Some("pragmatic"),
        _ => None,
    }
}

fn skill_inputs_for_session_kind(session_kind: SessionKind) -> Vec<Value> {
    match session_kind {
        SessionKind::Coder => vec![
            skill_input(
                "disciplined-coding",
                PathBuf::from(DISCIPLINED_CODING_SKILL_PATH),
            ),
            skill_input(
                "minions-coder",
                repo_root().join("session-skills/coder/SKILL.md"),
            ),
        ],
        SessionKind::Researcher => vec![skill_input(
            "minions-researcher",
            repo_root().join("session-skills/researcher/SKILL.md"),
        )],
        _ => Vec::new(),
    }
}

fn skill_input(name: &str, path: PathBuf) -> Value {
    json!({
        "type": "skill",
        "name": name,
        "path": path.to_string_lossy()
    })
}

fn toml_string(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coder_turn_includes_disciplined_and_coder_skills() {
        let params = turn_start_params("thread-1", "fix the bug".to_owned(), SessionKind::Coder);
        let input = params["input"]
            .as_array()
            .expect("input should be an array");

        assert_eq!(input.len(), 3);
        assert_eq!(input[0]["type"], "skill");
        assert_eq!(input[0]["name"], "disciplined-coding");
        assert_eq!(input[0]["path"], DISCIPLINED_CODING_SKILL_PATH);
        assert_eq!(input[1]["type"], "skill");
        assert_eq!(input[1]["name"], "minions-coder");
        assert!(input[1]["path"]
            .as_str()
            .expect("coder skill path should be a string")
            .ends_with("/session-skills/coder/SKILL.md"));
        assert_eq!(
            input[2],
            json!({ "type": "text", "text": "fix the bug", "text_elements": [] })
        );
    }

    #[test]
    fn researcher_turn_includes_researcher_skill() {
        let params = turn_start_params(
            "thread-1",
            "research the options".to_owned(),
            SessionKind::Researcher,
        );
        let input = params["input"]
            .as_array()
            .expect("input should be an array");

        assert_eq!(input.len(), 2);
        assert_eq!(input[0]["type"], "skill");
        assert_eq!(input[0]["name"], "minions-researcher");
        assert!(input[0]["path"]
            .as_str()
            .expect("researcher skill path should be a string")
            .ends_with("/session-skills/researcher/SKILL.md"));
        assert_eq!(
            input[1],
            json!({ "type": "text", "text": "research the options", "text_elements": [] })
        );
    }

    #[test]
    fn other_kinds_do_not_include_session_skills() {
        let params = turn_start_params("thread-1", "hello".to_owned(), SessionKind::Reviewer);
        let input = params["input"]
            .as_array()
            .expect("input should be an array");

        assert_eq!(
            input,
            &vec![json!({ "type": "text", "text": "hello", "text_elements": [] })]
        );
    }

    #[test]
    fn coder_thread_start_sets_developer_instructions_and_personality() {
        let params = thread_start_params(
            Path::new("/tmp/workspace"),
            "bob",
            "Bob",
            SessionKind::Coder,
        );
        let developer_instructions = params["developerInstructions"]
            .as_str()
            .expect("developer instructions should be set");

        assert_eq!(params["personality"], "pragmatic");
        assert!(developer_instructions.contains("coder minion"));
        assert!(developer_instructions.contains("bob"));
        assert!(developer_instructions.contains("Bob"));
    }

    #[test]
    fn researcher_thread_start_sets_developer_instructions_and_personality() {
        let params = thread_start_params(
            Path::new("/tmp/workspace"),
            "kevin",
            "Kevin",
            SessionKind::Researcher,
        );
        let developer_instructions = params["developerInstructions"]
            .as_str()
            .expect("developer instructions should be set");

        assert_eq!(params["personality"], "pragmatic");
        assert!(developer_instructions.contains("researcher minion"));
        assert!(developer_instructions.contains("kevin"));
        assert!(developer_instructions.contains("Kevin"));
    }
}
