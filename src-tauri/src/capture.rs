//! Clipboard text -> deduped queue append.

use std::{io, sync::Mutex};

use crate::{
    clock,
    config::Config,
    queue::{self, QueueEntry},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureOutcome {
    /// A new item landed in `inbox.jsonl`.
    Queued { id: String, hash: String },
    /// The URL's hash is already in `archive.jsonl`.
    AlreadySaved { host: String, hash: String },
    /// Not a URL: silently ignored.
    NotAUrl,
}

/// Dedupe against the archive, then append to `inbox.jsonl`.
///
/// The `lock` is held across the read-modify-write so a concurrent drainer
/// rotation cannot race the append.
pub fn capture(cfg: &Config, lock: &Mutex<()>, text: &str) -> io::Result<CaptureOutcome> {
    if !queue::looks_like_url(text) {
        return Ok(CaptureOutcome::NotAUrl);
    }

    let url = text.trim().to_string();
    let hash = queue::url_hash(&url);

    let _guard = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());

    if queue::archived_hashes(&cfg.archive_jsonl())?.contains(&hash) {
        return Ok(CaptureOutcome::AlreadySaved {
            host: queue::host_of(&url),
            hash,
        });
    }

    let id = uuid::Uuid::new_v4().to_string();
    let entry = QueueEntry {
        id: id.clone(),
        url,
        source: "clipboard".into(),
        kind: "url".into(),
        saved_at: clock::now_iso8601(),
    };
    queue::append_json_line(&cfg.inbox_jsonl(), &entry)?;

    Ok(CaptureOutcome::Queued { id, hash })
}
