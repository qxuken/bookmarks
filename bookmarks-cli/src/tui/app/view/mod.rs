use ratatui::prelude::*;

use crate::tui::{app::state::AppState, event::AppEvent};

pub mod edit;
pub mod error;
pub mod loader;
pub mod main;

pub type ViewBoxed = Box<dyn View>;
pub enum EventState {
    Handled,
    PushStack(ViewBoxed),
    PushBlockStack(ViewBoxed),
    NotHandled,
}

pub trait View {
    fn handle_app_event(&mut self, state: &mut AppState, event: &AppEvent) -> EventState;
    fn render_statusline(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut AppState,
    ) -> Option<Position>;
    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &mut AppState) -> Option<Position>;
}

#[inline]
pub fn statusline_help<'a>(text: impl Into<Text<'a>>, area: Rect, buf: &mut Buffer) {
    text.into()
        .style(Style::new().dim().bold().blue())
        .render(area, buf);
}
