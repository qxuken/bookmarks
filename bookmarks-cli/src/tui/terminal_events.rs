use crossterm::event::KeyEventKind;
use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};
use futures::{FutureExt, StreamExt};
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};

#[derive(Debug)]
pub(super) struct TerminalPoller {
    tx: UnboundedSender<TerminalEvent>,
    poller: Option<JoinHandle<color_eyre::Result<()>>>,
}

#[derive(Debug)]
pub(super) enum TerminalEvent {
    Key(KeyCode, KeyModifiers),
    Render,
}

impl TerminalEvent {
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
    pub(super) fn new(tx: UnboundedSender<TerminalEvent>) -> Self {
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
                .and_then(TerminalEvent::from_crossterm)
            {
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
