# Feature: Ask

A chat surface in the dashboard. The model is selected from
`Settings -> Models`. The default is `claude-sonnet-4-5`.

Streams over Server-Sent Events. The client uses `getReader` to
read chunks. Sources are collapsible; the user can see exactly
which file the model read.

The model receives:
1. The system prompt from `Soul.md`.
2. The memory context from the `memory` table.
3. The user's question.
4. The set of available skills.
