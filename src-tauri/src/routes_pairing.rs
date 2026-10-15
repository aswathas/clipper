//! Pairing over HTTP (S7): where the phone should go, and rotating its token.
//!
//! The token is deliberately **never** returned by the JSON API — it lives only
//! inside the QR image that `/pair.png` renders. A Settings page can show the
//! URL and the QR; nothing else can read the secret.

use axum::{extract::State, routing::{get, post}, Json, Router};
use serde_json::{json, Value};

use crate::{clock, events, pairing, server::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/pairing", get(view))
        .route("/api/pairing/regenerate", post(regenerate))
}

/// Swap in a fresh token and tell everyone watching.
pub fn rotate(state: &AppState) -> i64 {
    let fresh = pairing::new_token();
    *state
        .token
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = fresh;

    let at = clock::now_unix();
    events::hub().emit("token_rotated", json!({ "at": at }));
    at
}

async fn view(State(state): State<AppState>) -> Json<Value> {
    let port = state.cfg.server.port;
    let host = &state.lan_ip;

    Json(json!({
        "lan_ip": host,
        "port": port,
        // No token here on purpose: the QR is the only place it lives.
        "pair_url": format!("http://{host}:{port}/pair"),
        "qr_path": "/pair.png",
        "token_length": state
            .token
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .chars()
            .count(),
    }))
}

async fn regenerate(State(state): State<AppState>) -> Json<Value> {
    let at = rotate(&state);
    Json(json!({ "ok": true, "rotated_at": at }))
}
