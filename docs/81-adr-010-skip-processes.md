# ADR 010: skip-processes allowlist

**Status:** accepted.

**Context.** The watcher would otherwise log every focus
change, including the lock screen and shell windows.

**Decision.** A configurable `skip_processes` list. Defaults:
`LockApp.exe`, `ShellExperienceHost.exe`.

**Consequences.** The user can grow the list as they find more
noise. The watcher checks the basename, not the path.
