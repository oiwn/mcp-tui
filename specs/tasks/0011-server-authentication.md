---
id: 0011-server-authentication
status: draft
scope: [src/cli.rs, src/client.rs, src/headless.rs, src/error.rs, README.md, tests/cli.rs]
created: 2026-10-17
source: User: support connecting to MCP servers with authentication; add as task
---
# Task: server authentication

## Plan

- [ ] Confirm the target server's authentication mechanism and approve the first slice: bearer token, custom API-key header, or OAuth/browser login. Refine scope before advancing.
- [ ] Add the chosen authentication inputs and configure the SDK HTTP transport consistently for TUI, headless commands, and reconnect.
- [ ] Read secrets from environment variables, keep values out of logs/errors/status output, and provide clear missing/invalid-credential errors.
- [ ] Test authenticated success, absent/rejected credentials, reconnect, secret redaction, and unchanged unauthenticated behavior with local fixtures; document usage and run quality gates.

## Context

Current mcpv has no authentication inputs. The pinned SDK supports authorization and custom headers. Environment-sourced bearer/API-key credentials are the recommended first slice; OAuth requires separate planning and is not implicitly included. No authentication method has been selected yet. Initial scope is provisional; add fixture or OAuth-specific files through specdev when the design is approved.

## Summary

Support authenticated MCP server connections with explicit credential handling.

## Acceptance

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test --all-targets --all-features`
- [ ] `typos`
- [ ] `gitleaks git --verbose`
- [ ] `specdev check`

## Log

- 2026-10-17 created: server authentication
- 2026-10-17 set source User: support connecting to MCP servers with authentication; add as task
- 2026-10-17 scope add src/cli.rs — Provisional authentication input, shared transport setup, error handling, documentation, and CLI checks; refine during draft planning
- 2026-10-17 scope add src/client.rs — Provisional authentication input, shared transport setup, error handling, documentation, and CLI checks; refine during draft planning
- 2026-10-17 scope add src/headless.rs — Provisional authentication input, shared transport setup, error handling, documentation, and CLI checks; refine during draft planning
- 2026-10-17 scope add src/error.rs — Provisional authentication input, shared transport setup, error handling, documentation, and CLI checks; refine during draft planning
- 2026-10-17 scope add README.md — Provisional authentication input, shared transport setup, error handling, documentation, and CLI checks; refine during draft planning
- 2026-10-17 scope add tests/cli.rs — Provisional authentication input, shared transport setup, error handling, documentation, and CLI checks; refine during draft planning
