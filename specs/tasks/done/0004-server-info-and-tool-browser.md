---
id: 0004-server-info-and-tool-browser
status: done
scope: [src/app.rs, src/ui.rs, README.md]
created: 2026-10-13
source: User mcpv MVP brief; Rust 2024 and toolchain 1.99
depends: [0002-http-client-lifecycle, 0003-terminal-shell-and-status]
---
# Task: server info and tool browser

## Plan

- [x] Add visible 1 Tools / 2 Server tabs; Tools retains list/details panes and Server uses the full body width. Tab/Shift-Tab cycle visible panes and logs; switching tabs selects their initial body pane.
- [x] Show selected-tool name, full description, and pretty JSON inputSchema; show full server metadata, capabilities JSON, instructions, and errors in Server, with explicit missing-data labels.
- [x] Keep independent detail/server scrolling, reset details on selection changes, preserve selection by name on catalog replacement, and clear selection/details on reconnect. Preserve cached/offline states, logs, narrow layouts, and control-character filtering.
- [x] Test navigation, scrolling, selection/catalog/reconnect behavior and rendered states; update README, run project gates, and inspect a real-server TUI session.

## Context

Task 0003 owns terminal cleanup, event loop, basic list selection, status, and logs. This task adds detailed inspection to that established screen. Tool editing/invocation/results remain task 0006; prompts/resources tabs remain task 0007.

## Manual checks

- [x] Inspect real-server info, the complete tools list, and a long schema using both navigation styles. — agent: isolated tmux PTY against the stateless development MCP server; inspected all three tools, echo schema through its final brace, Server metadata/capability JSON/instructions label, arrows/j/k/g/G, logs and tab focus, and 100x24/50x12 layouts. TUI and server exited afterward.

## Summary

Expose server information and a two-pane tool/schema browser.

## Acceptance

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo test --all-targets --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`
- [x] `specdev check`

## Review

Approved against the agreed plan and three-file implementation scope. Existing SDK metadata/catalogs supply all detail views without client or CLI changes. Checked tab/log focus, selection by name, reconnect/stale events, scrolling and resize clamping, missing/error/offline states, terminal control filtering, and README keys. All 36 ordinary tests pass; the intentional panic probe remains ignored. Full quality gates and the local real-server PTY check passed. No dependencies or unwrap/expect calls added.

## Log

- 2026-10-13 created: server info and tool browser
- 2026-10-13 scope add src/app.rs — MVP implementation slice
- 2026-10-13 scope add src/ui.rs — MVP implementation slice
- 2026-10-13 scope add src/client.rs — MVP implementation slice
- 2026-10-13 set source User mcpv MVP brief; Rust 2024 and toolchain 1.99
- 2026-10-13 set depends 0002-http-client-lifecycle, 0003-terminal-shell-and-status
- 2026-10-15 scope rm src/client.rs
- 2026-10-15 scope add README.md — Document approved Tools/Server tabs and detail navigation
- 2026-10-16 advance draft → ready
- 2026-10-16 scope approved: src/app.rs, src/ui.rs, README.md
- 2026-10-16 advance ready → in-progress/implement
- 2026-10-16 advance in-progress/implement → in-progress/verify
- 2026-10-16 advance in-progress/verify → in-progress/review
- 2026-10-16 advance in-progress/review → approval
- 2026-10-16 approved: looks good !
- 2026-10-16 advance approval → done
