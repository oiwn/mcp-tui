# Changelog

Completed tasks, newest entry at the top, dated. `specdev task done` adds an entry for each finished task file; work done directly in `specs/ctx.md` is moved here once every checkbox in its plan is done. Coexists with any conventional release notes already here.

<!-- Entry template:
## YYYY-MM-DD — <task title>

- what shipped
- files touched / decisions locked
-->

## 2026-10-16 — terminal shell and status

- Add a persistent MCP terminal screen with basic tool browsing, connection status, logs, and reliable cleanup.
- Task `0003-terminal-shell-and-status`; source: User mcpv MVP brief; Rust 2024 and toolchain 1.99

## 2026-10-16 — server info and tool browser

- Expose server information and a two-pane tool/schema browser.
- Task `0004-server-info-and-tool-browser`; source: User mcpv MVP brief; Rust 2024 and toolchain 1.99

## 2026-10-15 — agent cli and live server checks

- Add headless MCP inspection/tool calls, a repo-local agent skill, and verified HTTPS live-server checks.
- Task `0010-agent-cli-and-live-server-checks`; source: User requests a CLI/testing facility for agents; live target https://exodata.space/mcp

## 2026-10-14 — mock mcp server

- Provide a runnable SDK-backed MCP development server with deterministic tools and HTTP tests.
- Task `0009-mock-mcp-server`; source: User requested a runnable mock MCP server as the next task

## 2026-10-14 — http client lifecycle

- Add SDK-backed HTTP connection and reconnect lifecycle.
- Task `0002-http-client-lifecycle`; source: User mcpv MVP brief; Rust 2024 and toolchain 1.99

## 2026-10-14 — mcpv foundation

- Replace the database project with the mcpv Rust foundation.
- Task `0001-mcpv-foundation`; source: User mcpv MVP brief; Rust 2024 and toolchain 1.99
