//! The local REST API, and the dashboard it serves.
//!
//! Bound to `[server] bind`/`port` (default `127.0.0.1:7137`). The dashboard in
//! `src/ui/` is embedded in the binary and served from `/`, so its relative
//! `/api/...` fetches stay same-origin.

use std::sync::{Arc, Mutex, RwLock};
use std::process::Command;

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{clock, config::Config, db, pairing, queue};

mod ui_assets {
    include!(concat!(env!("OUT_DIR"), "/ui_assets.rs"));
}

/// Look up a file embedded from `src/ui` at build time.
fn ui_asset(route: &str) -> Option<(&'static str, &'static [u8])> {
    let route = if route == "/" { "/index.html" } else { route };
    ui_assets::UI_ASSETS
        .iter()
        .find(|(path, _, _)| *path == route)
        .map(|(_, mime, bytes)| (*mime, *bytes))
}

/// Only these origins may call the API from a browser. Everything else is
/// default-deny: no CORS headers are emitted, so the browser blocks it.
/// The two `tauri` entries are the app's own webview, which must be able to
/// reach the API from the bundled window.
const ALLOWED_ORIGINS: [&str; 4] = [
    "http://127.0.0.1:7137",
    "http://localhost:7137",
    "http://tauri.localhost",
    "tauri://localhost",
];

#[derive(Clone)]
pub struct AppState {
    pub cfg: Config,
    pub db: db::Db,
    pub queue_lock: Arc<Mutex<()>>,
    /// Rotating session token that the phone must present to `/save`.
    pub token: Arc<RwLock<String>>,
    /// LAN address to advertise in the QR code.
    pub lan_ip: String,
}

/// Build the router. Split out so the self-check can serve it on port 0.
pub fn router(state: AppState) -> Router {
    Router::new()
        // api
        .route("/healthz", get(healthz))
        .route("/api/items", get(list_items))
        .route("/api/items/{id}", get(get_item))
        .route("/api/items/{id}/status", post(set_status))
        .route("/api/items/{id}/tags", post(set_tags))
        .route("/api/clips", post(create_clip))
        .route("/api/import", post(import_batch))
        .route("/api/muse/state", get(get_muse_state).post(set_muse_state))
        .route("/api/muse/activity", get(get_muse_activity))
        .route("/api/muse/heatmap", get(get_muse_heatmap))
        .route("/api/muse/digest", get(get_muse_digest))
        .route("/api/muse/config", get(get_muse_config).post(set_muse_config))
        .route("/api/concepts", get(get_concepts))
        .route("/api/items/{id}/related", get(get_related))
        .route("/api/today", get(today))
        .route("/api/stats", get(stats))
        // phone pairing
        .route("/pair", get(pair_page))
        .route("/pair.png", get(pair_qr))
        .route("/qr", get(pair_qr_page))
        .route("/save", post(save_clip))
        // slice routers
        .merge(crate::routes_skills::router())
        .merge(crate::routes_memory::router())
        .merge(crate::routes_soul::router())
        .merge(crate::routes_self::router())
        .merge(crate::routes_providers::router())
        .merge(crate::routes_pairing::router())
        .merge(crate::routes_chat::router())
        // everything else is a file embedded from src/ui
        .fallback(ui_static)
        .layer(middleware::from_fn(cors))
        .with_state(state)
}

/// Bind synchronously so a taken port fails loudly at startup rather than
/// silently picking another one.
pub fn bind(cfg: &Config) -> std::io::Result<std::net::TcpListener> {
    std::net::TcpListener::bind((cfg.server.bind.as_str(), cfg.server.port))
}

/// Serve until shutdown. Drains in-flight requests on the way out.
pub async fn serve(listener: std::net::TcpListener, state: AppState) {
    let _ = listener.set_nonblocking(true);
    let listener = match tokio::net::TcpListener::from_std(listener) {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("server: could not adopt listener: {e}");
            return;
        }
    };

    if let Ok(addr) = listener.local_addr() {
        println!("server: listening on http://{addr}");
    }

    if let Err(e) = axum::serve(listener, router(state))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
    {
        eprintln!("server: {e}");
    }
}

