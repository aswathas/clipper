# Database: vault index

The drainer is not the only writer to the vault. The user can open
Obsidian and rename a file. The index must follow.

The indexer (`vaultwatch`) is a `notify` watcher on the vault
root. Every change re-parses the touched file, upserts the row,
and refreshes the FTS index. The drainer is the fast path; the
indexer is the safety net.
