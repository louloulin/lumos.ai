//! Utility functions for memory vector storage

use lumosai_vector_core::prelude::*;

/// Utility functions for vector operations
pub mod vector_utils {
    use super::*;
    
    /// Normalize a vector to unit length
    pub fn normalize_vector(vector: &mut [f32]) -> Result<()> {
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm == 0.0 {
            return Err(VectorError::InvalidVector("Cannot normalize zero vector".to_string()));
        }
        
        for x in vector.iter_mut() {
            *x /= norm;
        }
        
        Ok(())
    }
    
    /// Calculate the magnitude of a vector
    pub fn vector_magnitude(vector: &[f32]) -> f32 {
        vector.iter().map(|x| x * x).sum::<f32>().sqrt()
    }
    
    /// Check if two vectors have the same dimension
    pub fn check_dimension_match(a: &[f32], b: &[f32]) -> Result<()> {
        if a.len() != b.len() {
            return Err(VectorError::dimension_mismatch(a.len(), b.len()));
        }
        Ok(())
    }
    
    /// Validate that a vector has the expected dimension
    pub fn validate_vector_dimension(vector: &[f32], expected_dim: usize) -> Result<()> {
        if vector.len() != expected_dim {
            return Err(VectorError::dimension_mismatch(expected_dim, vector.len()));
        }
        Ok(())
    }
    
    /// Check if a vector contains any NaN or infinite values
    pub fn validate_vector_values(vector: &[f32]) -> Result<()> {
        for (i, &value) in vector.iter().enumerate() {
            if value.is_nan() {
                return Err(VectorError::InvalidVector(format!("NaN value at index {}", i)));
            }
            if value.is_infinite() {
                return Err(VectorError::InvalidVector(format!("Infinite value at index {}", i)));
            }
        }
        Ok(())
    }
    
    /// Create a random vector for testing purposes
    #[cfg(test)]
    pub fn create_random_vector(dimension: usize, seed: u64) -> Vec<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut vector = Vec::with_capacity(dimension);
        for i in 0..dimension {
            let mut hasher = DefaultHasher::new();
            (seed + i as u64).hash(&mut hasher);
            let hash = hasher.finish();
            let value = (hash as f32) / (u64::MAX as f32) * 2.0 - 1.0; // Range [-1, 1]
            vector.push(value);
        }
        vector
    }
}

/// Utility functions for metadata operations
pub mod metadata_utils {
    use super::*;
    use std::collections::HashMap;
    
    /// Create metadata from key-value pairs
    pub fn create_metadata(pairs: Vec<(&str, MetadataValue)>) -> Metadata {
        pairs.into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect()
    }
    
    /// Merge two metadata objects
    pub fn merge_metadata(base: &Metadata, overlay: &Metadata) -> Metadata {
        let mut result = base.clone();
        for (key, value) in overlay {
            result.insert(key.clone(), value.clone());
        }
        result
    }
    
    /// Extract string value from metadata
    pub fn get_string_value(metadata: &Metadata, key: &str) -> Option<String> {
        match metadata.get(key) {
            Some(MetadataValue::String(s)) => Some(s.clone()),
            _ => None,
        }
    }
    
    /// Extract numeric value from metadata
    pub fn get_numeric_value(metadata: &Metadata, key: &str) -> Option<f64> {
        match metadata.get(key) {
            Some(MetadataValue::Integer(i)) => Some(*i as f64),
            Some(MetadataValue::Float(f)) => Some(*f),
            _ => None,
        }
    }
    
    /// Extract boolean value from metadata
    pub fn get_boolean_value(metadata: &Metadata, key: &str) -> Option<bool> {
        match metadata.get(key) {
            Some(MetadataValue::Boolean(b)) => Some(*b),
            _ => None,
        }
    }
    
    /// Check if metadata contains a specific key
    pub fn has_key(metadata: &Metadata, key: &str) -> bool {
        metadata.contains_key(key)
    }
    
