# Security: permissions

Windows capabilities the daemon requests:
- **Microphone** — only when `record_calls = true`. Required for
  capturing the user's voice on a call.
- **Accessibility** — `SetWinEventHook` does not require it on
  Windows 10+; we still request it because some UWP apps need it
  to fire events.
- **Filesystem** — read/write the vault folder. No broad access.

The Tauri capabilities are listed in
`src-tauri/capabilities/main.json`. We do not request any other
Windows capability.
