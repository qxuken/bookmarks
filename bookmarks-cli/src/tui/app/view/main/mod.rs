use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    prelude::*,
    symbols::scrollbar,
    widgets::{
        Block, BorderType, List, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Wrap,
    },
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
struct SelectedContent {
    item_index: usize,
    content_height: usize,
    viewport_height: usize,
    scroll_state: ScrollbarState,
    scroll_value: usize,
}

#[derive(Debug, Default)]
pub struct MainView {
    selected_block: SelectedBlock,
    items_state: ListState,
    selected_content: Option<SelectedContent>,
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
                AppEvent::Key(KeyCode::Char('l'), _) if self.selected_content.is_some() => {
                    self.selected_block = SelectedBlock::Content;
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Enter, _)
                    if let Some(item_index) = self.items_state.selected() =>
                {
                    self.selected_content = Some(SelectedContent {
                        item_index,
                        ..Default::default()
                    });
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Char(' '), _)
                    if let Some(item_index) = self.items_state.selected() =>
                {
                    self.selected_block = SelectedBlock::Content;
                    self.selected_content = Some(SelectedContent {
                        item_index,
                        ..Default::default()
                    });
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
                AppEvent::Key(KeyCode::Char('j'), _)
                    if let Some(selected_content) = self.selected_content.as_mut() =>
                {
                    selected_content.scroll_value =
                        selected_content.scroll_value.saturating_add(1).min(
                            selected_content
                                .content_height
                                .saturating_sub(selected_content.viewport_height),
                        );
                    selected_content.scroll_state = selected_content
                        .scroll_state
                        .position(selected_content.scroll_value);
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Char('k'), _)
                    if let Some(selected_content) = self.selected_content.as_mut() =>
                {
                    selected_content.scroll_value = selected_content.scroll_value.saturating_sub(1);
                    selected_content.scroll_state = selected_content
                        .scroll_state
                        .position(selected_content.scroll_value);
                    EventState::Handled
                }
                AppEvent::Key(KeyCode::Esc, _) => {
                    self.selected_block = SelectedBlock::List;
                    self.selected_content = None;
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
                        self.selected_content = Some(SelectedContent {
                            item_index: it.0,
                            ..Default::default()
                        });
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
            AppEvent::Key(KeyCode::Char('o'), _) => {
                if let Some(selected_index) = self.items_state.selected()
                    && let Some(item) = state.items.get(selected_index)
                {
                    let _ = open::that(item.content.url.clone());
                }
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
            SelectedBlock::List if self.selected_content.is_some() => {
                statusline_help(
                    "Quit: q | Next: j | Prev: k | Open: o | Select: return | Focus Select: space | Search: / | Focus Content: l",
                    content_area,
                    buf,
                );
            }
            SelectedBlock::List => {
                statusline_help(
                    "Quit: q | Next: j | Prev: k | Open: o | Select: return | Focus Select: space | Search: /",
                    content_area,
                    buf,
                );
            }
            SelectedBlock::Content => {
                statusline_help(
                    "Quit: q | Open: o | Focus List: h | Up: k | Down: j | Close: esc",
                    content_area,
                    buf,
                );
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
        let selected_block_style = Style::new().fg(Color::Yellow);

        let list_area = if let Some(selected_content) = self.selected_content.as_mut()
            && let Some(record_file) = state.items.get(selected_content.item_index)
        {
            let screen = Layout::vertical([Constraint::Min(6), Constraint::Max(15)]).split(area);

            let mut content_style = Block::bordered().border_type(BorderType::Rounded);
            if matches!(self.selected_block, SelectedBlock::Content) {
                content_style = content_style.border_style(selected_block_style);
            }

            if let Some(title) = record_file.content.title.as_ref() {
                content_style =
                    content_style.title(Line::from(title.clone().bold()).left_aligned());
            }
            if let Some(relative_path) = record_file.relative_path.to_str() {
                content_style =
                    content_style.title(Line::from(relative_path.dim().gray()).right_aligned());
            }

            let mut text = Text::default();
            text.push_line(Line::from(
                record_file.content.url.clone().blue().underlined(),
            ));

            if let Some(tags) = &record_file.content.tags {
                let mut line = Line::from("Tags:");
                for tag in tags {
                    line.push_span(" ");
                    line.push_span(" ".on_dark_gray());
                    line.push_span(tag.clone().dark_gray().reversed());
                    line.push_span(" ".on_dark_gray());
                }
                text.push_line(line);
            }

            if let Some(description) = &record_file.content.description {
                for line in description.lines() {
                    text.push_line(line);
                }
            }

            let content = Paragraph::new(text)
                .block(content_style)
                .wrap(Wrap { trim: true });

            selected_content.viewport_height = screen[1].height as usize;
            selected_content.content_height = content.line_count(screen[1].width);
            selected_content.scroll_state = selected_content.scroll_state.content_length(
                selected_content
                    .content_height
                    .saturating_sub(selected_content.viewport_height),
            );

            content
                .scroll((selected_content.scroll_value as u16, 0))
                .render(screen[1], buf);

            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .symbols(scrollbar::VERTICAL)
                .begin_symbol(None)
                .end_symbol(None)
                .render(
                    screen[1].inner(Margin {
                        horizontal: 0,
                        vertical: 1,
                    }),
                    buf,
                    &mut selected_content.scroll_state,
                );

            screen[0]
        } else {
            area
        };

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
        StatefulWidget::render(list, list_area, buf, &mut self.items_state);

        None
    }
}
