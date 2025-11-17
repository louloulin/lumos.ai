//! Memory storage type resolver for progressive API
//!
//! This module provides functionality to resolve memory storage type names (strings)
//! to memory instances, enabling a more user-friendly API where users can specify
//! memory storage by name instead of manually creating memory instances.

use crate::{Error, Result};
use crate::memory::{BasicMemory, Memory};
use std::sync::Arc;

/// Resolve a memory storage type name to a memory instance
///
/// This function maps common memory storage type names (strings) to their
/// corresponding memory instances. It supports basic memory types and provides
/// a way to extend with custom storage backends.
///
/// # Arguments
///
/// * `storage_type` - The name of the storage type (e.g., "basic", "memory", "working")
///
/// # Returns
///
/// An `Arc<dyn Memory>` if the storage type is recognized, or an error if not found.
///
/// # Examples
///
/// ```rust
/// use lumosai_core::agent::memory_resolver::resolve_memory_storage;
///
/// let memory = resolve_memory_storage("basic")?;
/// let memory = resolve_memory_storage("working")?;
/// ```
pub fn resolve_memory_storage(storage_type: &str) -> Result<Arc<dyn Memory>> {
    match storage_type.to_lowercase().as_str() {
        // Basic in-memory storage (default)
        "basic" | "memory" | "default" => {
            Ok(Arc::new(BasicMemory::new(None, None)))
        }
        
        // Working memory (short-term with capacity)
        "working" | "short_term" | "short-term" => {
            // Create working memory with default capacity
            // Note: UnifiedMemory::working() returns Memory struct, not Arc<dyn Memory>
            // For now, we use BasicMemory as a simple working memory
            // Future: Support proper working memory with Arc wrapper
            Ok(Arc::new(BasicMemory::new(None, None)))
        }
        
        // Semantic memory (long-term with vector search)
        "semantic" | "long_term" | "long-term" | "vector" => {
            // Create semantic memory
            // Note: UnifiedMemory::semantic() requires LLM provider for embeddings
            // For now, we use BasicMemory as a fallback
            // Future: Support proper semantic memory with LLM provider
            Ok(Arc::new(BasicMemory::new(None, None)))
        }
        
        // Note: For now, we only support basic memory types
        // Future: Support "postgres", "sqlite", "mongodb", etc.
        _ => Err(Error::NotFound(format!(
            "Memory storage type '{}' not found. Available types: basic, working, semantic",
            storage_type
        ))),
    }
}

/// Get list of available memory storage type names
///
/// Returns a list of all storage type names that can be resolved by `resolve_memory_storage`.
pub fn get_available_memory_storage_types() -> Vec<&'static str> {
    vec![
        "basic",
        "memory",
        "default",
        "working",
        "short_term",
        "semantic",
        "long_term",
        "vector",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_memory_storage() {
        // Test valid storage types
        assert!(resolve_memory_storage("basic").is_ok());
        assert!(resolve_memory_storage("memory").is_ok());
        assert!(resolve_memory_storage("working").is_ok());
        assert!(resolve_memory_storage("semantic").is_ok());
        
        // Test case insensitive
        assert!(resolve_memory_storage("Basic").is_ok());
        assert!(resolve_memory_storage("WORKING").is_ok());
        
        // Test aliases
        assert!(resolve_memory_storage("default").is_ok());
        assert!(resolve_memory_storage("short_term").is_ok());
        assert!(resolve_memory_storage("long_term").is_ok());
        
        // Test invalid storage type
        assert!(resolve_memory_storage("unknown").is_err());
    }

    #[test]
    fn test_get_available_memory_storage_types() {
        let types = get_available_memory_storage_types();
        assert!(!types.is_empty());
        assert!(types.contains(&"basic"));
        assert!(types.contains(&"working"));
        assert!(types.contains(&"semantic"));
    }
}

