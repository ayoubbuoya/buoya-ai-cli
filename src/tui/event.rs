use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent};
use std::time::Duration;

pub type Event = CrosstermEvent;

pub fn poll_event(timeout: Duration) -> Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}
