use std::{
    borrow::Cow,
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

use rmcp::{
    ErrorData, RoleServer, ServerHandler,
    model::{
        CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock,
        Implementation, ListPromptsResult, ListResourcesResult, ListToolsResult,
        PaginatedRequestParams, Prompt, ProtocolVersion, Resource,
        ServerCapabilities, ServerConfig, Tool,
    },
    service::RequestContext,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService,
        session::local::LocalSessionManager,
    },
};
use serde_json::{Value, json};
use tokio::{net::TcpListener, task::JoinHandle};
use tokio_util::sync::CancellationToken;

#[derive(Debug, thiserror::Error)]
pub enum FixtureError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),
    #[error("server shutdown timed out")]
    ShutdownTimeout,
    #[error(transparent)]
    Initialize(Box<rmcp::service::ClientInitializeError>),
    #[error(transparent)]
    Request(#[from] rmcp::ServiceError),
    #[error(transparent)]
    Timeout(#[from] tokio::time::error::Elapsed),
}

impl From<rmcp::service::ClientInitializeError> for FixtureError {
    fn from(error: rmcp::service::ClientInitializeError) -> Self {
        Self::Initialize(Box::new(error))
    }
}

#[derive(Clone)]
pub struct FixtureConfig {
    pub stateful: bool,
    pub tools_enabled: bool,
    pub empty_tools: bool,
    pub page_size: Option<usize>,
    pub list_error: bool,
    pub list_delay: Duration,
    pub list_started: Arc<tokio::sync::Notify>,
    pub list_requests: Arc<AtomicUsize>,
    pub http_requests: Arc<AtomicUsize>,
}

impl Default for FixtureConfig {
    fn default() -> Self {
        Self {
            stateful: false,
            tools_enabled: true,
            empty_tools: false,
            page_size: None,
            list_error: false,
            list_delay: Duration::ZERO,
            list_started: Arc::new(tokio::sync::Notify::new()),
            list_requests: Arc::new(AtomicUsize::new(0)),
            http_requests: Arc::new(AtomicUsize::new(0)),
        }
    }
}

#[derive(Clone)]
struct DevServer {
    config: FixtureConfig,
    tools: Arc<Vec<Tool>>,
}

impl ServerHandler for DevServer {
    fn get_info(&self) -> ServerConfig {
        let mut info = ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .enable_resources()
                .build(),
        );
        info.server_info = Implementation::new("mcpv-dev-server", "0.1.0");
        if !self.config.tools_enabled {
            info.capabilities.tools = None;
        }
        if self.config.stateful {
            info.protocol_version = ProtocolVersion::V_2025_11_25;
        }
        info
    }

    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        if self.config.stateful {
            Cow::Owned(vec![ProtocolVersion::V_2025_11_25])
        } else {
            Cow::Borrowed(ProtocolVersion::KNOWN_VERSIONS)
        }
    }

    async fn list_tools(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        self.config.list_requests.fetch_add(1, Ordering::SeqCst);
        self.config.list_started.notify_one();
        tokio::time::sleep(self.config.list_delay).await;
        if self.config.list_error {
            return Err(ErrorData::internal_error("fixture catalog error", None));
        }
        if self.config.empty_tools {
            return Ok(ListToolsResult::default());
        }
        let start = request
            .and_then(|params| params.cursor)
            .map(|cursor| cursor.parse::<usize>())
            .transpose()
            .map_err(|_| ErrorData::invalid_params("invalid cursor", None))?
            .unwrap_or(0);
        if start > self.tools.len() {
            return Err(ErrorData::invalid_params("invalid cursor", None));
        }
        let end = start
            .saturating_add(
                self.config.page_size.unwrap_or(self.tools.len()).max(1),
            )
            .min(self.tools.len());
        Ok(ListToolsResult {
            tools: self.tools[start..end].to_vec(),
            next_cursor: (end < self.tools.len()).then(|| end.to_string()),
            ..Default::default()
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let arguments = request.arguments.unwrap_or_default();
        let result = match request.name.as_ref() {
            "ping" => CallToolResult::success(vec![ContentBlock::text(
                r#"{"ok":true,"server":"mcpv-dev-server"}"#,
            )]),
            "echo"
                if arguments.get("fail").and_then(Value::as_bool)
                    == Some(true) =>
            {
                CallToolResult::error(vec![ContentBlock::text(
                    "echo rejected arguments: fail must not be true",
                )])
            }
            "echo" => match arguments.get("text").and_then(Value::as_str) {
                Some(text) => {
                    CallToolResult::success(vec![ContentBlock::text(text)])
                }
                None => CallToolResult::error(vec![ContentBlock::text(
                    "echo requires a string text argument",
                )]),
            },
            "slow" => {
                let delay = match arguments.get("delay_ms") {
                    None => 250,
                    Some(value) => match value
                        .as_u64()
                        .filter(|delay| *delay <= 5000)
                    {
                        Some(delay) => delay,
                        None => return Ok(CallToolResult::error(vec![
                            ContentBlock::text(
                                "delay_ms must be an integer between 0 and 5000",
                            ),
                        ])
                        .into()),
                    },
                };
                tokio::time::sleep(Duration::from_millis(delay)).await;
                CallToolResult::success(vec![ContentBlock::text(
                    json!({"delay_ms": delay}).to_string(),
                )])
            }
            _ => return Err(ErrorData::invalid_params("unknown tool", None)),
        };
        Ok(result.into())
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, ErrorData> {
        Ok(ListPromptsResult {
            prompts: vec![Prompt::new(
                "debug",
                Some("A sample debugging prompt"),
                None,
            )],
            ..Default::default()
        })
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        Ok(ListResourcesResult {
            resources: vec![Resource::new("mock://status", "status")],
            ..Default::default()
        })
    }
}

fn tools() -> Result<Vec<Tool>, serde_json::Error> {
    [
        ("ping", "Return deterministic JSON; accepts {}", json!({"type":"object","properties":{}})),
        ("echo", "Echo text or reject fail=true", json!({"type":"object","properties":{"text":{"type":"string","examples":["hello"]},"fail":{"type":"boolean","default":false}},"required":["text"]})),
        ("slow", "Delay a response to exercise latency", json!({"type":"object","properties":{"delay_ms":{"type":"integer","default":250,"minimum":0,"maximum":5000}}})),
    ].into_iter().map(|(name, description, schema)| {
        Ok(Tool::new(name, description, serde_json::from_value::<serde_json::Map<String, Value>>(schema)?))
    }).collect()
}

pub struct Fixture {
    pub address: SocketAddr,
    pub session_header_seen: Arc<AtomicBool>,
    cancellation: CancellationToken,
    task: Option<JoinHandle<Result<(), std::io::Error>>>,
}

impl Fixture {
    pub async fn start(
        address: SocketAddr,
        config: FixtureConfig,
    ) -> Result<Self, FixtureError> {
        let server = DevServer {
            config: config.clone(),
            tools: Arc::new(tools()?),
        };
        let cancellation = CancellationToken::new();
        let service: StreamableHttpService<DevServer, LocalSessionManager> =
            StreamableHttpService::new(
                move || Ok(server.clone()),
                Default::default(),
                StreamableHttpServerConfig::default()
                    .with_legacy_session_mode(config.stateful)
                    .with_json_response(true)
                    .with_cancellation_token(cancellation.child_token()),
            );
        let session_header_seen = Arc::new(AtomicBool::new(false));
        let router = axum::Router::new().nest_service("/mcp", service).layer(
            axum::middleware::from_fn_with_state(
                (session_header_seen.clone(), config.http_requests.clone()),
                observe_headers,
            ),
        );
        let listener = TcpListener::bind(address).await?;
        let address = listener.local_addr()?;
        let shutdown = cancellation.clone();
        let task = tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(shutdown.cancelled_owned())
                .await
        });
        Ok(Self {
            address,
            session_header_seen,
            cancellation,
            task: Some(task),
        })
    }

    pub fn url(&self) -> String {
        format!("http://{}/mcp", self.address)
    }

    pub async fn stop(mut self) -> Result<(), FixtureError> {
        self.cancellation.cancel();
        if let Some(mut task) = self.task.take() {
            match tokio::time::timeout(Duration::from_secs(2), &mut task).await {
                Ok(result) => result??,
                Err(_) => {
                    task.abort();
                    let _ = task.await;
                    return Err(FixtureError::ShutdownTimeout);
                }
            }
        }
        Ok(())
    }
}

async fn observe_headers(
    axum::extract::State((seen, requests)): axum::extract::State<(
        Arc<AtomicBool>,
        Arc<AtomicUsize>,
    )>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    requests.fetch_add(1, Ordering::SeqCst);
    let response = next.run(request).await;
    if response.headers().contains_key("mcp-session-id") {
        seen.store(true, Ordering::SeqCst);
    }
    response
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.cancellation.cancel();
        if let Some(task) = &self.task {
            task.abort();
        }
    }
}
