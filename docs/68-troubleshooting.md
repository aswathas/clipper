# Troubleshooting

**Port 7137 is busy.** Edit `<config>/config.toml`, change
`server.port` to `0`, restart. The new port is in the tray
tooltip.

**The dashboard shows 0 items.** The vault may be empty. Press
`Ctrl+Shift+C` with a URL in the clipboard. If the tray tooltip
does not flash, the hotkey is owned by another app.

**The Muse watcher is silent.** Check `Ctrl+Shift+M` did not
pause it. Open Muse settings, confirm `skip_processes` does not
include the foreground app.

**A window pops up every second.** A previous Clipper build is
still running. `taskkill //F //IM clipper.exe //T`.
