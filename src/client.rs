use std::{sync::Arc, time::Duration};

use rmcp::{
    RoleClient, ServiceExt,
    model::{
        CallToolRequestParams, CallToolResult, ClientConfig, ServerPeerInfo, Tool,
    },
    service::{QuitReason, RunningService},
    transport::StreamableHttpClientTransport,
};
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::error::{AppError, Operation, Result};

#[derive(Debug, Clone, Copy)]
pub struct ClientOptions {
    pub initialize_timeout: Duration,
    pub catalog_timeout: Duration,
    pub shutdown_timeout: Duration,
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self {
            initialize_timeout: Duration::from_secs(10),
            catalog_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(2),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionState {
    Connecting,
    Connected,
    #[default]
    Disconnected,
}

#[derive(Debug, Clone, Default)]
pub enum CatalogState {
    #[default]
    Loading,
    Unsupported,
    Ready(Arc<Vec<Tool>>),
    Failed(Arc<AppError>),
}

#[derive(Debug, Clone)]
pub enum EventKind {
    Connecting,
    Connected(Arc<ServerPeerInfo>),
    Catalog(CatalogState),
    Disconnected(Option<Arc<AppError>>),
    CleanupFailed(Arc<AppError>),
}

#[derive(Debug, Clone)]
pub struct ClientEvent {
    pub generation: u64,
    pub kind: EventKind,
}

/// State reducer shared by the summary CLI and future terminal UI.
#[derive(Debug, Default)]
pub struct ClientState {
    pub generation: u64,
    pub connection: ConnectionState,
    pub server: Option<Arc<ServerPeerInfo>>,
    pub catalog: CatalogState,
    pub error: Option<Arc<AppError>>,
}

impl ClientState {
    /// Call immediately after submitting reconnect to reject queued old events.
    pub fn begin_generation(&mut self, generation: u64) {
        if generation > self.generation {
            *self = Self {
                generation,
                connection: ConnectionState::Connecting,
                ..Default::default()
            };
        }
    }

    pub fn apply(&mut self, event: ClientEvent) -> bool {
        if event.generation < self.generation {
            return false;
        }
        self.begin_generation(event.generation);
        match event.kind {
            EventKind::Connecting => {
                self.connection = ConnectionState::Connecting
            }
            EventKind::Connected(info) => {
                self.connection = ConnectionState::Connected;
                self.server = Some(info);
            }
            EventKind::Catalog(catalog) => self.catalog = catalog,
            EventKind::Disconnected(error) => {
                self.connection = ConnectionState::Disconnected;
                self.error = error;
            }
            EventKind::CleanupFailed(error) => self.error = Some(error),
        }
        true
    }
}

enum Command {
    Reconnect(u64),
}

pub struct ClientHandle {
    commands: mpsc::Sender<Command>,
    shutdown: CancellationToken,
    worker: Option<JoinHandle<Result<()>>>,
    generation: u64,
    shutdown_timeout: Duration,
}

impl ClientHandle {
    pub fn spawn(
        url: &str,
        options: ClientOptions,
    ) -> Result<(Self, mpsc::UnboundedReceiver<ClientEvent>)> {
        let parsed = validate_url(url)?;
        let (commands, receiver) = mpsc::channel(16);
        let (events, event_receiver) = mpsc::unbounded_channel();
        let shutdown = CancellationToken::new();
        let worker = tokio::spawn(worker(
            parsed,
            options,
            receiver,
            events,
            shutdown.clone(),
        ));
        Ok((
            Self {
                commands,
                shutdown,
                worker: Some(worker),
                generation: 1,
                shutdown_timeout: options.shutdown_timeout,
            },
            event_receiver,
        ))
    }

    pub fn reconnect(&mut self) -> Result<u64> {
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(AppError::GenerationExhausted)?;
        self.commands
            .try_send(Command::Reconnect(generation))
            .map_err(|error| match error {
                mpsc::error::TrySendError::Full(_) => AppError::QueueFull,
                mpsc::error::TrySendError::Closed(_) => AppError::ChannelClosed,
            })?;
        self.generation = generation;
        Ok(generation)
    }

    pub async fn shutdown(mut self) -> Result<()> {
        self.shutdown.cancel();
        if let Some(mut worker) = self.worker.take() {
            let deadline = self.shutdown_timeout;
            match tokio::time::timeout(deadline, &mut worker).await {
                Ok(result) => result??,
                Err(_) => {
                    worker.abort();
                    let _ = worker.await;
                    return Err(AppError::Timeout {
                        operation: Operation::Shutdown,
                        duration: deadline,
                    });
                }
            }
        }
        Ok(())
    }
}

impl Drop for ClientHandle {
    fn drop(&mut self) {
        self.shutdown.cancel();
    }
}

fn emit(
    events: &mpsc::UnboundedSender<ClientEvent>,
    generation: u64,
    kind: EventKind,
) {
    let _ = events.send(ClientEvent { generation, kind });
}

struct Attempt {
    cancellation: CancellationToken,
    task: JoinHandle<Result<()>>,
}

impl Attempt {
    fn start(
        url: String,
        options: ClientOptions,
        events: mpsc::UnboundedSender<ClientEvent>,
        generation: u64,
    ) -> Self {
        let cancellation = CancellationToken::new();
        let task = tokio::spawn(run_generation(
            url,
            options,
            events,
            generation,
            cancellation.clone(),
        ));
        Self { cancellation, task }
    }

    async fn stop(mut self, timeout: Duration) -> Result<()> {
        self.cancellation.cancel();
        let bound = timeout + Duration::from_millis(100);
        match tokio::time::timeout(bound, &mut self.task).await {
            Ok(Ok(Err(
                error @ AppError::Timeout {
                    operation: Operation::Shutdown,
                    ..
                },
            )))
            | Ok(Ok(Err(error @ AppError::Worker(_)))) => Err(error),
            Ok(Err(error)) => Err(AppError::Worker(error)),
            // A failed attempt may finish just before reconnect/shutdown arrives.
            // Its connection error is not a new cleanup failure.
            Ok(Ok(_)) => Ok(()),
            Err(_) => {
                self.task.abort();
                let _ = (&mut self.task).await;
                Err(AppError::Timeout {
                    operation: Operation::Shutdown,
                    duration: bound,
                })
            }
        }
    }
}

impl Drop for Attempt {
    fn drop(&mut self) {
        self.cancellation.cancel();
        self.task.abort();
    }
}

async fn worker(
    url: String,
    options: ClientOptions,
    mut commands: mpsc::Receiver<Command>,
    events: mpsc::UnboundedSender<ClientEvent>,
    shutdown: CancellationToken,
) -> Result<()> {
    let mut generation = 1;
    let mut attempt = Some(Attempt::start(
        url.clone(),
        options,
        events.clone(),
        generation,
    ));
    loop {
        tokio::select! {
            biased;
            _ = shutdown.cancelled() => break,
            _ = events.closed() => break,
            command = commands.recv() => match command {
                Some(Command::Reconnect(next)) => {
                    if let Some(active) = attempt.take()
                        && let Err(error) = active.stop(options.shutdown_timeout).await {
                        emit(&events, generation, EventKind::CleanupFailed(Arc::new(error)));
                    }
                    generation = next;
                    attempt = Some(Attempt::start(url.clone(), options, events.clone(), generation));
                }
                None => break,
            },
            result = async {
                match attempt.as_mut() {
                    Some(active) => (&mut active.task).await,
                    None => std::future::pending().await,
                }
            } => {
                attempt.take();
                let error = match result {
                    Ok(Ok(())) => None,
                    Ok(Err(error)) => Some(Arc::new(error)),
                    Err(error) => Some(Arc::new(AppError::Worker(error))),
                };
                emit(&events, generation, EventKind::Disconnected(error));
            }
        }
    }
    if let Some(active) = attempt.take() {
        active.stop(options.shutdown_timeout).await?;
    }
    emit(&events, generation, EventKind::Disconnected(None));
    Ok(())
}

pub type McpService = RunningService<RoleClient, ClientConfig>;

pub fn validate_url(url: &str) -> Result<String> {
    let parsed = url::Url::parse(url)?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none()
    {
        return Err(AppError::UnsupportedUrl);
    }
    Ok(parsed.to_string())
}

pub async fn initialize(url: &str, deadline: Duration) -> Result<McpService> {
    let url = validate_url(url)?;
    tokio::time::timeout(
        deadline,
        ClientConfig::default()
            .serve(StreamableHttpClientTransport::from_uri(url)),
    )
    .await
    .map_err(|_| AppError::Timeout {
        operation: Operation::Initialize,
        duration: deadline,
    })?
    .map_err(|error| AppError::Initialize(Box::new(error)))
}

pub async fn close_service(
    service: &mut McpService,
    deadline: Duration,
) -> Result<()> {
    match service.close_with_timeout(deadline).await? {
        Some(QuitReason::JoinError(error)) => Err(AppError::Worker(error)),
        Some(_) => Ok(()),
        None => Err(AppError::Timeout {
            operation: Operation::Shutdown,
            duration: deadline,
        }),
    }
}

pub async fn list_all_tools(
    peer: &rmcp::service::Peer<RoleClient>,
    deadline: Duration,
) -> Result<Vec<Tool>> {
    tokio::time::timeout(deadline, peer.list_all_tools())
        .await
        .map_err(|_| AppError::Timeout {
            operation: Operation::ListTools,
            duration: deadline,
        })?
        .map_err(AppError::Request)
}

pub async fn call_tool(
    service: &McpService,
    params: CallToolRequestParams,
    deadline: Duration,
) -> Result<CallToolResult> {
    tokio::time::timeout(deadline, service.call_tool(params))
        .await
        .map_err(|_| AppError::Timeout {
            operation: Operation::CallTool,
            duration: deadline,
        })?
        .map_err(AppError::Request)
}

async fn run_generation(
    url: String,
    options: ClientOptions,
    events: mpsc::UnboundedSender<ClientEvent>,
    generation: u64,
    cancellation: CancellationToken,
) -> Result<()> {
    emit(&events, generation, EventKind::Connecting);
    let service = tokio::select! {
        biased;
        _ = cancellation.cancelled() => return Ok(()),
        result = initialize(&url,options.initialize_timeout) => result?,
    };
    let peer = service.peer().clone();
    let session = Session::new(service);
    let operation = async {
        let info = peer.peer_info().ok_or(AppError::MissingMetadata)?;
        emit(&events, generation, EventKind::Connected(info.clone()));
        if info.capabilities.tools.is_some() {
            let tools = list_all_tools(&peer, options.catalog_timeout).await;
            match tools {
                Ok(tools) => emit(
                    &events,
                    generation,
                    EventKind::Catalog(CatalogState::Ready(Arc::new(tools))),
                ),
                Err(
                    error @ AppError::Request(rmcp::ServiceError::McpError(_)),
                ) => emit(
                    &events,
                    generation,
                    EventKind::Catalog(CatalogState::Failed(Arc::new(error))),
                ),
                Err(error) => return Err(error),
            }
        } else {
            emit(
                &events,
                generation,
                EventKind::Catalog(CatalogState::Unsupported),
            );
        }
        std::future::pending::<Result<()>>().await
    };
    session
        .run(operation, &cancellation, options.shutdown_timeout)
        .await
}

/// Owns completion monitoring and guarantees cancellation when an attempt is aborted.
struct Session {
    cancellation: Option<rmcp::service::RunningServiceCancellationToken>,
    monitor: JoinHandle<std::result::Result<QuitReason, tokio::task::JoinError>>,
}

impl Session {
    fn new(service: McpService) -> Self {
        Self {
            cancellation: Some(service.cancellation_token()),
            monitor: tokio::spawn(service.waiting()),
        }
    }

    async fn run(
        mut self,
        operation: impl std::future::Future<Output = Result<()>>,
        cancellation: &CancellationToken,
        timeout: Duration,
    ) -> Result<()> {
        let result = tokio::select! {
            biased;
            _ = cancellation.cancelled() => Ok(()),
            result = &mut self.monitor => return match result {
                Ok(Ok(QuitReason::JoinError(error))) | Ok(Err(error)) | Err(error) => Err(AppError::Worker(error)),
                _ => Err(AppError::ConnectionClosed),
            },
            result = operation => result,
        };
        if let Some(token) = self.cancellation.take() {
            token.cancel();
        }
        let cleanup = match tokio::time::timeout(timeout, &mut self.monitor).await
        {
            Ok(Ok(Ok(QuitReason::JoinError(error))))
            | Ok(Ok(Err(error)))
            | Ok(Err(error)) => Err(AppError::Worker(error)),
            Ok(_) => Ok(()),
            Err(_) => Err(AppError::Timeout {
                operation: Operation::Shutdown,
                duration: timeout,
            }),
        };
        result.and(cleanup)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if let Some(token) = self.cancellation.take() {
            token.cancel();
        }
        self.monitor.abort();
    }
}
