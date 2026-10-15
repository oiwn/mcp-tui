#[path = "../examples/support/server.rs"]
mod server;

use std::{sync::Arc, time::Duration};

use mcpv::{
    client::{
        CatalogState, ClientEvent, ClientHandle, ClientOptions, ClientState,
        ConnectionState, EventKind,
    },
    error::{AppError, Operation},
};
use server::{Fixture, FixtureConfig, FixtureError};
use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Debug, thiserror::Error)]
enum TestError {
    #[error(transparent)]
    Fixture(#[from] FixtureError),
    #[error(transparent)]
    Client(#[from] AppError),
    #[error(transparent)]
    Timeout(#[from] tokio::time::error::Elapsed),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),
    #[error("event receiver closed")]
    Closed,
}

async fn event(
    events: &mut UnboundedReceiver<ClientEvent>,
) -> Result<ClientEvent, TestError> {
    tokio::time::timeout(Duration::from_secs(5), events.recv())
        .await?
        .ok_or(TestError::Closed)
}

async fn ready(
    events: &mut UnboundedReceiver<ClientEvent>,
    generation: u64,
) -> Result<ClientState, TestError> {
    let mut state = ClientState::default();
    state.begin_generation(generation);
    loop {
        state.apply(event(events).await?);
        if let Some(error) = &state.error {
            return Err(AppError::Reported(error.clone()).into());
        }
        if matches!(
            state.catalog,
            CatalogState::Ready(_)
                | CatalogState::Unsupported
                | CatalogState::Failed(_)
        ) {
            return Ok(state);
        }
    }
}

#[tokio::test]
async fn connects_in_both_modes_and_reconnects() -> Result<(), TestError> {
    for stateful in [false, true] {
        let fixture = Fixture::start(
            ([127, 0, 0, 1], 0).into(),
            FixtureConfig {
                stateful,
                ..Default::default()
            },
        )
        .await?;
        let (mut client, mut events) =
            ClientHandle::spawn(&fixture.url(), ClientOptions::default())?;
        let state = ready(&mut events, 1).await?;
        assert_eq!(state.connection, ConnectionState::Connected);
        assert!(
            matches!(state.catalog, CatalogState::Ready(ref tools) if tools.len() == 3)
        );
        assert!(state.server.as_ref().is_some_and(|info| {
            info.server_info
                .as_ref()
                .is_some_and(|identity| identity.name == "mcpv-dev-server")
        }));
        assert_eq!(
            fixture
                .session_header_seen
                .load(std::sync::atomic::Ordering::SeqCst),
            stateful
        );
        let generation = client.reconnect()?;
        assert_eq!(generation, 2);
        ready(&mut events, generation).await?;
        client.shutdown().await?;
        fixture.stop().await?;
    }
    Ok(())
}

#[tokio::test]
async fn catalog_capabilities_empty_pagination_and_protocol_failure()
-> Result<(), TestError> {
    let configurations = [
        FixtureConfig {
            tools_enabled: false,
            ..Default::default()
        },
        FixtureConfig {
            empty_tools: true,
            ..Default::default()
        },
        FixtureConfig {
            page_size: Some(1),
            ..Default::default()
        },
        FixtureConfig {
            list_error: true,
            ..Default::default()
        },
    ];
    for (index, config) in configurations.into_iter().enumerate() {
        let requests = config.list_requests.clone();
        let fixture = Fixture::start(([127, 0, 0, 1], 0).into(), config).await?;
        let (client, mut events) =
            ClientHandle::spawn(&fixture.url(), ClientOptions::default())?;
        let state = ready(&mut events, 1).await?;
        assert_eq!(state.connection, ConnectionState::Connected);
        match index {
            0 => {
                assert!(matches!(state.catalog, CatalogState::Unsupported));
                assert_eq!(requests.load(std::sync::atomic::Ordering::SeqCst), 0);
            }
            1 => assert!(
                matches!(state.catalog, CatalogState::Ready(ref tools) if tools.is_empty())
            ),
            2 => {
                assert!(
                    matches!(state.catalog, CatalogState::Ready(ref tools) if tools.len()==3)
                );
                assert_eq!(requests.load(std::sync::atomic::Ordering::SeqCst), 3);
            }
            _ => {
                assert!(
                    matches!(state.catalog, CatalogState::Failed(ref error) if matches!(error.as_ref(), AppError::Request(rmcp::ServiceError::McpError(_))))
                );
                assert!(state.server.is_some());
            }
        }
        client.shutdown().await?;
        fixture.stop().await?;
    }
    Ok(())
}

#[tokio::test]
async fn catalog_deadline_and_pending_reconnect() -> Result<(), TestError> {
    let config = FixtureConfig {
        list_delay: Duration::from_secs(1),
        ..Default::default()
    };
    let started = config.list_started.clone();
    let fixture = Fixture::start(([127, 0, 0, 1], 0).into(), config).await?;
    let options = ClientOptions {
        catalog_timeout: Duration::from_millis(50),
        ..Default::default()
    };
    let (mut client, mut events) = ClientHandle::spawn(&fixture.url(), options)?;
    tokio::time::timeout(Duration::from_secs(5), started.notified()).await?;
    let generation = client.reconnect()?;
    let mut state = ClientState::default();
    state.begin_generation(generation);
    loop {
        state.apply(event(&mut events).await?);
        if let Some(error) = &state.error {
            assert!(matches!(
                error.as_ref(),
                AppError::Timeout {
                    operation: Operation::ListTools,
                    ..
                }
            ));
            assert_eq!(state.generation, generation);
            assert_eq!(state.connection, ConnectionState::Disconnected);
            break;
        }
    }
    client.shutdown().await?;
    fixture.stop().await?;
    Ok(())
}

/// A bound HTTP endpoint deliberately never responds; no custom MCP wire layer.
async fn stalled_endpoint() -> Result<
    (
        String,
        tokio::task::JoinHandle<()>,
        Arc<tokio::sync::Notify>,
    ),
    std::io::Error,
> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let url = format!("http://{}/mcp", listener.local_addr()?);
    let started = Arc::new(tokio::sync::Notify::new());
    let notification = started.clone();
    let router = axum::Router::new().fallback(move || {
        let notification = notification.clone();
        async move {
            notification.notify_one();
            std::future::pending::<()>().await;
            ""
        }
    });
    let task = tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    });
    Ok((url, task, started))
}

