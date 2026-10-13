use crate::{cli::Args, error::AppError, error::Result};

pub async fn run(_args: Args) -> Result<()> {
    Err(AppError::FeatureUnavailable)
}
