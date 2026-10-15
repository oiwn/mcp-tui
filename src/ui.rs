use mcpv::client::{CatalogState, ConnectionState};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::{Focus, Level, Screen};

pub fn single_line(text: &str) -> String {
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

pub fn connection(screen: &Screen) -> &'static str {
    match screen.client.connection {
        ConnectionState::Connecting => "CONNECTING",
        ConnectionState::Connected => "CONNECTED",
        ConnectionState::Disconnected => "DISCONNECTED",
    }
}

fn offline(screen: &Screen) -> &'static str {
    if screen.client.connection == ConnectionState::Disconnected
        && screen.client.server.is_some()
    {
        " [cached/offline]"
    } else {
        ""
    }
}

fn catalog(screen: &Screen) -> String {
    match &screen.client.catalog {
        CatalogState::Loading
            if screen.client.connection == ConnectionState::Disconnected =>
        {
            "unavailable".into()
        }
        CatalogState::Loading => "loading".into(),
        CatalogState::Unsupported => "unsupported".into(),
        CatalogState::Ready(tools) => tools.len().to_string(),
        CatalogState::Failed(_) => "failed".into(),
    }
}

fn block(title: String, focused: bool) -> Block<'static> {
    Block::default()
        .title(if focused {
            format!("[focus] {title}")
        } else {
            title
        })
        .borders(Borders::ALL)
        .border_style(if focused {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        })
}

fn identity(screen: &Screen) -> String {
    screen
        .client
        .server
        .as_ref()
        .and_then(|server| server.server_info.as_ref())
        .map(|identity| {
            single_line(&format!("{} {}", identity.name, identity.version))
        })
        .unwrap_or_else(|| "unknown server".into())
}

fn protocol(screen: &Screen) -> String {
    screen
        .client
        .server
        .as_ref()
        .map(|info| info.protocol_version.to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn capabilities(screen: &Screen) -> String {
    let Some(info) = &screen.client.server else {
        return "unknown".into();
    };
    let mut labels = Vec::new();
    if info.capabilities.tools.is_some() {
        labels.push("tools");
    }
    if info.capabilities.prompts.is_some() {
        labels.push("prompts");
    }
    if info.capabilities.resources.is_some() {
        labels.push("resources");
    }
    if info.capabilities.logging.is_some() {
        labels.push("logging");
    }
    if info.capabilities.completions.is_some() {
        labels.push("completions");
    }
    if labels.is_empty() {
        "none".into()
    } else {
        labels.join(", ")
    }
}

pub fn draw(frame: &mut Frame<'_>, screen: &mut Screen) {
    let area = frame.area();
    if area.width < 32 || area.height < 8 {
        frame.render_widget(
            Paragraph::new("Resize terminal\nq quit | r reconnect")
                .wrap(Wrap { trim: false }),
            area,
        );
        return;
    }
    let [header, body, status, hints] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(format!("mcpv — {}{}", identity(screen), offline(screen)))
                .bold(),
            Line::from(format!(
                "Protocol: {} | Capabilities: {}",
                protocol(screen),
                capabilities(screen)
            )),
        ]),
        header,
    );

    if screen.logs_open && area.height >= 16 {
        let drawer_height = (body.height / 3)
            .clamp(4, 10)
            .min(body.height.saturating_sub(3));
        let [main, drawer] = Layout::vertical([
            Constraint::Min(3),
            Constraint::Length(drawer_height),
        ])
        .areas(body);
        body_panes(frame, screen, main, area.width >= 80);
        logs(frame, screen, drawer);
    } else if screen.logs_open && screen.focus == Focus::Logs {
        logs(frame, screen, body);
    } else {
        body_panes(frame, screen, body, area.width >= 80 && area.height >= 16);
    }
    let status_style = if screen.error().is_some() {
        Style::default().fg(Color::Red)
    } else {
        Style::default()
    };
    frame.render_widget(
        Paragraph::new(format!(
            "{} | Last call: — | {}",
            connection(screen),
            single_line(&screen.url)
        ))
        .style(status_style),
        status,
    );
    let movement = match screen.focus {
        Focus::Tools => "select",
        Focus::Summary => "scroll summary",
        Focus::Logs => "scroll logs",
    };
    frame.render_widget(Paragraph::new(format!("q quit  r reconnect  l logs  Tab focus  ↑↓/jk {movement}  g/G ends  Ctrl-C")),hints);
}

