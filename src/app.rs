use std::{
    collections::VecDeque,
    process::ExitCode,
    sync::Arc,
    time::{Duration, Instant},
};

use crossterm::event::{
    Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
};
use futures_util::StreamExt;
use mcpv::{
    client::{
        CatalogState, ClientEvent, ClientHandle, ClientOptions, ClientState,
        ConnectionState, EventKind,
    },
    error::{AppError, Result},
};
use ratatui::widgets::ListState;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{cli::Args, terminal::TerminalGuard, ui};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Tools,
    Details,
    Summary,
    Logs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Tools,
    Server,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Info,
    Error,
}

pub struct LogEntry {
    pub elapsed: Duration,
    pub level: Level,
    pub message: String,
}

pub struct Screen {
    pub url: String,
    pub client: ClientState,
    pub focus: Focus,
    pub view: View,
    previous_focus: Focus,
    pub tools: ListState,
    pub summary_scroll: usize,
    pub summary_max_scroll: usize,
    pub details_scroll: usize,
    pub details_max_scroll: usize,
    pub logs_open: bool,
    pub logs: VecDeque<LogEntry>,
    pub log_scroll: usize,
    pub log_view_height: usize,
    pub log_follow: bool,
    pub local_error: Option<Arc<AppError>>,
    started: Instant,
}

impl Screen {
    pub fn new(url: String) -> Self {
        Self {
            url,
            client: ClientState {
                generation: 1,
                connection: ConnectionState::Connecting,
                ..Default::default()
            },
            focus: Focus::Tools,
            view: View::Tools,
            previous_focus: Focus::Tools,
            tools: ListState::default(),
            summary_scroll: 0,
            summary_max_scroll: 0,
            details_scroll: 0,
            details_max_scroll: 0,
            logs_open: false,
            logs: VecDeque::new(),
            log_scroll: 0,
            log_view_height: 1,
            log_follow: true,
            local_error: None,
            started: Instant::now(),
        }
    }

    pub fn log(&mut self, level: Level, message: impl Into<String>) {
        if self.logs.len() == 1000 {
            self.logs.pop_front();
            self.log_scroll = self.log_scroll.saturating_sub(1);
        }
        self.logs.push_back(LogEntry {
            elapsed: self.started.elapsed(),
            level,
            message: ui::single_line(&message.into()),
        });
        self.clamp_logs();
    }

    pub fn clamp_logs(&mut self) {
        let last = self.logs.len().saturating_sub(self.log_view_height);
        self.log_scroll = if self.log_follow {
            last
        } else {
            self.log_scroll.min(last)
        };
    }

    pub fn error(&self) -> Option<&AppError> {
        self.local_error
            .as_deref()
            .or(self.client.error.as_deref())
            .or_else(|| match &self.client.catalog {
                CatalogState::Failed(error) => Some(error.as_ref()),
                _ => None,
            })
    }

    pub fn fail(&mut self, error: AppError) {
        self.log(Level::Error, error.to_string());
        self.local_error = Some(Arc::new(error));
    }

    pub fn reconnect(&mut self, generation: u64) {
        self.client.begin_generation(generation);
        self.tools = ListState::default();
        self.summary_scroll = 0;
        self.summary_max_scroll = 0;
        self.reset_details();
        self.local_error = None;
        self.log(
            Level::Info,
            format!("Reconnect requested (generation {generation})"),
        );
    }