    /// Get all string keys from metadata
    pub fn get_string_keys(metadata: &Metadata) -> Vec<String> {
        metadata.iter()
            .filter_map(|(k, v)| {
                if matches!(v, MetadataValue::String(_)) {
                    Some(k.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Utility functions for performance monitoring
pub mod perf_utils {
    use std::time::{Duration, Instant};
    
    /// Simple performance timer
    pub struct Timer {
        start: Instant,
    }
    
    impl Timer {
        /// Start a new timer
        pub fn start() -> Self {
            Self {
                start: Instant::now(),
            }
        }
        
        /// Get elapsed time in milliseconds
        pub fn elapsed_ms(&self) -> u64 {
            self.start.elapsed().as_millis() as u64
        }
        
        /// Get elapsed time as Duration
        pub fn elapsed(&self) -> Duration {
            self.start.elapsed()
        }
        
        /// Reset the timer
        pub fn reset(&mut self) {
            self.start = Instant::now();
        }
    }
    
    /// Memory usage utilities
    pub mod memory {
        /// Estimate memory usage of a string
        pub fn string_memory_usage(s: &str) -> usize {
            s.len() + std::mem::size_of::<String>()
        }
        
        /// Estimate memory usage of a vector
        pub fn vector_memory_usage<T>(v: &[T]) -> usize {
            v.len() * std::mem::size_of::<T>() + std::mem::size_of::<Vec<T>>()
        }
        
        /// Convert bytes to human-readable format
        pub fn format_bytes(bytes: u64) -> String {
            const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
            let mut size = bytes as f64;
            let mut unit_index = 0;
            
            while size >= 1024.0 && unit_index < UNITS.len() - 1 {
                size /= 1024.0;
                unit_index += 1;
            }
            
            if unit_index == 0 {
                format!("{} {}", bytes, UNITS[unit_index])
            } else {
                format!("{:.2} {}", size, UNITS[unit_index])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vector_normalization() {
        let mut vector = vec![3.0, 4.0, 0.0];
        vector_utils::normalize_vector(&mut vector).unwrap();
        
        let magnitude = vector_utils::vector_magnitude(&vector);
        assert!((magnitude - 1.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_vector_validation() {
        let valid_vector = vec![1.0, 2.0, 3.0];
        assert!(vector_utils::validate_vector_values(&valid_vector).is_ok());
        
        let invalid_vector = vec![1.0, f32::NAN, 3.0];
        assert!(vector_utils::validate_vector_values(&invalid_vector).is_err());
        
        let infinite_vector = vec![1.0, f32::INFINITY, 3.0];
        assert!(vector_utils::validate_vector_values(&infinite_vector).is_err());
    }
    
    #[test]
    fn test_metadata_utils() {
        let metadata = metadata_utils::create_metadata(vec![
            ("name", MetadataValue::String("test".to_string())),
            ("score", MetadataValue::Float(0.95)),
            ("active", MetadataValue::Boolean(true)),
        ]);
        
        assert_eq!(metadata_utils::get_string_value(&metadata, "name"), Some("test".to_string()));
        assert_eq!(metadata_utils::get_numeric_value(&metadata, "score"), Some(0.95));
        assert_eq!(metadata_utils::get_boolean_value(&metadata, "active"), Some(true));
        assert!(metadata_utils::has_key(&metadata, "name"));
        assert!(!metadata_utils::has_key(&metadata, "nonexistent"));
    }
    
    #[test]
    fn test_memory_formatting() {
        assert_eq!(perf_utils::memory::format_bytes(512), "512 B");
        assert_eq!(perf_utils::memory::format_bytes(1024), "1.00 KB");
        assert_eq!(perf_utils::memory::format_bytes(1536), "1.50 KB");
        assert_eq!(perf_utils::memory::format_bytes(1048576), "1.00 MB");
    }
    
    #[test]
    fn test_timer() {
        let timer = perf_utils::Timer::start();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 10);
    }
}
