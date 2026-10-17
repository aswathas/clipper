# Security overview

Threat model: a malicious file on the user's disk and a malicious
web page in the browser. The daemon trusts neither.

- The static pages are served with a strict CSP that forbids
  inline scripts and remote origins.
- Markdown from the vault is rendered through DOMPurify. User
  content can be markdown; it cannot be script.
- WASAPI loopback captures the default render device. There is
  no synthetic sink.
- The pairing token is bound to the LAN IP. A request from a
  different origin gets a 403.
