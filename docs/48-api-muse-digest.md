# API: muse/digest

- `GET /api/muse/digest?date=YYYY-MM-DD` — the day's one-line
  digest as written by `claude -p`. Cached for 6 hours.

The endpoint shells out to `claude -p` with
`CREATE_NO_WINDOW`. On a fresh install the first call may take
2-3 s while the CLI warms up.