// ---------------------------------------------------------------- middleware

async fn cors(request: axum::extract::Request, next: Next) -> Response {
    let origin = request
        .headers()
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let allowed = origin
        .as_deref()
        .map(|origin| ALLOWED_ORIGINS.contains(&origin))
        .unwrap_or(false);

    let mut response = next.run(request).await;

    if allowed {
        if let Some(origin) = origin {
            if let Ok(value) = origin.parse() {
                response.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, value);
            }
        }
        response
            .headers_mut()
            .insert(header::VARY, "Origin".parse().unwrap());
        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            "GET, POST, OPTIONS".parse().unwrap(),
        );
        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            "content-type, x-clipper-token".parse().unwrap(),
        );
    }

    response
}

// ------------------------------------------------------------------- errors

pub struct ApiError(StatusCode, String);

impl ApiError {
    pub fn new(code: StatusCode, message: impl Into<String>) -> Self {
        Self(code, message.into())
    }

    pub fn internal(e: impl std::fmt::Display) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": self.1 }))).into_response()
    }
}

pub fn with_db<T>(
    state: &AppState,
    query: impl FnOnce(&rusqlite::Connection) -> rusqlite::Result<T>,
) -> Result<T, ApiError> {
    let conn = state.db.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    query(&conn).map_err(ApiError::internal)
}

// ----------------------------------------------------------- static dashboard