#[tokio::test]
async fn initialization_deadline_reconnect_and_handle_drop()
-> Result<(), TestError> {
    let (url, stalled, started) = stalled_endpoint().await?;
    let options = ClientOptions {
        initialize_timeout: Duration::from_millis(80),
        ..Default::default()
    };
    let (mut client, mut events) = ClientHandle::spawn(&url, options)?;
    assert!(matches!(
        event(&mut events).await?.kind,
        EventKind::Connecting
    ));
    tokio::time::timeout(Duration::from_secs(5), started.notified()).await?;
    let generation = client.reconnect()?;
    loop {
        let event = event(&mut events).await?;
        if event.generation == generation
            && let EventKind::Disconnected(Some(error)) = event.kind
        {
            assert!(matches!(
                error.as_ref(),
                AppError::Timeout {
                    operation: Operation::Initialize,
                    ..
                }
            ));
            break;
        }
    }
    client.shutdown().await?;
    let (client, mut events) =
        ClientHandle::spawn(&url, ClientOptions::default())?;
    event(&mut events).await?;
    drop(client);
    tokio::time::timeout(Duration::from_secs(3), async {
        while events.recv().await.is_some() {}
    })
    .await?;
    stalled.abort();
    Ok(())
}

#[tokio::test]
async fn reconnect_queue_has_bounded_nonblocking_submission()
-> Result<(), TestError> {
    let (mut client, events) = ClientHandle::spawn(
        "http://127.0.0.1:7400/mcp",
        ClientOptions::default(),
    )?;
    // The current-thread test runtime cannot run the worker until we yield.
    for _ in 0..16 {
        client.reconnect()?;
    }
    assert!(matches!(client.reconnect(), Err(AppError::QueueFull)));
    drop(events);
    client.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn dropping_an_initialized_handle_cleans_up() -> Result<(), TestError> {
    let fixture = Fixture::start(
        ([127, 0, 0, 1], 0).into(),
        FixtureConfig {
            stateful: true,
            ..Default::default()
        },
    )
    .await?;
    let (client, mut events) =
        ClientHandle::spawn(&fixture.url(), ClientOptions::default())?;
    ready(&mut events, 1).await?;
    drop(client);
    tokio::time::timeout(Duration::from_secs(3), async {
        while events.recv().await.is_some() {}
    })
    .await?;
    fixture.stop().await?;
    Ok(())
}

#[tokio::test]
async fn help_and_version_work_without_a_terminal() -> Result<(), TestError> {
    for flag in ["--help", "--version"] {
        let output = tokio::process::Command::new(env!("CARGO_BIN_EXE_mcpv"))
            .arg(flag)
            .output()
            .await?;
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("mcpv"));
    }
    Ok(())
}

