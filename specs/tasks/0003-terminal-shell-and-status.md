---
id: 0003-terminal-shell-and-status
status: draft
scope: [src/terminal.rs, src/main.rs, src/app.rs, src/ui.rs, src/error.rs]
created: 2026-10-13
source: User mcpv MVP brief; Rust 2024 and toolchain 1.99
depends: [0001-mcpv-foundation, 0002-http-client-lifecycle]
---
# Task: terminal shell and status

## Plan

- [ ] Add terminal guard and panic hook that restore raw mode, alternate screen, and cursor before diagnostics, including partial startup failures.
- [ ] Build a responsive event loop with resize, quit, r reconnect, explicit focus/modes, arrows and j/k/g/G for read-only scrolling.
- [ ] Render connection state, URL, last-call latency, and a bounded scrollable log toggled with l; handle narrow windows and empty panes.
- [ ] Use terminal-default backgrounds and readable selection/error labels for dark and light terminals.

## Manual checks

- [ ] Resize and exit normally, then induce a controlled panic and confirm terminal restoration.
- [ ] Connect to an unavailable URL and verify responsive status/log failure handling without a panic.

## Summary

Add a resilient terminal shell with status, logs, and navigation.

## Acceptance

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo test --all-targets`
- [ ] `specdev check`

## Log

- 2026-10-13 created: terminal shell and status
- 2026-10-13 scope add src/terminal.rs — MVP implementation slice
- 2026-10-13 scope add src/main.rs — MVP implementation slice
- 2026-10-13 scope add src/app.rs — MVP implementation slice
- 2026-10-13 scope add src/ui.rs — MVP implementation slice
- 2026-10-13 scope add src/error.rs — MVP implementation slice
- 2026-10-13 set source User mcpv MVP brief; Rust 2024 and toolchain 1.99
- 2026-10-13 set depends 0001-mcpv-foundation, 0002-http-client-lifecycle
