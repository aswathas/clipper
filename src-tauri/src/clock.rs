//! Time helpers. Timestamps are UTC everywhere.

use time::{format_description::well_known::Rfc3339, OffsetDateTime};

/// ISO-8601 UTC, e.g. `2026-10-15T12:34:56Z`. Used for `saved_at`.
pub fn now_iso8601() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| String::new())
}

/// Unix seconds, for the index.
pub fn now_unix() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

/// `YYYY-MM-DD_HHMM` (UTC) for vault filenames.
///
/// Derived by slicing the RFC-3339 stamp rather than by a format macro, so it
/// does not care whether the offset renders as `Z` or `+00:00`.
pub fn file_stamp() -> String {
    let iso = now_iso8601();
    let date = iso.get(0..10).unwrap_or("0000-00-00");
    let hour = iso.get(11..13).unwrap_or("00");
    let minute = iso.get(14..16).unwrap_or("00");
    format!("{date}_{hour}{minute}")
}

/// `YYYY-MM-DD` (UTC) — the date stamp the muse watcher uses for its
/// `<vault>/Activity/<date>.md` filename.
pub fn today_yyyy_mm_dd() -> String {
    now_iso8601()
        .get(0..10)
        .unwrap_or("0000-00-00")
        .to_string()
}
