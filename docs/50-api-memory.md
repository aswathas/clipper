# API: memory

- `GET /api/memory` — JSON view of `<vault>/Memory/user.md`.
- `PUT /api/memory` — full replace, with `If-Match` mtime check.
- `GET /api/memory/people` — list of `People` entries.
- `POST /api/memory/people` — add a person.

The file is the source of truth. The table is a cache.
