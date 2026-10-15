//! Memory files over HTTP: read and edit `soul.md`, `user.md`, `memory.md`.
//!
//! Files stay the source of truth. `memory_file` is an mtime/hash cache so a
//! GET can report a version without re-reading, and a PUT can tell whether the
//! client's copy is still current.
//!
//! Concurrency uses `If-Match` with a **content hash**, not the mtime: some
//! filesystems only tick mtime once per second (or two), which would let two
//! edits in the same tick silently overwrite each other. A hash cannot.

use std::{
    fs,
    io,
    path::{Path, PathBuf},
};

use axum::{
    extract::{Path as UrlPath, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{clock, config::Config, db, memory, queue, server::AppState};

/// The only names that can be addressed. An allow-list, so no path handling is
/// involved and traversal is impossible by construction.
pub const PARTS: [&str; 3] = [memory::SOUL, memory::USER, memory::MEMORY];

/// Cap on a single part, to keep a runaway client from filling the disk.
const MAX_PART_BYTES: usize = 256 * 1024;

pub fn valid_part(part: &str) -> bool {
    PARTS.contains(&part)
}

pub fn part_path(cfg: &Config, part: &str) -> PathBuf {
    cfg.memory_dir().join(format!("{part}.md"))
}

/// The version a client must echo back in `If-Match`.
pub fn etag(content: &str) -> String {
    queue::sha256_16(content)
}

fn modified_seconds(path: &Path) -> i64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or_else(clock::now_unix)
}

/// Read a part and refresh its cache row.
pub fn read_part(cfg: &Config, db: &db::Db, part: &str) -> io::Result<(String, i64, String)> {
    let path = part_path(cfg, part);
    memory::seed_defaults(&cfg.state_dir())?;
    let content = fs::read_to_string(&path)?;
    let mtime = modified_seconds(&path);
    let hash = etag(&content);

    let conn = db.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    db::upsert_memory_file(&conn, &path.to_string_lossy(), mtime, &hash).map_err(io::Error::other)?;

    Ok((content, mtime, hash))
}

/// Write a part atomically and refresh its cache row.
pub fn write_part(cfg: &Config, db: &db::Db, part: &str, content: &str) -> io::Result<(i64, String)> {
    let dir = cfg.memory_dir();
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{part}.md"));
    let tmp = path.with_extension("md.tmp");
    fs::write(&tmp, content)?;
    fs::rename(&tmp, &path)?;

    let mtime = modified_seconds(&path);
    let hash = etag(content);
    let conn = db.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    db::upsert_memory_file(&conn, &path.to_string_lossy(), mtime, &hash).map_err(io::Error::other)?;

    Ok((mtime, hash))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/memory", get(list_memory))
        .route("/api/memory/{part}", get(get_memory).put(put_memory))
}

#[derive(Debug, Deserialize)]
pub struct WriteBody {
    pub content: String,
    /// The `etag` the client last read. Omit to force the write.
    #[serde(default)]
    pub if_match: Option<String>,
}

async fn list_memory(State(state): State<AppState>) -> Result<Json<Value>, crate::server::ApiError> {
    let cfg = state.cfg.clone();
    let handle = std::sync::Arc::clone(&state.db);

    let parts = tokio::task::spawn_blocking(move || {
        let mut out = Vec::new();
        for part in PARTS {
            match read_part(&cfg, &handle, part) {
                Ok((content, mtime, hash)) => out.push(json!({
                    "part": part,
                    "bytes": content.len(),
                    "mtime": mtime,
                    "etag": hash,
                })),
                Err(e) => return Err(e),
            }
        }
        Ok(out)
    })
    .await
    .map_err(crate::server::ApiError::internal)?
    .map_err(crate::server::ApiError::internal)?;

    Ok(Json(json!({ "parts": parts })))
}

async fn get_memory(
    State(state): State<AppState>,
    UrlPath(part): UrlPath<String>,
) -> Result<Json<Value>, crate::server::ApiError> {
    if !valid_part(&part) {
        return Err(crate::server::ApiError::new(
            StatusCode::BAD_REQUEST,
            "unknown memory part",
        ));
    }

    let cfg = state.cfg.clone();
    let handle = std::sync::Arc::clone(&state.db);
    let task_part = part.clone();
    let (content, mtime, hash) =
        tokio::task::spawn_blocking(move || read_part(&cfg, &handle, &task_part))
            .await
            .map_err(crate::server::ApiError::internal)?
            .map_err(crate::server::ApiError::internal)?;

    Ok(Json(json!({
        "part": part,
        "content": content,
        "mtime": mtime,
        "etag": hash,
    })))
}

async fn put_memory(
    State(state): State<AppState>,
    UrlPath(part): UrlPath<String>,
    Json(body): Json<WriteBody>,
) -> Result<Json<Value>, crate::server::ApiError> {
    if !valid_part(&part) {
        return Err(crate::server::ApiError::new(
            StatusCode::BAD_REQUEST,
            "unknown memory part",
        ));
    }
    if body.content.len() > MAX_PART_BYTES {
        return Err(crate::server::ApiError::new(
            StatusCode::BAD_REQUEST,
            "content is too large",
        ));
    }

    let cfg = state.cfg.clone();
    let handle = std::sync::Arc::clone(&state.db);
    let task_part = part.clone();

    // Read, check the precondition and write while holding the same blocking
    // task, so two concurrent PUTs cannot interleave.
    let result = tokio::task::spawn_blocking(move || -> Result<(i64, String), PutError> {
        let (_, _, current) = read_part(&cfg, &handle, &task_part).map_err(PutError::Io)?;

        if let Some(expected) = body.if_match.as_deref() {
            if expected != current {
                return Err(PutError::Precondition(current));
            }
        }

        write_part(&cfg, &handle, &task_part, &body.content).map_err(PutError::Io)
    })
    .await
    .map_err(crate::server::ApiError::internal)?;

    match result {
        Ok((mtime, hash)) => Ok(Json(json!({
            "ok": true,
            "part": part,
            "mtime": mtime,
            "etag": hash,
        }))),
        Err(PutError::Precondition(current)) => Err(crate::server::ApiError::new(
            StatusCode::PRECONDITION_FAILED,
            format!("your copy is stale; current etag is {current}"),
        )),
        Err(PutError::Io(e)) => Err(crate::server::ApiError::internal(e)),
    }
}

enum PutError {
    Precondition(String),
    Io(io::Error),
}

/// The soul's PUT. Identical rules to `/api/memory/{part}`, fixed to `soul`.
pub async fn put_soul(
    State(state): State<AppState>,
    Json(body): Json<WriteBody>,
) -> Result<Json<Value>, crate::server::ApiError> {
    put_memory(State(state), UrlPath(memory::SOUL.to_string()), Json(body)).await
}
