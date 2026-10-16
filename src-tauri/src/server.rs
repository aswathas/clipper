//! The local REST API, and the dashboard it serves.
//!
//! Bound to `[server] bind`/`port` (default `127.0.0.1:7137`). The dashboard in
//! `src/ui/` is embedded in the binary and served from `/`, so its relative
//! `/api/...` fetches stay same-origin.

use std::sync::{Arc, Mutex, RwLock};

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
