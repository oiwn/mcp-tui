---
id: 0007-prompts-and-resources-tabs
status: draft
scope: [src/client.rs, src/app.rs, src/ui.rs]
created: 2026-10-13
source: User mcpv MVP brief; Rust 2024 and toolchain 1.99
depends: [0004-server-info-and-tool-browser, 0006-tool-editor-and-results]
---
# Task: prompts and resources tabs

## Plan

- [ ] Add list-only tabs using list_all_prompts/list_all_resources when advertised by server capabilities.
- [ ] Show names/descriptions and resource URIs with existing navigation; represent unsupported/empty/error states explicitly without breaking tools.
- [ ] Refresh catalogs after reconnect, reusing layout/events/colors and keeping network work asynchronous.

## Context

Cheap should-have scope only: no prompt execution, resource reading/subscriptions, or new transports/auth. If nontrivial, report the finding for a scope decision rather than expanding the MVP.

## Summary

Add capability-aware list-only prompts and resources tabs.

## Acceptance

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo test --all-targets`
- [ ] `specdev check`

## Log

- 2026-10-13 created: prompts and resources tabs
- 2026-10-13 scope add src/client.rs — MVP implementation slice
- 2026-10-13 scope add src/app.rs — MVP implementation slice
- 2026-10-13 scope add src/ui.rs — MVP implementation slice
- 2026-10-13 set source User mcpv MVP brief; Rust 2024 and toolchain 1.99
- 2026-10-13 set depends 0004-server-info-and-tool-browser, 0006-tool-editor-and-results
