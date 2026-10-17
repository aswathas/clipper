# Architecture: indexer

A `notify` watcher on the vault root. The drainer's writes flow
through the same `notify` events as a hand edit. Both paths go
through the same `parse + index` step. The result: an out-of-band
edit (the user opened Obsidian and tweaked a note) is reflected
in the dashboard without a restart.
