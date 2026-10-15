#[path = "../examples/support/server.rs"]
mod server;

use serde_json::{Value, json};
use server::{Fixture, FixtureConfig, FixtureError};
use std::{process::Stdio, sync::atomic::Ordering, time::Duration};
use tokio::{io::AsyncWriteExt, process::Command};

#[derive(Debug, thiserror::Error)]
enum TestError {
    #[error(transparent)]
    Fixture(#[from] FixtureError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Timeout(#[from] tokio::time::error::Elapsed),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("stdin or process ID unavailable")]
    Missing,
}

async fn run(
    url: &str,
    args: &[&str],
    input: Option<&[u8]>,
) -> Result<std::process::Output, TestError> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mcpv"))
        .arg(url)
        .args(args)
        .kill_on_drop(true)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Some(input) = input {
        let mut stdin = child.stdin.take().ok_or(TestError::Missing)?;
        if let Err(error) = stdin.write_all(input).await
            && error.kind() != std::io::ErrorKind::BrokenPipe
        {
            return Err(error.into());
        }
    } else {
        child.stdin.take();
    }
    Ok(
        tokio::time::timeout(Duration::from_secs(10), child.wait_with_output())
            .await??,
    )
}

fn document(
    output: &std::process::Output,
    status: i32,
    stdout: bool,
) -> Result<Value, TestError> {
    assert_eq!(
        output.status.code(),
        Some(status),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let (stream, other) = if stdout {
        (&output.stdout, &output.stderr)
    } else {
        (&output.stderr, &output.stdout)
    };
    assert!(other.is_empty(), "unexpected secondary output");
    assert!(!stream.contains(&27), "ANSI in JSON output");
    let result: Value = serde_json::from_slice(stream)?;
    assert_eq!(result["schemaVersion"], 1);
    assert!(result["elapsedMs"].is_u64());
    assert_eq!(result["ok"], status == 0);
    Ok(result)
}

#[tokio::test]
async fn inspect_catalog_schema_and_call_in_both_modes() -> Result<(), TestError>
{
    for stateful in [false, true] {
        let config = FixtureConfig {
            stateful,
            page_size: Some(1),
            ..Default::default()
        };
        let lists = config.list_requests.clone();
        let fixture = Fixture::start(([127, 0, 0, 1], 0).into(), config).await?;
        let url = fixture.url();
        let info =
            document(&run(&url, &["info", "--json"], None).await?, 0, true)?;
        assert_eq!(info["data"]["serverInfo"]["name"], "mcpv-dev-server");
        assert_eq!(lists.load(Ordering::SeqCst), 0, "info must not list tools");
        let tools = document(
            &run(&url, &["tools-list", "--json"], None).await?,
            0,
            true,
        )?;
        assert_eq!(tools["data"].as_array().map(Vec::len), Some(3));
        assert_eq!(lists.load(Ordering::SeqCst), 3);
        let tool = document(
            &run(&url, &["tools-get", "echo", "--json"], None).await?,
            0,
            true,
        )?;
        assert_eq!(
            tool["data"]["inputSchema"]["properties"]["text"]["type"],
            "string"
        );
        let ping = document(
            &run(&url, &["tools-call", "ping", "--json"], None).await?,
            0,
            true,
        )?;
        assert!(
            ping["data"]["content"][0]["text"]
                .as_str()
                .is_some_and(|text| text.contains("mcpv-dev-server"))
        );
        let echo = document(
            &run(
                &url,
                &[
                    "tools-call",
                    "echo",
                    "--args",
                    r#"{"text":"hello"}"#,
                    "--json",
                ],
                None,
            )
            .await?,
            0,
            true,
        )?;
        assert_eq!(echo["data"]["content"][0]["text"], "hello");
        let stdin = document(
            &run(
                &url,
                &["tools-call", "echo", "--args", "-", "--json"],
                Some(br#"{"text":"from stdin"}"#),
            )
            .await?,
            0,
            true,
        )?;
        assert_eq!(stdin["data"]["content"][0]["text"], "from stdin");
        assert_eq!(fixture.session_header_seen.load(Ordering::SeqCst), stateful);
        fixture.stop().await?;
    }
    Ok(())
}

#[tokio::test]
async fn rejects_invalid_arguments_before_any_http_request()
-> Result<(), TestError> {
    let config = FixtureConfig::default();
    let requests = config.http_requests.clone();
    let fixture = Fixture::start(([127, 0, 0, 1], 0).into(), config).await?;
    for input in ["{", "[]", "null", "\"text\""] {
        let output = run(
            &fixture.url(),
            &["tools-call", "ping", "--args", input, "--json"],
            None,
        )
        .await?;
        let error = document(&output, 2, false)?;
        assert_eq!(error["error"]["kind"], "arguments");
    }
    let oversized = vec![b'a'; 1024 * 1024 + 1];
    document(
        &run(
            &fixture.url(),
            &["tools-call", "ping", "--args", "-", "--json"],
            Some(&oversized),
        )
        .await?,
        2,
        false,
    )?;
    assert_eq!(requests.load(Ordering::SeqCst), 0);
    fixture.stop().await?;
    Ok(())
}

#[tokio::test]
async fn distinguishes_content_protocol_and_lookup_failures()
-> Result<(), TestError> {
    let fixture =
        Fixture::start(([127, 0, 0, 1], 0).into(), FixtureConfig::default())
            .await?;
    let url = fixture.url();
    let failure = document(
        &run(
            &url,
            &[
                "tools-call",
                "echo",
                "--args",
                r#"{"text":"hello","fail":true}"#,
                "--json",
            ],
            None,
        )
        .await?,
        3,
        true,
    )?;
    assert_eq!(failure["data"]["isError"], true);
    assert_eq!(failure["error"]["kind"], "tool_content");
    assert!(
        failure["data"]["content"][0]["text"]
            .as_str()
            .is_some_and(|text| text.contains("rejected"))
    );
    let protocol = document(
        &run(&url, &["tools-call", "unknown", "--json"], None).await?,
        1,
        false,
    )?;
    assert_eq!(protocol["error"]["kind"], "protocol");
    let unknown = document(
        &run(&url, &["tools-get", "unknown", "--json"], None).await?,
        2,
        false,
    )?;
    assert_eq!(unknown["error"]["kind"], "tool_not_found");
    fixture.stop().await?;
    Ok(())
}

#[tokio::test]
async fn empty_unsupported_catalogs_and_text_output() -> Result<(), TestError> {
    for (config, status) in [
        (
            FixtureConfig {
                empty_tools: true,
                ..Default::default()
            },
            0,
        ),
        (
            FixtureConfig {
                tools_enabled: false,
                ..Default::default()
            },
            1,
        ),
    ] {
        let fixture = Fixture::start(([127, 0, 0, 1], 0).into(), config).await?;
        let output = run(&fixture.url(), &["tools-list", "--json"], None).await?;
        let result = document(&output, status, status == 0)?;
        if status == 0 {
            assert_eq!(result["data"], json!([]));
        } else {
            assert_eq!(result["error"]["kind"], "unsupported_capability");
        }
        fixture.stop().await?;
    }
    let fixture =
        Fixture::start(([127, 0, 0, 1], 0).into(), FixtureConfig::default())
            .await?;
    let output = run(&fixture.url(), &["tools-list"], None).await?;
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("ping"));
    assert!(text.contains("3 tools"));
    assert!(!output.stdout.contains(&27));
    fixture.stop().await?;
    Ok(())
}

#[tokio::test]
async fn request_deadline_override_and_cli_usage() -> Result<(), TestError> {
    let fixture =
        Fixture::start(([127, 0, 0, 1], 0).into(), FixtureConfig::default())
            .await?;
    let url = fixture.url();
    let failure = document(
        &run(
            &url,
            &[
                "tools-call",
                "slow",
                "--args",
                r#"{"delay_ms":1300}"#,
                "--timeout",
                "1",
                "--json",
            ],
            None,
        )
        .await?,
        1,
        false,
    )?;
    assert_eq!(failure["error"]["kind"], "timeout");
    let success = document(
        &run(
            &url,
            &[
                "tools-call",
                "slow",
                "--args",
                r#"{"delay_ms":1300}"#,
                "--timeout",
                "3",
                "--json",
            ],
            None,
        )
        .await?,
        0,
        true,
    )?;
    assert!(
        success["elapsedMs"]
            .as_u64()
            .is_some_and(|value| value >= 1000)
    );
    for args in [
        vec!["--json"],
        vec!["info", "--timeout", "0"],
        vec!["info", "--timeout", "18446744073709551615"],
        vec!["tools-get"],
    ] {
        let output = run(&url, &args, None).await?;
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
    }
    fixture.stop().await?;
    Ok(())
}

#[tokio::test]
async fn unfinished_stdin_has_a_deadline() -> Result<(), TestError> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mcpv"))
        .args([
            "http://127.0.0.1:1/mcp",
            "tools-call",
            "ping",
            "--args",
            "-",
            "--json",
            "--timeout",
            "1",
        ])
        .kill_on_drop(true)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdin = child.stdin.take().ok_or(TestError::Missing)?;
    let output =
        tokio::time::timeout(Duration::from_secs(4), child.wait_with_output())
            .await??;
    drop(stdin);
    assert_eq!(document(&output, 1, false)?["error"]["kind"], "timeout");
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn ctrl_c_exits_even_with_an_unfinished_stdin_pipe() -> Result<(), TestError>
{
    let mut child = Command::new(env!("CARGO_BIN_EXE_mcpv"))
        .args([
            "http://127.0.0.1:1/mcp",
            "tools-call",
            "ping",
            "--args",
            "-",
            "--json",
        ])
        .kill_on_drop(true)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdin = child.stdin.take().ok_or(TestError::Missing)?;
    stdin.write_all(b"{").await?;
    // Exceed pipe capacity: completion proves the reader started after the
    // biased select registered SIGINT. Keep EOF pending without arbitrary sleeps.
    stdin.write_all(&vec![b' '; 512 * 1024]).await?;
    let pid = child.id().ok_or(TestError::Missing)?;
    let status = Command::new("/bin/kill")
        .args(["-INT", &pid.to_string()])
        .status()
        .await?;
    assert!(status.success());
    let output =
        tokio::time::timeout(Duration::from_secs(4), child.wait_with_output())
            .await??;
    drop(stdin);
    assert_eq!(
        document(&output, 130, false)?["error"]["kind"],
        "interrupted"
    );
    Ok(())
}
