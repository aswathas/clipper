# Feature: phone pairing

The tray menu has "Show QR". Click it, point the phone camera.
The QR encodes a one-time token; the phone opens
`http://<lan-ip>:7137/pair?token=...`. From then on the phone
session carries the token as a cookie. Tokens rotate every hour.

The LAN IP is detected at startup; the tooltip shows it. If the
laptop moves networks, the next tray hover re-derives it.
