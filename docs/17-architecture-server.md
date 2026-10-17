# Architecture: server

`axum` on `127.0.0.1:7137`. The bind port is configurable; `0`
asks the OS for a free port. The real port is published to the
tray and the single-instance callback.

Routes are organized by feature (`/api/muse/...`, `/api/clip/...`,
`/api/memory/...`). The pairing middleware is the only one
required: a request without a valid token is rejected unless the
route is in the allow list (the static pages, the pair endpoint,
and the SSE stream after handshake).
