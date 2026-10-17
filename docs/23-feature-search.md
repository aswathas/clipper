# Feature: search

`/` opens the Cmd+K palette. Typing issues a GET to
`/api/search?q=...`. The handler queries SQLite FTS5 over
`title`, `body`, and `tags`. Results stream in as the user types.

Phases:
1. FTS5 keyword match.
2. (v0.9) Hybrid: FTS5 + vector recall, fused with reciprocal rank.
3. (v1) Optional re-rank with a small cross-encoder.
