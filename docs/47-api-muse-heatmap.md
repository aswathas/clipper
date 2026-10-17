# API: muse/heatmap

- `GET /api/muse/heatmap?days=84` — daily counts for the last N
  days. The dashboard renders this as an SVG grid.

Response:
```json
{ "days": [
  { "date": "2026-10-01", "count": 47 },
  ...
] }
```

`days` defaults to 84 (12 weeks). Max 365.
