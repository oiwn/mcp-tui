use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about = "Inspect and drive a running MCP server")]
pub struct Args {
    /// Streamable HTTP MCP server URL.
    #[arg(value_name = "SERVER_URL")]
    pub server_url: String,
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
    fn requires_exactly_one_argument() {
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
