use clap::{CommandFactory, Parser, Subcommand, error::ErrorKind};
use std::time::{Duration, Instant};

#[derive(Debug, Parser)]
#[command(version, about = "Inspect and drive a running MCP server")]
pub struct Args {
    /// Streamable HTTP MCP server URL.
    #[arg(value_name = "SERVER_URL")]
    pub server_url: String,
    /// Emit one machine-readable envelope (headless commands only).
    #[arg(long, global = true)]
    pub json: bool,
    /// Headless initialization/request deadline in whole seconds (default 30).
    #[arg(long, global = true, value_parser = parse_timeout)]
    pub timeout: Option<Duration>,
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Show handshake metadata, capabilities, and server instructions.
    Info,
    /// Fetch every page of tool definitions.
    ToolsList,
    /// Show the full definition of an exactly named tool.
    ToolsGet { name: String },
    /// Call one tool with a JSON object; use --args - to read stdin.
    ToolsCall {
        name: String,
        #[arg(long, allow_hyphen_values = true)]
        args: Option<String>,
    },
}

impl Command {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::ToolsList => "tools-list",
            Self::ToolsGet { .. } => "tools-get",
            Self::ToolsCall { .. } => "tools-call",
        }
    }
}

impl Args {
    pub fn validate_mode(&self) -> Result<(), clap::Error> {
        if self.command.is_none() && (self.json || self.timeout.is_some()) {
            return Err(Self::command().error(
                ErrorKind::ArgumentConflict,
                "--json and --timeout require a headless command",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
enum TimeoutError {
    #[error("timeout must be a positive whole number of seconds: {0}")]
    Number(#[from] std::num::ParseIntError),
    #[error("timeout must be positive and representable")]
    Range,
}

fn parse_timeout(value: &str) -> Result<Duration, TimeoutError> {
    let duration = Duration::from_secs(value.parse()?);
    if duration.is_zero() || Instant::now().checked_add(duration).is_none() {
        return Err(TimeoutError::Range);
    }
    Ok(duration)
}

#[cfg(test)]
mod tests {
    use clap::{Parser, error::ErrorKind};

    use super::Args;

    #[test]
    fn preserves_server_url() -> Result<(), clap::Error> {
        let url = "http://127.0.0.1:7400/mcp?test=1";
        let args = Args::try_parse_from(["mcpv", url])?;
        assert_eq!(args.server_url, url);
        Ok(())
    }

    #[test]
    fn requires_url_and_rejects_unknown_commands() {
        for (arguments, expected) in [
            (vec!["mcpv"], ErrorKind::MissingRequiredArgument),
            (
                vec!["mcpv", "http://localhost/mcp", "extra"],
                ErrorKind::UnknownArgument,
            ),
        ] {
            let error = Args::try_parse_from(arguments).err();
            assert_eq!(error.as_ref().map(clap::Error::kind), Some(expected));
            assert_eq!(error.as_ref().map(clap::Error::exit_code), Some(2));
        }
    }

    #[test]
    fn help_and_version_need_no_url() {
        for (flag, expected) in [
            ("--help", ErrorKind::DisplayHelp),
            ("--version", ErrorKind::DisplayVersion),
        ] {
            let error = Args::try_parse_from(["mcpv", flag]).err();
            assert_eq!(error.as_ref().map(clap::Error::kind), Some(expected));
            assert_eq!(error.as_ref().map(clap::Error::exit_code), Some(0));
        }
    }
}
