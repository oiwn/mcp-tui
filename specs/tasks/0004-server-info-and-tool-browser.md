---
id: 0004-server-info-and-tool-browser
status: draft
scope: [src/app.rs, src/ui.rs, src/client.rs]
created: 2026-10-13
source: User mcpv MVP brief; Rust 2024 and toolchain 1.99
depends: [0002-http-client-lifecycle, 0003-terminal-shell-and-status]
---
# Task: server info and tool browser

## Plan

- [ ] Extend task 0003's server summary into a scrollable Server view with full advertised capabilities and server instructions, retaining identity/protocol/count information.
- [ ] Reuse the existing complete tools list and replace its summary-only right pane with selected-tool full description and pretty JSON inputSchema; provide Tools/Server view switching.
- [ ] Reuse focus and arrows/j/k/g/G navigation to scroll details independently, handle missing descriptions/empty catalogs, and preserve valid selection after refresh.

## Context

Task 0003 owns terminal cleanup, event loop, basic list selection, status, and logs. This task adds detailed inspection to that established screen. Tool editing/invocation/results remain task 0006; prompts/resources tabs remain task 0007.

## Manual checks

- [ ] Inspect real-server info, the complete tools list, and a long schema using both navigation styles.

## Summary

Expose server information and a two-pane tool/schema browser.

## Acceptance

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo test --all-targets`
- [ ] `specdev check`

## Log

- 2026-10-13 created: server info and tool browser
- 2026-10-13 scope add src/app.rs — MVP implementation slice
- 2026-10-13 scope add src/ui.rs — MVP implementation slice
- 2026-10-13 scope add src/client.rs — MVP implementation slice
- 2026-10-13 set source User mcpv MVP brief; Rust 2024 and toolchain 1.99
- 2026-10-13 set depends 0002-http-client-lifecycle, 0003-terminal-shell-and-status
