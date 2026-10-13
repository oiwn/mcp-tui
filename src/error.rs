use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("connection and terminal UI are not implemented yet")]
    FeatureUnavailable,
}

pub type Result<T> = std::result::Result<T, AppError>;
