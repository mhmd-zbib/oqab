use std::path::{Path, PathBuf};
use std::sync::mpsc;
use thiserror::Error;

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
        Self { rx }
    }
}
