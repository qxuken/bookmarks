use crossterm::event::{Event, KeyEvent, KeyEventKind, KeyEventState};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::tui::{
    app::{
        state::AppState,
        view::{EventState, View, statusline_help},
    },
    event::AppEvent,
};

#[derive(Debug, Default)]
pub struct EditView {
    item_index: Option<usize>,
    title: Input,
}

impl EditView {
    pub fn new(item_index: Option<usize>) -> Self {
        Self {
            item_index,
            ..Default::default()
        }
    }
}

impl View for EditView {
    fn handle_app_event(&mut self, _state: &mut AppState, event: &AppEvent) -> EventState {
        match event {
            AppEvent::Key(code, modifier) => {
                self.title.handle_event(&Event::Key(KeyEvent {
                    code: *code,
                    modifiers: *modifier,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }));
                EventState::Handled
            }
            _ => EventState::NotHandled,
        }
    }

    fn render_statusline(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        _state: &mut AppState,
    ) -> Option<Position> {
        statusline_help("Close: q | Quit Application: c-q", area, buf);
        None
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, _state: &mut AppState) -> Option<Position> {
        let popup_area = Rect {
            x: area.width / 8,
            y: area.height / 8,
            width: area.width * 3 / 4,
            height: area.height * 3 / 4,
        };
        Clear.render(popup_area, buf);
        Paragraph::new(format!("{:?}", self.item_index))
            .wrap(Wrap { trim: true })
            .block(
                Block::new()
                    .title("Edit")
                    .title_style(Style::new().blue().bold())
                    .borders(Borders::ALL)
                    .border_style(Style::new().blue()),
            )
            .render(popup_area, buf);

        let [title_area] = Layout::vertical([Constraint::Length(3)])
            .margin(1)
            .areas(popup_area);
        let title_scroll = self.title.visual_scroll(
            (title_area.width as usize)
                .checked_sub(3)
                .unwrap_or_default(),
        );
        Paragraph::new(self.title.value())
            .scroll((0, title_scroll as u16))
            .block(Block::bordered().title("Title"))
            .render(title_area, buf);
        let x =
            self.title.visual_cursor().max(title_scroll) - title_scroll + 1 + title_area.x as usize;

        Some(Position {
            x: x as u16,
            y: title_area.y + 1,
        })
    }
}
