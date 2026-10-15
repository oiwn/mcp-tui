use std::{
    io::{self, Read, Write},
    process::ExitCode,
    time::{Duration, Instant},
};

use mcpv::{
    client::{McpService, close_service, initialize, validate_url},
    error::{AppError, Operation, Result},
};
use rmcp::model::{CallToolRequestParams, JsonObject};
use serde::Serialize;
use serde_json::Value;

use crate::cli::{Args, Command};

const MAX_ARGUMENT_BYTES: usize = 1024 * 1024;

#[derive(Serialize)]
struct ErrorDocument {
    kind: &'static str,
    message: String,
}

impl ErrorDocument {
    fn new(error: &AppError) -> Self {
        Self {
            kind: category(error),
            message: error.to_string(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Report {
    schema_version: u8,
    command: &'static str,
    server_url: String,
    ok: bool,
    elapsed_ms: u64,
    data: Option<Value>,
    error: Option<ErrorDocument>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cleanup_error: Option<ErrorDocument>,
}

fn category(error: &AppError) -> &'static str {
    match error {
        AppError::Initialize(error) if matches!(error.as_ref(),
            rmcp::service::ClientInitializeError::JsonRpcError(_)
            | rmcp::service::ClientInitializeError::NoCompatibleProtocolVersion{..}
            | rmcp::service::ClientInitializeError::ExpectedInitResponse(_)
            | rmcp::service::ClientInitializeError::ExpectedInitResult(_)
            | rmcp::service::ClientInitializeError::ConflictInitResponseId(..)
            | rmcp::service::ClientInitializeError::UncorrelatedErrorResponse{..}
        ) => "protocol",
        AppError::ArgumentJson(_)
        | AppError::ArgumentRoot
        | AppError::ArgumentSize => "arguments",
        AppError::ToolNotFound(_) => "tool_not_found",
        AppError::ToolsUnsupported => "unsupported_capability",
        AppError::ToolContent => "tool_content",
        AppError::Interrupted => "interrupted",
        AppError::Timeout { .. } => "timeout",
        AppError::Initialize(_)
        | AppError::ConnectionClosed
        | AppError::Url(_)
        | AppError::UnsupportedUrl => "connection",
        AppError::Request(rmcp::ServiceError::McpError(_) | rmcp::ServiceError::UnexpectedResponse) => "protocol",
        AppError::Request(rmcp::ServiceError::Timeout { .. }) => "timeout",
        AppError::Request(_) => "connection",
        AppError::Io(_) => "io",
        AppError::Serialization(_) => "serialization",
        AppError::Reported(error) => category(error),
        _ => "internal",
    }
}

fn code(error: &AppError) -> u8 {
    match error {
        AppError::ArgumentJson(_)
        | AppError::ArgumentRoot
        | AppError::ArgumentSize
        | AppError::ToolNotFound(_) => 2,
        AppError::ToolContent => 3,
        AppError::Interrupted => 130,
        AppError::Reported(error) => code(error),
        _ => 1,
    }
}

fn elapsed(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}

async fn arguments(command: &Command) -> Result<JsonObject> {
    let Command::ToolsCall { args, .. } = command else {
        return Ok(JsonObject::new());
    };
    let Some(input) = args else {
        return Ok(JsonObject::new());
    };
    let bytes = if input == "-" {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        // Unlike Tokio's stdin blocking task, this thread cannot hold up runtime shutdown.
        std::thread::Builder::new()
            .name("mcpv-stdin".into())
            .spawn(move || {
                let mut bytes = Vec::new();
                let result = io::stdin()
                    .lock()
                    .take((MAX_ARGUMENT_BYTES + 1) as u64)
                    .read_to_end(&mut bytes)
                    .map(|_| bytes);
                let _ = sender.send(result);
            })?;
        receiver
            .await
            .map_err(|_| AppError::ArgumentReaderClosed)??
    } else {
        input.as_bytes().to_vec()
    };
    if bytes.len() > MAX_ARGUMENT_BYTES {
        return Err(AppError::ArgumentSize);
    }
    match serde_json::from_slice(&bytes).map_err(AppError::ArgumentJson)? {
        Value::Object(object) => Ok(object),
        _ => Err(AppError::ArgumentRoot),
    }
}

struct Output {
    data: Value,
    tool_failure: bool,
}

async fn execute(
    service: &McpService,
    command: &Command,
    arguments: JsonObject,
    deadline: Duration,
) -> Result<Output> {
    let info = service.peer_info().ok_or(AppError::MissingMetadata)?;
    if matches!(command, Command::Info) {
        return Ok(Output {
            data: serde_json::to_value(info.as_ref())?,
            tool_failure: false,
        });
    }
    if info.capabilities.tools.is_none() {
        return Err(AppError::ToolsUnsupported);
    }
    let data = match command {
        Command::ToolsList => serde_json::to_value(
            mcpv::client::list_all_tools(service.peer(), deadline).await?,
        )?,
        Command::ToolsGet { name } => {
            let tool = mcpv::client::list_all_tools(service.peer(), deadline)
                .await?
                .into_iter()
                .find(|tool| tool.name.as_ref() == name)
                .ok_or_else(|| AppError::ToolNotFound(name.clone()))?;
            serde_json::to_value(tool)?
        }
        Command::ToolsCall { name, .. } => {
            let result = mcpv::client::call_tool(
                service,
                CallToolRequestParams::new(name.clone())
                    .with_arguments(arguments),
                deadline,
            )
            .await?;
            let tool_failure = result.is_error == Some(true);
            return Ok(Output {
                data: serde_json::to_value(result)?,
                tool_failure,
            });
        }
        Command::Info => serde_json::to_value(info.as_ref())?,
    };
    Ok(Output {
        data,
        tool_failure: false,
    })
}

pub async fn run(args: &Args, command: Command) -> ExitCode {
    let deadline = args.timeout.unwrap_or(Duration::from_secs(30));
    let mut report = Report {
        schema_version: 1,
        command: command.name(),
        server_url: args.server_url.clone(),
        ok: false,
        elapsed_ms: 0,
        data: None,
        error: None,
        cleanup_error: None,
    };
    let interrupt = tokio::signal::ctrl_c();
    tokio::pin!(interrupt);
    let mut service = None;
    let mut timing_start = None;
    let operation = async {
        validate_url(&args.server_url)?;
        let arguments = tokio::time::timeout(deadline, arguments(&command))
            .await
            .map_err(|_| AppError::Timeout {
                operation: Operation::ReadArguments,
                duration: deadline,
            })??;
        let initialized = Instant::now();
        timing_start = Some(initialized);
        service = Some(initialize(&args.server_url, deadline).await?);
        let connection = service.as_ref().ok_or(AppError::ConnectionClosed)?;
        let start = if matches!(command, Command::Info) {
            initialized
        } else {
            Instant::now()
        };
        timing_start = Some(start);
        execute(connection, &command, arguments, deadline).await
    };
    let outcome = tokio::select! {
        biased;
        signal=&mut interrupt=>match signal {Ok(())=>Err(AppError::Interrupted),Err(error)=>Err(AppError::Signal(error))},
        outcome=operation=>outcome,
    };
    report.elapsed_ms = timing_start.map(elapsed).unwrap_or(0);
    let mut error = match outcome {
        Ok(output) => {
            report.data = Some(output.data);
            if output.tool_failure {
                Some(AppError::ToolContent)
            } else {
                None
            }
        }
        Err(error) => Some(error),
    };
    if let Some(service) = service.as_mut() {
        let cleanup = tokio::select! {
            biased;
            signal=&mut interrupt, if !matches!(error,Some(AppError::Interrupted))=>{
                let primary=match signal {Ok(())=>AppError::Interrupted,Err(error)=>AppError::Signal(error)};
                report.cleanup_error=error.as_ref().map(ErrorDocument::new);
                error=Some(primary);
                close_service(service,Duration::from_secs(2)).await
            },
            cleanup=close_service(service,Duration::from_secs(2))=>cleanup,
        };
        if let Err(cleanup) = cleanup {
            if error.is_some() {
                report.cleanup_error = Some(ErrorDocument::new(&cleanup));
            } else {
                error = Some(cleanup);
            }
        }
    }
    let exit = error.as_ref().map(code).unwrap_or(0);
    report.ok = error.is_none();
    report.error = error.as_ref().map(ErrorDocument::new);
    // MCP content failures remain result documents on stdout; execution failures use stderr.
    let result_stream = exit == 0 || exit == 3;
    let output = write_report(&report, args.json, result_stream);
    if output.is_err() {
        return ExitCode::FAILURE;
    }
    ExitCode::from(exit)
}

fn write_report(report: &Report, json: bool, result_stream: bool) -> Result<()> {
    let mut stream: Box<dyn Write> = if result_stream {
        Box::new(io::stdout().lock())
    } else {
        Box::new(io::stderr().lock())
    };
    if json {
        serde_json::to_writer(&mut stream, report)?;
        writeln!(stream)?;
    } else if let Some(data) = &report.data {
        match report.command {
            "tools-list" => {
                if let Some(tools) = data.as_array() {
                    for tool in tools {
                        writeln!(
                            stream,
                            "{} — {}",
                            single_line(
                                tool.get("name")
                                    .and_then(Value::as_str)
                                    .unwrap_or("unknown")
                            ),
                            single_line(
                                tool.get("description")
                                    .and_then(Value::as_str)
                                    .unwrap_or("")
                            )
                        )?;
                    }
                    writeln!(stream, "{} tools", tools.len())?;
                }
            }
            _ => writeln!(stream, "{}", serde_json::to_string_pretty(data)?)?,
        }
        if let Some(error) = &report.error {
            writeln!(
                stream,
                "ERROR [{}]: {}",
                error.kind,
                single_line(&error.message)
            )?;
        }
        writeln!(stream, "Elapsed: {} ms", report.elapsed_ms)?;
    } else if let Some(error) = &report.error {
        writeln!(
            stream,
            "ERROR [{}]: {}",
            error.kind,
            single_line(&error.message)
        )?;
    }
    stream.flush()?;
    Ok(())
}

fn single_line(text: &str) -> String {
    text.chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect()
}
