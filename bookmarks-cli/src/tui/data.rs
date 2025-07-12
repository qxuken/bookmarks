use std::{io, path::PathBuf};

use bookmarks_data::BookmarkFile;
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};

#[derive(Debug)]
pub(super) struct DataWorker {
    tx: UnboundedSender<DataEvent>,
    loader: Option<JoinHandle<color_eyre::Result<()>>>,
}

#[derive(Debug)]
pub(super) enum DataEvent {
    NewFile(BookmarkFile),
    Loaded,
    LoadError(io::Error),
}

impl DataWorker {
    pub(super) fn new(tx: UnboundedSender<DataEvent>) -> Self {
        Self { tx, loader: None }
    }

    pub(super) fn load_items(&mut self, data: PathBuf) {
        let tx = self.tx.clone();

        let handle = tokio::spawn(async move {
            match bookmarks_data::load_from_fs(data) {
                Ok(iter) => {
                    for file in iter {
                        tx.send(DataEvent::NewFile(file))?;
                    }
                    tx.send(DataEvent::Loaded)?;
                }
                Err(err) => {
                    tx.send(DataEvent::LoadError(err))?;
                }
            }
            Ok(())
        });
        self.loader = Some(handle);
    }
}

impl Drop for DataWorker {
    fn drop(&mut self) {
        if let Some(loader) = &self.loader
            && !loader.is_finished()
        {
            loader.abort();
        }
    }
}
