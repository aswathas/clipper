# Contributing

## One agent, one branch
Open a branch per task. Merge to `master` immediately when green.
Branches that drift cost more than they buy.

## Commit message style
```
feat(muse): drag-to-move the popover
fix(clip): race in dedup when title contains the URL
docs: adr-007 pick whisper base.en
chore: bump tauri to 2.0.4
```

## Before opening a PR
- `cargo build --release` is green.
- The dashboard loads at `http://127.0.0.1:7137/` without console
  errors.
- Any new route is documented under `docs/`.
- Any new dependency is justified in the next ADR.

## Code review
- One reviewer. One round. We do not chase perfection.
- Style: `cargo fmt`, `cargo clippy -- -D warnings`.
