# Error codes

- `400` — bad input (schema mismatch, unknown field)
- `401` — missing pairing token
- `403` — token wrong or rotated
- `404` — id not found
- `409` — duplicate hash (dedup)
- `412` — `If-Match` mtime stale
- `422` — markdown parse error
- `500` — daemon panic; the response body has the cause

Every 500 is also written to `<vault>/.cache/clipper.log`.