    pub fn apply(&mut self, event: ClientEvent) {
        if event.generation < self.client.generation {
            return;
        }
        let new_generation = event.generation > self.client.generation;
        let selected_name = if new_generation {
            None
        } else {
            self.selected_tool().map(|tool| tool.name.to_string())
        };
        let catalog_changed = matches!(event.kind, EventKind::Catalog(_));
        let (level, message) = match &event.kind {
            EventKind::Connecting => (
                Level::Info,
                format!("Connecting (generation {})", event.generation),
            ),
            EventKind::Connected(info) => (
                Level::Info,
                format!("Handshake complete; protocol {}", info.protocol_version),
            ),
            EventKind::Catalog(CatalogState::Ready(tools)) => {
                (Level::Info, format!("Listed {} tools", tools.len()))
            }
            EventKind::Catalog(CatalogState::Unsupported) => {
                (Level::Info, "Tools unsupported".into())
            }
            EventKind::Catalog(CatalogState::Failed(error))
            | EventKind::CleanupFailed(error) => {
                (Level::Error, error.to_string())
            }
            EventKind::Catalog(CatalogState::Loading) => {
                (Level::Info, "Loading tools".into())
            }
            EventKind::Disconnected(Some(error)) => {
                (Level::Error, error.to_string())
            }
            EventKind::Disconnected(None) => (Level::Info, "Disconnected".into()),
        };
        self.client.apply(event);
        if new_generation || catalog_changed {
            let selected = match &self.client.catalog {
                CatalogState::Ready(tools) if !tools.is_empty() => Some(
                    selected_name
                        .as_deref()
                        .and_then(|name| {
                            tools.iter().position(|tool| tool.name == name)
                        })
                        .unwrap_or(0),
                ),
                _ => None,
            };
            self.tools = ListState::default().with_selected(selected);
            // A replacement definition may change even when the name is retained.
            self.reset_details();
            if new_generation {
                self.summary_scroll = 0;
                self.summary_max_scroll = 0;
            }
        }
        self.log(level, message);
    }

    pub fn cycle_focus(&mut self, reverse: bool) {
        let mut panes = match self.view {
            View::Tools => vec![Focus::Tools, Focus::Details],
            View::Server => vec![Focus::Summary],
        };
        if self.logs_open {
            panes.push(Focus::Logs);
        }
        let current = panes
            .iter()
            .position(|focus| *focus == self.focus)
            .unwrap_or(0);
        let step = if reverse { panes.len() - 1 } else { 1 };
        self.focus = panes[(current + step) % panes.len()];
    }

    pub fn switch_view(&mut self, view: View) {
        self.view = view;
        self.focus = match view {
            View::Tools => Focus::Tools,
            View::Server => Focus::Summary,
        };
        self.previous_focus = self.focus;
    }

    pub fn selected_tool(&self) -> Option<&rmcp::model::Tool> {
        match &self.client.catalog {
            CatalogState::Ready(tools) => {
                self.tools.selected().and_then(|index| tools.get(index))
            }
            _ => None,
        }
    }

    fn reset_details(&mut self) {
        self.details_scroll = 0;
        self.details_max_scroll = 0;
    }

    pub fn toggle_logs(&mut self) {
        self.logs_open = !self.logs_open;
        if self.logs_open {
            self.previous_focus = self.focus;
            self.focus = Focus::Logs;
        } else {
            self.focus = self.previous_focus;
        }
    }

    pub fn navigate(&mut self, key: KeyCode) {
        let (position, last) = match self.focus {
            Focus::Tools => {
                let CatalogState::Ready(tools) = &self.client.catalog else {
                    return;
                };
                if tools.is_empty() {
                    self.tools.select(None);
                    return;
                }
                (self.tools.selected().unwrap_or(0), tools.len() - 1)
            }
            Focus::Summary => (self.summary_scroll, self.summary_max_scroll),
            Focus::Details => (self.details_scroll, self.details_max_scroll),
            Focus::Logs => (
                self.log_scroll,
                self.logs.len().saturating_sub(self.log_view_height),
            ),
        };
        let next = match key {
            KeyCode::Up | KeyCode::Char('k') => position.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => {
                position.saturating_add(1).min(last)
            }
            KeyCode::Char('g') => 0,
            KeyCode::Char('G') => last,
            _ => return,
        };
        match self.focus {
            Focus::Tools => {
                if self.tools.selected() != Some(next) {
                    self.reset_details();
                }
                self.tools.select(Some(next));
            }
            Focus::Summary => self.summary_scroll = next,
            Focus::Details => self.details_scroll = next,
            Focus::Logs => {
                self.log_scroll = next;
                self.log_follow = next == last;
            }
        }
    }
}

