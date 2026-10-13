use std::process::ExitCode;

use clap::Parser;

mod app;
mod cli;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> ExitCode {
    let args = cli::Args::parse();
    match app::run(args).await {
        Ok(code) => code,
        Err(error) => {
            eprintln!("mcpv: {error}");
            ExitCode::FAILURE
        }
    }
}
