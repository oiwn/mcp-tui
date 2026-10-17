# Project Overview: mcpv

## Purpose and scope

mcpv replaces the abandoned redb-tui project. It inspects and drives one running local or remote MCP server: mcpv http://127.0.0.1:7400/mcp. Legacy source is archived in tmp/legacy-redb.

- Rust edition 2024, toolchain 1.99 (pin 1.99.0 in rust-toolchain.toml), tokio rt-multi-thread, ratatui, and crossterm.
- Official SDK only: rmcp = { version = "=3.5.1", features = ["client", "transport-streamable-http-client-reqwest"] }. No hand-written JSON-RPC.
- One required positional server URL; no command opens the TUI. Optional headless info/tools-list/tools-get/tools-call commands and --json support agents. No auth/OAuth, stdio/child processes, server-state editor, multiple servers, or config files. Explicit tool invocation is in scope.
- Errors use a thiserror enum with typed sources, never String. Avoid unwrap/expect except documented impossible conditions.

## Architecture

- An asynchronous SDK worker owns connection lifecycle; typed commands/events update UI state without blocking keyboard input.
- Initialize with ClientConfig::default().serve(StreamableHttpClientTransport::from_uri(url)). Read server metadata through the SDK peer-info equivalent.
- Fetch complete catalogs with list_all_tools; tool count comes from the fetched list. Reconnect cancels old work and initializes again; stale responses cannot overwrite current state.
- Calls use CallToolRequestParams::new(name).with_arguments(parsed_json), with an object argument root.
- Headless commands reuse SDK initialization, catalog/call deadlines, URL validation, and bounded cleanup. Enable rmcp reqwest for rustls HTTPS. Default headless input/initialization/request deadlines are 30 seconds (--timeout overrides); TUI deadlines stay 10 seconds and shutdown stays 2 seconds.
- Terminal guard and panic hook restore raw mode, alternate screen, and cursor before diagnostics; resize recalculates layout.

## Data Flow

- Server info: name, version, negotiated protocol version, capabilities, and tool count.
- Left tools pane: every tool name and one-line description. Right pane: full description and pretty JSON inputSchema, scrollable.
- Enter opens an editable JSON buffer with a schema skeleton or {}. Ctrl+S submits (F5/Ctrl-Enter are alternatives); invalid JSON is rejected with the parse error. Esc saves the draft and returns to browsing.
- Arrows and j/k/g/G navigate lists/read-only panes. Editor mode preserves those characters as input.
- Render response text, pretty-printed when valid JSON. Some(true) is_error means failure: red with an explicit label. None/Some(false) mean no tool failure.
- Status: connection state, last call latency, URL. l toggles a bounded scrollable log; connection failures never panic.
- Cheap additions: capability-aware list-only prompts/resources tabs, per-tool call count/latency, and terminal-default colors readable on dark/light backgrounds.

## Gotchas / Knowledge

- Stateless servers need no Mcp-Session-Id; rmcp owns session handling. After killing an idle stateless server, trigger a request to observe failure; passive disconnect detection is not guaranteed.
- Tool-content failures are distinct from protocol/transport errors and must remain visible without a crash.
- Ctrl+S is the primary submit key; F5 remains an alternative for terminals that cannot distinguish Ctrl-Enter. Skeletons are suggestions, not schema validation.
- Agent skill: .agents/skills/mcpv/SKILL.md. Headless JSON envelopes preserve native SDK data; tool-content failures exit 3 on stdout, execution errors use stderr. Input is capped at 1 MiB; stdin uses a separate reader thread so interruption cannot stall Tokio shutdown.
- Agent/live-server test target: https://exodata.space/mcp. Automated tests stay local; public-endpoint smoke checks are opt-in. Inspect actual schemas before selecting read-only calls.
- Acceptance requires a real-server demo: connect, info, tools, schema, call {}, success, rejected arguments in red, stop/request failure, restart/r reconnect, plus stateless operation.
- Quality gates: cargo fmt --all -- --check; cargo clippy --all-targets -- -D warnings; cargo test --all-targets; specdev check. Schema helper unit tests are required.
- Latest published rmcp verified as 3.5.1 (2026-10-05): https://docs.rs/crate/rmcp/latest. Pinned API documentation: https://docs.rs/rmcp/3.5.1/rmcp/.
