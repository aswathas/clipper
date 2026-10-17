# Clipper v0.5.3 — silent watcher + working activity panel

## Highlights

- **No more popup console windows** (`00f55d9`). The Muse watcher used to spawn a visible PowerShell + claude.exe console on every 5s tick. Both spawns now carry `CREATE_NO_WINDOW` (0x0800_0000) via `std::os::windows::process::CommandExt::creation_flags()`. Silent.
- **Muse page now shows real activity** (`37cc923`). `GET /api/muse/state` also reads the last 30 lines of today's `<vault>/Activity/YYYY-MM-DD.md` and returns them as `{ts, process, window_title, summary}`. The dashboard's "Recent Activity" panel was always empty before because the watcher only writes to the markdown file, never to `muse_state.json`.
- **Local time, newest-first, date picker** (`2040619`):
  - Watcher now uses `time::OffsetDateTime::now_local()` (with `local-offset` feature added to Cargo.toml) so `HH:MM` is the user's local clock, not UTC.
  - Parser emits naive ISO so the browser displays local time, not UTC.
  - `GET /api/muse/activity?date=YYYY-MM-DD&limit=N` reads any day's log, defaulting to today.
  - Dashboard Muse page: date selector (last 7 days) and a sort toggle (newest-first / oldest-first).

## Bug fixes

- **Console popups killing the desk** — PowerShell + claude.exe windows stopped opening on every 5s poll. User-reported and now silent.
- **Dashboard Muse page always empty** — handler returned `activity: []` from JSON, ignored the markdown log.
- **Wrong timezone** — markdown was stamped UTC; user in IST saw 7-hour skew.
- **Oldest-first sort** — list rendered oldest at the top, newest at the bottom. Inverted.

## Files changed

- `src-tauri/Cargo.toml` — added `local-offset` feature to `time` crate.
- `src-tauri/src/musewatch.rs` — local time; dropped unused `format_ymd` / `is_leap` helpers.
- `src-tauri/src/server.rs` — `get_muse_state` reads markdown; new `get_muse_activity` route; helper `read_recent_activity(state, date, limit)`; parser handles empty process (`****` in markdown).
- `src-tauri/src/clock.rs` — new `today_yyyy_mm_dd()` helper.
- `src/ui/index.html` — Muse page: date picker, sort button, hint about tray "Show muse".
- `src/ui/app.js` — `pageMuse()`: `dateOptions`, `sortedActivity`, `loadActivity()`, `toggleSort()`.

## Known issues

- **Empty process for some windows**: `GetModuleBaseName` returns `""` for certain foreground windows (Brave, Munder Difflin); the markdown stamps `****` for process and packs process info into the title. Cosmetic; pending `PROCESS_QUERY_LIMITED_INFORMATION` fallback.
- **5s polling lag**: a real `SetWinEventHook` would react in <100ms. Deferred to v0.6.
- **No screenshot**: titles lie; a PrintWindow capture + Claude with image would be more useful. Deferred to v0.6.

## Migration

No DB or config migration. Just relaunch clipper.
