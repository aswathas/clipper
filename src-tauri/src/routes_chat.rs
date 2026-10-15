//! Ask (S8): streaming chat about one saved clip.
//!
//! `claude -p` is spawned per question with the full prompt template — soul,
//! user, memory, skill context, the clip itself, then the question. The child's
//! stdout is read line by line on a blocking thread and pushed through a
//! **bounded** channel that becomes the SSE stream: the bound *is* the
//! backpressure. If the client stalls, the reader blocks rather than buffering
//! an unbounded amount of model output in memory.
//!
//! Providers differ — some print plain text, some emit NDJSON (one JSON object
//! per line). Both are normalised to `event: chunk` frames.

use std::{
    fs,
    sync::{Mutex, OnceLock},
    time::Duration,
};

use axum::{
    body::Body,
    extract::{Path as UrlPath, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use futures_util::stream;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{clock, db, memory, server::AppState};

/// Frames that may queue before the provider's reader blocks.
const FRAME_BACKLOG: usize = 64;
/// Turns kept in the session ring.
const HISTORY_LIMIT: usize = 200;

/// One turn of a chat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatTurn {
    pub item_id: String,
    pub role: String,
    pub content: String,
    pub ts: i64,
}

/// Session-scoped chat history, as a bounded ring.
pub fn history() -> &'static Mutex<Vec<ChatTurn>> {
    static HISTORY: OnceLock<Mutex<Vec<ChatTurn>>> = OnceLock::new();
    HISTORY.get_or_init(|| Mutex::new(Vec::new()))
}

fn remember(turn: ChatTurn) {
    let mut log = history().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    log.push(turn);
    let len = log.len();
    if len > HISTORY_LIMIT {
        log.drain(..len - HISTORY_LIMIT);
    }
}

/// The prompt sent to the provider: persona first, then the clip, then the ask.
pub fn build_chat_prompt(state: &AppState, item: &db::Item, question: &str) -> String {
    let loaded = memory::load_now(&state.cfg.memory_dir()).unwrap_or_else(|_| memory::Memory {
        soul: String::new(),
        user: String::new(),
        memory: String::new(),
    });

    let skills = crate::server::with_db(state, db::list_skills).unwrap_or_default();
    let skill_lines: Vec<String> = skills
        .iter()
        .map(|skill| match &skill.description {
            Some(description) => format!("- {}: {description}", skill.name),
            None => format!("- {}", skill.name),
        })
        .collect();

    let system = format!(
        "Answer questions about one saved clip. Be concrete, quote the clip when it helps, \
         and say so plainly when the clip does not answer the question.\n\n\
         ## Skills available\n{}",
        if skill_lines.is_empty() {
            "(none)".to_string()
        } else {
            skill_lines.join("\n")
        }
    );

    let body = item
        .vault_path
        .as_deref()
        .and_then(|path| fs::read_to_string(path).ok())
        .unwrap_or_default();

    let clip = format!(
        "## The clip\n- title: {}\n- type: {}\n- url: {}\n- tags: {}\n- summary: {}\n\n\
         ## Note body\n{}",
        item.title.clone().unwrap_or_default(),
        item.kind.clone().unwrap_or_default(),
        item.url,
        item.tags.join(", "),
        item.summary.clone().unwrap_or_default(),
        if body.is_empty() { "(not on disk)" } else { &body },
    );

    format!(
        "{}\n\n{clip}\n\n## Question\n{question}\n",
        loaded.build_claude_prompt(&system)
    )
}

/// One provider stdout line as an SSE chunk frame.
fn frame_from_line(line: &str) -> String {
    let text = match serde_json::from_str::<Value>(line) {
        Ok(value) => value
            .get("text")
            .or_else(|| value.get("delta").and_then(|delta| delta.get("text")))
            .or_else(|| value.get("result"))
            .and_then(|found| found.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| line.to_string()),
        Err(_) => line.to_string(),
    };
    format!("event: chunk\ndata: {}\n\n", json!({ "text": text }))
}

