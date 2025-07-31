use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// An iterator that recursively finds all TOML files in a directory tree.
pub struct TomlFileIterator {
    root: PathBuf,
    work_stack: Vec<fs::ReadDir>,
}

impl TomlFileIterator {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let root: PathBuf = path.as_ref().to_path_buf();
        let read_dir = fs::read_dir(path)?;
        Ok(TomlFileIterator {
            root,
            work_stack: vec![read_dir],
        })
    }
}

impl Iterator for TomlFileIterator {
    type Item = io::Result<PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(current_dir) = self.work_stack.last_mut() {
            match current_dir.next() {
                Some(Ok(entry)) => {
                    let path = entry.path();
                    if path.is_dir() {
                        match fs::read_dir(path) {
                            Ok(new_read_dir) => {
                                self.work_stack.push(new_read_dir);
                            }
                            Err(e) => {
                                return Some(Err(e));
                            }
                        }
                    } else if path.is_file()
                        && let Some(ext) = path.extension()
                        && ext == "toml"
                    {
                        let relative_path = path.strip_prefix(&self.root).map(|p| p.to_path_buf());
                        tracing::info!("relative_path: {relative_path:?}");
                        return Some(Ok(path));
                    }
                }
                Some(Err(e)) => {
                    return Some(Err(e));
                }
                None => {
                    self.work_stack.pop();
                }
            }
        }
        None
    }
}
