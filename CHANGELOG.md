# Changelog

## v0.4.0 — 2026-10-17

- fix(api): get_item returns full markdown body split by frontmatter
- feat(ui): 365-day contribution calendar on Today
- feat(ui): tag color map (12-color deterministic palette, WCAG AA)
- feat(ui): Cmd+K command palette (fuzzy search + recent + tags + quick-save)
- feat(ui): inline hover preview on cards (600ms)
- fix(selfcheck): align selfcheck with v0.3.0 `.fallback(ui_static)` routing
- fix(ui): sparkline via x-html (SVG <template> namespace bug)
- fix(ui): status=all treated as no-filter in list_items
- chore: color contrast tokens meet WCAG AA in light theme

## v0.3.2 — 2026-10-16

- fix(api): `get_item` returns markdown body from `vault_path`

## v0.3.1 — 2026-10-16

- fix(ui): sparkline via x-html (SVG-namespace <template> binding bug)
- fix(server): treat `status=all` as no-filter in list_items
- fix(ui): guard `editingSkill` null on init

## v0.3.0 — 2026-10-15 — "The Library"

- Headless Tauri daemon: tray opens dashboard in default browser at `http://127.0.0.1:7137`
- 6 pages: Today / Inbox / Library / Ask / Skills / Settings
- Impeccable redesign: warm-paper palette, Spectral + Inter + JetBrains Mono
- Custom SVG charts (sparkline, decay bars, type-color accent bar)
- 9 a11y fix commits
- DESIGN.md ships in repo

## v0.2.0 — 2026-10-15

- Tauri 2 desktop app with webview dashboard
- axum REST + SSE
- SQLite + FTS5
- Claude Code CLI as brain
- Drainer / queue / deadletter
- Per-slice self-checks behind `--<n>-smoke` flags

## v0.1.1 — 2026-10-15

- Tauri webview routing fix (frontendDist path)

## v0.1.0 — 2026-10-15

- Initial Tauri scaffolding
- Capture URL → SQLite → markdown file
- Mock dashboard
