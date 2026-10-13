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

- [ ] Render server name/version, negotiated protocol, capabilities, and complete tool count from SDK events.
- [ ] List every tool on the left with name and one-line description; support absent descriptions and empty catalogs.
- [ ] Navigate with arrows and j/k/g/G; provide explicit focus switching and scroll the full description/pretty JSON inputSchema on the right; preserve valid selection after refresh.

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
