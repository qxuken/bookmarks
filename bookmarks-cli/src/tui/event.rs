use crossterm::event::{KeyCode, KeyModifiers};

#[derive(Debug)]
pub(super) enum AppEvent {
    Key(KeyCode, KeyModifiers),
    Render,
    Tick,
}
