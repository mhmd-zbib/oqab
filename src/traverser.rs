use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};
use futures::Stream;
use thiserror::Error;
use tokio::sync::mpsc;
use walkdir::WalkDir;

#[derive(Error, Debug)]
pub enum TraverserError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Permission denied: {0}")]
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
                    Err(err) => {
                        let path = err.path().map(PathBuf::from).unwrap_or_default();
                        let error = if let Some(io_err) = err.io_error() {
                            if io_err.kind() == std::io::ErrorKind::PermissionDenied {
                                TraverserError::PermissionDenied(path)
                            } else {
                                TraverserError::Io(std::io::Error::new(
                                    io_err.kind(),
                                    io_err.to_string(),
                                ))
                            }
                        } else {
                            TraverserError::InvalidPath(path)
                        };
                        if tx.send(Err(error)).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });
        Self { rx }
    }
}

impl Stream for Traverser {
    type Item = Result<PathBuf, TraverserError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}
