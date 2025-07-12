use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

use serde::{Deserialize, Serialize};
use tracing::Level;

mod toml_file_iterator;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkRecord {
    pub title: Option<String>,
    pub url: String,
    pub tags: Option<Vec<String>>,
    pub description: Option<String>,
    pub embeddings: Option<Vec<f32>>,
}

impl BookmarkRecord {
    pub fn fuzzy_string(&self) -> String {
        let mut parts: Vec<&str> = vec![];

        if let Some(tags) = &self.tags {
            parts.extend(tags.iter().filter(|t| !t.is_empty()).map(|s| s.as_str()));
        }

        if let Some(title) = &self.title {
            parts.push(title);
        }

        parts.push(&self.url);

        if let Some(description) = &self.description {
            parts.push(description);
        }

        parts.join(" ")
    }
}

#[derive(Debug, Clone)]
pub struct BookmarkFile {
    pub content: BookmarkRecord,
    pub path: PathBuf,
}

#[tracing::instrument]
pub fn load_from_fs<P>(path: P) -> io::Result<impl Iterator<Item = BookmarkFile>>
where
    P: AsRef<Path> + fmt::Debug,
{
    let toml_path_iterator = toml_file_iterator::TomlFileIterator::new(path)?;
    let files = toml_path_iterator.filter_map(|path_result| match path_result {
        Ok(path) => {
            let file = match fs::read_to_string(&path) {
                Ok(file) => file,
                Err(e) => {
                    tracing::warn!("Failed to read {path:?}. {e}");
                    return None;
                }
            };

            let content = match toml::from_str(&file) {
                Ok(content) => content,
                Err(e) => {
                    tracing::warn!("Failed to parse {path:?}. {e}");
                    return None;
                }
            };

            tracing::trace!("Processed {path:?}. {content:?}");
            Some(BookmarkFile { path, content })
        }
        Err(e) => {
            tracing::warn!("Failed {e}");
            None
        }
    });
    Ok(files)
}

#[tracing::instrument]
pub fn save_to_fs(bookmark: &BookmarkFile) -> io::Result<()> {
    let str_content = toml::to_string_pretty(&bookmark.content)
        .map_err(|err| io::Error::other(err.to_string()))?;
    fs::write(&bookmark.path, str_content)
}

#[tracing::instrument(skip(records), ret(level = Level::DEBUG))]
pub fn search(
    needle: &str,
    records: impl IntoIterator<Item = BookmarkRecord>,
) -> impl Iterator<Item = (BookmarkRecord, i64)> {
    let matcher = SkimMatcherV2::default();
    let mut keys: Vec<_> = records
        .into_iter()
        .filter_map(|r| {
            let fuzz = r.fuzzy_string();
            Some(r).zip(matcher.fuzzy_match(&fuzz, needle))
        })
        .collect();
    keys.sort_unstable_by_key(|r| r.1);
    keys.into_iter().rev()
}
