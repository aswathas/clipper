//! The Soul, with its frontmatter surfaced as structured settings.
//!
//! `soul.md` is one of the three memory files, so reads and writes go through
//! [`crate::routes_memory`] — this module only adds the parsed view a Settings
//! UI needs, and keeps the existing `If-Match` rules.

use axum::{
    extract::State,
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde_json::{json, Value};

use crate::{frontmatter, memory, routes_memory, server::AppState};

/// The keys a client may rely on being surfaced.
const SETTINGS: [&str; 4] = ["tone", "language", "max_summary_words", "avoid"];

/// Pull the human-editable settings out of the soul's frontmatter.
pub fn settings_from(content: &str) -> Value {
    let parsed = frontmatter::parse(content).unwrap_or_default();
    let mut out = serde_json::Map::new();

    for key in SETTINGS {
        let Some(raw) = parsed.get(key) else {
            continue;
        };
        if key == "avoid" {
            out.insert("avoid".into(), json!(frontmatter::parse_inline_list(raw)));
        } else if key == "max_summary_words" {
            match raw.trim().parse::<i64>() {
                Ok(number) => out.insert(key.into(), json!(number)),
                Err(_) => out.insert(key.into(), json!(raw)),
            };
        } else {
            out.insert(key.into(), json!(raw));
        }
    }

    Value::Object(out)
}

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/api/soul",
        get(get_soul).put(routes_memory::put_soul),
    )
}

async fn get_soul(State(state): State<AppState>) -> Result<Json<Value>, crate::server::ApiError> {
    let cfg = state.cfg.clone();
    let handle = std::sync::Arc::clone(&state.db);

    let (content, mtime, etag) =
        tokio::task::spawn_blocking(move || routes_memory::read_part(&cfg, &handle, memory::SOUL))
            .await
            .map_err(crate::server::ApiError::internal)?
            .map_err(crate::server::ApiError::internal)?;

    if content.is_empty() {
        return Err(crate::server::ApiError::new(
            StatusCode::NOT_FOUND,
            "soul.md is empty",
        ));
    }

    Ok(Json(json!({
        "part": memory::SOUL,
        "content": content,
        "settings": settings_from(&content),
        "mtime": mtime,
        "etag": etag,
    })))
}
