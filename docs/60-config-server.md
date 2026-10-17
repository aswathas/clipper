# Config: server

```toml
[server]
bind = "127.0.0.1"
port = 7137
```

`port = 0` means "ask the OS for a free port". The chosen port is
written to the tray tooltip and the single-instance callback so
the dashboard URL is always right.