enum Action {
    None,
    Quit(ExitCode),
    Reconnect,
}

fn key_action(screen: &mut Screen, key: KeyEvent) -> Action {
    if key.kind == KeyEventKind::Release {
        return Action::None;
    }
    let navigation = matches!(
        key.code,
        KeyCode::Up | KeyCode::Down | KeyCode::Char('j' | 'k' | 'g' | 'G')
    );
    if key.kind == KeyEventKind::Repeat && !navigation {
        return Action::None;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL)
        && key.code == KeyCode::Char('c')
    {
        return Action::Quit(ExitCode::from(130));
    }
    if key
        .modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
    {
        return Action::None;
    }
    match key.code {
        KeyCode::Char('1') => {
            screen.switch_view(View::Tools);
            Action::None
        }
        KeyCode::Char('2') => {
            screen.switch_view(View::Server);
            Action::None
        }
        KeyCode::Char('q') => Action::Quit(ExitCode::SUCCESS),
        KeyCode::Char('r') => Action::Reconnect,
        KeyCode::Char('l') => {
            screen.toggle_logs();
            Action::None
        }
        KeyCode::Tab => {
            screen.cycle_focus(false);
            Action::None
        }
        KeyCode::BackTab => {
            screen.cycle_focus(true);
            Action::None
        }
        code => {
            screen.navigate(code);
            Action::None
        }
    }
}

pub async fn run(args: Args) -> Result<ExitCode> {
    // Validate through the existing API before taking over the terminal.
    let (handle, events) =
        ClientHandle::spawn(&args.server_url, ClientOptions::default())?;
    let mut terminal = match TerminalGuard::init() {
        Ok(terminal) => terminal,
        Err(error) => {
            let _ = handle.shutdown().await;
            return Err(error);
        }
    };
    let mut handle = Some(handle);
    let outcome = event_loop(
        &mut terminal,
        &mut handle,
        events,
        Screen::new(args.server_url),
    )
    .await;
    // Restore immediately; diagnostics and bounded network cleanup use normal terminal mode.
    let restored = terminal.restore();
    let cleanup = match handle.take() {
        Some(handle) => handle.shutdown().await,
        None => Ok(()),
    };
    match outcome {
        Ok(code) if code == ExitCode::from(130) => {
            if let Err(error) = restored.and(cleanup) {
                eprintln!("mcpv: {error}");
            }
            Ok(code)
        }
        Ok(code) => {
            restored?;
            cleanup?;
            Ok(code)
        }
        Err(error) => Err(error),
    }
}

