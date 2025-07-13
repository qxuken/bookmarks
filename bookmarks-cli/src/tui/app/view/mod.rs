use ratatui::{prelude::*, widgets::Paragraph};

use crate::tui::{app::state::AppState, event::AppEvent};

pub mod error;
pub mod main;

pub type ViewBoxed = Box<dyn View>;

pub enum EventState {
    Handled,
    PushStack(ViewBoxed),
    NotHandled,
}

pub trait View {
    fn handle_app_event(&mut self, state: &mut AppState, event: &AppEvent) -> EventState;
    fn render_statusline(&mut self, area: Rect, buf: &mut Buffer, state: &mut AppState);
    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &mut AppState);
}

#[inline]
fn statusline_help<'a>(text: impl Into<Text<'a>>, area: Rect, buf: &mut Buffer) {
    Paragraph::new(text).dim().bold().blue().render(area, buf);
}
