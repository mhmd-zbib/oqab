use std::path::Path;
use std::time::{UNIX_EPOCH};
use chrono::{NaiveDate};

use crate::filters::{Filter, FilterResult};

/// Filter that matches files by their modification date
#[derive(Debug)]
pub struct DateFilter {
    /// Files must be newer than this timestamp (in seconds since UNIX epoch)
    newer_than: Option<i64>,
    /// Files must be older than this timestamp (in seconds since UNIX epoch)
    older_than: Option<i64>,
}

impl DateFilter {
    /// Create a new date filter
    pub fn new(newer_than: Option<i64>, older_than: Option<i64>) -> Self {
        Self { newer_than, older_than }
    }
    
    /// Create a filter for files newer than the given date string (YYYY-MM-DD)
    pub fn newer_than(date_str: &str) -> Result<Self, chrono::ParseError> {
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
        let datetime = date.and_hms_opt(0, 0, 0).unwrap();
        // Convert to UTC and get timestamp
        let timestamp = datetime.and_utc().timestamp();
        
        Ok(Self {
            newer_than: Some(timestamp),
            older_than: None,
        })
    }
    
    /// Create a filter for files older than the given date string (YYYY-MM-DD)
    pub fn older_than(date_str: &str) -> Result<Self, chrono::ParseError> {
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
        let datetime = date.and_hms_opt(23, 59, 59).unwrap();
        // Convert to UTC and get timestamp
        let timestamp = datetime.and_utc().timestamp();
        
        Ok(Self {
            newer_than: None,
            older_than: Some(timestamp),
        })
    }
    
    /// Create a filter for files within a date range (YYYY-MM-DD)
    pub fn date_range(
        newer_than: &str,
        older_than: &str,
    ) -> Result<Self, chrono::ParseError> {
        let newer_date = NaiveDate::parse_from_str(newer_than, "%Y-%m-%d")?;
        let newer_datetime = newer_date.and_hms_opt(0, 0, 0).unwrap();
        // Convert to UTC and get timestamp
        let newer_timestamp = newer_datetime.and_utc().timestamp();
        
        let older_date = NaiveDate::parse_from_str(older_than, "%Y-%m-%d")?;
        let older_datetime = older_date.and_hms_opt(23, 59, 59).unwrap();
        // Convert to UTC and get timestamp
        let older_timestamp = older_datetime.and_utc().timestamp();
        
        Ok(Self {
            newer_than: Some(newer_timestamp),
            older_than: Some(older_timestamp),
        })
    }
}

impl Filter for DateFilter {
    fn filter(&self, path: &Path) -> FilterResult {
        // Get file metadata
        let metadata = match std::fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(_) => return FilterResult::Reject,
        };
        
        // Get modification time
        let modified = match metadata.modified() {
            Ok(time) => time,
            Err(_) => return FilterResult::Reject,
        };
        
        // Convert to timestamp
        let modified_secs = match modified.duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_secs() as i64,
            Err(_) => return FilterResult::Reject,
        };
        
        // Check if file is newer than the specified date
        if let Some(newer_than) = self.newer_than {
            if modified_secs < newer_than {
                return FilterResult::Reject;
            }
        }
        
        // Check if file is older than the specified date
        if let Some(older_than) = self.older_than {
            if modified_secs > older_than {
                return FilterResult::Reject;
            }
        }
        
        FilterResult::Accept
    }
} 