---
id: 0003-terminal-shell-and-status
status: done
scope: [src/terminal.rs, src/main.rs, src/app.rs, src/ui.rs, src/error.rs, README.md, Cargo.toml, Cargo.lock, tests/http_client.rs]
created: 2026-10-13
source: User mcpv MVP brief; Rust 2024 and toolchain 1.99
depends: [0001-mcpv-foundation, 0002-http-client-lifecycle]
---
# Task: terminal shell and status

## Plan

- [x] Add terminal guard and panic hook that restore raw mode, alternate screen, and cursor before diagnostics, including partial startup failures; use bounded client shutdown on exit.
- [x] Replace summary-and-exit with a persistent responsive TUI event loop using the existing worker/state reducer; handle resize, q/Ctrl-C, r reconnect, Tab/Shift-Tab focus, and arrows plus j/k/g/G in read-only panes.
- [x] Render a compact server header and two-pane body: all fetched tools with names/one-line descriptions on the left, connection/server summary on the right; maintain valid selection and scroll positions.
- [x] Add a connection/status line, context-sensitive key hints, and a bounded timestamped activity drawer toggled with l; keep failures visible and reconnect available without closing the UI.
- [x] Support loading/unsupported/empty/failed catalogs, narrow terminals, and missing metadata with terminal-default colors and explicit labels; show last-call latency as unavailable until calls exist.
- [x] Test navigation/focus, stale reconnect state, log bounds, and rendering of empty/error/narrow screens; update README and run all quality gates.

## Screen and interaction

- **Header** — server name/version when known, negotiated protocol, and compact advertised capability labels. Unknown values remain explicit during connection.
- **Tools pane** — name and one-line description for every fetched tool, selected-row highlighting, and tool count. Initial focus is here; absent descriptions use the name alone.
- **Summary pane** — full server URL, current connection state, server identity/protocol, catalog state/count, and the latest failure. Full instructions/capability JSON and selected-tool schemas arrive in task 0004.
- **Activity drawer** — l opens/closes a bottom pane with up to 1000 lifecycle/catalog/error entries and elapsed-time timestamps. New entries follow the tail unless the user has scrolled away; G returns to the tail. Focus moves into the drawer on open and back to the prior visible pane on close.
- **Status and hints** — connection state, URL, last-call latency (— for now), and only currently available actions. Tab/Shift-Tab cycles visible panes; arrows and j/k move/scroll, g/G go first/last, r reconnects, q quits, Ctrl-C exits 130 after cleanup. Editor/result modes are later work.
- **Resize** — use a 40%/60% split at width >=80 and height >=16; smaller screens show the focused body pane, with focused logs replacing it below height 16. Drawer height is one-third of the body, clamped to 4–10 rows. Below width 32 or height 8, show a resize hint while global controls remain active. Truncate rows/status safely; retain the full URL in the scrollable summary.

## Context

Reuse task 0002's typed generation-tagged events and ClientState; call begin_generation immediately after reconnect submission. Clear old connection/catalog selection when a new generation starts; never apply late results from the previous connection. An unavailable server keeps the TUI open with a readable error and reconnect hint. Unsupported capabilities are not empty catalogs; count 0 only for a successfully fetched empty list. Retain fetched tools/metadata after disconnect and label them cached/offline until reconnect begins.

Logs contain client lifecycle/catalog events; no server logging, JSON-RPC tracing, heartbeat, or automatic reconnect. Hide unavailable schema/edit/call/result/tab controls. Pair color with focus/error labels.

## Manual checks

- [x] Resize and exit normally, then induce a controlled panic and confirm terminal restoration. — agent: isolated tmux PTYs at 100x24, 50x12, and 31x7; stty -g matched before/after q (0), Ctrl-C (130), and the ignored panic probe (expected 101), with panic diagnostics on the restored screen.
- [x] Connect to an unavailable URL and verify responsive status/log failure handling without a panic. — agent: port 1 and stopped port 7400 remained in the UI with error summary/logs; navigation, log toggle, reconnect, and quit worked. Red ANSI error styles were verified with NO_COLOR absent; labels remained visible with NO_COLOR=1.
- [x] Use the development server to inspect/navigate all three tools, change focus, scroll/toggle logs, and reconnect after server restart; check dark/light and narrow layouts. — agent: inspected real PTY captures, exercised j/G/Tab/l and g/G logs, stopped/restarted the fixture and recovered with r; visually inspected captured terminal cells on dark/light defaults and checked narrow/tiny captures.

## Summary

Add a persistent MCP terminal screen with basic tool browsing, connection status, logs, and reliable cleanup.

## Acceptance

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `cargo test --all-targets`
- [x] `specdev check`

## Review

Approved against the plan. All 24 ordinary tests and cargo build --locked pass; the ignored panic probe was verified separately. The worker API is unchanged. Async input uses crossterm event-stream/futures-util; wrapped scrolling uses ratatui rendered-line-info. No unwrap/expect was added. Review corrected narrow key hints, disconnected catalog labels, and drawing after panic restoration. Isolated PTYs/rendered captures replaced unavailable native Terminal access; test processes were stopped. Remaining warnings: justified nine-file scope and future docs/demo.md.

## Log

- 2026-10-13 created: terminal shell and status
- 2026-10-13 scope add src/terminal.rs — MVP implementation slice
- 2026-10-13 scope add src/main.rs — MVP implementation slice
- 2026-10-13 scope add src/app.rs — MVP implementation slice
- 2026-10-13 scope add src/ui.rs — MVP implementation slice
- 2026-10-13 scope add src/error.rs — MVP implementation slice
- 2026-10-13 set source User mcpv MVP brief; Rust 2024 and toolchain 1.99
- 2026-10-13 set depends 0001-mcpv-foundation, 0002-http-client-lifecycle
- 2026-10-14 scope add README.md — Document persistent TUI launch, navigation, reconnect, and log controls
- 2026-10-14 scope add Cargo.toml — Approved async terminal input dependencies and replacement of summary-only subprocess checks
- 2026-10-14 scope add Cargo.lock — Approved async terminal input dependencies and replacement of summary-only subprocess checks
- 2026-10-14 scope add tests/http_client.rs — Approved async terminal input dependencies and replacement of summary-only subprocess checks
- 2026-10-14 advance draft → ready
- 2026-10-14 scope approved: src/terminal.rs, src/main.rs, src/app.rs, src/ui.rs, src/error.rs, README.md, Cargo.toml, Cargo.lock, tests/http_client.rs
- 2026-10-14 advance ready → in-progress/implement
- 2026-10-14 advance in-progress/implement → in-progress/verify
- 2026-10-14 advance in-progress/verify → in-progress/review
- 2026-10-14 advance in-progress/review → approval
- 2026-10-16 approved: let's close 0003
- 2026-10-16 advance approval → done
