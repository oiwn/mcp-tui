# Project Overview: mcpv

## Purpose and scope

mcpv replaces the abandoned redb-tui project. It inspects and drives one running MCP server for localhost development: mcpv http://127.0.0.1:7400/mcp. Legacy source may move to tmp/legacy-redb during implementation.

- Rust edition 2024, toolchain 1.99 (pin 1.99.0 in rust-toolchain.toml), tokio rt-multi-thread, ratatui, and crossterm.
- Official SDK only: rmcp = { version = "=3.5.1", features = ["client", "transport-streamable-http-client-reqwest"] }. No hand-written JSON-RPC.
- One required positional server URL. No auth/OAuth, stdio/child processes, server-state editor, multiple servers, or config files. Explicit tool invocation is in scope.
- Errors use a thiserror enum with typed sources, never String. Avoid unwrap/expect except documented impossible conditions.

## Architecture

- An asynchronous SDK worker owns connection lifecycle; typed commands/events update UI state without blocking keyboard input.
- Initialize with ClientConfig::default().serve(StreamableHttpClientTransport::from_uri(url)). Read server metadata through the SDK peer-info equivalent.
- Fetch complete catalogs with list_all_tools; tool count comes from the fetched list. Reconnect cancels old work and initializes again; stale responses cannot overwrite current state.
- Calls use CallToolRequestParams::new(name).with_arguments(parsed_json), with an object argument root.
- Terminal guard and panic hook restore raw mode, alternate screen, and cursor before diagnostics; resize recalculates layout.

## Data Flow

- Server info: name, version, negotiated protocol version, capabilities, and tool count.
- Left tools pane: every tool name and one-line description. Right pane: full description and pretty JSON inputSchema, scrollable.
- Enter opens an editable JSON buffer with a schema skeleton or {}. Ctrl-Enter/F5 submits; invalid JSON is rejected with the parse error. Esc cancels.
- Arrows and j/k/g/G navigate lists/read-only panes. Editor mode preserves those characters as input.
- Render response text, pretty-printed when valid JSON. Some(true) is_error means failure: red with an explicit label. None/Some(false) mean no tool failure.
- Status: connection state, last call latency, URL. l toggles a bounded scrollable log; connection failures never panic.
- Cheap additions: capability-aware list-only prompts/resources tabs, per-tool call count/latency, and terminal-default colors readable on dark/light backgrounds.

## Gotchas / Knowledge

- Stateless servers need no Mcp-Session-Id; rmcp owns session handling. After killing an idle stateless server, trigger a request to observe failure; passive disconnect detection is not guaranteed.
- Tool-content failures are distinct from protocol/transport errors and must remain visible without a crash.
- F5 covers terminals that cannot distinguish Ctrl-Enter. Skeletons are suggestions, not schema validation.
- Acceptance requires a real-server demo: connect, info, tools, schema, call {}, success, rejected arguments in red, stop/request failure, restart/r reconnect, plus stateless operation.
- Quality gates: cargo fmt --all -- --check; cargo clippy --all-targets -- -D warnings; cargo test --all-targets; specdev check. Schema helper unit tests are required.
- Latest published rmcp verified as 3.5.1 (2026-10-05): https://docs.rs/crate/rmcp/latest. Pinned API documentation: https://docs.rs/rmcp/3.5.1/rmcp/.
