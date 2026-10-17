# API: chat (SSE)

- `POST /api/chat/stream` — `{ "messages": [...] }` returns a
  text/event-stream.

Events:
- `message` — token delta
- `tool_call` — model wants to invoke a tool
- `tool_result` — local result for the call
- `done` — stream finished
- `error` — terminal error

The model is selected from `Settings -> Models`.