#[tokio::test]
async fn invalid_urls_and_connection_refusal() -> Result<(), TestError> {
    for url in ["not a url", "file:///tmp/server", "ftp://localhost/mcp"] {
        assert!(ClientHandle::spawn(url, ClientOptions::default()).is_err());
    }
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let url = format!("http://{}/mcp", listener.local_addr()?);
    drop(listener);
    let (client, mut events) =
        ClientHandle::spawn(&url, ClientOptions::default())?;
    loop {
        if let EventKind::Disconnected(Some(error)) =
            event(&mut events).await?.kind
        {
            assert!(matches!(error.as_ref(), AppError::Initialize(_)));
            break;
        }
    }
    client.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn recovers_after_server_restart_and_receiver_drop() -> Result<(), TestError>
{
    let fixture =
        Fixture::start(([127, 0, 0, 1], 0).into(), FixtureConfig::default())
            .await?;
    let address = fixture.address;
    let (mut client, mut events) =
        ClientHandle::spawn(&fixture.url(), ClientOptions::default())?;
    ready(&mut events, 1).await?;
    fixture.stop().await?;
    let generation = client.reconnect()?;
    loop {
        let event = event(&mut events).await?;
        if event.generation == generation
            && matches!(event.kind, EventKind::Disconnected(Some(_)))
        {
            break;
        }
    }
    let fixture = Fixture::start(address, FixtureConfig::default()).await?;
    let generation = client.reconnect()?;
    ready(&mut events, generation).await?;
    drop(events);
    // Receiver closure stops the worker; explicit shutdown still joins it.
    client.shutdown().await?;
    fixture.stop().await?;
    Ok(())
}

#[test]
fn stale_events_cannot_override_new_generation() {
    let mut state = ClientState::default();
    state.begin_generation(2);
    assert!(!state.apply(ClientEvent {
        generation: 1,
        kind: EventKind::Disconnected(Some(Arc::new(AppError::ConnectionClosed)))
    }));
    assert_eq!(state.connection, ConnectionState::Connecting);
    assert!(state.error.is_none());
}

#[tokio::test]
async fn noninteractive_tui_headless_dispatch_and_url_failures_are_clean()
-> Result<(), TestError> {
    let fixture =
        Fixture::start(([127, 0, 0, 1], 0).into(), FixtureConfig::default())
            .await?;
    let url = fixture.url();
    let headless_url = url.clone();
    let output = tokio::task::spawn_blocking(move || {
        std::process::Command::new(env!("CARGO_BIN_EXE_mcpv"))
            .arg(url)
            .output()
    })
    .await??;
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("requires interactive")
    );
    assert!(output.stdout.is_empty());
    let headless = tokio::task::spawn_blocking(move || {
        std::process::Command::new(env!("CARGO_BIN_EXE_mcpv"))
            .arg(headless_url)
            .args(["info", "--json"])
            .output()
    })
    .await??;
    assert!(headless.status.success());
    let metadata: serde_json::Value =
        serde_json::from_slice(&headless.stdout).map_err(FixtureError::Json)?;
    assert_eq!(metadata["data"]["serverInfo"]["name"], "mcpv-dev-server");
    assert!(headless.stderr.is_empty());
    let output = tokio::task::spawn_blocking(|| {
        std::process::Command::new(env!("CARGO_BIN_EXE_mcpv"))
            .arg("file:///tmp/server")
            .output()
    })
    .await??;
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("HTTP or HTTPS"));
    fixture.stop().await?;
    Ok(())
}
