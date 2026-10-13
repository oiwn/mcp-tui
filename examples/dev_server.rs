#[path = "support/server.rs"]
mod server;

use clap::Parser;
use server::{Fixture, FixtureConfig, FixtureError};

#[derive(Parser)]
#[command(about = "Run the mcpv development MCP server")]
struct Args {
    /// Enable legacy MCP sessions (protocol 2025-11-25).
    #[arg(long)]
    stateful: bool,
}

#[tokio::main]
async fn main() -> Result<(), FixtureError> {
    let args = Args::parse();
    let fixture = Fixture::start(
        std::net::SocketAddr::from(([127, 0, 0, 1], 7400)),
        FixtureConfig {
            stateful: args.stateful,
            ..Default::default()
        },
    )
    .await?;
    println!(
        "mcpv-dev-server: {} ({})",
        fixture.url(),
        if args.stateful {
            "legacy sessions"
        } else {
            "stateless"
        }
    );
    // Keep header observation available to shared fixture tests.
    let _ = &fixture.session_header_seen;
    tokio::signal::ctrl_c().await?;
    fixture.stop().await
}
