use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, List, ListState, Paragraph},
};

#[cfg(debug_assertions)]
use crate::tui::app::view::error::ErrorView;
use crate::tui::{
    app::{
        AppState,
        view::{EventState, View, statusline_help},
    },
    event::AppEvent,
};

#[derive(Debug, Default)]
enum SelectedBlock {
    #[default]
    List,
    Content,
    Search,
}

#[derive(Debug, Default)]
struct Search {
    value: String,
    cursor_pos: usize,
    items: Vec<(usize, i64)>,
    item_ids: HashMap<usize, usize>,
    latest_focused: usize,
}

#[derive(Debug, Default)]
pub struct MainView {
    selected_block: SelectedBlock,
    items_state: ListState,
    content_item: Option<usize>,
    search: Option<Search>,
}

impl View for MainView {
    fn handle_app_event(&mut self, state: &mut AppState, event: &AppEvent) -> EventState {
        if !state.items.is_empty() && self.items_state.selected().is_none() {
            self.items_state.select(Some(0));
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
                AppEvent::Key(KeyCode::Char('g'), _) => {
                    self.items_state.select_first();
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Char('G'), _) => {
                    self.items_state.select_last();
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Char('l'), _) if self.content_item.is_some() => {
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
                    self.search = Some(Default::default());
                    self.selected_block = SelectedBlock::Search;
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Esc, _) if self.search.is_some() => {
                    self.search = None;
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Char('n'), _) if let Some(search) = self.search.as_mut() => {
                    search.latest_focused = (search.latest_focused + 1) % search.items.len();
                    self.items_state
                        .select(search.items.get(search.latest_focused).map(|it| it.0));
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Char('N'), _) if let Some(search) = self.search.as_mut() => {
                    search.latest_focused = if search.latest_focused == 0 {
                        search.items.len() - 1
                    } else {
                        search.latest_focused - 1
                    };
                    self.items_state
                        .select(search.items.get(search.latest_focused).map(|it| it.0));
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
            SelectedBlock::Search if let Some(search) = self.search.as_mut() => match event {
                AppEvent::Key(KeyCode::Backspace, KeyModifiers::NONE)
                    if search.cursor_pos == 0 && search.value.is_empty() =>
                {
                    self.search = None;
                    self.selected_block = SelectedBlock::List;
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Backspace, KeyModifiers::NONE) if search.cursor_pos > 0 => {
                    search.value.remove(search.cursor_pos - 1);
                    search.cursor_pos -= 1;
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::End, _)
                | AppEvent::Key(KeyCode::Right, KeyModifiers::SUPER) => {
                    search.cursor_pos = search.value.len();
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Right, KeyModifiers::NONE)
                    if search.cursor_pos < search.value.len() =>
                {
                    search.cursor_pos += 1;
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Home, _)
                | AppEvent::Key(KeyCode::Left, KeyModifiers::SUPER) => {
                    search.cursor_pos = 0;
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Left, KeyModifiers::NONE) if search.cursor_pos > 0 => {
                    search.cursor_pos -= 1;
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Char(ch), _) => {
                    search.value.insert(search.cursor_pos, *ch);
                    search.cursor_pos += 1;
                    search.items = bookmarks_data::search(
                        &search.value,
                        state.items.iter().map(|it| &it.content),
                    )
                    .collect();
                    search.item_ids = search
                        .items
                        .iter()
                        .enumerate()
                        .map(|(i_local, (i_global, _score))| (*i_global, i_local))
                        .collect();
                    if let Some(it) = search.items.first() {
                        self.items_state.select(Some(it.0));
                    }
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Esc, _) => {
                    self.search = None;
                    self.selected_block = SelectedBlock::List;
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Enter, _) => {
                    search.latest_focused = 0;
                    if let Some(it) = search.items.first() {
                        self.items_state.select(Some(it.0));
                        self.content_item = Some(it.0);
                    }
                    self.selected_block = SelectedBlock::List;
                    EventState::Handled
                }
                _ => EventState::NotHandled,
            },
            SelectedBlock::Search => {
                self.search = Some(Default::default());
                EventState::NotHandled
            }
        };
        if !matches!(event_state, EventState::NotHandled) {
            return event_state;
        }
        match event {
            #[cfg(debug_assertions)]
            AppEvent::Key(KeyCode::Char('e'), KeyModifiers::ALT) => {
                EventState::PushStack(Box::new(ErrorView("Test error".to_string())))
            }
            #[cfg(debug_assertions)]
            AppEvent::Key(KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                EventState::PushBlockStack(Box::new(ErrorView("Test error".to_string())))
            }
            // AppEvent::Key(KeyCode::Char('o'), _) => {
            //     if let Some(selected_index) = self.items_state.selected()
            //         && let Some(item) = state.items.get(selected_index)
            //     {
            //         let _ = open::that(item.content.url.clone());
            //     }
            //     EventState::Handled
            // }
            _ => EventState::NotHandled,
        }
    }

    fn render_statusline(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        _state: &mut AppState,
    ) -> Option<Position> {
        let content_area = if !matches!(self.selected_block, SelectedBlock::Search)
            && let Some(search) = self.search.as_ref()
        {
            let hint = format!(
                "[{}/{}] {:?} Next: n | Prev: N",
                search.latest_focused + 1,
                search.items.len(),
                search.value
            );
            let layout = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Length(hint.len().try_into().unwrap_or_default()),
            ])
            .split(area);

            Text::styled(hint, Style::new().fg(Color::Blue)).render(layout[2], buf);
            layout[0]
        } else {
            area
        };

