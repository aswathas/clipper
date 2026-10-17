# API: search

- `GET /api/search?q=&limit=` — FTS5 results.
- `GET /api/search?q=&mode=hybrid` — FTS5 + vector (v0.9).

Results are sorted by `bm25` score. `limit` defaults to 20, max
100.
