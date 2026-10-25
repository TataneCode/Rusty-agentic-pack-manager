use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use std::time::Duration;

/// Terminal events, synchronous version of the rusty-console EventHandler.
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
}

pub struct EventHandler {
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> EventHandler {
        EventHandler { tick_rate }
    }

    pub fn next(&self) -> Result<AppEvent, std::io::Error> {
        if event::poll(self.tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    return Ok(AppEvent::Key(key));
                }
            }
        }
        Ok(AppEvent::Tick)
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        EventHandler::new(Duration::from_millis(250))
    }
}
