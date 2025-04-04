use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
    thread,
};

/// Message type sent between threads during file search
#[derive(Debug)]
pub enum WorkerMessage {
    /// Process a directory
    Directory(PathBuf),
    /// A file that matches search criteria
    File(PathBuf),
    /// No more items to process
    Done,
}

/// Thread pool for processing directories and files
pub struct WorkerPool {
    workers: Vec<thread::JoinHandle<()>>,
    directory_tx: Sender<WorkerMessage>,
    file_tx: Sender<WorkerMessage>,
    stopped: Arc<AtomicBool>,
}

impl WorkerPool {
    /// Create a new worker pool with the given number of threads
    pub fn new(
        num_threads: usize,
        directory_consumer: impl Fn(PathBuf) + Send + Clone + 'static,
        file_consumer: impl Fn(PathBuf) + Send + Clone + 'static,
    ) -> Self {
        let (directory_tx, directory_rx) = channel();
        let (file_tx, file_rx) = channel();
        let directory_rx = Arc::new(Mutex::new(directory_rx));
        let file_rx = Arc::new(Mutex::new(file_rx));
        let stopped = Arc::new(AtomicBool::new(false));

        let workers = (0..num_threads)
            .map(|_| {
                let directory_rx = Arc::clone(&directory_rx);
                let file_rx = Arc::clone(&file_rx);
                let directory_tx = directory_tx.clone();
                let file_tx = file_tx.clone();
                let stopped = Arc::clone(&stopped);
                let directory_consumer = directory_consumer.clone();
                let file_consumer = file_consumer.clone();

                thread::spawn(move || {
                    while !stopped.load(Ordering::Relaxed) {
                        // Process directories first
                        if let Ok(message) = directory_rx.lock().unwrap().try_recv() {
                            match message {
                                WorkerMessage::Directory(dir) => {
                                    directory_consumer(dir);
                                }
                                WorkerMessage::File(file) => {
                                    let _ = file_tx.send(WorkerMessage::File(file));
                                }
                                WorkerMessage::Done => {
                                    let _ = directory_tx.send(WorkerMessage::Done);
                                    break;
                                }
                            }
                            continue;
                        }

                        // Then process files
                        if let Ok(message) = file_rx.lock().unwrap().try_recv() {
                            match message {
                                WorkerMessage::File(file) => {
                                    file_consumer(file);
                                }
                                WorkerMessage::Directory(dir) => {
                                    let _ = directory_tx.send(WorkerMessage::Directory(dir));
                                }
                                WorkerMessage::Done => {
                                    let _ = file_tx.send(WorkerMessage::Done);
                                    break;
                                }
                            }
                            continue;
                        }

                        // If nothing to process, sleep briefly
                        thread::sleep(std::time::Duration::from_millis(1));
                    }
                })
            })
            .collect();

        WorkerPool {
            workers,
            directory_tx,
            file_tx,
            stopped,
        }
    }

    /// Submit a directory for processing
    pub fn submit_directory(&self, path: &Path) {
        if !self.stopped.load(Ordering::Relaxed) {
            let _ = self.directory_tx.send(WorkerMessage::Directory(path.to_path_buf()));
        }
    }

    /// Submit a file for processing
    pub fn submit_file(&self, path: &Path) {
        if !self.stopped.load(Ordering::Relaxed) {
            let _ = self.file_tx.send(WorkerMessage::File(path.to_path_buf()));
        }
    }

    /// Signal that there are no more items to process
    pub fn complete(&self) {
        let _ = self.directory_tx.send(WorkerMessage::Done);
        let _ = self.file_tx.send(WorkerMessage::Done);
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Relaxed);
        self.complete();

        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
} 