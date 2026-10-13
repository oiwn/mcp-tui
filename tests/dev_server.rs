#[path = "../examples/support/server.rs"]
mod server;

use rmcp::{
    ServiceExt,
    model::{CallToolRequestParams, ClientConfig},
    transport::StreamableHttpClientTransport,
};
use serde_json::json;
use server::{Fixture, FixtureConfig};

#[tokio::test]
async fn catalogs_and_tool_results() -> Result<(), server::FixtureError> {
    for stateful in [false, true] {
        let fixture = Fixture::start(
            ([127, 0, 0, 1], 0).into(),
            FixtureConfig {
                stateful,
                ..Default::default()
            },
        )
        .await?;
        let client = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            ClientConfig::default()
                .serve(StreamableHttpClientTransport::from_uri(fixture.url())),
        )
        .await??;
        assert_eq!(
            client.peer_info().and_then(|info| info
                .server_info
                .as_ref()
                .map(|identity| identity.name.clone())),
            Some("mcpv-dev-server".into())
        );
        assert_eq!(client.list_all_tools().await?.len(), 3);
        assert_eq!(client.list_all_prompts().await?.len(), 1);
        assert_eq!(client.list_all_resources().await?.len(), 1);
        let ping = client
            .call_tool(
                CallToolRequestParams::new("ping")
                    .with_arguments(Default::default()),
            )
            .await?;
        assert_ne!(ping.is_error, Some(true));
        let echo =
            client
                .call_tool(CallToolRequestParams::new("echo").with_arguments(
                    serde_json::from_value(json!({"text":"hello"}))?,
                ))
                .await?;
        assert_eq!(
            echo.content
                .first()
                .and_then(|content| content.as_text())
                .map(|text| text.text.as_str()),
            Some("hello")
        );
        let error = client
            .call_tool(CallToolRequestParams::new("echo").with_arguments(
                serde_json::from_value(json!({"text":"hello","fail":true}))?,
            ))
            .await?;
        assert_eq!(error.is_error, Some(true));
        let slow =
            client
                .call_tool(CallToolRequestParams::new("slow").with_arguments(
                    serde_json::from_value(json!({"delay_ms":0}))?,
                ))
                .await?;
        assert_ne!(slow.is_error, Some(true));
        let slow_error = client
            .call_tool(CallToolRequestParams::new("slow").with_arguments(
                serde_json::from_value(json!({"delay_ms":5001}))?,
            ))
            .await?;
        assert_eq!(slow_error.is_error, Some(true));
        assert_eq!(
            fixture
                .session_header_seen
                .load(std::sync::atomic::Ordering::SeqCst),
            stateful
        );
        client.cancel().await?;
        fixture.stop().await?;
    }
    Ok(())
}
