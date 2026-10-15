//! Cross-cutting API contract gates for Clipper (HAG-12).
//!
//! These lock the contracts that every later slice builds on:
//!
//! * `saved_at` is Unix **seconds**, returned as a JSON *number* (the frontend
//!   multiplies by 1000 before formatting)
//! * the FTS5 external-content triggers stay in sync across insert/update/delete
//! * the indexer populates `items.type` from the note frontmatter
//! * the CORS allow-list is closed (no foreign origin gets a CORS header)
//! * the API binds loopback only
//! * hostile FTS query input cannot 500 the API
//!
//! The assertions live in the library as
//! [`clipper_lib::selfcheck::integration_api`], so there is exactly one
//! implementation. This file is the `cargo test` entry point.
//!
//! NOTE: on this project's Windows GNU toolchain a Rust test-harness binary
//! that links the Tauri stack cannot be **loaded**
//! (`STATUS_ENTRYPOINT_NOT_FOUND`), so `cargo test` cannot execute this file
//! here. Use the equivalent binary flag, which runs the identical assertions:
//!
//! ```text
//! clipper.exe --integration-smoke
//! ```
//!
//! `scripts/smoke.ps1` does not invoke either; this is a developer gate.

#[test]
fn api_contracts_hold() {
    assert_eq!(
        clipper_lib::selfcheck::integration_api(),
        0,
        "cross-cutting API contract gates failed — run `clipper.exe --integration-smoke` for detail"
    );
}
