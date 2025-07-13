use ratatui::{prelude::*, widgets::Paragraph};

pub mod error;
pub mod main;

#[inline]
fn statusline_help<'a>(text: impl Into<Text<'a>>, area: Rect, buf: &mut Buffer) {
    Paragraph::new(text).dim().bold().blue().render(area, buf);
}
