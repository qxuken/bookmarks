use bookmarks_data::BookmarkFile;

#[derive(Default, Debug, Clone)]
pub struct AppState {
    pub items: Vec<BookmarkFile>,
    pub items_loaded: bool,
}
