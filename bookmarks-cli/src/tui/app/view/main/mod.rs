use crossterm::event::KeyCode;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, List, ListState, Paragraph},
};

use crate::tui::{
    app::{
        AppState,
        view::{EventState, View, error::ErrorView, statusline_help},
    },
    event::AppEvent,
};
use selected_block::SelectedBlock;

mod loader;
mod selected_block;

#[derive(Default)]
pub struct MainView {
    selected_block: SelectedBlock,
    items_state: ListState,
    content_item: Option<usize>,
    loader: loader::Loader,
}

impl View for MainView {
    fn handle_app_event(&mut self, state: &mut AppState, event: &AppEvent) -> EventState {
        match event {
            AppEvent::Key(KeyCode::Char('k'), _) => {
                self.items_state.select_previous();
                EventState::Handled
            }
            AppEvent::Key(KeyCode::Char('j'), _) => {
                self.items_state.select_next();
                EventState::Handled
            }
            AppEvent::Key(KeyCode::Char('l'), _) => {
                self.selected_block = self.selected_block.next();
                EventState::Handled
            }
            AppEvent::Key(KeyCode::Char('h'), _) => {
                self.selected_block = self.selected_block.prev();
                EventState::Handled
            }
            AppEvent::Key(KeyCode::Char(' ') | KeyCode::Enter, _) => {
                self.content_item = self.items_state.selected();
                EventState::Handled
            }
            AppEvent::Key(KeyCode::Esc, _) => {
                self.content_item = None;
                EventState::Handled
            }
            AppEvent::Key(KeyCode::Tab, _) => {
                EventState::PushStack(Box::new(ErrorView("Test error".to_string())))
            }
            AppEvent::Tick if !state.items_loaded => {
                self.loader.next();
                EventState::Handled
            }
            _ => EventState::NotHandled,
        }
    }

    fn render_statusline(&mut self, area: Rect, buf: &mut Buffer, _state: &mut AppState) {
        statusline_help(
            "Quit: q | Next: j | Prev: k | Select: space | Close: esc",
            area,
            buf,
        )
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
        let screen = if self.content_item.is_some() {
            Layout::vertical([Constraint::Min(5), Constraint::Max(20)]).split(area)
        } else {
            Layout::vertical([Constraint::Fill(1), Constraint::Length(0)]).split(area)
        };
        let selected_block_style = Style::new().fg(Color::Yellow);

        let mut list_title = vec![Span::raw("Bookmarks")];
        if !state.items_loaded {
            list_title.push(Span::styled(
                format!(" {} ", self.loader),
                Style::new().dim(),
            ));
        }

        let mut list_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Line::from(list_title));
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
