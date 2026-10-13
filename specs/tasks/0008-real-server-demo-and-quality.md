---
id: 0008-real-server-demo-and-quality
status: draft
scope: [examples/dev_server.rs, tests/http_client.rs, docs/demo.md, Cargo.toml, Cargo.lock, README.md]
created: 2026-10-13
source: User mcpv MVP brief; Rust 2024 and toolchain 1.99
depends: [0002-http-client-lifecycle, 0003-terminal-shell-and-status, 0004-server-info-and-tool-browser, 0005-schema-json-skeleton, 0006-tool-editor-and-results, 0007-prompts-and-resources-tabs]
---
# Task: real server demo and quality

## Plan

- [ ] Reuse the dev-server fixture delivered by task 0009 for the final demo; make only scoped fixture adjustments needed by acceptance findings.
- [ ] Run focused HTTP integration tests for handshake/catalog, success/content failure, stop/request failure, restart/reconnect, and missing Mcp-Session-Id.
- [ ] Run all quality gates on toolchain 1.99.0, review typed errors and unwrap/expect usage, and record reproducible live TUI demo evidence in docs/demo.md.
- [ ] Update README launch/key/error/demo instructions; fix any findings through scoped revisions.

## Manual checks

- [ ] Show connect → info → tools → schema → call {} → result, then rejected arguments rendered red.
- [ ] Kill server, trigger a request, inspect status/logs without panic, restart and r reconnect; repeat stateless operation.
- [ ] Confirm integrated resize/scrolling, dark/light readability, and normal/panic terminal restoration.

## Context

Fixture configuration is test infrastructure; the product still takes only a URL. Record only actual evidence. Automated tests do not prove visual checks. No acceptance boxes are ticked during planning.

## Summary

Verify the full mcpv MVP against a real server and document the demo.

## Acceptance

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo test --all-targets`
- [ ] `specdev check`

## Log

- 2026-10-13 created: real server demo and quality
- 2026-10-13 scope add examples/dev_server.rs — MVP implementation slice
- 2026-10-13 scope add tests/http_client.rs — MVP implementation slice
- 2026-10-13 scope add docs/demo.md — MVP implementation slice
- 2026-10-13 scope add Cargo.toml — MVP implementation slice
- 2026-10-13 scope add Cargo.lock — MVP implementation slice
- 2026-10-13 scope add README.md — MVP implementation slice
- 2026-10-13 set source User mcpv MVP brief; Rust 2024 and toolchain 1.99
- 2026-10-13 set depends 0002-http-client-lifecycle, 0003-terminal-shell-and-status, 0004-server-info-and-tool-browser, 0005-schema-json-skeleton, 0006-tool-editor-and-results, 0007-prompts-and-resources-tabs
