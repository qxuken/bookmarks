use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::tui::{
    app::{
        state::AppState,
        view::{EventState, View, statusline_help},
    },
    event::AppEvent,
};

#[derive(Debug, Clone)]
pub struct EditView(pub Option<usize>);

impl View for EditView {
    fn handle_app_event(&mut self, _state: &mut AppState, _event: &AppEvent) -> EventState {
        EventState::NotHandled
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
            x: area.width / 3,
            y: area.height / 4,
            width: area.width / 3,
            height: area.height / 4,
        };
        Clear.render(popup_area, buf);
        Paragraph::new(format!("{:?}", self.0))
            .wrap(Wrap { trim: true })
            .style(Style::new().red())
            .block(
                Block::new()
                    .title("Error")
                    .title_style(Style::new().red().bold())
                    .borders(Borders::ALL)
                    .border_style(Style::new().red()),
            )
            .render(popup_area, buf);
        None
    }
}