fn static_file(content_type: &'static str, bytes: &'static [u8]) -> Response {
    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .body(Body::from(bytes))
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

/// Serve an embedded `src/ui` file, or 404.
///
/// This is the router's fallback: the desktop app is headless, so the web app
/// under `src/ui` is the only UI and it is served entirely from here.
async fn ui_static(uri: axum::http::Uri) -> Response {
    match ui_asset(uri.path()) {
        Some((mime, bytes)) => static_file(mime, bytes),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// `GET /pair` — a short alias for the phone PWA page.
async fn pair_page() -> Response {
    match ui_asset("/pwa/index.html") {
        Some((mime, bytes)) => static_file(mime, bytes),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

// ------------------------------------------------------------------ handlers

async fn healthz() -> &'static str {
    "ok"
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    limit: Option<i64>,
    offset: Option<i64>,
    #[serde(rename = "type")]
    kind: Option<String>,
    tag: Option<String>,
    status: Option<String>,
    q: Option<String>,
}

async fn list_items(
    State(state): State<AppState>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Value>, ApiError> {
    let query = db::Query {
        limit: params.limit.unwrap_or(50),
        offset: params.offset.unwrap_or(0),
        kind: params.kind.filter(|v| !v.is_empty()),
        tag: params.tag.filter(|v| !v.is_empty()),
        status: params.status.filter(|v| !v.is_empty() && v != "all"),
        q: params.q.filter(|v| !v.is_empty()),
    };

    let (items, total) = with_db(&state, |conn| db::list(conn, &query))?;
    Ok(Json(json!({ "items": items, "total": total })))
}

async fn get_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let Some(item) = with_db(&state, |conn| db::get(conn, &id))? else {
        return Err(ApiError::new(StatusCode::NOT_FOUND, "no such item"));
    };

    let mut val = serde_json::to_value(&item).map_err(ApiError::internal)?;
    if let Some(obj) = val.as_object_mut() {
        let raw_file = item.vault_path.as_deref().and_then(|vp| {
            let p = std::path::Path::new(vp);
            if p.is_absolute() {
                std::fs::read_to_string(p).ok()
            } else {
                std::fs::read_to_string(p)
                    .or_else(|_| std::fs::read_to_string(state.cfg.vault_path().join(p)))
                    .ok()
            }
        });

        match raw_file {
            Some(text) => {
                let (raw_fm, body) = crate::frontmatter::split(&text);
                let fm_map = crate::frontmatter::parse(&text)
                    .map(|fm| json!(fm.fields()))
                    .unwrap_or(Value::Null);
                let body_trimmed = body.trim().to_string();
                let content_str = if body_trimmed.is_empty() {
                    text.clone()
                } else {
                    body_trimmed
                };
                obj.insert("content".into(), Value::String(content_str.clone()));
                obj.insert("body_md".into(), Value::String(content_str));
                obj.insert(
                    "raw_frontmatter".into(),
                    raw_fm.map(|s| Value::String(s.to_string())).unwrap_or(Value::Null),
                );
                obj.insert("frontmatter".into(), fm_map);
            }
            None => {
                obj.insert("content".into(), Value::Null);
                obj.insert("body_md".into(), Value::Null);
                obj.insert("raw_frontmatter".into(), Value::Null);
                obj.insert("frontmatter".into(), Value::Null);
            }
        }
    }

    Ok(Json(val))
}

#[derive(Debug, Deserialize)]
struct StatusBody {
    status: String,
}

async fn set_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<StatusBody>,
) -> Result<Json<Value>, ApiError> {
    if !db::STATUSES.contains(&body.status.as_str()) {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            format!("status must be one of {}", db::STATUSES.join(", ")),
        ));
    }

    if with_db(&state, |conn| db::set_status(conn, &id, &body.status))? {
        Ok(Json(json!({ "ok": true })))
    } else {
        Err(ApiError::new(StatusCode::NOT_FOUND, "no such item"))
    }
}

#[derive(Debug, Deserialize)]
struct TagsBody {
    tags: Vec<String>,
}

async fn set_tags(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<TagsBody>,
) -> Result<Json<Value>, ApiError> {
    if with_db(&state, |conn| db::set_tags(conn, &id, &body.tags))? {
        Ok(Json(json!({ "ok": true, "tags": body.tags })))
    } else {
        Err(ApiError::new(StatusCode::NOT_FOUND, "no such item"))
    }
}

#[derive(Debug, Deserialize)]
struct ClipBody {
    url: String,
    #[serde(rename = "type")]
    kind: Option<String>,
}

async fn create_clip(
    State(state): State<AppState>,
    Json(body): Json<ClipBody>,
) -> Result<Json<Value>, ApiError> {
    enqueue_clip(&state, body.url, body.kind, "manual")
}

#[derive(Debug, Deserialize)]
struct ImportItem {
    url: String,
    #[serde(rename = "type")]
    kind: Option<String>,
    title: Option<String>,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct ImportBody {
    items: Vec<ImportItem>,
    source: Option<String>,
}

async fn import_batch(
    State(state): State<AppState>,
    Json(body): Json<ImportBody>,
) -> Result<Json<Value>, ApiError> {
    let source = body.source.unwrap_or_else(|| "import".to_string());
    let mut imported = 0u32;
    let mut failed = 0u32;
    let mut errors: Vec<String> = Vec::new();
    for item in body.items {
        if !queue::looks_like_url(&item.url) {
            failed += 1;
            errors.push(format!("bad url: {}", item.url));
            continue;
        }
        // Dedup: skip if URL already exists in items table.
        let dup = with_db(&state, |conn| {
            let mut stmt = conn.prepare("SELECT 1 FROM items WHERE url = ? LIMIT 1")?;
            let exists: bool = stmt.exists([&item.url])?;
            Ok(exists)
        })?;
        if dup {
            failed += 1;
            continue;
        }
        match enqueue_clip(&state, item.url.clone(), item.kind, &source) {
            Ok(_) => imported += 1,
            Err(e) => {
                failed += 1;
                errors.push(format!("{}: {}", item.url, e.1));
            }
        }
    }
    let total = imported + failed;
    Ok(Json(json!({
        "total": total,
        "imported": imported,
        "failed": failed,
        "errors": errors,
    })))
}

/// Shared by `POST /api/clips` and the phone's `POST /save`.
fn enqueue_clip(
    state: &AppState,
    url: String,
    kind: Option<String>,
    source: &str,
) -> Result<Json<Value>, ApiError> {
    if !queue::looks_like_url(&url) {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "url must start with http:// or https://",
        ));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let entry = queue::QueueEntry {
        id: id.clone(),
        url: Some(url.trim().to_string()),
        path: None,
        source: source.to_string(),
        kind: kind.unwrap_or_else(|| "manual".to_string()),
        saved_at: clock::now_iso8601(),
    };

    let _guard = state
        .queue_lock
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    queue::append_json_line(&state.cfg.inbox_jsonl(), &entry).map_err(ApiError::internal)?;

    Ok(Json(json!({ "id": id, "status": "queued" })))
}

async fn today(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let items = with_db(&state, |conn| db::today(conn, 3))?;
    Ok(Json(json!({ "items": items })))
}

async fn stats(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let stats = with_db(&state, db::stats)?;
    Ok(Json(json!(stats)))
}

// --------------------------------------------------------------- phone pairing

/// A QR code for the pairing URL, as PNG.
async fn pair_qr(State(state): State<AppState>) -> Response {
    let url = pair_url_for(&state);
    match pairing::qr_png(&url) {
        Ok(png) => Response::builder()
            .header(header::CONTENT_TYPE, "image/png")
            .header(header::CACHE_CONTROL, "no-store")
            .body(Body::from(png))
            .unwrap_or_else(|_| Response::new(Body::empty())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("could not render QR: {e}"),
        )
            .into_response(),
    }
}

/// A tiny page showing that QR, for the tray's "Show QR" window.
async fn pair_qr_page(State(state): State<AppState>) -> Response {
    let url = pair_url_for(&state);
    let html = format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
         <title>Clipper — pair phone</title><style>\
         html,body{{height:100%}}body{{margin:0;display:grid;place-items:center;\
         background:#0f172a;color:#e2e8f0;font-family:system-ui,sans-serif}}\
         img{{background:#fff;border-radius:12px;padding:12px}}\
         code{{word-break:break-all;font-size:12px;color:#94a3b8}}\
         h1{{font-size:1rem;font-weight:600;text-align:center}}</style></head>\
         <body><div><h1>Scan to pair your phone</h1>\
         <img src=\"/pair.png\" width=\"304\" height=\"304\" alt=\"Pairing QR code\">\
         <p><code>{url}</code></p></div></body></html>"
    );
    static_string("text/html; charset=utf-8", html)
}

#[derive(Debug, Deserialize)]
struct SaveQuery {
    token: Option<String>,
}

/// The phone's save endpoint. Requires the pairing token, in the query string
/// or in `X-Clipper-Token`.
async fn save_clip(
    State(state): State<AppState>,
    Query(query): Query<SaveQuery>,
    headers: HeaderMap,
    Json(body): Json<ClipBody>,
) -> Result<Json<Value>, ApiError> {
    let supplied = query.token.or_else(|| {
        headers
            .get("x-clipper-token")
            .and_then(|value| value.to_str().ok())
            .map(str::to_string)
    });

    let expected = state
        .token
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();

    let authorised = supplied
        .as_deref()
        .map(|supplied| !expected.is_empty() && constant_time_eq(supplied, &expected))
        .unwrap_or(false);

    if !authorised {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "invalid or missing pairing token",
        ));
    }

    enqueue_clip(&state, body.url, body.kind, "phone")
}

fn pair_url_for(state: &AppState) -> String {
    let token = state
        .token
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    pairing::pair_url(&state.cfg, &state.lan_ip, &token)
}

/// Compare without an early exit, so a wrong token leaks no timing signal.
fn constant_time_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut difference = 0u8;
    for (left, right) in a.iter().zip(b.iter()) {
        difference |= left ^ right;
    }
    difference == 0
}

fn static_string(content_type: &'static str, body: String) -> Response {
    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

fn muse_state_path(state: &AppState) -> std::path::PathBuf {
    state.cfg.memory_dir().parent().map(|p| p.join("muse_state.json")).unwrap_or_else(|| std::path::PathBuf::from("muse_state.json"))
}

async fn get_muse_state(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let path = muse_state_path(&state);
    let mut value: Value = match std::fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|_| json!({ "state": "off", "activity": [] })),
        Err(_) => json!({ "state": "off", "activity": [] }),
    };
    // The watcher appends to <vault>/Activity/<YYYY-MM-DD>.md, not to muse_state.json,
    // so on every read we also surface the last N entries from today's file. This is
    // what the dashboard's "Recent Activity" panel renders.
    let recent = read_recent_activity(&state, clock::today_yyyy_mm_dd(), 30);
    if let Some(obj) = value.as_object_mut() {
        obj.insert("activity".to_string(), json!(recent));
    }
    Ok(Json(value))
}

#[derive(Debug, Deserialize)]
struct ActivityQuery {
    date: Option<String>,
    limit: Option<usize>,
}

async fn get_muse_activity(
    State(state): State<AppState>,
    Query(q): Query<ActivityQuery>,
) -> Result<Json<Value>, ApiError> {
    let date = q.date.unwrap_or_else(clock::today_yyyy_mm_dd);
    let limit = q.limit.unwrap_or(200);
    let entries = read_recent_activity(&state, date.clone(), limit);
    Ok(Json(json!({ "date": date, "activity": entries })))
}

/// Counts the activity entries per day for the last `days` days, for the calendar
/// heatmap on the Muse page. The list is sorted oldest-first.
async fn get_muse_heatmap(
    State(state): State<AppState>,
    Query(q): Query<HeatmapQuery>,
) -> Result<Json<Value>, ApiError> {
    let days = q.days.unwrap_or(365).clamp(7, 730);
    let activity_dir = state.cfg.vault_path().join("Activity");
    let today = clock::today_yyyy_mm_dd();
    let mut cells: Vec<Value> = Vec::with_capacity(days);
    for i in (0..days).rev() {
        let d = shift_date(&today, -(i as i64));
        let path = activity_dir.join(format!("{d}.md"));
        let count = std::fs::read_to_string(&path)
            .map(|t| t.lines().filter(|l| l.starts_with("- `")).count())
            .unwrap_or(0);
        cells.push(json!({ "date": d, "count": count }));
    }
    Ok(Json(json!({ "today": today, "days": days, "cells": cells })))
}

#[derive(Debug, Deserialize)]
struct HeatmapQuery {
    days: Option<usize>,
}

fn shift_date(yyyy_mm_dd: &str, delta_days: i64) -> String {
    // Tiny Gregorian date shift, mirrors format_ymd in musewatch.rs but in
    // reverse: parse the date, walk by `delta_days` days, reformat.
    let parts: Vec<&str> = yyyy_mm_dd.split('-').collect();
    if parts.len() != 3 {
        return yyyy_mm_dd.to_string();
    }
    let y: i64 = parts[0].parse().unwrap_or(1970);
    let m: i64 = parts[1].parse().unwrap_or(1);
    let d: i64 = parts[2].parse().unwrap_or(1);
    let days_from_epoch = |year: i64, month: i64, day: i64| -> i64 {
        let mut total = 0i64;
        for yy in 1970..year {
            total += if is_leap(yy) { 366 } else { 365 };
        }
        let mdays = [31, if is_leap(year) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        for mm in 1..month {
            total += mdays[(mm - 1) as usize];
        }
        total + (day - 1)
    };
    let epoch_day = days_from_epoch(y, m, d);
    let target = epoch_day + delta_days;
    // Convert epoch day back to Y-M-D.
    let mut year = 1970i64;
    let mut remaining = target;
    loop {
        let dy = if is_leap(year) { 366 } else { 365 };
        if remaining < dy {
            break;
        }
        remaining -= dy;
        year += 1;
    }
    let mdays = [31, if is_leap(year) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1i64;
    while month <= 12 && remaining >= mdays[(month - 1) as usize] {
        remaining -= mdays[(month - 1) as usize];
        month += 1;
    }
    let day = remaining + 1;
    format!("{:04}-{:02}-{:02}", year, month, day)
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

#[derive(Debug, Deserialize)]
struct DigestQuery {
    date: Option<String>,
}

/// Shell out to `claude -p` with the given prompt, capture the first line of
/// stdout, and suppress the console window. Used by the daily-digest endpoint.
fn run_claude_silent(prompt: &str) -> String {
    let mut cmd = Command::new("claude");
    cmd.arg("-p")
        .arg(prompt)
        .arg("--output-format")
        .arg("text")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let Ok(mut child) = cmd.spawn() else {
        return String::new();
    };
    let mut out = String::new();
    if let Some(mut s) = child.stdout.take() {
        let _ = std::io::Read::read_to_string(&mut s, &mut out);
    }
    let _ = child.wait();
    out.lines().next().unwrap_or("").trim().to_string()
}

/// Read the current Muse configuration. The dashboard uses this to populate
/// the settings popover; the values mirror the `[muse]` section in `config.toml`.
async fn get_muse_config(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    Ok(Json(serde_json::to_value(&state.cfg.muse).map_err(ApiError::internal)?))
}

#[derive(Debug, Deserialize)]
struct MuseConfigBody {
    skip_processes: Option<Vec<String>>,
    smart_dedup: Option<bool>,
    debounce_secs: Option<u32>,
    screenshots: Option<bool>,
}

/// Update one or more Muse settings. Persists to `config.toml`. The in-memory
/// config is not reloaded (the watcher reads from disk on its next 5s tick,
/// and the next process restart picks up the change too).
async fn set_muse_config(
    State(state): State<AppState>,
    Json(body): Json<MuseConfigBody>,
) -> Result<Json<Value>, ApiError> {
    let mut cfg = state.cfg.clone();
    if let Some(v) = body.skip_processes {
        cfg.muse.skip_processes = v;
    }
    if let Some(v) = body.smart_dedup {
        cfg.muse.smart_dedup = v;
    }
    if let Some(v) = body.debounce_secs {
        cfg.muse.debounce_secs = v;
    }
    if let Some(v) = body.screenshots {
        cfg.muse.screenshots = v;
    }
    let path = crate::config::config_path();
    let toml_str = toml::to_string_pretty(&cfg).map_err(ApiError::internal)?;
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&path, toml_str).map_err(ApiError::internal)?;
    Ok(Json(serde_json::to_value(&cfg.muse).map_err(ApiError::internal)?))
}

/// Summarize the day's activity log via `claude -p` for the end-of-day digest.
/// Returns a single string with a 2-3 sentence overview.
async fn get_muse_digest(
    State(state): State<AppState>,
    Query(q): Query<DigestQuery>,
) -> Result<Json<Value>, ApiError> {
    let date = q.date.unwrap_or_else(clock::today_yyyy_mm_dd);
    let path = state.cfg.vault_path().join("Activity").join(format!("{date}.md"));
    let body = std::fs::read_to_string(&path).unwrap_or_default();
    if body.trim().is_empty() {
        return Ok(Json(json!({ "date": date, "summary": "No activity logged.", "entries": 0 })));
    }
    let entries = body.lines().filter(|l| l.starts_with("- `")).count();
    let prompt = format!(
        "Below is a chronological log of foreground windows the user switched to today. \
         Reply with a 2-3 sentence summary of what they worked on, grouping by task rather \
         than by window. Be specific. If the log is mostly noise (calls, social apps), say so.\n\n{}",
        body
    );
    let summary = run_claude_silent(&prompt);
    Ok(Json(json!({ "date": date, "summary": summary, "entries": entries })))
}

/// Read the last `limit` lines of the requested day's activity markdown and
/// parse them into `{ts, process, window_title, summary}` objects, newest first.
/// Lines look like:
///   - `14:58` **Brave** — GitHub — clipper/aswathas — likely drafting launch post.
fn read_recent_activity(state: &AppState, date: String, limit: usize) -> Vec<Value> {
    let activity_dir = state.cfg.vault_path().join("Activity");
    let path = activity_dir.join(format!("{date}.md"));
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let day = date;
    text.lines()
        .rev()
        .filter(|l| l.starts_with("- `"))
        .take(limit)
        .filter_map(|line| parse_activity_line(line, &day))
        .collect()
}

fn parse_activity_line(line: &str, today: &str) -> Option<Value> {
    // Skip the leading "- `" and find the closing backtick that ends the timestamp.
    let rest = line.strip_prefix("- `")?;
    let end = rest.find('`')?;
    let hhmm = &rest[..end];
    let after_ts = rest[end + 1..].trim_start();
    // after_ts: **process** — title — summary. The process may be empty, which
    // renders as `****`; strip one `**` and look for the next pair.
    let (process, rest) = if let Some(after) = after_ts.strip_prefix("**") {
        if let Some(end) = after.find("**") {
            (after[..end].to_string(), after[end + 2..].trim_start())
        } else {
            // Empty process (****): skip four asterisks and the separator.
            let stripped = after.trim_start_matches('*').trim_start();
            (String::new(), stripped)
        }
    } else {
        (String::new(), after_ts)
    };
    // `rest` starts with " — "; drop that, then split on the next " — " into title/summary.
    let rest = rest.strip_prefix("—").unwrap_or(rest).trim_start();
    let mut parts = rest.splitn(2, " — ");
    let title = parts.next().unwrap_or("").trim().to_string();
    let summary = parts.next().unwrap_or("").trim().to_string();
    // The watcher's HH:MM is the user's local clock; emit a naive ISO stamp
    // (no Z) so `new Date(ts)` parses it as browser-local time, not UTC.
    let ts = format!("{today}T{hhmm}:00");
    Some(json!({
        "ts": ts,
        "process": process,
        "window_title": title,
        "summary": summary,
    }))
}

#[derive(Debug, Deserialize)]
struct MuseStateBody {
    state: String,
}

async fn set_muse_state(State(state): State<AppState>, Json(body): Json<MuseStateBody>) -> Result<Json<Value>, ApiError> {
    let new_state = match body.state.as_str() {
        "off" | "watching" | "paused" => body.state,
        _ => return Err(ApiError::new(StatusCode::BAD_REQUEST, "state must be off|watching|paused")),
    };
    let path = muse_state_path(&state);
    if let Some(dir) = path.parent() { let _ = std::fs::create_dir_all(dir); }
    let payload = json!({ "state": new_state, "updated_at": clock::now_iso8601(), "activity": [] });
    std::fs::write(&path, serde_json::to_string_pretty(&payload).map_err(ApiError::internal)?).map_err(ApiError::internal)?;
    Ok(Json(payload))
}

async fn get_concepts(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let concepts = with_db(&state, |conn| {
        let mut stmt = conn.prepare(
            "SELECT je.value AS tag, COUNT(*) AS n
             FROM items, json_each(items.tags) AS je
             GROUP BY je.value
             ORDER BY n DESC
             LIMIT 200"
        )?;
        let rows = stmt.query_map([], |r| {
            let tag: String = r.get(0)?;
            let n: i64 = r.get(1)?;
            Ok(json!({ "tag": tag, "count": n }))
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    })?;
    Ok(Json(json!({ "concepts": concepts, "total": concepts.len() })))
}

async fn get_related(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let item = match with_db(&state, |conn| db::get(conn, &id))? {
        Some(it) => it,
        None => return Err(ApiError::new(StatusCode::NOT_FOUND, "no such item")),
    };
    let tags: Vec<String> = item.tags.clone();
    let related = if tags.is_empty() {
        Vec::new()
    } else {
        with_db(&state, |conn| {
            let placeholders = std::iter::repeat("?").take(tags.len()).collect::<Vec<_>>().join(",");
            let sql = format!(
                "SELECT DISTINCT items.id, items.title, items.url, items.saved_at, items.summary
                 FROM items, json_each(items.tags) AS je
                 WHERE je.value IN ({placeholders}) AND items.id != ?
                 ORDER BY items.saved_at DESC
                 LIMIT 12"
            );
            let mut stmt = conn.prepare(&sql)?;
            let mut params: Vec<&dyn rusqlite::ToSql> = tags.iter().map(|t| t as &dyn rusqlite::ToSql).collect();
            let id_ref = id.clone();
            params.push(&id_ref as &dyn rusqlite::ToSql);
            let rows = stmt.query_map(&*params, |r| {
                Ok(json!({
                    "id": r.get::<_, String>(0)?,
                    "title": r.get::<_, String>(1)?,
                    "url": r.get::<_, String>(2)?,
                    "saved_at": r.get::<_, i64>(3)?,
                    "summary": r.get::<_, String>(4)?,
                }))
            })?.collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })?
    };
    Ok(Json(json!({ "tags": tags, "related": related, "total": related.len() })))
}