        match self.selected_block {
            SelectedBlock::List if self.content_item.is_some() => {
                statusline_help(
                    "Quit: q | Next: j | Prev: k | Select: return | Focus Select: space | Search: / | Focus Content: l",
                    content_area,
                    buf,
                );
            }
            SelectedBlock::List => {
                statusline_help(
                    "Quit: q | Next: j | Prev: k | Select: return | Focus Select: space | Search: /",
                    content_area,
                    buf,
                );
            }
            SelectedBlock::Content => {
                statusline_help("Quit: q | Focus List: h | Close: esc", content_area, buf);
            }
            SelectedBlock::Search if let Some(search) = self.search.as_ref() => {
                Paragraph::new(format!("/{}", search.value)).render(content_area, buf);
                return Some(Position::new(
                    content_area.x + u16::try_from(search.cursor_pos).unwrap_or_default() + 1,
                    content_area.y,
                ));
            }
            SelectedBlock::Search => {}
        };
        None
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &mut AppState) -> Option<Position> {
        let screen = if self.content_item.is_some() {
            Layout::vertical([Constraint::Min(5), Constraint::Max(20)]).split(area)
        } else {
            Layout::vertical([Constraint::Fill(1), Constraint::Length(0)]).split(area)
        };
        let selected_block_style = Style::new().fg(Color::Yellow);

        let mut list_title = vec![Span::raw("Bookmarks")];
        if !state.items_loaded {
            list_title.push(Span::styled(
                format!(" {} ", state.loader),
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
            .enumerate()
            .map(|(i, it)| {
                let mut text = Text::default();
                let mut title_line = Line::default();
                for (i, part) in it.content.path.iter().enumerate() {
                    if i > 0 {
                        title_line.push_span("/".gray().dim());
                    }
                    title_line.push_span(part.clone().gray());
                }
                if !it.content.path.is_empty() {
                    title_line.push_span(" :".gray().dim());
                    title_line.push_span(" ");
                }
                if let Some(title) = it.content.title.as_ref() {
                    title_line.push_span(title);
                }
                if let Some(search) = self.search.as_ref() {
                    if let Some(local_i) = search.item_ids.get(&i)
                        && let Some((_, score)) = search.items.get(*local_i)
                    {
                        title_line.push_span(" ");
                        title_line.push_span(
                            format!("[{}/{}]", local_i + 1, search.item_ids.len()).blue(),
                        );
                        title_line.push_span(" ");
                        title_line.push_span(format!("[score: {score}]").blue());
                    } else {
                        text = text.dim();
                    }
                };
                text.push_line(title_line);
                text.push_line(
                    Span::styled(&it.content.url, Style::new().fg(Color::DarkGray))
                        .into_left_aligned_line(),
                );
                text
            })
            .collect::<List>()
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
        None
    }
}
