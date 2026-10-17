# Dev loop

```
cargo run --release          # one terminal
code .                       # editor
http://127.0.0.1:7137/       # browser
```

Changes to a Rust file trigger a rebuild and a daemon restart. The
single-instance plugin brings the dashboard up in the browser
when the new build takes over the port. Hot-reload of the static
pages is `F5`.

If the build is locked (`clipper.exe` is the current process):
`taskkill //F //IM clipper.exe //T` (Git Bash mangles `/F`; use the
double-slash form).
