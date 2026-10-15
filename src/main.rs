use std::process::ExitCode;

use clap::Parser;

mod app;
mod cli;
mod headless;
mod terminal;
mod ui;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> ExitCode {
    let mut args = cli::Args::parse();
    if let Err(error) = args.validate_mode() {
        error.exit();
    }
    if let Some(command) = args.command.take() {
        return headless::run(&args, command).await;
    }
    match app::run(args).await {
        Ok(code) => code,
        Err(error) => {
            eprintln!("mcpv: {error}");
            ExitCode::FAILURE
        }
    }
}
