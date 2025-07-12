use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    widgets::{Paragraph, Widget},
};

use crate::tui::{app::state::AppState, terminal_events::TerminalEvent};

type Item = Box<dyn AppStackItem>;

pub enum HandleResult {
    Handled,
    PushStack(Item),
    NotHandled,
}

#[derive(Default)]
pub struct AppStack {
    pub should_quit: bool,
    stack: Vec<Item>,
}

pub trait AppStackItem {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &TerminalEvent,
    ) -> HandleResult;
    fn help(&self, state: &AppState) -> String;
    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &mut AppState);
}

impl AppStack {
    pub fn handle_terminal_event(&mut self, state: &mut AppState, event: TerminalEvent) {
        let Some(cur) = self.stack.last_mut() else {
            return;
        };
        match cur.handle_terminal_event(state, &event) {
            HandleResult::Handled => {}
            HandleResult::PushStack(it) => {
                self.push(it);
            }
            HandleResult::NotHandled => match event {
                TerminalEvent::Key(KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                    self.should_quit = true;
                }
                TerminalEvent::Key(KeyCode::Char('q'), _) => {
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

    pub fn push(&mut self, value: Item) {
        self.stack.push(value);
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
        let screen = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

        for it in self.stack.iter_mut() {
            it.render(screen[0], buf, state);
        }
        if let Some(it) = self.stack.last() {
            Paragraph::new(it.help(state))
                .dim()
                .bold()
                .blue()
                .render(screen[1], buf);
        }
    }
}
