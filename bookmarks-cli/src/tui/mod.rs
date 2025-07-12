use std::path::PathBuf;

use crate::tui::app::App;

mod app;
mod data;
mod terminal_events;

#[tokio::main]
pub async fn run(data: PathBuf) -> color_eyre::Result<()> {
    let app = App::try_new()?;
    app.run(data).await
}
