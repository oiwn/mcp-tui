---
id: 0006-tool-editor-and-results
status: approval
scope: [src/editor.rs, src/result.rs, src/app.rs, src/ui.rs, src/client.rs, src/error.rs, src/main.rs, src/terminal.rs, README.md, tests/http_client.rs]
created: 2026-10-13
source: User mcpv MVP brief; Rust 2024 and toolchain 1.99
depends: [0002-http-client-lifecycle, 0003-terminal-shell-and-status, 0004-server-info-and-tool-browser, 0005-schema-json-skeleton]
---
# Task: tool editor and results

## Plan

- [x] Add full-body editor/result screens: Enter edits, Esc saves drafts/returns to browsing, v reopens the latest result, and Enter from results edits that tool. Retain per-tool drafts until reconnect/exit.
- [x] Support Unicode-safe multiline editing, arrows/Home/End, Tab indentation, bracketed paste, visible cursor on scroll/resize, Ctrl-R skeleton reset, and Ctrl-U {}. Ordinary shortcut characters remain text; Ctrl-C exits.
- [x] F5/Ctrl-Enter validates JSON object arguments up to 1 MiB and submits asynchronously; preserve invalid buffers with local errors. Enforce one active call without a submission queue and keep browsing/logs/reconnect/quit responsive.
- [x] Add typed call IDs and generation-tagged started/completed events with a configurable 10-second call deadline; cancel local work on reconnect/shutdown and reject stale/mismatched responses. Preserve headless contracts.
- [x] Render ordered text blocks (pretty JSON or plain text), structuredContent, explicit empty/non-text labels, and safe terminal text. Label Some(true) tool-content failures red; distinguish execution failures and never retry automatically.
- [x] Track dispatched counts and completion latency including failures; show selected-tool metrics and last-call status. Clear drafts/results/metrics on reconnect, test editor/results/worker behavior, update README, run gates, and verify the live PTY flow.

## Context

Retain only the latest result, not history. Returning to browsing leaves a call running; completion never steals focus. The editor hides the activity drawer temporarily. Integration scope includes module wiring, terminal paste lifecycle, README, and HTTP worker tests as one coherent feature.

## Manual checks

- [x] Edit/send with F5, reject malformed JSON in-UI, and display a success plus red tool-content failure. — agent: local stateless fixture in isolated tmux; malformed } stayed editable with a JSON error, ping {} succeeded, Unicode bracketed paste and echo failures rendered correctly. Captures without NO_COLOR showed red ANSI styling and TOOL ERROR; NO_COLOR retained the label.
- [x] Browse during a slow call, reopen results, reconnect, and inspect narrow editor/result layouts. — agent: Server stayed responsive during a 5-second slow call and remained open on completion; v reopened the result. Echo draft reopened unchanged, reconnect reset metrics/results, and 110x28/50x12 captures showed full-body editor/results. Test TUI sessions and fixture stopped afterward.

## Summary

Call tools from a JSON editor and display results/failures with timing.

## Acceptance

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo test --all-targets --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`
- [x] `specdev check`

## Review

Approved against the plan. All 60 ordinary tests pass; the intentional panic probe remains ignored. Reviewed draft/input handling, full-body rendering, terminal paste restoration, call identity/generation checks, the one-call permit, elapsed/count metrics, bounded cancellation, and unchanged headless behavior. Integration tests cover both session modes, content/protocol/transport failures, timeout recovery, duplicate rejection, and reconnect/shutdown while pending. Review corrected browser focus after results/reconnect and pending completion on unexpected worker-channel closure. No dependencies or unwrap/expect calls added. Warnings: the justified ten-file integration scope and existing future docs/demo.md.

## Log

- 2026-10-13 created: tool editor and results
- 2026-10-13 scope add src/editor.rs — MVP implementation slice
- 2026-10-13 scope add src/result.rs — MVP implementation slice
- 2026-10-13 scope add src/app.rs — MVP implementation slice
- 2026-10-13 scope add src/ui.rs — MVP implementation slice
- 2026-10-13 scope add src/client.rs — MVP implementation slice
- 2026-10-13 scope add src/error.rs — MVP implementation slice
- 2026-10-13 set source User mcpv MVP brief; Rust 2024 and toolchain 1.99
- 2026-10-13 set depends 0002-http-client-lifecycle, 0003-terminal-shell-and-status, 0004-server-info-and-tool-browser, 0005-schema-json-skeleton
- 2026-10-16 scope add src/main.rs — Approved editor/result module wiring, bracketed-paste restoration, usage documentation, and asynchronous call integration tests
- 2026-10-16 scope add src/terminal.rs — Approved editor/result module wiring, bracketed-paste restoration, usage documentation, and asynchronous call integration tests
- 2026-10-16 scope add README.md — Approved editor/result module wiring, bracketed-paste restoration, usage documentation, and asynchronous call integration tests
- 2026-10-16 scope add tests/http_client.rs — Approved editor/result module wiring, bracketed-paste restoration, usage documentation, and asynchronous call integration tests
- 2026-10-16 advance draft → ready
- 2026-10-16 scope approved: src/editor.rs, src/result.rs, src/app.rs, src/ui.rs, src/client.rs, src/error.rs, src/main.rs, src/terminal.rs, README.md, tests/http_client.rs
- 2026-10-16 advance ready → in-progress/implement
- 2026-10-16 advance in-progress/implement → in-progress/verify
- 2026-10-16 advance in-progress/verify → in-progress/review
- 2026-10-16 advance in-progress/review → approval
