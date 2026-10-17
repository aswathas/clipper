# Observability

- Logs: `println!` in the daemon, visible in the launching
  terminal. The packaged build writes to
  `%APPDATA%/Clipper/clipper.log`.
- Stats: `<vault>/.cache/stats.json` — daily counts, top hosts,
  top tags. Refreshed at startup and at 03:00 local.
- Errors: tray tooltip flashes the message; the dashboard's
  `/api/self/last` exposes the last index run.

There is no remote telemetry. There is no event upload.
