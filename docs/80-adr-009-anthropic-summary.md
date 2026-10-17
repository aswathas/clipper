# ADR 009: Claude CLI for summary

**Status:** accepted.

**Context.** The watcher summarizes a foreground window title.
The choices are the Anthropic SDK, the Claude CLI, or a local
model.

**Decision.** `claude -p` with `CREATE_NO_WINDOW`.

**Consequences.** The user already has the CLI installed for
Ask. The shell-out is two flags; no SDK wiring. The downside
is the cold-start cost (2-3 s); the watcher is debounced, so
this is invisible.

**Alternatives.** SDK (more code, no behavior gain), local model
(privacy wins, accuracy cost).