fn frame_done(exit_code: i32) -> String {
    format!("event: done\ndata: {}\n\n", json!({ "exit_code": exit_code }))
}

fn frame_error(message: &str) -> String {
    format!("event: error\ndata: {}\n\n", json!({ "message": message }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/items/{id}/chat", post(chat))
        .route("/api/chat/history", get(chat_history))
}

#[derive(Debug, Deserialize)]
struct ChatBody {
    message: String,
}

async fn chat(
    State(state): State<AppState>,
    UrlPath(id): UrlPath<String>,
    Json(body): Json<ChatBody>,
) -> Result<Response, crate::server::ApiError> {
    let question = body.message.trim().to_string();
    if question.is_empty() {
        return Err(crate::server::ApiError::new(
            StatusCode::BAD_REQUEST,
            "message is required",
        ));
    }

    let item = crate::server::with_db(&state, |conn| db::get(conn, &id))?
        .ok_or_else(|| crate::server::ApiError::new(StatusCode::NOT_FOUND, "no such item"))?;

    let prompt = build_chat_prompt(&state, &item, &question);
    remember(ChatTurn {
        item_id: id.clone(),
        role: "user".to_string(),
        content: question,
        ts: clock::now_unix(),
    });

    let provider = state.cfg.provider.claude.clone();
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(FRAME_BACKLOG);

    // The child is driven on a blocking thread; the bounded channel paces it.
    std::thread::spawn(move || {
        use std::{
            io::{BufRead, BufReader, Write},
            process::{Command, Stdio},
        };

        let mut child = match Command::new(&provider.command)
            .args(&provider.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                let _ = tx.blocking_send(frame_error(&format!(
                    "could not start {}: {e}",
                    provider.command
                )));
                let _ = tx.blocking_send(frame_done(-1));
                return;
            }
        };

        if let Some(mut stdin) = child.stdin.take() {
            // Feed stdin off-thread so a large prompt cannot deadlock the reader.
            std::thread::spawn(move || {
                let _ = stdin.write_all(prompt.as_bytes());
                let _ = stdin.flush();
            });
        }

        let mut reply = String::new();
        if let Some(stdout) = child.stdout.take() {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                let line = line.trim_end().to_string();
                if line.is_empty() {
                    continue;
                }
                reply.push_str(&line);
                reply.push('\n');
                if tx.blocking_send(frame_from_line(&line)).is_err() {
                    // The client hung up: stop reading and drop the child.
                    break;
                }
            }
        }

        let code = child
            .wait()
            .ok()
            .and_then(|status| status.code())
            .unwrap_or(-1);
        let _ = tx.blocking_send(frame_done(code));

        remember(ChatTurn {
            item_id: id,
            role: "assistant".to_string(),
            content: reply.trim().to_string(),
            ts: clock::now_unix(),
        });

        // Give the last frame a moment to flush before the thread exits.
        std::thread::sleep(Duration::from_millis(20));
    });

    let frames = stream::unfold(rx, |mut rx| async move {
        rx.recv()
            .await
            .map(|frame| (Ok::<_, std::convert::Infallible>(frame), rx))
    });

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/event-stream")
        .header("cache-control", "no-cache")
        .header("x-accel-buffering", "no")
        .body(Body::from_stream(frames))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()))
}

#[derive(Debug, Deserialize)]
struct HistoryQuery {
    item_id: Option<String>,
    limit: Option<usize>,
}

async fn chat_history(Query(params): Query<HistoryQuery>) -> Json<Value> {
    let limit = params.limit.unwrap_or(50).clamp(1, HISTORY_LIMIT);
    let log = history().lock().unwrap_or_else(|poisoned| poisoned.into_inner());

    let matching: Vec<ChatTurn> = log
        .iter()
        .filter(|turn| {
            params
                .item_id
                .as_deref()
                .map(|id| id == turn.item_id)
                .unwrap_or(true)
        })
        .cloned()
        .collect();
    let start = matching.len().saturating_sub(limit);

    Json(json!({
        "turns": &matching[start..],
        "total": matching.len(),
    }))
}
