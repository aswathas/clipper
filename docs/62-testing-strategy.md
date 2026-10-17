# Testing strategy

- **Unit** — every public function in `musewatch`, `drainer`,
  `capture`, `indexer`. No mocks of stdlib.
- **Integration** — boot the daemon in `--mock` mode and run a
  shell script that hits every route.
- **Manual smoke** — the maintainer runs the dashboard for 10
  minutes before any release.

Coverage target: 70% on `src-tauri/src`. The UI is not measured.
