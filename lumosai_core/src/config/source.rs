//! Configuration source implementations
//!
//! This module provides various sources for loading configuration data
//! including files, environment variables, and remote sources.

use crate::{Error, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Configuration source trait
pub trait ConfigSource {
    /// Load configuration data as key-value pairs
    fn load(&self) -> Result<HashMap<String, serde_json::Value>>;

    /// Get source name for debugging
    fn name(&self) -> &'static str;
}

/// File-based configuration source
pub struct FileConfigSource {
    path: String,
}

impl FileConfigSource {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_string_lossy().to_string(),
        }
    }
}

impl ConfigSource for FileConfigSource {
    fn load(&self) -> Result<HashMap<String, serde_json::Value>> {
        let content = fs::read_to_string(&self.path).map_err(|e| {
            Error::Config(format!("Failed to read config file {}: {}", self.path, e))
        })?;

        let config: HashMap<String, serde_json::Value> = serde_yaml::from_str(&content)
            .map_err(|e| Error::Config(format!("Failed to parse YAML: {}", e)))?;

        Ok(config)
    }

    fn name(&self) -> &'static str {
        "file"
    }
}

/// Environment variable configuration source
pub struct EnvConfigSource {
    prefix: String,
}

impl EnvConfigSource {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }
}

impl ConfigSource for EnvConfigSource {
    fn load(&self) -> Result<HashMap<String, serde_json::Value>> {
        let mut config = HashMap::new();

        for (key, value) in std::env::vars() {
            if key.starts_with(&self.prefix) {
                let config_key = key
                    .strip_prefix(&self.prefix)
                    .unwrap_or(&key)
                    .to_lowercase();

                // Try to parse as JSON, fallback to string
                let parsed_value = serde_json::from_str(&value)
                    .unwrap_or_else(|_| serde_json::Value::String(value));

                config.insert(config_key, parsed_value);
            }
        }

        Ok(config)
    }

    fn name(&self) -> &'static str {
        "env"
    }
}

/// In-memory configuration source
pub struct MemoryConfigSource {
    config: HashMap<String, serde_json::Value>,
}

impl MemoryConfigSource {
    pub fn new(config: HashMap<String, serde_json::Value>) -> Self {
        Self { config }
    }
}

impl ConfigSource for MemoryConfigSource {
    fn load(&self) -> Result<HashMap<String, serde_json::Value>> {
        Ok(self.config.clone())
    }

    fn name(&self) -> &'static str {
        "memory"
    }
}
