# API: pairing

- `GET /api/pair/qr` — SVG QR for the current token.
- `POST /api/pair` — `{ "token": "..." }` returns the session
  cookie.

The token rotates every hour. A session is valid until the next
rotation; the rotation middleware re-issues a new cookie on the
next request.
