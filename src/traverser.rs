use futures::Stream;
use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::mpsc;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Clone)]
pub struct TraverserConfig {
    pub max_depth: Option<usize>,
    pub follow_symlinks: bool,
    pub exclude_patterns: Vec<String>,
}

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
    #[error("Max depth reached: {0}")]
    MaxDepthReached(PathBuf),
}

pub struct Traverser {
    rx: mpsc::Receiver<Result<PathBuf, TraverserError>>,
    start_time: Instant,
    completion_time: Option<Instant>,
}

impl Traverser {
    pub fn new<P: AsRef<Path>>(root: P, config: TraverserConfig) -> Self {
        let (tx, rx) = mpsc::channel(1024);
        let root_path = root.as_ref().to_path_buf();

        tokio::spawn(async move {
            let mut queue = VecDeque::new();
            let mut visited = HashSet::new();
            queue.push_back(root_path.clone());

            while let Some(current_path) = queue.pop_front() {
                if visited.contains(&current_path) {
                    continue;
                }
                visited.insert(current_path.clone());

                let walker = WalkDir::new(&current_path)
                    .max_depth(1)
                    .follow_links(config.follow_symlinks)
                    .into_iter();

                for entry in walker {
                    match entry {
                        Ok(entry) => {
                            if Self::should_include(&entry, &config) {
                                let path = entry.into_path();
                                if path.is_dir() {
                                    queue.push_back(path.clone());
                                }
                                if tx.send(Ok(path)).await.is_err() {
                                    return;
                                }
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
                                return;
                            }
                        }
                    }
                }
            }

            let _ = tx.send(Err(TraverserError::ChannelClosed)).await;
        });

        Self {
            rx,
            start_time: Instant::now(),
            completion_time: None,
        }
    }

    fn should_include(entry: &DirEntry, config: &TraverserConfig) -> bool {
        let path = entry.path();
        !config
            .exclude_patterns
            .iter()
            .any(|pattern| path.to_string_lossy().contains(pattern))
    }

    pub fn elapsed_time(&self) -> Option<Duration> {
        self.completion_time.map(|end| end.duration_since(self.start_time))
    }
}

impl Stream for Traverser {
    type Item = Result<PathBuf, TraverserError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.rx).poll_recv(cx) {
            Poll::Ready(Some(Err(TraverserError::ChannelClosed))) => {
                self.completion_time = Some(Instant::now());
                Poll::Ready(None)
            }
            Poll::Ready(item) => Poll::Ready(item),
            Poll::Pending => Poll::Pending,
        }
    }
}
