# mcpv

A terminal client for inspecting and driving a single running MCP server during local development. Built with Rust, tokio, ratatui/crossterm, and the official rmcp SDK pinned to 3.5.1.

## Development

Install Rust through rustup. The repository selects Rust 1.99.0 with rustfmt and clippy; the package uses edition 2024.

```sh
cargo build --locked
cargo run --locked -- --help
cargo run --locked -- http://127.0.0.1:7400/mcp
```

The CLI foundation accepts one required server URL and standard help/version flags. Connection handling and the terminal UI are pending: normal launch currently prints that message to stderr and exits with code 1, without contacting the server or changing terminal modes. Missing or extra arguments exit with code 2; help/version exit with code 0. URL validation will arrive with connection handling.

The MVP targets localhost debugging over Streamable HTTP. Auth, stdio/child-process transports, multiple servers, config files, and server-state editing are outside its scope.

## Checks

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
specdev check
```

Implementation tasks live in specs/tasks/. Obsolete redb sources, documentation snapshots, and artifacts are preserved in tmp/legacy-redb/ and are outside Cargo targets.
