use crossterm::event::KeyEventKind;
use crossterm::event::{Event, EventStream};
use futures::{FutureExt, StreamExt};
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};

use crate::tui::event::AppEvent;

#[derive(Debug)]
pub(super) struct TerminalPoller {
    tx: UnboundedSender<AppEvent>,
    poller: Option<JoinHandle<color_eyre::Result<()>>>,
}

impl AppEvent {
    fn from_crossterm(event: Event) -> Option<Self> {
        match event {
            Event::Key(key) if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) => {
                Some(Self::Key(key.code, key.modifiers))
            }
            Event::FocusGained | Event::Resize(_, _) => Some(Self::Render),
            _ => None,
        }
    }
}

impl TerminalPoller {
    pub(super) fn new(tx: UnboundedSender<AppEvent>) -> Self {
        Self { tx, poller: None }
    }

    pub(super) fn init_poller(&mut self) {
        let tx = self.tx.clone();
        let handle = tokio::spawn(async move {
            let mut reader = EventStream::new();
            while let Some(evt) = reader
                .next()
                .fuse()
                .await
                .and_then(|e| e.ok())
                .and_then(AppEvent::from_crossterm)
            {
                tracing::trace!("{:?}", evt);
                tx.send(evt)?;
            }
            Ok(())
        });
        self.poller = Some(handle);
    }
}

impl Drop for TerminalPoller {
    fn drop(&mut self) {}
}
