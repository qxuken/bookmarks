use crossterm::event::{KeyCode, KeyModifiers};

#[derive(Debug)]
pub enum AppEvent {
    Key(KeyCode, KeyModifiers),
    Render,
    Tick,
}
