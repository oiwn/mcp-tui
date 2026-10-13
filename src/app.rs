use std::{
    io::{self, Write},
    process::ExitCode,
};

use crate::cli::Args;
use mcpv::{
    client::{
        CatalogState, ClientHandle, ClientOptions, ClientState, ConnectionState,
    },
    error::{AppError, Result},
};

pub async fn run(args: Args) -> Result<ExitCode> {
    let (handle, mut events) =
        ClientHandle::spawn(&args.server_url, ClientOptions::default())?;
    let mut state = ClientState::default();
    let interrupted = tokio::signal::ctrl_c();
    tokio::pin!(interrupted);
    let outcome = loop {
        tokio::select! {
            biased;
            signal = &mut interrupted => {
                break signal.map(|()| ExitCode::from(130)).map_err(AppError::Signal);
            },
            event = events.recv() => {
                let Some(event) = event else { break Err(AppError::ChannelClosed) };
                state.apply(event);
                if let Some(error) = &state.error { break Err(AppError::Reported(error.clone())) }
                match &state.catalog {
                    CatalogState::Failed(error) => break Err(AppError::Reported(error.clone())),
                    CatalogState::Ready(tools) => break summary(&state, &tools.len().to_string()),
                    CatalogState::Unsupported => break summary(&state, "unsupported"),
                    CatalogState::Loading => if state.connection == ConnectionState::Disconnected {
                        break Err(AppError::ConnectionClosed);
                    },
                }
            }
        }
    };
    let cleanup = handle.shutdown().await;
    outcome.and_then(|code| cleanup.map(|()| code))
}

fn summary(state: &ClientState, tools: &str) -> Result<ExitCode> {
    let info = state.server.as_ref().ok_or(AppError::MissingMetadata)?;
    let identity = info
        .server_info
        .as_ref()
        .map(|identity| format!("{} {}", identity.name, identity.version))
        .unwrap_or_else(|| "unknown server".to_owned());
    writeln!(
        io::stdout().lock(),
        "Connected: {identity} | Protocol: {} | Tools: {tools}",
        info.protocol_version
    )?;
    Ok(ExitCode::SUCCESS)
}
