# API: soul

- `GET /api/soul` — current `Soul.md` (voice + persona).
- `PUT /api/soul` — replace. The body is markdown; the server
  splits on the first `##` heading.

The file is the source of truth; the API is a thin wrapper.
