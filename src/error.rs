use std::{sync::Arc, time::Duration};

use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub enum Operation {
    Initialize,
    ListTools,
    Shutdown,
}

impl std::fmt::Display for Operation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{}",
            match self {
                Self::Initialize => "initialization",
                Self::ListTools => "tool listing",
                Self::Shutdown => "shutdown",
            }
        )
    }
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid server URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("server URL must use HTTP or HTTPS and include a host")]
    UnsupportedUrl,
    #[error("initialization failed: {0}")]
    Initialize(#[source] Box<rmcp::service::ClientInitializeError>),
    #[error("request failed: {0}")]
    Request(#[from] rmcp::ServiceError),
    #[error("{operation} timed out after {duration:?}")]
    Timeout {
        operation: Operation,
        duration: Duration,
    },
    #[error("server handshake metadata is missing")]
    MissingMetadata,
    #[error("server connection closed")]
    ConnectionClosed,
    #[error("client command queue is full")]
    QueueFull,
    #[error("client worker channel closed")]
    ChannelClosed,
    #[error("connection generation counter exhausted")]
    GenerationExhausted,
    #[error("client worker failed: {0}")]
    Worker(#[from] tokio::task::JoinError),
    #[error("signal handler failed: {0}")]
    Signal(#[source] std::io::Error),
    #[error("I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Reported(#[source] Arc<AppError>),
}

pub type Result<T> = std::result::Result<T, AppError>;
