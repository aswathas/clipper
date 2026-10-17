# Build

```
cd src-tauri
cargo build --release
./target/release/clipper.exe
```

The first build is slow (~6 min) because of `tauri` + `windows-sys`
+ `rusqlite` + `reqwest`. Subsequent builds are incremental.

Cross-compile is not supported. Clipper is a Windows desktop
product; macOS and Linux are out of scope.
