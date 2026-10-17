# mcpv

A terminal client for inspecting and driving a single running MCP server during local development. Built with Rust, tokio, ratatui/crossterm, and the official rmcp SDK pinned to 3.5.1.

## Development

Install Rust through rustup. The repository selects Rust 1.99.0 with rustfmt and clippy; the package uses edition 2024.

```sh
cargo build --locked
cargo run --locked -- --help
cargo run --locked -- http://127.0.0.1:7400/mcp
```

The CLI accepts one required absolute HTTP/HTTPS server URL and standard help/version flags. Without a command it opens a persistent terminal screen, requiring interactive stdin/stdout. It initializes the server and lists all advertised tools alongside connection/server information; failures remain visible in the screen and activity logs. Missing or extra arguments exit 2; invalid URLs/non-interactive TUI launches exit 1; help/version exit 0. Terminal modes are restored on normal exit, startup/I/O failures, and panic.

Initialization, full tool listing, and TUI calls each have a 10-second deadline; cleanup is bounded to 2 seconds. There is no automatic retry or heartbeat; an idle stateless HTTP server can disappear without an immediate disconnect event. rmcp owns protocol and session handling.

## Keys and screen

- Arrows or j/k select tools or scroll the focused pane; g/G jump to its beginning/end.
- 1/2 switches between Tools and Server tabs; each switch focuses that tab's initial body pane.
- Tab/Shift-Tab changes focus between the active tab's panes and the activity drawer when open.
- l toggles activity logs; G resumes following their tail after scrolling away.
- r cancels old work and reconnects. Cached tools/metadata remain labeled offline after disconnect and clear when reconnect starts.
- q quits with exit 0; Ctrl-C cancels and exits 130.
- Enter from the Tools list/details opens a full-body argument editor. Ctrl+S submits; F5/Ctrl-Enter are alternatives. Esc saves the draft and returns to browsing. Plain Enter inserts a newline.
- In the editor, arrows/Home/End move, Enter inserts a newline, Tab indents, Ctrl-R resets the schema skeleton, and Ctrl-U sets {}. Paste supports multiline text. Ordinary shortcuts including q/r/l/1/2 are text; Ctrl-C still exits.
- Accepted submissions open a full-body result screen. Esc returns to browsing while a pending call continues; v reopens the latest result, and Enter from results edits that tool's draft. One call can be active; further submissions are rejected until it completes. Completion never changes the active screen.

The Tools tab shows a tool list alongside the selected tool's full description and formatted inputSchema. Changing selection resets detail scrolling; catalog replacement retains selection by name when possible. The Server tab uses the full width for URL, connection state, identity/protocol, catalog count, full capabilities, instructions, and errors. Both detail views scroll independently.

Arguments must be a JSON object of at most 1 MiB. Invalid arguments stay in the editor with an explicit error and are never sent. The skeleton is an editable suggestion, not validation. Drafts are saved per tool; only the latest call result is retained. Reconnect clears drafts/results/metrics and cancels local pending work. Calls are never automatically retried; cancelling local work does not undo server-side effects.

Results show all text blocks, formatted JSON, structuredContent when supplied, and labels for non-text/empty content. Only isError=true is a tool-content failure (TOOL ERROR in red); protocol/transport/timeout failures have an EXECUTION ERROR label. Tool details show dispatched-call counts and latest completion latency, including failures; status shows the latest call's state and latency.

Narrow screens show the focused pane; short screens show focused logs in place of the body. The editor temporarily hides the activity drawer. Tiny screens show a resize hint while mode-specific controls remain active. Prompts/resources browsing remains planned.

Colors inherit terminal defaults. Errors use red plus an ERROR label; if NO_COLOR is set, the label remains visible without color.

The MVP supports local and remote debugging over Streamable HTTP, with verified HTTPS through the SDK's rustls backend. Auth, stdio/child-process transports, multiple servers, config files, and server-state editing are outside its scope.

## Headless CLI for agents and scripts

Build once, then use the executable directly to keep Cargo output separate from machine-readable results:

```sh
cargo build --locked
target/debug/mcpv https://exodata.space/mcp info --json
target/debug/mcpv https://exodata.space/mcp tools-list --json
target/debug/mcpv https://exodata.space/mcp tools-get describe_catalog --json
target/debug/mcpv https://exodata.space/mcp tools-call describe_catalog --args '{"table":"exoplanets","columns":["pl_name","hostname"]}' --json
printf '{"table":"exoplanets","columns":["pl_name"]}\n' | target/debug/mcpv https://exodata.space/mcp tools-call describe_catalog --args - --json
```

Inspect the catalog/schema before selecting a tool and supplying arguments. Only explicit tools-call commands invoke tools. Omitted arguments mean {}; --args - reads stdin. Arguments must be an object and fit within 1 MiB. Headless commands do not require a terminal. Their initialization, input, and request deadlines default to 30 seconds; --timeout 60 overrides them with positive whole seconds. Cleanup remains bounded to 2 seconds, and Ctrl-C exits even with a stalled stdin pipe. Calls are not automatically replayed after failure.

JSON mode emits one envelope with schemaVersion (1), command, serverUrl, ok, elapsedMs, data, and error. data preserves native SDK fields/content, including structuredContent and isError. elapsedMs measures the requested operation (the handshake for info), excluding argument reading and cleanup. error has kind and message; a secondary cleanup failure can add cleanupError. JSON result documents go to stdout; execution failures go to stderr. No ANSI/progress output is emitted. Help/version and clap syntax errors remain text; --json/--timeout require a headless command.

- Exit 0: success.
- Exit 1: connection/protocol, unsupported capability, timeout, or I/O failure.
- Exit 2: usage, malformed/non-object/oversized arguments, or a tool not found by tools-get.
- Exit 3: tool-content failure (only is_error Some(true)); result data stays on stdout with ok=false.
- Exit 130: interruption with cleanup attempted.

Omit --json for readable metadata, tool rows, and pretty definitions/results. The repo-local [mcpv skill](.agents/skills/mcpv/SKILL.md) guides agents through discovery, retrieval, and diagnosis. Normal tests use local fixtures; public endpoint checks are opt-in and require no changes to the default test suite.

## Development server

Run `cargo run --locked --example dev_server` for a real stateless MCP server at `http://127.0.0.1:7400/mcp`. Add `-- --stateful` to exercise legacy sessions using protocol 2025-11-25; current MCP protocol revisions are stateless. Stop with Ctrl-C and restart with the same command.

Tools: `ping` accepts `{}` and returns JSON; `echo` accepts `{"text":"hello"}` and returns text, or `{"text":"hello","fail":true}` to return a tool-content error; `slow` accepts `{"delay_ms":250}` (0–5000 ms). The server also lists a `debug` prompt and `mock://status` resource. No external services are required.

## Quality checks

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
specdev check
```

Implementation tasks live in specs/tasks/. Obsolete redb sources, documentation snapshots, and artifacts are preserved in tmp/legacy-redb/ and are outside Cargo targets.
