---
id: 0006-tool-editor-and-results
status: draft
scope: [src/editor.rs, src/result.rs, src/app.rs, src/ui.rs, src/client.rs, src/error.rs]
created: 2026-10-13
source: User mcpv MVP brief; Rust 2024 and toolchain 1.99
depends: [0002-http-client-lifecycle, 0003-terminal-shell-and-status, 0004-server-info-and-tool-browser, 0005-schema-json-skeleton]
---
# Task: tool editor and results

## Plan

- [ ] Enter opens a multiline JSON editor with a schema skeleton or {}; support cursor movement, insertion/deletion, and Esc back to browsing.
- [ ] Ctrl-Enter/F5 parses an object and sends CallToolRequestParams::new(name).with_arguments(parsed_json); show syntax/type errors locally and send nothing invalid.
- [ ] Display all response text blocks, pretty-printing valid JSON with scroll support; handle empty/non-text responses gracefully.
- [ ] Render is_error Some(true) distinctly in red with an error label; keep protocol/transport failures separate in status/logs without crashing.
- [ ] Track per-tool dispatched-call counts and completed-call latency, including failures; test invalid input, Some(true)/Some(false)/None, and JSON/plain/multiple text blocks.

## Context

Editor input must preserve j/k/g/G/l/r as text. F5 is the portable submit key. Allow an explicit {} for the demo. Ignore stale results after reconnect and keep the UI responsive while calls run.

## Manual checks

- [ ] Edit/send with F5, reject malformed JSON in-UI, and display a success plus red tool-content failure.

## Summary

Call tools from a JSON editor and display results/failures with timing.

## Acceptance

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo test --all-targets`
- [ ] `specdev check`

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
