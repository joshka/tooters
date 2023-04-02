mod authentication;
mod home;
mod root;
mod status_bar;
mod title_bar;

pub use authentication::*;
pub use root::*;

use crate::event::Event;
use ratatui::{backend::Backend, layout::Rect, Frame};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventOutcome {
    Consumed,
    NotConsumed,
}

pub trait Component {
    fn draw(&self, f: &mut Frame<impl Backend>, area: Rect);

    /// Handle an event.
    /// Return `EventOutcome::Handled` if the event was handled, `EventOutcome::Unhandled` otherwise.
    /// This allows the caller to decide whether to propagate the event to other components.
    fn handle_event(&mut self, _event: &Event) -> EventOutcome {
        EventOutcome::NotConsumed
    }

    fn start(&mut self);
}
