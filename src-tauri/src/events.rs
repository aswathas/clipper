//! The event hub: one broadcast channel that everything publishes to and
//! `/api/events` streams out as Server-Sent Events.
//!
//! SSE rather than WebSocket, per the v0.2 synthesis: the traffic is
//! server-to-client only, and a plain HTTP response needs no upgrade handshake.

use std::convert::Infallible;

use axum::{
    body::Body,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use futures_util::stream;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::broadcast;

use crate::clock;

/// How many events a slow client may fall behind before it starts dropping.
const CHANNEL_CAPACITY: usize = 256;

/// An event waiting to be delivered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub kind: String,
    pub at: i64,
    #[serde(default)]
    pub payload: Value,
}

impl Event {
    pub fn new(kind: impl Into<String>, payload: Value) -> Self {
        Self {
            kind: kind.into(),
            at: clock::now_unix(),
            payload,
        }
    }

    /// One SSE frame.
    pub fn to_frame(&self) -> String {
        let data = serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string());
        format!("event: {}\ndata: {data}\n\n", self.kind)
    }
}

/// A cheap, cloneable handle to the hub.
#[derive(Clone)]
pub struct Hub {
    tx: broadcast::Sender<Event>,
}

impl Default for Hub {
    fn default() -> Self {
        Self::new()
    }
}

/// The process-wide hub. One channel for the whole app, so any slice can
/// publish without every constructor having to thread a handle through.
pub fn hub() -> &'static Hub {
    static HUB: std::sync::OnceLock<Hub> = std::sync::OnceLock::new();
    HUB.get_or_init(Hub::new)
}

impl Hub {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self { tx }
    }

    /// Publish. Errors only when nobody is listening, which is normal.
    pub fn emit(&self, kind: impl Into<String>, payload: Value) {
        let _ = self.tx.send(Event::new(kind, payload));
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.tx.subscribe()
    }

    /// `GET /api/events` — the SSE stream.
    pub fn stream_response(&self) -> Response {
        // Announce the connection so a client can tell it is live.
        let hello = Event::new("hello", serde_json::json!({ "ok": true }));
        let rx = self.subscribe();

        let body = stream::unfold(
            (rx, Some(hello.to_frame()), tokio::time::interval(
                std::time::Duration::from_secs(15),
            )),
            |(mut rx, mut pending, mut keepalive)| async move {
                if let Some(frame) = pending.take() {
                    return Some((Ok::<_, Infallible>(frame), (rx, None, keepalive)));
                }

                loop {
                    tokio::select! {
                        received = rx.recv() => match received {
                            Ok(event) => {
                                return Some((Ok(event.to_frame()), (rx, None, keepalive)));
                            }
                            // A slow client simply misses events; keep streaming.
                            Err(broadcast::error::RecvError::Lagged(_)) => continue,
                            Err(broadcast::error::RecvError::Closed) => return None,
                        },
                        _ = keepalive.tick() => {
                            // A comment frame keeps proxies from closing idle streams.
                            return Some((Ok(": keep-alive\n\n".to_string()), (rx, None, keepalive)));
                        }
                    }
                }
            },
        );

        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/event-stream")
            .header(header::CACHE_CONTROL, "no-cache")
            .header("x-accel-buffering", "no")
            .body(Body::from_stream(body))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
    }
}
