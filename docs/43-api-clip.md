# API: clip

- `POST /api/clip` — `{ url, title? }` returns `{ id, status }`.
- `GET /api/clips?from=&to=&host=&tag=&limit=` — list.
- `GET /api/clips/:id` — single row.
- `DELETE /api/clips/:id` — archive (soft).

`POST` dedupes by SHA-256 of the canonical URL. If the row exists,
the response is `200` with the existing id, not `201`.
