use std::path::{Path, PathBuf};
use std::fs::Metadata;
use std::io;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct MockPath {
    path: PathBuf,
    exists: bool,
    is_dir: bool,
    is_file: bool,
    file_name: Option<String>,
    extension: Option<String>,
    size: u64,
    modified: SystemTime,
}

impl MockPath {
    pub fn new(path_str: &str) -> Self {
        let path = PathBuf::from(path_str);
        let file_name = path.file_name().map(|f| f.to_string_lossy().to_string());
        let extension = path.extension().map(|e| e.to_string_lossy().to_string());
        
        MockPath {
            path,
            exists: true,
            is_dir: false,
            is_file: true,
            file_name,
            extension,
            size: 0,
            modified: SystemTime::now(),
        }
    }
    
    pub fn with_size(mut self, size: u64) -> Self {
        self.size = size;
        self
    }
    
    pub fn as_dir(mut self) -> Self {
        self.is_dir = true;
        self.is_file = false;
        self
    }
    
    pub fn metadata(&self) -> io::Result<MockMetadata> {
        Ok(MockMetadata {
            size: self.size,
            is_dir: self.is_dir,
            is_file: self.is_file,
            modified: self.modified,
        })
    }
    
    pub fn exists(&self) -> bool {
        self.exists
    }
    
    pub fn is_dir(&self) -> bool {
        self.is_dir
    }
    
    pub fn is_file(&self) -> bool {
        self.is_file
    }
    
    pub fn file_name(&self) -> Option<&str> {
        self.file_name.as_deref()
    }
    
    pub fn extension(&self) -> Option<&str> {
        self.extension.as_deref()
    }
}

#[derive(Debug)]
pub struct MockMetadata {
    size: u64,
    is_dir: bool,
    is_file: bool,
    modified: SystemTime,
}

impl MockMetadata {
    pub fn len(&self) -> u64 {
        self.size
    }
    
    pub fn is_dir(&self) -> bool {
        self.is_dir
    }
    
    pub fn is_file(&self) -> bool {
        self.is_file
    }
    
    pub fn modified(&self) -> io::Result<SystemTime> {
        Ok(self.modified)
    }
    
    pub fn created(&self) -> io::Result<SystemTime> {
        Ok(UNIX_EPOCH)
    }
} 