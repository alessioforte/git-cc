use crossterm::event::{self, Event};
use std::time::Duration;

pub fn next_event(timeout: Duration) -> Option<Event> {
    if event::poll(timeout).ok()? {
        event::read().ok()
    } else {
        None
    }
}
