# Testing: unit

```bash
cd src-tauri
cargo test --lib
```

The watcher tests use a fake `mpsc` and assert on the markdown
output. The drainer tests use a `tempfile` vault and assert on
the SQLite row.
