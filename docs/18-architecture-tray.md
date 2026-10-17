# Architecture: tray

Single icon. Menu: Show dashboard, Show QR, Show muse, Hide muse,
Quit. The icon is the only user surface on the daemon — every
other action is a hotkey, a URL bar, or the dashboard.

The tray tooltip is the live status line. If the daemon is
healthy, it shows `Clipper — 192.168.1.42:7137`. If the bind
failed, it shows the error. The tooltip is the heartbeat.
