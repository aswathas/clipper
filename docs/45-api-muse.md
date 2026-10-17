# API: muse (root)

- `GET /api/muse/state` — current state (`off` | `watching` |
  `paused`) + last 30 entries from today's activity log.
- `POST /api/muse/state` — `{ "state": "watching" }` toggles the
  watcher.

The watcher itself runs in the daemon process; this endpoint is
the only way the dashboard talks to it.
