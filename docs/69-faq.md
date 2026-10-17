# FAQ

**Why headless?** A desktop app that owns a window has to fight
the window manager. A daemon that serves a URL gets the user's
browser for free, with their extensions and their fonts.

**Why markdown?** The user owns the file. They can grep, rsync,
edit in any editor. The drainer writes the file, the indexer
reads it. There is no second source of truth.

**Why SQLite?** One file. The user can copy it. `sqlite3` is on
every dev machine. FTS5 is built in. Vector recall is a v0.9
addition.

**Why `claude -p` instead of the SDK?** The CLI is what the
user already has installed. The shell-out is `CREATE_NO_WINDOW`
so there is no popup. The SDK is a post-v1 consideration.
