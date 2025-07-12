use crossterm::event::KeyCode;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, List, ListState, Paragraph},
};

use crate::tui::{
    app::{
        AppState,
        stack::{AppStackItem, HandleResult},
        view::error::ErrorView,
    },
    terminal_events::TerminalEvent,
};
use selected_block::SelectedBlock;

mod selected_block;

#[derive(Default)]
pub struct MainView {
    selected_block: SelectedBlock,
    items_state: ListState,
    content_item: Option<usize>,
}

impl AppStackItem for MainView {
    fn handle_terminal_event(
        &mut self,
        _state: &mut AppState,
        event: &TerminalEvent,
    ) -> HandleResult {
        match event {
            TerminalEvent::Key(KeyCode::Char('k'), _) => {
                self.items_state.select_previous();
                HandleResult::Handled
            }
            TerminalEvent::Key(KeyCode::Char('j'), _) => {
                self.items_state.select_next();
                HandleResult::Handled
            }
            TerminalEvent::Key(KeyCode::Char('l'), _) => {
                self.selected_block = self.selected_block.next();
                HandleResult::Handled
            }
            TerminalEvent::Key(KeyCode::Char('h'), _) => {
                self.selected_block = self.selected_block.prev();
                HandleResult::Handled
            }
            TerminalEvent::Key(KeyCode::Char(' ') | KeyCode::Enter, _) => {
                self.content_item = self.items_state.selected();
                HandleResult::Handled
            }
            TerminalEvent::Key(KeyCode::Esc, _) => {
                self.content_item = None;
                HandleResult::Handled
            }
            TerminalEvent::Key(KeyCode::Tab, _) => {
                HandleResult::PushStack(Box::new(ErrorView("Test error".to_string())))
            }
            _ => HandleResult::NotHandled,
        }
    }

    fn help(&self, _state: &AppState) -> String {
        "Quit: q | Next: j | Prev: k | Select: space | Close: esc".to_string()
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
        let screen = if self.content_item.is_some() {
            Layout::vertical([Constraint::Min(5), Constraint::Max(20)]).split(area)
        } else {
            Layout::vertical([Constraint::Fill(1), Constraint::Length(0)]).split(area)
        };
        let selected_block_style = Style::new().fg(Color::Yellow);

        let list_title = Line::from(vec![
            Span::raw("Bookmarks"),
            Span::styled(
                if !state.items_loaded {
                    " (Loading...)"
                } else {
                    Default::default()
                },
                Style::new().dim(),
            ),
        ]);
        let mut list_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(list_title);
        if matches!(self.selected_block, SelectedBlock::List) {
            list_block = list_block.border_style(selected_block_style);
        }
        let list = state
            .items
            .iter()
            .map(|it| {
                it.content
                    .title
                    .as_ref()
                    .unwrap_or(&it.content.url)
                    .as_str()
            })
            .collect::<List>()
            .block(list_block)
            .highlight_style(Style::new().reversed());

        let mut content_style = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("Selected");
        if matches!(self.selected_block, SelectedBlock::Content) {
            content_style = content_style.border_style(selected_block_style);
        }
        let content = Paragraph::new(format!(
            "{:#?}",
            self.content_item.and_then(|i| state.items.get(i))
        ))
        .block(content_style);

        StatefulWidget::render(list, screen[0], buf, &mut self.items_state);
        content.render(screen[1], buf);
    }
}
