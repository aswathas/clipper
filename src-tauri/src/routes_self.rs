//! Self-improvement (S6): what the user does, the topics that implies, and the
//! rules derived from them.
//!
//! `user_topic` and `user_pattern` are **pure functions of the index**, so a
//! recompute replaces them wholesale instead of merging — there is nothing to
//! reconcile and no drift to accumulate.

use std::{collections::BTreeMap, sync::Arc, time::Duration};

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{clock, db, events, server::AppState};

/// How many topics to keep. The tail is noise.
pub const TOPIC_LIMIT: usize = 12;

/// Derive topics and patterns from the item index, and persist them.
pub fn recompute(
    conn: &mut rusqlite::Connection,
) -> rusqlite::Result<(Vec<db::UserTopic>, Vec<db::UserPattern>)> {
    let rows = db::item_type_tags(conn)?;
    let total = rows.len() as f64;

    let mut tag_counts: BTreeMap<String, f64> = BTreeMap::new();
    let mut type_counts: BTreeMap<String, f64> = BTreeMap::new();

    for (kind, tags_json) in &rows {
        for tag in serde_json::from_str::<Vec<String>>(tags_json).unwrap_or_default() {
            *tag_counts.entry(tag).or_insert(0.0) += 1.0;
        }
        *type_counts
            .entry(kind.clone().unwrap_or_else(|| "unknown".to_string()))
            .or_insert(0.0) += 1.0;
    }

    let now = clock::now_unix();

    let mut topics: Vec<db::UserTopic> = tag_counts
        .into_iter()
        .map(|(label, weight)| db::UserTopic {
            label,
            weight,
            last_seen: now,
        })
        .collect();
    topics.sort_by(|a, b| {
        b.weight
            .partial_cmp(&a.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.label.cmp(&b.label))
    });
    topics.truncate(TOPIC_LIMIT);

    let mut patterns = Vec::new();

    if rows.is_empty() {
        patterns.push(db::UserPattern {
            rule_md: "Nothing captured yet — patterns appear once you save a few things.".into(),
            confidence: 0.0,
            added_at: now,
        });
    } else {
        if let Some(top) = topics.first() {
            patterns.push(db::UserPattern {
                rule_md: format!(
                    "Most of what you save is about `{}` — {} item(s).",
                    top.label, top.weight as i64
                ),
                confidence: (top.weight / total).min(1.0),
                added_at: now,
            });
        }
        if let Some((kind, count)) = type_counts
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        {
            patterns.push(db::UserPattern {
                rule_md: format!(
                    "You mostly save `{kind}` items — {} of {}.",
                    *count as i64,
                    rows.len()
                ),
                confidence: (*count / total).min(1.0),
                added_at: now,
            });
        }
    }

    db::replace_topics(conn, &topics)?;
    db::replace_patterns(conn, &patterns)?;
    Ok((topics, patterns))
}

/// Run one recompute against the shared connection.
pub async fn recompute_now(handle: &db::Db) -> Result<(Vec<db::UserTopic>, Vec<db::UserPattern>), String> {
    let handle = Arc::clone(handle);
    tokio::task::spawn_blocking(move || {
        let mut conn = handle.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        recompute(&mut conn)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

/// Recompute once at startup, then once a day. A task and a sleep — no cron.
pub async fn nightly(db: db::Db) {
    loop {
        match recompute_now(&db).await {
            Ok((topics, patterns)) => {
                println!(
                    "self: recomputed {} topic(s), {} pattern(s)",
                    topics.len(),
                    patterns.len()
                );
                events::hub().emit(
                    "self_recomputed",
                    json!({ "topics": topics.len(), "patterns": patterns.len() }),
                );
            }
            Err(e) => eprintln!("self: recompute failed: {e}"),
        }
        tokio::time::sleep(Duration::from_secs(24 * 60 * 60)).await;
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/events", get(sse_stream).post(record_event))
        .route("/api/self/summary", get(summary))
        .route("/api/self/recompute", post(recompute_route))
}

/// `GET /api/events` — the SSE stream every slice publishes into.
async fn sse_stream(State(_state): State<AppState>)  -> axum::response::Response {
    events::hub().stream_response()
}

#[derive(Debug, Deserialize)]
struct EventBody {
    kind: String,
    #[serde(default)]
    payload: Value,
}

/// `POST /api/events` — the frontend reporting something the user did.
async fn record_event(
    State(state): State<AppState>,
    Json(body): Json<EventBody>,
) -> Result<Json<Value>, crate::server::ApiError> {
    let kind = body.kind.trim().to_string();
    if kind.is_empty() || kind.len() > 64 {
        return Err(crate::server::ApiError::new(
            StatusCode::BAD_REQUEST,
            "kind is required (max 64 chars)",
        ));
    }

    let (ts, payload) = (clock::now_unix(), body.payload.clone());
    crate::server::with_db(&state, |conn| db::record_event(conn, ts, &kind, &payload))?;
    events::hub().emit("user_event", json!({ "kind": kind }));

    Ok(Json(json!({ "ok": true })))
}

async fn summary(State(state): State<AppState>) -> Result<Json<Value>, crate::server::ApiError> {
    let (topics, patterns, recent, count) = crate::server::with_db(&state, |conn| {
        Ok((
            db::list_topics(conn)?,
            db::list_patterns(conn)?,
            db::recent_events(conn, 20)?,
            db::event_count(conn)?,
        ))
    })?;

    Ok(Json(json!({
        "topics": topics,
        "patterns": patterns,
        "recent_events": recent,
        "event_count": count,
    })))
}

async fn recompute_route(
    State(state): State<AppState>,
) -> Result<Json<Value>, crate::server::ApiError> {
    let (topics, patterns) = recompute_now(&state.db)
        .await
        .map_err(crate::server::ApiError::internal)?;

    events::hub().emit(
        "self_recomputed",
        json!({ "topics": topics.len(), "patterns": patterns.len() }),
    );

    Ok(Json(json!({ "ok": true, "topics": topics, "patterns": patterns })))
}
