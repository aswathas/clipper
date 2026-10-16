# Clipper v0.5.2 — Muse foreground watcher

## Highlights

- **Foreground watcher** (`src-tauri/src/musewatch.rs`) — polls the active window every 5s via PowerShell + Win32 P/Invoke (no new Rust deps). On a stable title for 2 ticks (10s), it shells out to `claude -p` for a 1-line summary and appends it to `<vault>/Activity/<YYYY-MM-DD>.md`. Skips screensaver/lockscreen/blank via the model's `SKIP` reply.
- **Toggle via the floating widget or the dashboard Muse page** — both write `muse_state.json` which the watcher reads each cycle.
- All 11 existing `--<n>-smoke` gates still green; 22 `smoke.ps1` checks still green.

## Deferred to v0.6

- `--muse-smoke` self-check gate (skipped this round due to scope pressure)
- Screenshot capture (PrintWindow / BitBlt of the foreground HWND) → Claude with image
- Global `Ctrl+.` kill switch (hard pause from any focus)
- Floating widget settings popover (screenshot on/off, pause timers, quit)

## Known issues

- `claude` CLI must be on `PATH` for summaries to land; if not, the watcher still records nothing (silent skip).
- The PowerShell spawn takes 1-2s each tick; in a 5s poll window the foreground can change twice before a new poll sees it. Acceptable for a passive note-taker.
- The watcher ignores the `screensaver`, `lockapp`, and empty foreground windows via the model's `SKIP` reply, not via a hard filter.
