# Feature: Muse

Muse is the second brain's peripheral vision. It watches the
foreground window, asks `claude -p` to summarize the title, and
writes a one-line entry to `<vault>/Activity/<date>.md`:

```
14:23  Visual Studio Code — Cargo.toml   editing deps
14:31  WhatsApp — Papa                    call
14:33  Notion — Q4 OKRs                   drafting
```

The watcher uses a 1 s debounce per `(process, normalized_title)`
group. A title that does not change for 30 s gets a single entry.

The dashboard reads the file. The daily digest asks `claude -p`
to summarize the whole day. The weekly review (planned) does
the same for a week.
