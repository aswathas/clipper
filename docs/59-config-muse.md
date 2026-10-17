# Config: muse

```toml
[muse]
skip_processes = ["LockApp.exe", "ShellExperienceHost.exe"]
smart_dedup = true
debounce_secs = 30
screenshots = true
record_calls = false
capture_whatsapp = false
```

`skip_processes` is matched by exact basename. `smart_dedup`
groups by `(process, normalized_title)`. `debounce_secs` is the
quiet period before a new title is logged.
