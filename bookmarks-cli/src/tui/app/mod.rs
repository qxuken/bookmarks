use std::path::PathBuf;

use futures::FutureExt;
use ratatui::DefaultTerminal;
use tokio::{select, sync::mpsc};

mod stack;
mod state;
mod view;

use crate::tui::{
    app::{
        stack::AppStack,
        state::AppState,
        view::{error::ErrorView, main::MainView},
    },
    data::{DataEvent, DataWorker},
    terminal_events::{TerminalEvent, TerminalPoller},
};

pub(super) struct App {
    terminal: DefaultTerminal,
}

impl App {
    pub fn try_new() -> color_eyre::Result<Self> {
        let terminal = ratatui::try_init()?;
        Ok(Self { terminal })
    }
}

impl Drop for App {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

impl App {
    pub async fn run(mut self, data: PathBuf) -> color_eyre::Result<()> {
        let tick_rate = std::time::Duration::from_secs(1);
        let mut interval = tokio::time::interval(tick_rate);
        let (e_tx, mut e_rx) = mpsc::unbounded_channel::<TerminalEvent>();
        let mut terminal_poller = TerminalPoller::new(e_tx);
        terminal_poller.init_poller();
        let (d_tx, mut d_rx) = mpsc::unbounded_channel::<DataEvent>();
        let mut data_worker = DataWorker::new(d_tx);
        data_worker.load_items(data);

        let mut state = AppState::default();
        let mut stack = AppStack::default();
        stack.push(Box::new(MainView::default()));
        while !stack.should_quit {
            let maybe_event = select! {
            maybe_event = d_rx.recv().fuse() => {
                if let Some(evt) = maybe_event  {
                    match evt {
                        DataEvent::NewFile(file) => {
                            state.items.push(file);
                        }
                        DataEvent::Loaded => {
                            state.items_loaded = true;
                        }
                        DataEvent::LoadError(err) => {
                            stack.push(Box::new(ErrorView(err.to_string())))
                        }
                    }
                }
                None
            }
            maybe_event = e_rx.recv().fuse() => {
                maybe_event
            }
            _ = interval.tick().fuse() => {
                None
            },
            };

            if let Some(event) = maybe_event {
                stack.handle_terminal_event(&mut state, event);
            }

            self.terminal
                .draw(|f| stack.render(f.area(), f.buffer_mut(), &mut state))?;
        }

        Ok(())
    }
}
