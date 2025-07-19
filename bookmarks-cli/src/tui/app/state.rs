use bookmarks_data::BookmarkFile;

use crate::tui::app::view::loader::Loader;

#[derive(Default, Debug, Clone)]
pub struct AppState {
    pub items: Vec<BookmarkFile>,
    pub items_loaded: bool,
    pub loader: Loader,
}
