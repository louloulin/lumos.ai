//! Configuration loading utilities
//!
//! Provides flexible configuration loading from multiple sources including
//! environment variables, files, and remote configuration services.

use crate::{Error, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

/// Configuration source enum
#[derive(Debug, Clone)]
pub enum ConfigSource {
    /// Load from file (auto-detect format)
    File(String),

    /// Load from environment variables with prefix
    Environment(String),

    /// Load from specific environment variables
    EnvironmentVars(Vec<String>),

    /// Load from remote URL
    Remote {
        url: String,
        auth_token: Option<String>,
        headers: HashMap<String, String>,
    },

    /// Load from multiple sources with merge strategy
    Merged {
        sources: Vec<ConfigSource>,
        strategy: MergeStrategy,
    },

    /// Auto-detect configuration source
    AutoDetect,
}

/// Configuration merge strategy
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeStrategy {
    /// First source takes precedence
    FirstWins,
    /// Last source takes precedence
    LastWins,
    /// Deep merge with conflict resolution
    DeepMerge,
}

/// Configuration loader with support for multiple sources and formats
#[derive(Debug, Clone)]
pub struct ConfigLoader {
    /// Default load timeout in seconds
    timeout: u64,

    /// Maximum number of retries for remote sources
    max_retries: u32,

    /// User agent for remote requests
    user_agent: String,
}

/// Trait for custom configuration source handlers
pub trait ConfigSourceHandler: Send + Sync {
    /// Load configuration from custom source
    fn load(&self, source: &str) -> Result<Value>;

    /// Get handler name
    fn name(&self) -> &str;

    /// Check if handler can handle the given source
    fn can_handle(&self, source: &str) -> bool;
}

impl ConfigLoader {
    /// Create a new configuration loader with default settings
    pub fn new() -> Self {
        Self {
            timeout: 30,
            max_retries: 3,
            user_agent: format!("lumosai/{}", env!("CARGO_PKG_VERSION")),
        }
    }

    /// Create loader with custom timeout and retry settings
    pub fn with_settings(timeout: u64, max_retries: u32) -> Self {
        Self {
            timeout,
            max_retries,
            user_agent: format!("lumosai/{}", env!("CARGO_PKG_VERSION")),
        }
    }

    /// Load raw configuration from the specified source
    pub async fn load_raw(&self, source: &ConfigSource) -> Result<Value> {
        match source {
            ConfigSource::File(path) => self.load_from_file(path).await,
            ConfigSource::Environment(prefix) => self.load_from_env(prefix).await,
            ConfigSource::EnvironmentVars(vars) => self.load_from_env_vars(vars).await,
            ConfigSource::Remote {
                url,
                auth_token,
                headers,
            } => {
                self.load_from_remote(url, auth_token.as_deref(), headers)
                    .await
            }
            ConfigSource::Merged { sources, strategy } => {
                self.load_and_merge(sources, strategy).await
            }
            ConfigSource::AutoDetect => self.load_auto_detect().await,
        }
    }

    /// Load configuration from file (auto-detect format)
    async fn load_from_file(&self, path: &str) -> Result<Value> {
        let path = Path::new(path);

        if !path.exists() {
            return Err(Error::Configuration(format!(
                "Configuration file not found: {}",
                path.display()
            )));
        }

        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| Error::Configuration(format!("Failed to read config file: {}", e)))?;

        // Auto-detect format based on file extension
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        match extension.as_deref() {
            Some("yaml") | Some("yml") => serde_yaml::from_str(&content)
                .map_err(|e| Error::Configuration(format!("Failed to parse YAML: {}", e))),
            Some("json") => serde_json::from_str(&content)
                .map_err(|e| Error::Configuration(format!("Failed to parse JSON: {}", e))),
            Some("toml") => {
                let toml_value: toml::Value = toml::from_str(&content)
                    .map_err(|e| Error::Configuration(format!("Failed to parse TOML: {}", e)))?;

                // Convert TOML to JSON value
                self.toml_to_json(toml_value)
            }
            _ => {
                // Try to detect format by parsing
                // Try YAML first
                if let Ok(value) = serde_yaml::from_str::<Value>(&content) {
                    return Ok(value);
                }

                // Try JSON
                if let Ok(value) = serde_json::from_str::<Value>(&content) {
                    return Ok(value);
                }

                // Try TOML
                if let Ok(toml_value) = toml::from_str::<toml::Value>(&content) {
                    return self.toml_to_json(toml_value);
                }

                Err(Error::Configuration(
                    "Failed to auto-detect configuration format".to_string(),
                ))
            }
        }
    }

    /// Load configuration from environment variables with prefix
    async fn load_from_env(&self, prefix: &str) -> Result<Value> {
        let mut config = HashMap::new();
        let prefix_upper = prefix.to_uppercase();
        let prefix_with_underscore = if prefix_upper.ends_with('_') {
            prefix_upper
        } else {
            format!("{}_", prefix_upper)
        };

        for (key, value) in std::env::vars() {
            if key.starts_with(&prefix_with_underscore) {
                let config_key = key[prefix_with_underscore.len()..].to_lowercase();

                // Convert underscore to dot for nested keys
                let config_key = config_key.replace('_', ".");

                // Try to parse as JSON, fallback to string
                let parsed_value = match serde_json::from_str::<Value>(&value) {
                    Ok(v) => v,
                    Err(_) => Value::String(value),
                };

                // Insert into nested structure
                self.insert_nested(&mut config, &config_key, parsed_value);
            }
        }

        Ok(Value::Object(
            config.into_iter().map(|(k, v)| (k, v)).collect(),
        ))
    }

    /// Load configuration from specific environment variables
    async fn load_from_env_vars(&self, vars: &[String]) -> Result<Value> {
        let mut config = HashMap::new();

        for var in vars {
            if let Ok(value) = std::env::var(var) {
                let config_key = var.to_lowercase().replace('_', ".");

                let parsed_value = match serde_json::from_str::<Value>(&value) {
                    Ok(v) => v,
                    Err(_) => Value::String(value),
                };

                self.insert_nested(&mut config, &config_key, parsed_value);
            }
        }

        Ok(Value::Object(
            config.into_iter().map(|(k, v)| (k, v)).collect(),
        ))
    }

    /// Load configuration from remote URL
    async fn load_from_remote(
        &self,
        url: &str,
        auth_token: Option<&str>,
        headers: &HashMap<String, String>,
    ) -> Result<Value> {
        let client = reqwest::Client::builder()
            .user_agent(&self.user_agent)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .build()
            .map_err(|e| Error::Configuration(format!("Failed to create HTTP client: {}", e)))?;

        let mut request = client.get(url);

        // Add auth token if provided
        if let Some(token) = auth_token {
            request = request.bearer_auth(token);
        }

        // Add custom headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        let mut attempts = 0;
        loop {
            match request.try_clone().unwrap().send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let text = response.text().await.map_err(|e| {
                            Error::Configuration(format!("Failed to read response: {}", e))
                        })?;

                        // Try to parse as JSON
                        return serde_json::from_str(&text).map_err(|e| {
                            Error::Configuration(format!("Failed to parse JSON response: {}", e))
                        });
                    } else {
                        return Err(Error::Configuration(format!(
                            "HTTP error {}: {}",
                            response.status(),
                            response.status().canonical_reason().unwrap_or("Unknown")
                        )));
                    }
                }
                Err(e) => {
                    attempts += 1;
                    if attempts >= self.max_retries {
                        return Err(Error::Configuration(format!(
                            "Failed to load remote config after {} attempts: {}",
                            self.max_retries, e
                        )));
                    }

                    // Wait before retrying with exponential backoff
                    let delay = std::time::Duration::from_millis(1000 * 2_u64.pow(attempts - 1));
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    /// Load and merge multiple configuration sources
    fn load_and_merge<'a>(
        &'a self,
        sources: &'a [ConfigSource],
        strategy: &'a MergeStrategy,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value>> + Send + 'a>> {
        Box::pin(async move {
            let mut merged = Value::Null;

            for source in sources {
                let config = self.load_raw(source).await?;
                merged = self.merge_values(merged, config, strategy);
            }

            Ok(merged)
        })
    }

    /// Auto-detect and load configuration
    async fn load_auto_detect(&self) -> Result<Value> {
        // Try environment variables first
        if let Ok(env_config) = self.load_from_env("lumosai").await {
            if !env_config.is_null() {
                return Ok(env_config);
            }
        }

        // Try configuration files
        let config_files = [
            "lumosai.yaml",
            "lumosai.yml",
            "lumosai.json",
            "lumosai.toml",
            ".lumosai.yaml",
            ".lumosai.yml",
            ".lumosai.json",
            ".lumosai.toml",
        ];

        for file in &config_files {
            if Path::new(file).exists() {
                return self.load_from_file(file).await;
            }
        }

        // Try default configuration
        Ok(Value::Object(serde_json::Map::new()))
    }

    /// Merge two configuration values
    fn merge_values(&self, base: Value, update: Value, strategy: &MergeStrategy) -> Value {
        match strategy {
            MergeStrategy::FirstWins => {
                if !base.is_null() {
                    base
                } else {
                    update
                }
            }
            MergeStrategy::LastWins => {
                if !update.is_null() {
                    update
                } else {
                    base
                }
            }
            MergeStrategy::DeepMerge => self.deep_merge(base, update),
        }
    }

    /// Deep merge two configuration values
    fn deep_merge(&self, base: Value, update: Value) -> Value {
        match (base, update) {
            (Value::Object(mut base_map), Value::Object(update_map)) => {
                for (key, value) in update_map {
                    if let Some(base_value) = base_map.get(&key) {
                        base_map.insert(key, self.deep_merge(base_value.clone(), value));
                    } else {
                        base_map.insert(key, value);
                    }
                }
                Value::Object(base_map)
            }
            (Value::Array(mut base_array), Value::Array(update_array)) => {
                base_array.extend(update_array);
                Value::Array(base_array)
            }
            (_, update) => update,
        }
    }

    /// Insert nested value using dot-separated key
    fn insert_nested(&self, map: &mut HashMap<String, Value>, key: &str, value: Value) {
        let parts: Vec<&str> = key.split('.').collect();

        if parts.len() == 1 {
            map.insert(key.to_string(), value);
            return;
        }

        // Build nested structure
        let mut nested_value = value;
        for part in parts.iter().rev().skip(1) {
            let mut inner_map = serde_json::Map::new();
            inner_map.insert(parts[parts.len() - 1].to_string(), nested_value);
            nested_value = Value::Object(inner_map);
        }

        map.insert(parts[0].to_string(), nested_value);
    }

    /// Convert TOML value to JSON value
    fn toml_to_json(&self, toml_value: toml::Value) -> Result<Value> {
        let toml_string = toml::to_string_pretty(&toml_value)
            .map_err(|e| Error::Configuration(format!("Failed to serialize TOML: {}", e)))?;

        serde_yaml::from_str(&toml_string)
            .map_err(|e| Error::Configuration(format!("Failed to convert TOML to JSON: {}", e)))
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_load_from_json_file() {
        let config_content = r#"
        {
            "test": {
                "value": "hello"
            }
        }
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();

        let loader = ConfigLoader::new();
        let result = loader
            .load_from_file(temp_file.path().to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(result["test"]["value"], "hello");
    }

    #[tokio::test]
    async fn test_load_from_yaml_file() {
        let config_content = r#"
        test:
            value: "hello"
        "#;

        let mut temp_file = NamedTempFile::with_suffix(".yaml").unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();

        let loader = ConfigLoader::new();
        let result = loader
            .load_from_file(temp_file.path().to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(result["test"]["value"], "hello");
    }

    #[tokio::test]
    async fn test_load_from_toml_file() {
        let config_content = r#"
        [test]
        value = "hello"
        "#;

        let mut temp_file = NamedTempFile::with_suffix(".toml").unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();

        let loader = ConfigLoader::new();
        let result = loader
            .load_from_file(temp_file.path().to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(result["test"]["value"], "hello");
    }

    #[tokio::test]
    async fn test_load_from_environment() {
        env::set_var("LUMOSAI_TEST_VALUE", "hello");
        env::set_var("LUMOSAI_NESTED_DEEP_VALUE", "world");

        let loader = ConfigLoader::new();
        let result = loader.load_from_env("lumosai").await.unwrap();

        assert_eq!(result["test"]["value"], "hello");
        assert_eq!(result["nested"]["deep"]["value"], "world");

        env::remove_var("LUMOSAI_TEST_VALUE");
        env::remove_var("LUMOSAI_NESTED_DEEP_VALUE");
    }

    #[tokio::test]
    async fn test_merge_strategy_last_wins() {
        let source1 = serde_json::json!({
            "shared": "value1",
            "unique1": "unique"
        });

        let source2 = serde_json::json!({
            "shared": "value2",
            "unique2": "unique"
        });

        let loader = ConfigLoader::new();
        let merged =
            loader.merge_values(source1.clone(), source2.clone(), &MergeStrategy::LastWins);

        assert_eq!(merged["shared"], "value2");
        assert_eq!(merged["unique1"], "unique");
        assert_eq!(merged["unique2"], "unique");

        let merged = loader.merge_values(source1, source2, &MergeStrategy::FirstWins);
        assert_eq!(merged["shared"], "value1");
    }

    #[tokio::test]
    async fn test_deep_merge() {
        let base = serde_json::json!({
            "nested": {
                "shared": "base_value",
                "base_only": "base"
            },
            "array": [1, 2]
        });

        let update = serde_json::json!({
            "nested": {
                "shared": "update_value",
                "update_only": "update"
            },
            "array": [3, 4]
        });

        let loader = ConfigLoader::new();
        let merged = loader.deep_merge(base, update);

        assert_eq!(merged["nested"]["shared"], "update_value");
        assert_eq!(merged["nested"]["base_only"], "base");
        assert_eq!(merged["nested"]["update_only"], "update");

        // Arrays are concatenated
        let array = merged["array"].as_array().unwrap();
        assert_eq!(array.len(), 4);
        assert_eq!(array[0], 1);
        assert_eq!(array[1], 2);
        assert_eq!(array[2], 3);
        assert_eq!(array[3], 4);
    }
}
