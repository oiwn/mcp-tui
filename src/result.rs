use mcpv::client::{CallId, CallOutcome};
use rmcp::model::{CallToolResult, ContentBlock};
use std::time::{Duration, Instant};

pub struct CallDisplay {
    pub id: CallId,
    pub tool: String,
    pub started: Option<Instant>,
    pub elapsed: Option<Duration>,
    pub outcome: Option<CallOutcome>,
    pub scroll: usize,
    pub max_scroll: usize,
    pub text: String,
}

impl CallDisplay {
    pub fn new(id: CallId, tool: String) -> Self {
        Self {
            id,
            tool,
            started: None,
            elapsed: None,
            outcome: None,
            scroll: 0,
            max_scroll: 0,
            text: "Waiting for tool response…".into(),
        }
    }

    pub fn pending(&self) -> bool {
        self.outcome.is_none()
    }

    pub fn failed(&self) -> bool {
        matches!(&self.outcome, Some(Err(_)))
            || matches!(&self.outcome, Some(Ok(result)) if result.is_error == Some(true))
    }

    pub fn label(&self) -> &'static str {
        match &self.outcome {
            None => "PENDING",
            Some(Err(_)) => "EXECUTION ERROR",
            Some(Ok(result)) if result.is_error == Some(true) => "TOOL ERROR",
            Some(Ok(_)) => "SUCCESS",
        }
    }

    pub fn finish(&mut self, result: CallOutcome, elapsed: Duration) {
        self.text = match &result {
            Ok(result) => format_result(result),
            Err(error) => format!("Execution failed: {error}"),
        };
        self.outcome = Some(result);
        self.elapsed = Some(elapsed);
        self.scroll = 0;
        self.max_scroll = 0;
    }
}

fn pretty(text: &str) -> String {
    serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .and_then(|value| serde_json::to_string_pretty(&value).ok())
        .unwrap_or_else(|| text.to_owned())
}

pub fn format_result(result: &CallToolResult) -> String {
    let mut blocks = Vec::new();
    for block in &result.content {
        blocks.push(match block {
            ContentBlock::Text(text) => {
                if text.text.is_empty() {
                    "[Empty text block]".into()
                } else {
                    pretty(&text.text)
                }
            }
            ContentBlock::Image(image) => format!("[Image: {}]", image.mime_type),
            ContentBlock::Audio(audio) => format!("[Audio: {}]", audio.mime_type),
            ContentBlock::Resource(_) => "[Embedded resource]".into(),
            ContentBlock::ResourceLink(resource) => {
                format!("[Resource link: {}]", resource.uri)
            }
            _ => "[Unsupported content block]".into(),
        });
    }
    if let Some(value) = &result.structured_content {
        blocks.push(format!(
            "structuredContent:\n{}",
            serde_json::to_string_pretty(value)
                .unwrap_or_else(|error| error.to_string())
        ));
    }
    if blocks.is_empty() {
        "[Empty response]".into()
    } else {
        blocks.join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_explicit_true_is_a_content_failure() {
        use std::sync::Arc;
        for flag in [None, Some(false), Some(true)] {
            let mut display = CallDisplay::new(CallId(1), "ping".into());
            let mut result = CallToolResult::success(vec![]);
            result.is_error = flag;
            display.finish(Ok(Arc::new(result)), Duration::from_millis(1));
            assert_eq!(display.failed(), flag == Some(true));
            assert_eq!(
                display.label(),
                if flag == Some(true) {
                    "TOOL ERROR"
                } else {
                    "SUCCESS"
                }
            );
        }
        let mut display = CallDisplay::new(CallId(1), "ping".into());
        display.finish(
            Err(Arc::new(mcpv::error::AppError::ConnectionClosed)),
            Duration::ZERO,
        );
        assert_eq!(display.label(), "EXECUTION ERROR");
        assert!(display.text.contains("connection closed"));
    }

    #[test]
    fn preserves_order_prettifies_json_and_labels_non_text() {
        let mut result = CallToolResult::success(vec![
            ContentBlock::text("plain"),
            ContentBlock::text("{\"ok\":true}"),
            ContentBlock::image("hidden-base64", "image/png"),
            ContentBlock::text(""),
        ]);
        result.structured_content = Some(serde_json::json!({"count":2}));
        let text = format_result(&result);
        assert!(text.starts_with("plain\n\n{\n  \"ok\": true\n}"));
        assert!(text.contains("[Image: image/png]"));
        assert!(!text.contains("hidden-base64"));
        assert!(text.contains("Empty text block"));
        assert!(text.contains("structuredContent:\n{\n  \"count\": 2\n}"));
        assert_eq!(
            format_result(&CallToolResult::success(vec![])),
            "[Empty response]"
        );
    }
}
