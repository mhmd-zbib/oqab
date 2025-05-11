use std::path::PathBuf;
use log::debug;

/// Platform-specific functionality for the search utility
/// 
/// This module provides platform-specific implementations for various
/// operations, ensuring the application works consistently across
/// different operating systems.
pub struct Platform;

impl Platform {
    /// Determine the root directory for the current platform
    /// 
    /// Returns the appropriate root directory path for the current OS:
    /// - "/" for Unix-like systems (Linux, macOS)
    /// - "C:\\" for Windows
    pub fn root_directory() -> PathBuf {
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
        {
            debug!("Using Unix-like root directory: /");
            PathBuf::from("/")
        }
        
        #[cfg(target_os = "windows")]
        {
            debug!("Using Windows root directory: C:\\");
            PathBuf::from("C:\\")
        }
        
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "freebsd", target_os = "windows")))]
        {
            debug!("Unknown OS, defaulting to current directory");
            PathBuf::from(".")
        }
    }
    
    /// Check if a path is the root directory for the current platform
    pub fn is_root_path(path: &str) -> bool {
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
        {
            path == "/"
        }
        
        #[cfg(target_os = "windows")]
        {
            path == "C:\\" || path.to_lowercase() == "c:\\" || path == "\\"
        }
        
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "freebsd", target_os = "windows")))]
        {
            false
        }
    }
    
    /// Get a list of common system directories to search based on the platform
    pub fn common_search_paths() -> Vec<PathBuf> {
        #[cfg(target_os = "linux")]
        {
            vec![
                PathBuf::from("/etc"),
                PathBuf::from("/usr"),
                PathBuf::from("/var"),
                PathBuf::from("/home"),
                PathBuf::from("/opt"),
            ]
        }
        
        #[cfg(target_os = "macos")]
        {
            vec![
                PathBuf::from("/Applications"),
                PathBuf::from("/Library"),
                PathBuf::from("/System"),
                PathBuf::from("/Users"),
                PathBuf::from("/usr"),
                PathBuf::from("/etc"),
            ]
        }
        
        #[cfg(target_os = "windows")]
        {
            vec![
                PathBuf::from("C:\\Program Files"),
                PathBuf::from("C:\\Program Files (x86)"),
                PathBuf::from("C:\\Users"),
                PathBuf::from("C:\\Windows"),
            ]
        }
        
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            vec![PathBuf::from(".")]
        }
    }
    
    /// Get the home directory for the current user
    pub fn home_directory() -> Option<PathBuf> {
        dirs::home_dir()
    }
    
    /// Get the current working directory
    pub fn current_directory() -> Option<PathBuf> {
        std::env::current_dir().ok()
    }
}
