# Clipper v0.6.0 — Muse becomes a real activity tracker

## Highlights

- **1-second polling** (`d80c623`) — watcher reacts to foreground changes in under 2s instead of up to 10s. PowerShell spawn is already silent via `CREATE_NO_WINDOW`, so the load is invisible.
- **Smart dedup** (`6ddbc49`) — consecutive entries for the same `(process, normalized_title)` collapse into one log line. No more `Brave → GitHub/aswathas/clipper` repeated 8 times.
- **Skip rules** — `[muse] skip_processes` in `config.toml` filters out known noise (zoom, whatsapp, slack, telegram, screensaver, lockapp) before summarization. Editable from the dashboard.
- **Calendar heatmap** — `/api/muse/heatmap?days=N` returns per-day entry counts; the Muse page renders a 7×N SVG grid in the purple palette. Last 365 days on the dashboard.
- **Daily digest** — `/api/muse/digest?date=YYYY-MM-DD` asks Claude to summarize the day's log into a 2-3 sentence overview. Click "Daily digest" on the Muse page.
- **PrintWindow screenshots** — when `cfg.muse.screenshots=true`, every new entry also captures a PNG of the foreground HWND to `<vault>/Activity/screenshots/<HHMMSS>_<process>.png`. The markdown line links to the file with a 📷 marker. Uses the GDI path with `PW_RENDERFULLCONTENT` so it works for most apps, including ones that `BitBlt` cannot reach.
- **Settings popover** — Muse page → "Settings" → edit smart_dedup, screenshots, debounce_secs, skip processes. Save writes to `config.toml` via `POST /api/muse/config`.
- **Ctrl+Shift+M global kill switch** (`daccd54`) — pause/resume the watcher from any focus, no need to open the dashboard.
- **Process-name fix** (`80164c8`) — fall back to `QueryFullProcessImageName` when `GetModuleBaseName` returns empty. Stops the `****` placeholder for Brave, Munder Difflin, and other apps whose process names `psapi` can't read.

## Files changed

- `src-tauri/src/config.rs` — new `MuseCfg` with skip_processes, smart_dedup, debounce_secs, screenshots.
- `src-tauri/src/musewatch.rs` — 1s poll, skip list, smart dedup, screenshot capture, normalize_title, dropped dead `format_ymd`/`is_leap`.
- `src-tauri/src/server.rs` — new routes `/api/muse/activity`, `/api/muse/heatmap`, `/api/muse/digest`, `/api/muse/config` (GET+POST). Helper `read_recent_activity(state, date, limit)`, `parse_activity_line`, `shift_date`, `is_leap`, `run_claude_silent`.
- `src-tauri/src/lib.rs` — `Ctrl+Shift+M` global hotkey + `toggle_muse_state()`.
- `src/ui/index.html` — Muse page: date picker, sort toggle, digest button, settings popover, 7×N heatmap SVG.
- `src/ui/app.js` — `pageMuse()`: `dateOptions`, `sortedActivity`, `loadActivity/Heatmap/Digest/Config`, `saveSettings`, `renderHeatmap`, `toggleSettings`.

## Pending

- **Real `SetWinEventHook`** — replace 1s poll with Win32 event hook for true <100ms reaction. Requires adding `windows-sys` and writing a native callback that posts to an mpsc channel. Defer to v0.6.1.
- **Screenshot → Claude** — currently screenshots are saved to disk but not fed to the model. `claude -p` doesn't accept image attachments on the command line; would need to inline base64 via `--file` (still experimental). Defer to v0.6.2.
- **Widget settings popover** — settings currently live on the dashboard. The muse floating widget itself is still display-only. Add a click-to-expand popover in v0.6.3.

## Known issues

- 1s polling means 60 PowerShell spawns/min when watching. CPU cost is small but non-zero; debounce + smart dedup keep the markdown log sparse.
- Screenshot capture adds ~200-500ms per debounced entry. The watcher doesn't sleep between capture and summarize.
- `claude -p` calls for digest + summary still go to the cloud. For local-first mode, swap to a local model (llama.cpp or whisper-ggml) — bigger work.

## Migration

No DB or schema migration. Pull, rebuild, relaunch. New `[muse]` section in `config.toml` is added on first save; defaults match the new behavior.
