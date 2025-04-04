use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Sender, TryRecvError},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use log::{debug, warn};

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
        
        // We need to share receivers between threads, so we'll wrap them in mutexes
        // for thread safety (mpsc::Receiver is !Sync)
        let directory_rx = Arc::new(Mutex::new(directory_rx));
        let file_rx = Arc::new(Mutex::new(file_rx));
        
        let stopped = Arc::new(AtomicBool::new(false));

        let workers = (0..num_threads)
            .map(|id| {
                // Clone the thread-specific resources
                let directory_rx = Arc::clone(&directory_rx);
                let file_rx = Arc::clone(&file_rx);
                let directory_tx = directory_tx.clone();
                let file_tx = file_tx.clone();
                let stopped = Arc::clone(&stopped);
                let directory_consumer = directory_consumer.clone();
                let file_consumer = file_consumer.clone();

                thread::spawn(move || {
                    debug!("Worker thread {} started", id);
                    
                    let timeout = Duration::from_millis(100);
                    
                    while !stopped.load(Ordering::Relaxed) {
                        let mut processed_message = false;
                        
                        // Process directories first with timeout
                        let dir_msg = match directory_rx.lock() {
                            Ok(rx) => {
                                match rx.try_recv() {
                                    Ok(msg) => Some(msg),
                                    Err(TryRecvError::Empty) => None,
                                    Err(TryRecvError::Disconnected) => {
                                        debug!("Directory channel disconnected for worker {}", id);
                                        break;
                                    }
                                }
                            },
                            Err(_) => {
                                warn!("Failed to acquire lock on directory_rx for worker {}", id);
                                None
                            }
                        };
                        
                        if let Some(message) = dir_msg {
                            match message {
                                WorkerMessage::Directory(dir) => {
                                    directory_consumer(dir);
                                    processed_message = true;
                                }
                                WorkerMessage::File(file) => {
                                    if let Err(e) = file_tx.send(WorkerMessage::File(file)) {
                                        warn!("Failed to forward file to file queue: {}", e);
                                    }
                                    processed_message = true;
                                }
                                WorkerMessage::Done => {
                                    debug!("Worker {} received Done message for directories", id);
                                    if let Err(e) = directory_tx.send(WorkerMessage::Done) {
                                        warn!("Failed to forward Done message: {}", e);
                                    }
                                    break;
                                }
                            }
                        }

                        // Then process files
                        let file_msg = match file_rx.lock() {
                            Ok(rx) => {
                                match rx.try_recv() {
                                    Ok(msg) => Some(msg),
                                    Err(TryRecvError::Empty) => None,
                                    Err(TryRecvError::Disconnected) => {
                                        debug!("File channel disconnected for worker {}", id);
                                        break;
                                    }
                                }
                            },
                            Err(_) => {
                                warn!("Failed to acquire lock on file_rx for worker {}", id);
                                None
                            }
                        };
                        
                        if let Some(message) = file_msg {
                            match message {
                                WorkerMessage::File(file) => {
                                    file_consumer(file);
                                    processed_message = true;
                                }
                                WorkerMessage::Directory(dir) => {
                                    if let Err(e) = directory_tx.send(WorkerMessage::Directory(dir)) {
                                        warn!("Failed to forward directory to directory queue: {}", e);
                                    }
                                    processed_message = true;
                                }
                                WorkerMessage::Done => {
                                    debug!("Worker {} received Done message for files", id);
                                    if let Err(e) = file_tx.send(WorkerMessage::Done) {
                                        warn!("Failed to forward Done message: {}", e);
                                    }
                                    break;
                                }
                            }
                        }
                        
                        // If no messages were processed this cycle, yield to other threads
                        if !processed_message {
                            thread::sleep(timeout);
                        }
                    }
                    
                    debug!("Worker thread {} shutting down", id);
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
    pub fn submit_directory(&self, path: &Path) -> bool {
        if self.stopped.load(Ordering::Relaxed) {
            debug!("Not submitting directory: worker pool is stopped");
            return false;
        }
        
        match self.directory_tx.send(WorkerMessage::Directory(path.to_path_buf())) {
            Ok(_) => true,
            Err(e) => {
                warn!("Failed to submit directory: {}", e);
                false
            }
        }
    }

    /// Submit a file for processing
    pub fn submit_file(&self, path: &Path) -> bool {
        if self.stopped.load(Ordering::Relaxed) {
            debug!("Not submitting file: worker pool is stopped");
            return false;
        }
        
        match self.file_tx.send(WorkerMessage::File(path.to_path_buf())) {
            Ok(_) => true,
            Err(e) => {
                warn!("Failed to submit file: {}", e);
                false
            }
        }
    }

    /// Signal that there are no more items to process
    pub fn complete(&self) {
        debug!("Signaling worker pool completion");
        
        // Send Done message to both queues
        if let Err(e) = self.directory_tx.send(WorkerMessage::Done) {
            warn!("Failed to send Done message to directory queue: {}", e);
        }
        
        if let Err(e) = self.file_tx.send(WorkerMessage::Done) {
            warn!("Failed to send Done message to file queue: {}", e);
        }
    }
    
    /// Wait for all worker threads to complete
    pub fn join(mut self) {
        debug!("Waiting for all worker threads to complete");
        self.stopped.store(true, Ordering::Relaxed);
        self.complete();

        while let Some(worker) = self.workers.pop() {
            if let Err(e) = worker.join() {
                warn!("Worker thread panicked: {:?}", e);
            }
        }
        debug!("All worker threads joined successfully");
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        debug!("WorkerPool being dropped, stopping workers");
        self.stopped.store(true, Ordering::Relaxed);
        self.complete();

        for worker in self.workers.drain(..) {
            // Don't block on join in the destructor, but log if there were problems
            if worker.is_finished() {
                if let Err(e) = worker.join() {
                    warn!("Worker thread panicked: {:?}", e);
                }
            }
        }
    }
} 