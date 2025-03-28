//! Storage module for persisting data in Lomusai
//! 
//! This module provides interfaces and implementations for storing
//! various types of data including workflows, threads, messages, and evaluation results.

mod constants;
mod types;
mod providers;

pub use constants::*;
pub use types::*;
pub use providers::*;

use crate::error::Result;
use std::sync::Arc;

/// Creates a new in-memory storage provider
pub fn create_memory_storage(name: String) -> Result<Arc<dyn Storage>> {
    Ok(Arc::new(providers::memory::MemoryStorage::new(name)))
}

/// Creates a file-based SQLite storage provider
#[cfg(feature = "sqlite")]
pub fn create_sqlite_storage(name: String, path: String) -> Result<Arc<dyn Storage>> {
    Ok(Arc::new(providers::sqlite::SqliteStorage::new(name, path)?))
} 