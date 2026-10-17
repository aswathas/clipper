# Clipper v0.6.1 — true hook, real vision, expandable widget

## Highlights

- **Win32 `SetWinEventHook`** (`64ed436`, `musehook.rs`) — native `EVENT_SYSTEM_FOREGROUND` callback posts HWND changes to an mpsc channel. The watcher now reacts in <50ms via `recv_timeout(1s)` instead of polling. The 1s poll remains as a safety net for windows the hook misses.
- **Multimodal screenshots** — when `cfg.muse.screenshots=true` and `ANTHROPIC_API_KEY` is set, each new entry is summarized via the Anthropic Messages API with the captured PNG as a base64 image content block. Model defaults to `claude-sonnet-4-5`; override with `MUSE_VISION_MODEL`. Falls back to the `claude -p` CLI when the API is unavailable.
- **Expandable widget popover** (`f59ea71`) — click the floating pill to grow the Tauri window from 240×72 to 320×440 and reveal: status + Watch/Pause, last 6 activity entries, quick settings (smart_dedup, screenshots), Open Dashboard, and the Ctrl+Shift+M hint. Click outside or the × to shrink back. Uses `plugin:window|resize` via the global Tauri API.

## Files

- `src-tauri/Cargo.toml` — `windows-sys 0.59`, `reqwest 0.12` (blocking+rustls), `base64 0.22`.
- `src-tauri/src/musehook.rs` (new) — native event hook.
- `src-tauri/src/musewatch.rs` — `recv_timeout` over the hook + 1s poll safety net; `summarize_with_image()` and `call_anthropic_with_image()`.
- `src-tauri/src/lib.rs` — calls `musehook::spawn()` and passes the receiver to `musewatch::spawn_with_hook()`.
- `src/ui/muse.html`, `muse.js`, `muse.css` — expandable popover.

## Pending

- **Position-aware resize** — the popover currently grows the Tauri window but doesn't adjust `(x, y)`, so it expands toward the top-left of the screen rather than staying bottom-right anchored. Cheap fix: subtract the size delta from x and y in the same `plugin:window|set_position` call.
- **Drag-to-move** — the popover has no drag handle. Tauri's `set_position` is exposed via the global Tauri API; bind it to mousedown on the popover header.
- **Animations** — popover snaps open/closed; a 150ms fade+scale would feel friendlier.
- **Background tint** — current transparent WebView lets the desktop bleed through; for a Cute Grok-bot feel, give the popover a soft pastel gradient (peach → cream) with a mascot that has eyes.
