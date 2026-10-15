use std::{
    io::{self, IsTerminal},
    sync::{
        Once,
        atomic::{AtomicU8, Ordering},
    },
};

use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use mcpv::error::{AppError, Result};
use ratatui::{DefaultTerminal, Terminal, backend::CrosstermBackend};

const RAW: u8 = 1;
const ALTERNATE: u8 = 2;
const CURSOR: u8 = 4;
static MODES: AtomicU8 = AtomicU8::new(0);
static HOOK: Once = Once::new();

pub struct TerminalGuard {
    terminal: Option<DefaultTerminal>,
}

impl TerminalGuard {
    pub fn init() -> Result<Self> {
        if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
            return Err(AppError::TerminalRequired);
        }
        HOOK.call_once(|| {
            let previous = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                let _ = restore_modes();
                previous(info);
            }));
        });
        // Construct before any side effect so partial failures unwind safely.
        let mut guard = Self { terminal: None };
        MODES.fetch_or(RAW, Ordering::SeqCst);
        enable_raw_mode()?;
        MODES.fetch_or(ALTERNATE, Ordering::SeqCst);
        execute!(io::stdout(), EnterAlternateScreen)?;
        MODES.fetch_or(CURSOR, Ordering::SeqCst);
        execute!(io::stdout(), Hide)?;
        guard.terminal =
            Some(Terminal::new(CrosstermBackend::new(io::stdout()))?);
        guard.get()?.clear()?;
        Ok(guard)
    }

    pub fn get(&mut self) -> Result<&mut DefaultTerminal> {
        // A panic on another runtime thread may already have restored the UI.
        if MODES.load(Ordering::SeqCst) == 0 {
            return Err(AppError::TerminalInactive);
        }
        self.terminal.as_mut().ok_or(AppError::TerminalInactive)
    }

    pub fn restore(&mut self) -> Result<()> {
        self.terminal.take();
        restore_modes().map_err(AppError::Io)
    }
}

fn restore_modes() -> io::Result<()> {
    let modes = MODES.swap(0, Ordering::SeqCst);
    let mut first_error = None;
    for result in [
        if modes & RAW != 0 {
            disable_raw_mode()
        } else {
            Ok(())
        },
        if modes & ALTERNATE != 0 {
            execute!(io::stdout(), LeaveAlternateScreen)
        } else {
            Ok(())
        },
        if modes & CURSOR != 0 {
            execute!(io::stdout(), Show)
        } else {
            Ok(())
        },
    ] {
        if let Err(error) = result {
            first_error.get_or_insert(error);
        }
    }
    match first_error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "intentional panic; run this probe in a PTY to inspect restoration"]
    fn panic_cleanup_probe() -> Result<()> {
        let _guard = TerminalGuard::init()?;
        panic!("intentional terminal restoration probe");
    }
}
