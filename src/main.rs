use std::process::ExitCode;

use clap::Parser;

mod app;
mod cli;
mod error;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> ExitCode {
    let args = cli::Args::parse();
    match app::run(args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("mcpv: {error}");
            ExitCode::FAILURE
        }
    }
}
