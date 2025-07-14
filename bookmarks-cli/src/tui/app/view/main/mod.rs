use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, List, ListState, Paragraph},
};

use crate::tui::{
    app::{
        AppState,
        view::{EventState, View, error::ErrorView, main::loader::Loader, statusline_help},
    },
    event::AppEvent,
};

mod loader;

#[derive(Default)]
enum SelectedBlock {
    #[default]
    List,
    Content,
}

#[derive(Default)]
struct Search {
    value: String,
    cursor_pos: usize,
    focused: bool,
    items: Vec<(usize, i64)>,
    items_state: ListState,
    selected_before: Option<usize>,
}

#[derive(Default)]
pub struct MainView {
    selected_block: SelectedBlock,
    items_state: ListState,
    content_item: Option<usize>,
    loader: Loader,
    search: Option<Search>,
}

impl View for MainView {
    fn handle_app_event(&mut self, state: &mut AppState, event: &AppEvent) -> EventState {
        if state.items_loaded && self.items_state.selected().is_none() {
            self.items_state.select(Some(0));
        }
        if let Some(search) = self.search.as_mut()
            && search.focused
        {
            let event_state = match event {
                AppEvent::Key(KeyCode::Backspace, _)
                    if search.cursor_pos == 0 && search.value.is_empty() =>
                {
                    self.search = None;
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Backspace, _) if search.cursor_pos > 0 => {
                    search.value.remove(search.cursor_pos - 1);
                    search.cursor_pos -= 1;
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Right, KeyModifiers::SUPER) => {
                    search.cursor_pos = search.value.len();
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Right, _) => {
                    search.cursor_pos = search.value.len().min(search.cursor_pos + 1);
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Left, KeyModifiers::SUPER) => {
                    search.cursor_pos = 0;
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Left, _) => {
                    search.cursor_pos = search.cursor_pos.checked_sub(1).unwrap_or_default();
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Char(ch), _) => {
                    search.items_state.select(None);
                    search.value.insert(search.cursor_pos, *ch);
                    search.cursor_pos += 1;
                    search.items = bookmarks_data::search(
                        &search.value,
                        state.items.iter().map(|it| &it.content),
                    )
                    .collect();
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Esc, _) => {
                    self.search = None;
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Enter, _) => {
                    self.selected_block = SelectedBlock::List;
                    self.items_state.select_first();
                    self.content_item = Some(0);
                    EventState::Handled
                }
                _ => EventState::NotHandled,
            };

            if !matches!(event_state, EventState::NotHandled) {
                return event_state;
            }
        }
        let event_state = match self.selected_block {
            SelectedBlock::List => match event {
                AppEvent::Key(KeyCode::Char('k'), _) => {
                    self.items_state.select_previous();
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Char('j'), _) => {
                    self.items_state.select_next();
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Char('l'), _) => {
                    self.selected_block = SelectedBlock::Content;
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Enter, _) => {
                    self.content_item = self.items_state.selected();
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Char(' '), _) => {
                    self.selected_block = SelectedBlock::Content;
                    self.content_item = self.items_state.selected();
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Char('/'), _) => {
                    if let Some(search) = self.search.as_mut() {
                        search.focused = true;
                    } else {
                        let search = Search {
                            focused: true,
                            ..Default::default()
                        };
                        self.search = Some(search);
                    }
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Esc, _) if self.search.is_some() => {
                    self.search = None;
                    EventState::Handled
                }
                _ => EventState::NotHandled,
            },
            SelectedBlock::Content => match event {
                AppEvent::Key(KeyCode::Char('h'), _) => {
                    self.selected_block = SelectedBlock::List;
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Esc, _) => {
                    self.selected_block = SelectedBlock::List;
                    self.content_item = None;
                    EventState::Handled
                }
                _ => EventState::NotHandled,
            },
        };
        if !matches!(event_state, EventState::NotHandled) {
            return event_state;
        }
        match event {
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
        if let Some(search) = self.search.as_ref() {
            Paragraph::new(format!("/{}", search.value)).render(area, buf);
            return;
        }
        let help_str = match self.selected_block {
            SelectedBlock::List => {
                if self.content_item.is_some() {
                    "Quit: q | Next: j | Prev: k | Select: return | Focus Select: space | Search: / | Close: esc | Focus Content: l"
                } else {
                    "Quit: q | Next: j | Prev: k | Select: return | Focus Select: space | Search: /"
                }
            }
            SelectedBlock::Content => "Quit: q | Focus List: h",
        };
        statusline_help(help_str, area, buf)
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
        let list = if let Some(search) = self.search.as_ref()
            && !search.value.is_empty()
        {
            search
                .items
                .iter()
                .map(|it| &state.items[it.0])
                .map(|it| {
                    let mut text = Text::default();
                    if let Some(title) = it.content.title.as_ref() {
                        text.push_span(title);
                    }
                    text.push_line(
                        Span::styled(&it.content.url, Style::new().dim()).into_left_aligned_line(),
                    );
                    text
                })
                .collect::<List>()
        } else {
            state
                .items
                .iter()
                .map(|it| {
                    let mut text = Text::default();
                    if let Some(title) = it.content.title.as_ref() {
                        text.push_span(title);
                    }
                    text.push_line(
                        Span::styled(&it.content.url, Style::new().dim()).into_left_aligned_line(),
                    );
                    text
                })
                .collect::<List>()
        }
        .block(list_block)
        .highlight_style(Style::new().reversed());
        StatefulWidget::render(list, screen[0], buf, &mut self.items_state);

        let mut content_style = Block::bordered().border_type(BorderType::Rounded);
        if matches!(self.selected_block, SelectedBlock::Content) {
            content_style = content_style.border_style(selected_block_style);
        }
        let content = Paragraph::new(format!(
            "{:#?}",
            self.content_item.and_then(|i| state.items.get(i))
        ))
        .block(content_style);
        content.render(screen[1], buf);
    }
}
