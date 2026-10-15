//! The local REST API, and the dashboard it serves.
//!
//! Bound to `[server] bind`/`port` (default `127.0.0.1:7137`). The dashboard in
//! `src/ui/` is embedded in the binary and served from `/`, so its relative
//! `/api/...` fetches stay same-origin.

use std::sync::{Arc, Mutex};

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{clock, config::Config, db, queue};

const DASHBOARD_HTML: &[u8] = include_bytes!("../../src/ui/index.html");
const DASHBOARD_JS: &[u8] = include_bytes!("../../src/ui/app.js");
const DASHBOARD_CSS: &[u8] = include_bytes!("../../src/ui/styles.css");

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
}

/// Build the router. Split out so the self-check can serve it on port 0.
pub fn router(state: AppState) -> Router {
    Router::new()
        // dashboard
        .route("/", get(dashboard_html))
        .route("/index.html", get(dashboard_html))
        .route("/app.js", get(dashboard_js))
        .route("/styles.css", get(dashboard_css))
        // api
        .route("/healthz", get(healthz))
        .route("/api/items", get(list_items))
        .route("/api/items/{id}", get(get_item))
        .route("/api/items/{id}/status", post(set_status))
        .route("/api/items/{id}/tags", post(set_tags))
        .route("/api/clips", post(create_clip))
        .route("/api/today", get(today))
        .route("/api/stats", get(stats))
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

struct ApiError(StatusCode, String);

impl ApiError {
    fn new(code: StatusCode, message: impl Into<String>) -> Self {
        Self(code, message.into())
    }

    fn internal(e: impl std::fmt::Display) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": self.1 }))).into_response()
    }
}

fn with_db<T>(
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

async fn dashboard_html() -> Response {
    static_file("text/html; charset=utf-8", DASHBOARD_HTML)
}

async fn dashboard_js() -> Response {
    static_file("text/javascript; charset=utf-8", DASHBOARD_JS)
}

async fn dashboard_css() -> Response {
    static_file("text/css; charset=utf-8", DASHBOARD_CSS)
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
        status: params.status.filter(|v| !v.is_empty()),
        q: params.q.filter(|v| !v.is_empty()),
    };

    let (items, total) = with_db(&state, |conn| db::list(conn, &query))?;
    Ok(Json(json!({ "items": items, "total": total })))
}

async fn get_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    match with_db(&state, |conn| db::get(conn, &id))? {
        Some(item) => Ok(Json(json!(item))),
        None => Err(ApiError::new(StatusCode::NOT_FOUND, "no such item")),
    }
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
    if !queue::looks_like_url(&body.url) {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "url must start with http:// or https://",
        ));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let entry = queue::QueueEntry {
        id: id.clone(),
        url: Some(body.url.trim().to_string()),
        path: None,
        source: "manual".to_string(),
        kind: body.kind.unwrap_or_else(|| "manual".to_string()),
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
