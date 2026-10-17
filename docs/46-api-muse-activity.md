# API: muse/activity

- `GET /api/muse/activity?date=YYYY-MM-DD&limit=N` — entries
  from the day's markdown, newest first.

The response is a JSON array of:
```json
{
  "time": "14:23",
  "title": "Cargo.toml",
  "process": "Code",
  "summary": "editing deps",
  "category": "dev",
  "tags": ["rust", "clipper"]
}
```

`date` defaults to today (local). `limit` defaults to 200.
