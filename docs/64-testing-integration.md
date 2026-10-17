# Testing: integration

```bash
cd src-tauri
cargo build --release
./target/release/clipper.exe --smoke &
SERVER=http://127.0.0.1:7137
curl -s $SERVER/api/health
curl -s -X POST $SERVER/api/clip -d '{"url":"https://example.com"}'
curl -s "$SERVER/api/search?q=example"
```

`--smoke` skips the foreground watcher and the tray. Routes
return mock data behind `?mock=1` when the daemon is in smoke
mode.
