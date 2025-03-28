use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;
use std::time::SystemTime;
use chrono::{DateTime, Utc};

use crate::error::{RagError, Result};
use crate::types::Metadata;

/// Trait for document loaders
#[async_trait]
pub trait DocumentLoader: Send + Sync {
    /// Load content from a source
    async fn load(&self, source: &str) -> Result<(String, Metadata)>;
}

/// Loader for loading documents from the file system
pub struct FileLoader {
    /// Base directory for relative paths
    base_dir: Option<PathBuf>,
}

impl FileLoader {
    /// Create a new file loader
    pub fn new() -> Self {
        Self { base_dir: None }
    }
    
    /// Create a new file loader with a base directory
    pub fn with_base_dir<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: Some(base_dir.as_ref().to_path_buf()),
        }
    }
    
    /// Resolve a path, applying the base directory if necessary
    fn resolve_path(&self, path: &str) -> PathBuf {
        if let Some(base_dir) = &self.base_dir {
            base_dir.join(path)
        } else {
            PathBuf::from(path)
        }
    }
}

#[async_trait]
impl DocumentLoader for FileLoader {
    async fn load(&self, source: &str) -> Result<(String, Metadata)> {
        let path = self.resolve_path(source);
        
        if !path.exists() {
            return Err(RagError::DocumentLoading(format!("File not found: {}", source)));
        }
        
        let content = fs::read_to_string(&path)
            .await
            .map_err(|e| RagError::DocumentLoading(format!("Failed to read file: {}", e)))?;
        
        let mut metadata = Metadata::new();
        metadata.source = Some(source.to_string());
        
        if let Ok(metadata_result) = fs::metadata(&path).await {
            if let Ok(modified) = metadata_result.modified() {
                // Convert SystemTime to DateTime<Utc>
                if let Ok(dt) = system_time_to_date_time(modified) {
                    metadata.created_at = Some(dt);
                }
            }
        }
        
        Ok((content, metadata))
    }
}

/// Convert SystemTime to DateTime<Utc>
fn system_time_to_date_time(time: SystemTime) -> Result<DateTime<Utc>> {
    // First convert to duration since UNIX_EPOCH
    let duration_since_epoch = time.duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| RagError::DocumentLoading(format!("Failed to convert time: {}", e)))?;
    
    // Convert to seconds and nanoseconds
    let secs = duration_since_epoch.as_secs();
    let nsecs = duration_since_epoch.subsec_nanos();
    
    // Create DateTime<Utc> from timestamp
    let dt = DateTime::from_timestamp(secs as i64, nsecs)
        .ok_or_else(|| RagError::DocumentLoading("Failed to create DateTime from timestamp".to_string()))?;
    
    Ok(dt)
}

impl Default for FileLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_file_loader() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        let mut file = File::create(&file_path).await.unwrap();
        file.write_all(b"Test content").await.unwrap();
        
        let loader = FileLoader::with_base_dir(temp_dir.path());
        let (content, metadata) = loader.load("test.txt").await.unwrap();
        
        assert_eq!(content, "Test content");
        assert_eq!(metadata.source, Some("test.txt".to_string()));
    }
    
    #[tokio::test]
    async fn test_file_not_found() {
        let loader = FileLoader::new();
        let result = loader.load("nonexistent.txt").await;
        assert!(result.is_err());
    }
} 