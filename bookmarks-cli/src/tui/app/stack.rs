use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
};

use crate::tui::{
    app::{
        state::AppState,
        view::{EventState, ViewBoxed, statusline_help},
    },
    event::AppEvent,
};

#[derive(Default)]
pub struct AppStack {
    pub should_quit: bool,
    is_blocked: bool,
    stack: Vec<ViewBoxed>,
}

impl AppStack {
    pub fn handle_app_event(&mut self, state: &mut AppState, event: AppEvent) {
        if self.is_blocked {
            if let AppEvent::Key(KeyCode::Char('q') | KeyCode::Char('Q'), _) = event {
                self.should_quit = true;
            }
            return;
        };
        let Some(cur) = self.stack.last_mut() else {
            return;
        };
        match cur.handle_app_event(state, &event) {
            EventState::Handled => {}
            EventState::PushStack(it) => {
                self.push(it);
            }
            EventState::PushBlockStack(it) => {
                self.push_block(it);
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

    pub fn push_block(&mut self, value: ViewBoxed) {
        self.stack.push(value);
        self.is_blocked = true;
    }

    pub fn render(&mut self, state: &mut AppState, frame: &mut Frame) {
        let screen =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(frame.area());

        for it in self.stack.iter_mut() {
            if let Some(new_cursor_pos) = it.render(screen[0], frame.buffer_mut(), state) {
                frame.set_cursor_position(new_cursor_pos);
            }
        }
        if self.is_blocked {
            statusline_help("Quit: q", screen[1], frame.buffer_mut());
        } else if let Some(it) = self.stack.last_mut()
            && let Some(new_cursor_pos) = it.render_statusline(screen[1], frame.buffer_mut(), state)
        {
            frame.set_cursor_position(new_cursor_pos);
        }
    }
}