async fn event_loop(
    terminal: &mut TerminalGuard,
    handle: &mut Option<ClientHandle>,
    mut events: UnboundedReceiver<ClientEvent>,
    mut screen: Screen,
) -> Result<ExitCode> {
    let mut input = EventStream::new();
    let interrupt = tokio::signal::ctrl_c();
    tokio::pin!(interrupt);
    let mut tick = tokio::time::interval(Duration::from_millis(100));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut dirty = true;
    let mut worker_closed = false;
    loop {
        tokio::select! {
            signal = &mut interrupt => {signal.map_err(AppError::Signal)?; return Ok(ExitCode::from(130));},
            event = input.next() => {
                match event.ok_or(AppError::InputClosed)?? {
                    Event::Key(key) => match key_action(&mut screen, key) {
                        Action::Quit(code) => return Ok(code),
                        Action::Reconnect => {
                            if worker_closed {
                                match ClientHandle::spawn(&screen.url, ClientOptions::default()) {
                                    Ok((fresh, receiver)) => {
                                        // Replacement discards every retired worker event.
                                        *handle = Some(fresh); events = receiver; worker_closed = false;
                                        screen.client = ClientState::default();screen.reconnect(1);
                                    }
                                    Err(error) => screen.fail(error),
                                }
                            } else if let Some(handle) = handle.as_mut() {
                                match handle.reconnect() {Ok(generation) => screen.reconnect(generation), Err(error) => screen.fail(error)}
                            }
                        }
                        Action::None => {},
                    },
                    Event::Resize(_,_) => {terminal.get()?.autoresize()?;},
                    _ => {},
                }
                dirty = true;
            },
            event = events.recv(), if !worker_closed => {
                match event {
                    Some(event) => screen.apply(event),
                    None => {worker_closed = true;screen.client.connection = ConnectionState::Disconnected;screen.fail(AppError::ChannelClosed);},
                }
                dirty = true;
            },
            _ = tick.tick(), if dirty => {
                terminal.get()?.draw(|frame| ui::draw(frame, &mut screen))?;
                dirty = false;
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::Tool;

    fn ready() -> Screen {
        let mut screen = Screen::new("http://localhost/mcp".into());
        screen.apply(ClientEvent {
            generation: 1,
            kind: EventKind::Catalog(CatalogState::Ready(Arc::new(
                ["first", "second", "third"]
                    .into_iter()
                    .map(|name| Tool::new(name, "", serde_json::Map::new()))
                    .collect(),
            ))),
        });
        screen
    }

    #[test]
    fn tool_navigation_clamps_and_empty_selection_is_safe() {
        let mut screen = ready();
        screen.navigate(KeyCode::Up);
        assert_eq!(screen.tools.selected(), Some(0));
        screen.navigate(KeyCode::Char('j'));
        assert_eq!(screen.tools.selected(), Some(1));
        screen.navigate(KeyCode::Char('G'));
        screen.navigate(KeyCode::Down);
        assert_eq!(screen.tools.selected(), Some(2));
        screen.navigate(KeyCode::Char('g'));
        assert_eq!(screen.tools.selected(), Some(0));
        screen.apply(ClientEvent {
            generation: 1,
            kind: EventKind::Catalog(CatalogState::Ready(Arc::new(Vec::new()))),
        });
        screen.navigate(KeyCode::Down);
        assert_eq!(screen.tools.selected(), None);
    }

    #[test]
    fn focus_cycles_and_drawer_restores_previous_focus() {
        let mut screen = ready();
        screen.cycle_focus(true);
        assert_eq!(screen.focus, Focus::Details);
        screen.toggle_logs();
        assert_eq!(screen.focus, Focus::Logs);
        screen.cycle_focus(true);
        assert_eq!(screen.focus, Focus::Details);
        screen.cycle_focus(true);
        assert_eq!(screen.focus, Focus::Tools);
        screen.cycle_focus(true);
        assert_eq!(screen.focus, Focus::Logs);
        screen.toggle_logs();
        assert_eq!(screen.focus, Focus::Details);
        screen.cycle_focus(false);
        assert_eq!(screen.focus, Focus::Tools);
    }

    #[test]
    fn tabs_have_valid_focus_with_logs_and_independent_scroll() {
        let mut screen = ready();
        screen.cycle_focus(false);
        screen.details_max_scroll = 20;
        screen.navigate(KeyCode::Char('G'));
        screen.toggle_logs();
        key_action(
            &mut screen,
            KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE),
        );
        assert_eq!(screen.view, View::Server);
        assert_eq!(screen.focus, Focus::Summary);
        screen.summary_max_scroll = 30;
        screen.navigate(KeyCode::Down);
        assert_eq!(screen.summary_scroll, 1);
        screen.cycle_focus(true);
        assert_eq!(screen.focus, Focus::Logs);
        screen.toggle_logs();
        assert_eq!(screen.focus, Focus::Summary);
        screen.cycle_focus(false);
        assert_eq!(screen.focus, Focus::Summary);
        key_action(
            &mut screen,
            KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE),
        );
        assert_eq!(screen.focus, Focus::Tools);
        assert_eq!(screen.details_scroll, 20);
        screen.navigate(KeyCode::Down);
        assert_eq!(screen.details_scroll, 0);
        assert_eq!(screen.summary_scroll, 1);
    }

    #[test]
    fn replacement_catalog_retains_name_and_resets_changed_details() {
        let mut screen = ready();
        screen.navigate(KeyCode::Down);
        screen.details_scroll = 12;
        screen.details_max_scroll = 20;
        let catalog = |names: &[&str]| ClientEvent {
            generation: 1,
            kind: EventKind::Catalog(CatalogState::Ready(Arc::new(
                names
                    .iter()
                    .map(|name| {
                        Tool::new((*name).to_owned(), "", serde_json::Map::new())
                    })
                    .collect(),
            ))),
        };
        screen.apply(catalog(&["third", "first", "second"]));
        assert_eq!(screen.tools.selected(), Some(2));
        assert_eq!(
            screen.selected_tool().map(|tool| tool.name.as_ref()),
            Some("second")
        );
        assert_eq!(screen.details_scroll, 0);
        screen.apply(catalog(&["third", "first"]));
        assert_eq!(screen.tools.selected(), Some(0));
        screen.details_scroll = 10;
        screen.apply(ClientEvent {
            generation: 2,
            kind: EventKind::Connecting,
        });
        assert!(screen.selected_tool().is_none());
        assert_eq!(screen.details_scroll, 0);
        screen.apply(catalog(&["second"]));
        assert!(screen.selected_tool().is_none());
    }

    #[test]
    fn reconnect_discards_old_events_and_resets_selection_but_keeps_logs() {
        let mut screen = ready();
        screen.navigate(KeyCode::Char('G'));
        screen.reconnect(2);
        let count = screen.logs.len();
        screen.apply(ClientEvent {
            generation: 1,
            kind: EventKind::Disconnected(Some(Arc::new(
                AppError::ConnectionClosed,
            ))),
        });
        assert_eq!(screen.logs.len(), count);
        assert!(screen.error().is_none());
        assert_eq!(screen.tools.selected(), None);
        assert_eq!(screen.client.connection, ConnectionState::Connecting);
    }

    #[test]
    fn logs_follow_pause_and_preserve_anchor_through_eviction() {
        let mut screen = ready();
        screen.log_view_height = 3;
        for index in 0..1005 {
            screen.log(Level::Info, index.to_string());
        }
        assert_eq!(screen.logs.len(), 1000);
        assert_eq!(screen.log_scroll, 997);
        screen.focus = Focus::Logs;
        screen.navigate(KeyCode::Up);
        assert!(!screen.log_follow);
        let anchor = screen.logs[screen.log_scroll].message.clone();
        screen.log(Level::Info, "new");
        assert_eq!(screen.logs[screen.log_scroll].message, anchor);
        screen.navigate(KeyCode::Char('G'));
        assert!(screen.log_follow);
        screen.log(Level::Info, "tail");
        assert_eq!(screen.log_scroll, 997);
    }

    #[test]
    fn key_repeats_only_move_and_ctrl_c_quits() {
        let mut screen = ready();
        let mut key = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE);
        key.kind = KeyEventKind::Repeat;
        assert!(matches!(key_action(&mut screen, key), Action::None));
        assert!(!screen.logs_open);
        key.code = KeyCode::Down;
        key_action(&mut screen, key);
        assert_eq!(screen.tools.selected(), Some(1));
        key.kind = KeyEventKind::Release;
        key_action(&mut screen, key);
        assert_eq!(screen.tools.selected(), Some(1));
        assert!(
            matches!(key_action(&mut screen,KeyEvent::new(KeyCode::Char('c'),KeyModifiers::CONTROL)),Action::Quit(code) if code==ExitCode::from(130))
        );
    }
}
