---
id: 0002-http-client-lifecycle
status: approval
scope: [src/client.rs, src/error.rs, src/app.rs, src/main.rs, tests/http_client.rs, src/lib.rs, Cargo.toml, Cargo.lock, README.md, examples/support/server.rs, examples/dev_server.rs, tests/dev_server.rs]
created: 2026-10-13
source: User mcpv MVP brief; Rust 2024 and toolchain 1.99
depends: [0001-mcpv-foundation, 0009-mock-mcp-server]
---
# Task: http client lifecycle

## Plan

- [x] Validate absolute HTTP/HTTPS URLs with hosts; initialize through ClientConfig::default().serve(StreamableHttpClientTransport::from_uri(url)), expose peer metadata, and fetch all tools only when advertised.
- [x] Add a reusable library worker with typed generation-tagged events, a 16-entry command queue, reconnect/shutdown, separate connection/catalog states, and SDK completion monitoring.
- [x] Apply 10-second initialization/catalog and 2-second cleanup deadlines with internal test overrides; cancel pending work on reconnect/drop, reject stale events, and preserve connected metadata on catalog protocol errors.
- [x] Replace the CLI stub with summary-and-exit, typed diagnostics and clean shutdown; Ctrl-C cancels pending work and exits 130; document the temporary flow.
- [x] Test real stateless/stateful fixtures, empty/unsupported/paginated/failing catalogs, timeouts, invalid/refused URLs, pending-operation reconnect, stop/restart recovery, stale events, and drop cleanup; run all quality gates.

## Context

Approved plan: connection states connecting/connected/disconnected; catalogs loading/unsupported/ready/failed. Unbounded event channel avoids blocking control commands. Protocol list errors keep the connection; transport/init failures disconnect. URL validation uses url; time/sync/signal are explicit tokio features. No automatic reconnect/heartbeat, extra product CLI flags, auth, or hand-written JSON-RPC. Integration tests reuse task 0009; visible TUI acceptance remains task 0008.

On success print identity (or unknown), protocol, and count/unsupported, shut down, exit 0; failure diagnostics exit 1; Ctrl-C exits 130. Add a library entry point for tests; errors retain typed SDK sources. Scope size is justified by the worker, binary integration, and fixture test controls; global limits remain unchanged.

## Summary

Add SDK-backed HTTP connection and reconnect lifecycle.

## Acceptance

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `cargo test --all-targets`
- [x] `specdev check`

## Review

Approved against the plan. Fifteen tests pass, including real HTTP lifecycle/catalog cases and a Ctrl-C subprocess that exits 130. cargo build --locked passes without dev server features; that binary connected to the example on port 7400, printed mcpv-dev-server 0.1.0, protocol 2025-11-25, and Tools: 3, then exited 0. SDK metadata is ServerPeerInfo with optional identity; absent identity renders unknown server. Errors preserve typed sources, and new source has no unwrap/expect. Review corrected an operation-error/cleanup-error race. Shutdown joins are bounded and drop guards cancel owned service tasks. specdev check reports zero errors; its two warnings are the declared replacement/integration scope size and future docs/demo.md. Scope includes adapting fixture consumers to configurable test controls. Task 0009 was implemented as the prerequisite and remains in approval; no user acceptance was invented.

## Log

- 2026-10-13 created: http client lifecycle
- 2026-10-13 scope add src/client.rs — MVP implementation slice
- 2026-10-13 scope add src/error.rs — MVP implementation slice
- 2026-10-13 scope add src/app.rs — MVP implementation slice
- 2026-10-13 scope add src/main.rs — MVP implementation slice
- 2026-10-13 scope add tests/http_client.rs — MVP implementation slice
- 2026-10-13 set source User mcpv MVP brief; Rust 2024 and toolchain 1.99
- 2026-10-13 set depends 0001-mcpv-foundation
- 2026-10-14 set depends 0001-mcpv-foundation, 0009-mock-mcp-server
- 2026-10-14 scope add src/lib.rs — Approved reusable worker, explicit dependency features, documented summary flow, and deterministic fixture controls
- 2026-10-14 scope add Cargo.toml — Approved reusable worker, explicit dependency features, documented summary flow, and deterministic fixture controls
- 2026-10-14 scope add Cargo.lock — Approved reusable worker, explicit dependency features, documented summary flow, and deterministic fixture controls
- 2026-10-14 scope add README.md — Approved reusable worker, explicit dependency features, documented summary flow, and deterministic fixture controls
- 2026-10-14 scope add examples/support/server.rs — Approved reusable worker, explicit dependency features, documented summary flow, and deterministic fixture controls
- 2026-10-14 advance draft → ready
- 2026-10-14 scope approved: src/client.rs, src/error.rs, src/app.rs, src/main.rs, tests/http_client.rs, src/lib.rs, Cargo.toml, Cargo.lock, README.md, examples/support/server.rs
- 2026-10-14 advance ready → in-progress/implement
- 2026-10-14 baseline: examples/dev_server.rs@4a23eee5af6c, tests/dev_server.rs@ebb97d3edc25
- 2026-10-14 scope add examples/dev_server.rs — Adapt fixture consumers to the shared configurable fixture used by lifecycle tests
- 2026-10-14 scope add tests/dev_server.rs — Adapt fixture consumers to the shared configurable fixture used by lifecycle tests
- 2026-10-14 advance in-progress/implement → in-progress/verify
- 2026-10-14 advance in-progress/verify → in-progress/review
- 2026-10-14 advance in-progress/review → approval
