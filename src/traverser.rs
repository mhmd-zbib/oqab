use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::sync::mpsc;
use walkdir::WalkDir;

#[derive(Error, Debug)]
pub enum TraverserError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Permission Denied: {0}")]
    PermissionDenied(PathBuf),
    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),
    #[error("Channel closed")]
    ChannelClosed,
}

pub struct Traverser {
    rx: mpsc::Receiver<Result<PathBuf, TraverserError>>,
}

impl Traverser {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        let (tx, rx) = mpsc::channel(1024);
        let root_path = root.as_ref().to_path_buf();

        tokio::spawn(async move {
            let walker = WalkDir::new(root_path).follow_links(true).into_iter();

            for entry in walker {
                match entry {
                    Ok(entry) => {
                        if tx.send(Ok(entry.into_path())).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break;
                }
            }
        });
        Self { rx }
    }
}