fn body_panes(
    frame: &mut Frame<'_>,
    screen: &mut Screen,
    area: Rect,
    wide: bool,
) {
    if wide {
        let [left, right] = Layout::horizontal([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ])
        .areas(area);
        tools(frame, screen, left);
        summary(frame, screen, right);
    } else {
        match screen.focus {
            Focus::Summary => summary(frame, screen, area),
            _ => tools(frame, screen, area),
        }
    }
}

fn tools(frame: &mut Frame<'_>, screen: &mut Screen, area: Rect) {
    let border = block(
        format!("Tools ({}){}", catalog(screen), offline(screen)),
        screen.focus == Focus::Tools,
    );
    match &screen.client.catalog {
        CatalogState::Ready(tools) if !tools.is_empty() => {
            let items: Vec<ListItem<'_>> = tools
                .iter()
                .map(|tool| {
                    let name = single_line(&tool.name);
                    let description = tool
                        .description
                        .as_deref()
                        .map(single_line)
                        .filter(|text| !text.trim().is_empty());
                    ListItem::new(match description {
                        Some(description) => format!("{name} — {description}"),
                        None => name,
                    })
                })
                .collect();
            let list = List::new(items)
                .block(border)
                .highlight_symbol("› ")
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::REVERSED | Modifier::BOLD),
                );
            frame.render_stateful_widget(list, area, &mut screen.tools);
        }
        catalog => {
            let message = match catalog {
                CatalogState::Loading
                    if screen.client.connection
                        == ConnectionState::Disconnected =>
                {
                    "Tools unavailable. Press r to reconnect."
                }
                CatalogState::Loading => "Loading tools…",
                CatalogState::Unsupported => "Tools unsupported by this server.",
                CatalogState::Ready(_) => "0 tools — the catalog is empty.",
                CatalogState::Failed(_) => {
                    "Tool listing failed. See summary or logs."
                }
            };
            frame.render_widget(
                Paragraph::new(message)
                    .block(border)
                    .wrap(Wrap { trim: false }),
                area,
            );
        }
    }
}

fn summary(frame: &mut Frame<'_>, screen: &mut Screen, area: Rect) {
    let border = block(
        format!("Server summary{}", offline(screen)),
        screen.focus == Focus::Summary,
    );
    let inner = border.inner(area);
    let mut lines = vec![
        Line::from(format!("URL: {}", single_line(&screen.url))),
        Line::from(format!("Connection: {}", connection(screen))),
        Line::from(format!("Server: {}", identity(screen))),
        Line::from(format!("Protocol: {}", protocol(screen))),
        Line::from(format!("Tools: {}", catalog(screen))),
    ];
    if let Some(error) = screen.error() {
        lines.push(Line::from(Span::styled(
            format!("ERROR: {}", single_line(&error.to_string())),
            Style::default().fg(Color::Red),
        )));
        lines.push(Line::from("Press r to reconnect; l opens activity logs."));
    }
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    screen.summary_max_scroll = paragraph
        .line_count(inner.width)
        .saturating_sub(usize::from(inner.height))
        .min(usize::from(u16::MAX));
    screen.summary_scroll = screen.summary_scroll.min(screen.summary_max_scroll);
    frame.render_widget(
        paragraph
            .block(border)
            .scroll((screen.summary_scroll as u16, 0)),
        area,
    );
}

