use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
};

use crate::tui::{
    app::{
        state::AppState,
        view::{EventState, ViewBoxed},
    },
    event::AppEvent,
};

#[derive(Default)]
pub struct AppStack {
    pub should_quit: bool,
    stack: Vec<ViewBoxed>,
}

impl AppStack {
    pub fn handle_app_event(&mut self, state: &mut AppState, event: AppEvent) {
        let Some(cur) = self.stack.last_mut() else {
            return;
        };
        match cur.handle_app_event(state, &event) {
            EventState::Handled => {}
            EventState::PushStack(it) => {
                self.push(it);
            }
            EventState::NotHandled => match event {
                AppEvent::Key(KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                    self.should_quit = true;
                }
                AppEvent::Key(KeyCode::Char('q'), _) => {
                    if self.stack.len() > 1 {
                        self.stack.pop();
                    } else {
                        self.should_quit = true;
                    }
                }
                _ => {}
            },
        }
    }

    pub fn push(&mut self, value: ViewBoxed) {
        self.stack.push(value);
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
        let screen = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

        for it in self.stack.iter_mut() {
            it.render(screen[0], buf, state);
        }
        if let Some(it) = self.stack.last_mut() {
            it.render_statusline(screen[1], buf, state);
        }
    }
}
