# Release process

1. Bump `version` in `src-tauri/Cargo.toml`.
2. `cargo build --release` — confirm green.
3. Smoke test: launch, capture a URL, watch Muse, open dashboard.
4. `git tag vX.Y.Z` and `git push --tags`.
5. Write `RELEASE_NOTES_vX.Y.Z.md` with the diff summary.
6. Post the announcement.

We do not have a CI build. The maintainer runs the build, the
maintainer signs the tag. Until there is a real team, automation
adds more risk than it removes.