fn logs(frame: &mut Frame<'_>, screen: &mut Screen, area: Rect) {
    let mode = if screen.log_follow {
        "following"
    } else {
        "paused; G follows"
    };
    let border = block(format!("Activity ({mode})"), screen.focus == Focus::Logs);
    screen.log_view_height = usize::from(border.inner(area).height).max(1);
    screen.clamp_logs();
    let lines: Vec<Line<'_>> = screen
        .logs
        .iter()
        .skip(screen.log_scroll)
        .take(screen.log_view_height)
        .map(|entry| {
            let seconds = entry.elapsed.as_secs();
            let (label, style) = match entry.level {
                Level::Info => ("INFO", Style::default()),
                Level::Error => ("ERROR", Style::default().fg(Color::Red)),
            };
            Line::styled(
                format!(
                    "+{:02}:{:02}:{:02} {label} {}",
                    seconds / 3600,
                    (seconds / 60) % 60,
                    seconds % 60,
                    entry.message
                ),
                style,
            )
        })
        .collect();
    frame.render_widget(Paragraph::new(lines).block(border), area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use mcpv::{
        client::{ClientEvent, EventKind},
        error::AppError,
    };
    use ratatui::{Terminal, backend::TestBackend};
    use rmcp::model::{
        Implementation, ProtocolVersion, ServerCapabilities, ServerPeerInfo, Tool,
    };
    use std::sync::Arc;

    fn screen() -> Screen {
        let mut screen = Screen::new("http://127.0.0.1:7400/mcp".into());
        let info = ServerPeerInfo::new(
            ProtocolVersion::V_2025_11_25,
            ServerCapabilities::builder().enable_tools().build(),
        )
        .with_server_info(Implementation::new("fixture", "1.0"));
        screen.apply(ClientEvent {
            generation: 1,
            kind: EventKind::Connected(Arc::new(info)),
        });
        screen.apply(ClientEvent {
            generation: 1,
            kind: EventKind::Catalog(CatalogState::Ready(Arc::new(vec![
                Tool::new("ping", "Return JSON", serde_json::Map::new()),
            ]))),
        });
        screen
    }

    fn render(
        screen: &mut Screen,
        width: u16,
        height: u16,
    ) -> Result<(String, bool), std::convert::Infallible> {
        let mut terminal = Terminal::new(TestBackend::new(width, height))?;
        terminal.draw(|frame| draw(frame, screen))?;
        let buffer = terminal.backend().buffer();
        let text = (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        let red = buffer.content.iter().any(|cell| cell.fg == Color::Red);
        Ok((text, red))
    }

    #[test]
    fn connected_layout_has_real_metadata_and_navigation()
    -> Result<(), std::convert::Infallible> {
        let (text, _) = render(&mut screen(), 100, 24)?;
        for expected in [
            "fixture 1.0",
            "2025-11-25",
            "Capabilities: tools",
            "ping",
            "Return JSON",
            "Server summary",
            "Last call: —",
            "Tab focus",
        ] {
            assert!(text.contains(expected), "missing {expected}");
        }
        assert!(!text.contains("F5"));
        Ok(())
    }

    #[test]
    fn empty_unsupported_failed_and_offline_are_distinct()
    -> Result<(), std::convert::Infallible> {
        let mut screen = screen();
        screen.client.catalog = CatalogState::Unsupported;
        assert!(render(&mut screen, 100, 24)?.0.contains("unsupported"));
        screen.client.catalog = CatalogState::Ready(Arc::new(Vec::new()));
        assert!(render(&mut screen, 100, 24)?.0.contains("0 tools"));
        screen.client.catalog =
            CatalogState::Failed(Arc::new(AppError::ConnectionClosed));
        let (text, red) = render(&mut screen, 100, 24)?;
        assert!(text.contains("ERROR:"));
        assert!(red);
        screen.apply(ClientEvent {
            generation: 1,
            kind: EventKind::Disconnected(Some(Arc::new(
                AppError::ConnectionClosed,
            ))),
        });
        assert!(render(&mut screen, 100, 24)?.0.contains("cached/offline"));
        screen.client.catalog = CatalogState::Loading;
        let text = render(&mut screen, 100, 24)?.0;
        assert!(text.contains("Tools (unavailable)"));
        assert!(!text.contains("Tools (loading)"));
        Ok(())
    }

    #[test]
    fn narrow_focus_and_tiny_sizes_are_safe()
    -> Result<(), std::convert::Infallible> {
        let mut screen = screen();
        assert!(render(&mut screen, 50, 16)?.0.contains("ping"));
        screen.focus = Focus::Summary;
        let text = render(&mut screen, 50, 16)?.0;
        assert!(text.contains("Server summary"));
        assert!(!text.contains("Return JSON"));
        screen.toggle_logs();
        assert!(render(&mut screen, 50, 10)?.0.contains("Activity"));
        for (width, height) in [(31, 7), (1, 1), (0, 0), (80, 8), (32, 8)] {
            render(&mut screen, width, height)?;
        }
        Ok(())
    }

    #[test]
    fn terminal_control_characters_do_not_reach_widgets() {
        assert_eq!(single_line("line\nnext\u{1b}[31m"), "line next [31m");
    }
}
