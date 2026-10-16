# Clipper v0.3.0 — The Library

Impeccable redesign. Headless Tauri daemon. Calm library aesthetic.

## Highlights

- **Headless daemon** — no webview, no window. Tray opens the dashboard in your default browser at 127.0.0.1:7137 (`HAG-20`, commit `17741ed`).
- **Impeccable redesign** — warm-paper palette, Spectral serif headings, Inter body, JetBrains Mono. 6 pages: Today, Inbox, Library, Ask, Skills, Settings. Custom SVG charts, no chart library. Light + dark, system pref, no dropdown. Mobile bottom tab bar (`HAG-21`, 12 commits by Pam culminating in `f783e31`).
- **9 a11y fix commits** — heading order, button semantics on clickable cards/rows, aria labels, sparkline role, focus rings, inbox j/k roving tabindex, slideover tag pill button, color contrast to WCAG AA, 44px touch targets on mobile, baked type-icon aria-labels (commits `f42020b`…`a5abf5f`).
- **Version bump** — `f18a5b1` (package.json, Cargo.toml, tauri.conf.json).
- **Selfcheck alignment** — `30ee57f`. v0.3.0 server uses `.fallback(ui_static)`; selfcheck.rs now asserts that routing instead of the old explicit route table. 11/11 `--<n>-smoke` gates green, `smoke.ps1` green (22 checks).
- **Atomic commit hygiene** — every commit is a `git add` of only the files it needed (no `-A`).
- **DESIGN.md** — 500-line design spec, source of truth, ships in the repo.

## Upgrade notes

Drop-in. Existing SQLite vault unchanged. Same API contract.

## Known issues

- Desktop theme toggle moved into the in-page Settings panel; the rail-bottom toggle is gone on viewports ≥ 900px (intentional, per audit).
- The HEADLESS_BUILD warning from GNU `ld` about multiple non-default manifests is benign (Tauri resource manifests).
