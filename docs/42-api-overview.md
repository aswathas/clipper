# API overview

All routes are under `/api/`. The static dashboard is at `/`. The
pairing QR is at `/qr`.

Routes return JSON. Errors return `{ "error": "<message>" }` with
the right status code. The pairing middleware allows `/`, `/qr`,
`/pair`, and the SSE stream without a token.

Versioning is by URL prefix; `v0` is implicit.
