---
id: 0002-http-client-lifecycle
status: draft
scope: [src/client.rs, src/error.rs, src/app.rs, src/main.rs, tests/http_client.rs]
created: 2026-10-13
source: User mcpv MVP brief; Rust 2024 and toolchain 1.99
depends: [0001-mcpv-foundation]
---
# Task: http client lifecycle

## Plan

- [ ] Initialize with ClientConfig::default().serve(StreamableHttpClientTransport::from_uri(url)); expose handshake metadata and list_all_tools through typed events, respecting capabilities.
- [ ] Own the SDK client in a nonblocking worker with bounded request timeouts, cancellation/shutdown, and connecting/connected/disconnected states.
- [ ] Implement reconnect teardown/reinitialize, ignoring responses from old connection generations; never require a session header.
- [ ] Test failure/timeout handling and stale-event rejection; leave real-server integration coverage to task 0008.

## Context

Errors produce status/log events while the UI stays usable. Empty catalogs are valid. Count tools from the complete fetched list. rmcp handles sessions and JSON-RPC.

## Summary

Add SDK-backed HTTP connection and reconnect lifecycle.

## Acceptance

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo test --all-targets`
- [ ] `specdev check`

## Log

- 2026-10-13 created: http client lifecycle
- 2026-10-13 scope add src/client.rs — MVP implementation slice
- 2026-10-13 scope add src/error.rs — MVP implementation slice
- 2026-10-13 scope add src/app.rs — MVP implementation slice
- 2026-10-13 scope add src/main.rs — MVP implementation slice
- 2026-10-13 scope add tests/http_client.rs — MVP implementation slice
- 2026-10-13 set source User mcpv MVP brief; Rust 2024 and toolchain 1.99
- 2026-10-13 set depends 0001-mcpv-foundation
