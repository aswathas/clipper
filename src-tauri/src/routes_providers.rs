//! Providers over HTTP (S7).
//!
//! Read-only plus a **test** action. Saving provider settings needs a handle on
//! the config file the app actually booted from, and the drainer would have to
//! reload its cached prompt — both deferred so this slice cannot half-apply a
//! change to a running extraction loop.

use std::{
    io::{Read, Write},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use axum::{
    extract::{Path as UrlPath, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};

use crate::server::AppState;

/// A test run is a smoke check, not a real extraction.
const TEST_TIMEOUT: Duration = Duration::from_secs(15);
const TEST_PROMPT: &str = "Reply with exactly: pong";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/providers", get(list_providers))
        .route("/api/providers/{name}/test", post(test_provider))
}

/// The configured providers, with nothing secret in the answer.
fn provider_view(state: &AppState) -> Value {
    let claude = &state.cfg.provider.claude;
    let image = &state.cfg.provider.image;

    json!({
        "claude": {
            "command": claude.command,
            "args": claude.args,
            "stdin": claude.stdin,
        },
        "image": {
            "backend": image.backend,
            "prompt_hint": image.prompt_hint,
            "command": image.command,
            "args": image.args,
        },
    })
}

async fn list_providers(State(state): State<AppState>) -> Json<Value> {
    Json(json!({ "providers": provider_view(&state), "editable": false }))
}

/// Run the named provider once with a trivial prompt and report what happened.
///
/// This is the Settings "Test" button, and it is the fastest way for a user to
/// see that their CLI is missing, slow, or writing noise to stdout.
async fn test_provider(
    State(state): State<AppState>,
    UrlPath(name): UrlPath<String>,
) -> Result<Json<Value>, crate::server::ApiError> {
    let cfg = state.cfg.clone();

    let (command, args) = match name.as_str() {
        "claude" => (
            cfg.provider.claude.command.clone(),
            cfg.provider.claude.args.clone(),
        ),
        "image" => match cfg.provider.image.command.clone() {
            Some(command) => (
                command,
                cfg.provider.image.args.clone().unwrap_or_default(),
            ),
            None => (
                cfg.provider.claude.command.clone(),
                cfg.provider.claude.args.clone(),
            ),
        },
        _ => {
            return Err(crate::server::ApiError::new(
                StatusCode::NOT_FOUND,
                "unknown provider",
            ))
        }
    };

    let run = tokio::task::spawn_blocking(move || run_once(&command, &args))
        .await
        .map_err(crate::server::ApiError::internal)?;

    let ok = run.exit_code == Some(0) && !run.timed_out;
    Ok(Json(json!({
        "ok": ok,
        "provider": name,
        "exit_code": run.exit_code,
        "timed_out": run.timed_out,
        "stdout_first_line": run.stdout.lines().next().unwrap_or("").trim(),
        "stderr_first_line": run.stderr.lines().next().unwrap_or("").trim(),
    })))
}

struct Run {
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    timed_out: bool,
}

/// Spawn `command`, write the prompt to its stdin, and wait with a cap.
fn run_once(command: &str, args: &[String]) -> Run {
    let mut child = match Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            return Run {
                exit_code: None,
                stdout: String::new(),
                stderr: format!("could not start {command}: {e}"),
                timed_out: false,
            }
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        let payload = TEST_PROMPT.as_bytes().to_vec();
        thread::spawn(move || {
            let _ = stdin.write_all(&payload);
            let _ = stdin.flush();
        });
    }

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stdout_reader = thread::spawn(move || drain(stdout));
    let stderr_reader = thread::spawn(move || drain(stderr));

    let deadline = Instant::now() + TEST_TIMEOUT;
    let mut timed_out = false;
    let exit = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    timed_out = true;
                    break child.wait().ok();
                }
                thread::sleep(Duration::from_millis(50));
            }
            Err(_) => break None,
        }
    };

    Run {
        exit_code: exit.and_then(|status| status.code()),
        stdout: stdout_reader.join().unwrap_or_default(),
        stderr: stderr_reader.join().unwrap_or_default(),
        timed_out,
    }
}

fn drain(pipe: Option<impl Read>) -> String {
    let mut buffer = Vec::new();
    if let Some(mut pipe) = pipe {
        let _ = pipe.read_to_end(&mut buffer);
    }
    String::from_utf8_lossy(&buffer).into_owned()
}
