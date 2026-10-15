---
id: 0010-agent-cli-and-live-server-checks
status: done
scope: [src/cli.rs, src/main.rs, src/headless.rs, src/client.rs, src/error.rs, Cargo.toml, Cargo.lock, tests/cli.rs, README.md, .agents/skills/mcpv/SKILL.md, AGENTS.md, examples/support/server.rs, tests/http_client.rs]
created: 2026-10-14
source: User requests a CLI/testing facility for agents; live target https://exodata.space/mcp
depends: [0002-http-client-lifecycle, 0003-terminal-shell-and-status, 0009-mock-mcp-server]
---
# Task: agent cli and live server checks

## Plan

- [x] Add non-TTY info/tools-list/tools-get/tools-call commands while preserving default TUI launch and standard help/version.
- [x] Add schemaVersion 1 JSON envelopes, native SDK data, separate result/error streams, readable text, and exit codes 0/1/2/3/130.
- [x] Share SDK initialization/catalog/call deadlines, URL validation and cleanup; retain rmcp =3.5.1 and enable rustls HTTPS through reqwest.
- [x] Validate object arguments before connecting, support explicit stdin/default {}, cap input at 1 MiB, and add positive --timeout seconds (default 30, cleanup 2).
- [x] Add repo-local mcpv skill and AGENTS/README discovery; validate and exercise schema-first read-only retrieval with inline/stdin arguments.
- [x] Test fixture modes, pagination, errors, stream/exit contracts, timeout overrides, unfinished-stdin interruption, and TUI/headless dispatch; run quality gates and live exodata checks.

## Interface

- mcpv https://exodata.space/mcp info --json
- mcpv https://exodata.space/mcp tools-list --json
- mcpv https://exodata.space/mcp tools-get <NAME> --json
- mcpv https://exodata.space/mcp tools-call <NAME> --args '{"key":"value"}' --json
- Explicit --args - reads stdin; discovery never automatically invokes tools.

JSON envelope: schemaVersion, command, serverUrl, ok, elapsedMs, data, error; optional cleanupError preserves secondary cleanup failures. Native SDK fields/content remain intact. Success/content failures use stdout (exit 0/3); execution errors use stderr. Help/version and clap syntax errors remain text. See README for the full contract.

## Context

Subcommands explicitly expand the URL-only CLI for agents while retaining default TUI behavior. info returns handshake metadata without fetching tools; tools-get resolves exact names from all catalog pages. Only Some(true) is_error fails a tool result. No auth, stdio, config, multi-server sessions, batch execution, or automatic replay is added.

The SDK reqwest feature enables verified rustls HTTPS. The expanded scope integrates CLI routing, shared client helpers, tests/instrumentation, documentation, and the agent skill; global quality limits remain unchanged. Dedicated stdin reader threads cannot hold up Tokio shutdown. Skill files use apply_patch; the Python validator only reads them.

Normal tests stay local. Live endpoints are opt-in; discover schemas/names before selecting read-only calls. The skill is .agents/skills/mcpv/SKILL.md, with automatic discovery and no global installation.

## Manual checks

- [x] From a non-TTY shell, run info/tools-list/tools-get against https://exodata.space/mcp over verified HTTPS and inspect parseable JSON plus exit codes. — agent: exit 0; exodata 0.4.1/protocol 2025-11-25, six tools, describe_catalog/query_catalog schemas, selected column descriptions, and read-only SELECT results via inline JSON and stdin.
- [x] Run a chosen fixture tool via inline JSON and stdin; demonstrate successful content and is_error Some(true) with distinct exit codes and preserved result content. — agent: echo returned inline/stdin text with exit 0; fail=true returned its error content, isError=true, ok=false, and exit 3 on stdout. Fixture stopped afterward.

## Summary

Add headless MCP inspection/tool calls, a repo-local agent skill, and verified HTTPS live-server checks.

## Acceptance

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `cargo test --all-targets`
- [x] `cargo build --locked`
- [x] `uv run --with pyyaml python /Users/alexch/.codex/skills/.system/skill-creator/scripts/quick_validate.py .agents/skills/mcpv`
- [x] `specdev check`

