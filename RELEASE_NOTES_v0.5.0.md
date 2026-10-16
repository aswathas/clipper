# Clipper v0.5.0 — Mass import · Muse scaffold · Connections

## Highlights

- **Mass import** — `POST /api/import` accepts a JSON array `{items: [{url, title?, tags?}]}`, dedups against the SQLite `items` table, and enqueues new URLs through the existing drainer. A new **Import** tab in Settings shows a JSON textarea, parse, and import-with-progress. Use this to bulk-load Pocket / Chrome bookmarks / Notion exports by converting them to the same JSON shape first.
- **Muse scaffold** — new `Muse` page in the dashboard with start/stop toggle, status badge, recent-activity feed, and a "what's coming" panel. State persists to `muse_state.json` via `GET/POST /api/muse/state`. The floating transparent widget, Win32 foreground hook, screenshot capture, and Claude-summarize pipeline are deferred to v0.6 — the dashboard surface is live now so the story is complete.
- **Wikilink interlinking** — `GET /api/concepts` returns the top 200 tags by item count. `GET /api/items/{id}/related` returns up to 12 items sharing any of the item's tags, ordered by recency. The slide-over's **Related** section now fetches live data instead of the mock list.
- **Docs** — `CHANGELOG.md`, `docs/ARCHITECTURE.md`, README "What's new" section for v0.4.0.

## What v0.5 does NOT include

- The floating Tauri widget window (transparent, always-on-top) — risky to ship in one commit. Deferred to v0.6.
- Win32 foreground hooks + screenshot capture + Claude-CLI summarize — depends on the widget window. Deferred to v0.6.
- Body-text [[wikilink]] rewriting on capture — entity extraction during note generation. Deferred to v0.6.
- The top-5 brainstorm UI ideas (hover preview, command-palette quick-save, FTS5 vault search, etc.) — already shipped in v0.4.0.

## Upgrade notes

Drop-in. No migration. The new `muse_state.json` is created on first toggle. The Import tab is a no-op until you paste JSON.

## Known issues

- Import dedups against the `items` table only, not the pending queue. A second import of the same URL while the first is still in the queue will create a duplicate after the drainer runs. Manual dedup is needed for now.
- The 5 brainstorm UI ideas from the v0.5 design plan are already in v0.4.0, not v0.5.0.
