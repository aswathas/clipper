# ADR 003: SSE for chat

**Status:** accepted.

**Context.** The chat surface streams tokens from the model.
Candidates are WebSocket, SSE, and long-poll.

**Decision.** Server-Sent Events.

**Consequences.** One-way, plain HTTP, works through every
proxy, easy to debug with `curl`. The downside is no
client-to-server streaming beyond the initial POST; we do not
need it for now.

**Alternatives.** WebSocket (heavier, harder to proxy), long-poll
(works but is slow).
