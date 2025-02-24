use std::path::PathBuf;
use thiserror::Error;

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
