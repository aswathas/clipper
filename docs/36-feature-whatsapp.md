# Feature: WhatsApp capture

Toggle `capture_whatsapp` in Muse settings. While on, a hidden
message-only window listens for `WM_CLIPBOARDUPDATE`. When the
foreground process is `whatsapp.exe` and the clipboard changed,
the listener appends to `<vault>/WhatsApp/<date>.md`:

```markdown
## 14:31 · Papa

> don't forget the keys

---
```

The contact name is read from the foreground window title. The
listener uses `AddClipboardFormatListener` (Windows 8+).
