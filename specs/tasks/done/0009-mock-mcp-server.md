---
id: 0009-mock-mcp-server
status: done
scope: [examples/**, tests/dev_server.rs, Cargo.toml, Cargo.lock, README.md]
created: 2026-10-14
source: User requested a runnable mock MCP server as the next task
depends: [0001-mcpv-foundation]
---
# Task: mock mcp server

## Plan

- [x] Add cargo run --locked --example dev_server: a real rmcp =3.5.1 Streamable HTTP server at http://127.0.0.1:7400/mcp, stateless by default, with --stateful for session coverage and Ctrl-C shutdown.
- [x] Provide deterministic ping ({} returns JSON text), echo (text plus optional fail flag; fail true returns is_error Some(true) content), and slow (bounded delay_ms for latency/pending-call checks) tools with descriptions and schemas; add one listable prompt and resource for later tabs.
- [x] Share fixture construction with SDK-client HTTP tests on ephemeral loopback ports; verify handshake metadata, full catalogs, {} success, plain text echo, explicit content failure, stateless no-session-header operation, and stateful initialization/shutdown.
- [x] Keep server-only SDK/HTTP dependencies dev-only, use typed errors and bounded startup/request waits, and document run/call/restart examples in README.

## Context

This task extracts fixture creation from task 0008 so client development can use it immediately. The fixture speaks real MCP through the official SDK; it does not fake JSON-RPC. Its CLI options are development infrastructure and do not change the mcpv URL-only CLI. Share support code under examples/ with tests, avoiding duplicate server implementations.

Use stable server name/version and deterministic payloads. Slow calls default to 250 ms and reject delays above 5000 ms. Use an explicit fail flag to exercise tool-content errors rather than relying on SDK argument-decoding errors, which may be protocol errors. Test readiness through bound listener/explicit readiness signals, not arbitrary sleeps. This task does not require the unfinished TUI; task 0008 owns visible red rendering and interactive recovery evidence.

## Manual checks

- [x] Run the documented example, inspect its startup URL, stop with Ctrl-C, and restart on the same port. — agent: ran cargo run --locked --example dev_server in a PTY twice on port 7400; both Ctrl-C exits returned 0.

## Summary

Provide a runnable SDK-backed MCP development server with deterministic tools and HTTP tests.

## Acceptance

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `cargo test --all-targets`
- [x] `specdev check`

## Review

Fixture uses the official SDK and shared construction, dev-only server dependencies, typed errors, bounded shutdown, and actual header observation. HTTP tests cover both modes and deterministic catalogs/results. Legacy-session mode narrows protocol support to 2025-11-25 because newer MCP versions remove sessions. Quality gates pass; only the future docs/demo.md scope warning remains. No product CLI or terminal behavior changed.

## Log

- 2026-10-14 created: mock mcp server
- 2026-10-14 set source User requested a runnable mock MCP server as the next task
- 2026-10-14 set depends 0001-mcpv-foundation
- 2026-10-14 scope add examples/** — Runnable SDK-backed fixture and HTTP verification
- 2026-10-14 scope add tests/dev_server.rs — Runnable SDK-backed fixture and HTTP verification
- 2026-10-14 scope add Cargo.toml — Runnable SDK-backed fixture and HTTP verification
- 2026-10-14 scope add Cargo.lock — Runnable SDK-backed fixture and HTTP verification
- 2026-10-14 scope add README.md — Runnable SDK-backed fixture and HTTP verification
- 2026-10-14 advance draft → ready
- 2026-10-14 scope approved: examples/**, tests/dev_server.rs, Cargo.toml, Cargo.lock, README.md
- 2026-10-14 advance ready → in-progress/implement
- 2026-10-14 advance in-progress/implement → in-progress/verify
- 2026-10-14 advance in-progress/verify → in-progress/review
- 2026-10-14 advance in-progress/review → approval
- 2026-10-14 approved: ok it works
- 2026-10-14 advance approval → done
