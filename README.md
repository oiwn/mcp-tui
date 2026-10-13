# mcpv

A terminal client for inspecting and driving a single running MCP server during local development. Built with Rust, tokio, ratatui/crossterm, and the official rmcp SDK pinned to 3.5.1.

## Development

Install Rust through rustup. The repository selects Rust 1.99.0 with rustfmt and clippy; the package uses edition 2024.

```sh
cargo build --locked
cargo run --locked -- --help
cargo run --locked -- http://127.0.0.1:7400/mcp
```

The CLI accepts one required absolute HTTP/HTTPS server URL and standard help/version flags. It initializes the server, fetches all advertised tools, prints server identity, negotiated protocol, and tool count (or unsupported), then shuts down and exits 0. Connection/catalog errors print diagnostics and exit 1; Ctrl-C cancels pending work and exits 130. Missing or extra arguments exit 2; help/version exit 0. The terminal UI is pending; no terminal modes are changed yet.

Initialization and full tool listing each have a 10-second deadline; cleanup is bounded to 2 seconds. The reusable client worker supports reconnect with stale-event rejection for the future r key. There is no automatic retry or heartbeat; an idle stateless HTTP server can disappear without an immediate disconnect event. rmcp owns protocol and session handling.

The MVP targets localhost debugging over Streamable HTTP. Auth, stdio/child-process transports, multiple servers, config files, and server-state editing are outside its scope.

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