## Review

Approved against the execution plan. All 31 ordinary tests pass; the earlier intentional TUI panic probe remains ignored. Shared SDK helpers preserve worker behavior, native result fields, typed errors, and bounded cancellation. No unwrap/expect was added. The skill validated and its schema-first/filtered-query workflow succeeded against exodata with inline/stdin JSON and --timeout 60. Review covered stream/exit contracts, request-free local rejection, certificate verification, unchanged default TUI dispatch, and stalled-input interruption. Fixture processes were stopped. Remaining warnings are the justified CLI/skill integration scope, existing task 0003 scope, and future docs/demo.md. Task 0003 remains awaiting acceptance.

## Log

- 2026-10-14 created: agent cli and live server checks
- 2026-10-14 set source User requests a CLI/testing facility for agents; live target https://exodata.space/mcp
- 2026-10-14 set depends 0002-http-client-lifecycle, 0003-terminal-shell-and-status, 0009-mock-mcp-server
- 2026-10-14 scope add src/cli.rs — Headless MCP inspection/calls, HTTPS support, subprocess verification and usage documentation
- 2026-10-14 scope add src/main.rs — Headless MCP inspection/calls, HTTPS support, subprocess verification and usage documentation
- 2026-10-14 scope add src/headless.rs — Headless MCP inspection/calls, HTTPS support, subprocess verification and usage documentation
- 2026-10-14 scope add src/client.rs — Headless MCP inspection/calls, HTTPS support, subprocess verification and usage documentation
- 2026-10-14 scope add src/error.rs — Headless MCP inspection/calls, HTTPS support, subprocess verification and usage documentation
- 2026-10-14 scope add Cargo.toml — Headless MCP inspection/calls, HTTPS support, subprocess verification and usage documentation
- 2026-10-14 scope add Cargo.lock — Headless MCP inspection/calls, HTTPS support, subprocess verification and usage documentation
- 2026-10-14 scope add tests/cli.rs — Headless MCP inspection/calls, HTTPS support, subprocess verification and usage documentation
- 2026-10-14 scope add README.md — Headless MCP inspection/calls, HTTPS support, subprocess verification and usage documentation
- 2026-10-14 scope add .agents/skills/mcpv/SKILL.md — Approved agent skill, shared session API, CLI regression coverage and request instrumentation
- 2026-10-14 scope add AGENTS.md — Approved agent skill, shared session API, CLI regression coverage and request instrumentation
- 2026-10-14 scope add src/lib.rs — Approved agent skill, shared session API, CLI regression coverage and request instrumentation
- 2026-10-14 scope add tests/http_client.rs — Approved agent skill, shared session API, CLI regression coverage and request instrumentation
- 2026-10-14 scope add examples/support/server.rs — Approved agent skill, shared session API, CLI regression coverage and request instrumentation
- 2026-10-14 advance draft → ready
- 2026-10-14 scope approved: src/cli.rs, src/main.rs, src/headless.rs, src/client.rs, src/error.rs, Cargo.toml, Cargo.lock, tests/cli.rs, README.md, .agents/skills/mcpv/SKILL.md, AGENTS.md, src/lib.rs, tests/http_client.rs, examples/support/server.rs
- 2026-10-14 advance ready → in-progress/implement
- 2026-10-14 baseline: src/app.rs@5e17c60ad215, src/terminal.rs@f9237f109f76, src/ui.rs@a9258c105c90
- 2026-10-14 scope rm src/lib.rs
- 2026-10-14 scope rm tests/http_client.rs
- 2026-10-14 scope add tests/http_client.rs — Verify default TUI rejection and explicit headless dispatch together against the existing HTTP fixture
- 2026-10-14 advance in-progress/implement → in-progress/verify
- 2026-10-14 advance in-progress/verify → in-progress/review
- 2026-10-14 advance in-progress/review → approval
- 2026-10-15 approved: ok it works, close the task
- 2026-10-15 advance approval → done
