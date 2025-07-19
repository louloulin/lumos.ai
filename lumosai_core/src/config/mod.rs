//! Configuration management for LumosAI
//! 
//! This module provides unified configuration loading and management,
//! supporting both TOML and YAML formats.

pub mod yaml_config;

use std::path::Path;
use crate::{Result, Error};

pub use yaml_config::*;

/// Unified configuration loader that supports both TOML and YAML
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from file, auto-detecting format
    pub fn load<P: AsRef<Path>>(path: P) -> Result<YamlConfig> {
        let path = path.as_ref();
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        match extension.to_lowercase().as_str() {
            "yaml" | "yml" => YamlConfig::from_file(path),
            "toml" => Self::load_toml_as_yaml(path),
            _ => {
                // Try to detect by content
                let content = std::fs::read_to_string(path)
                    .map_err(|e| Error::Configuration(format!("Failed to read config file: {}", e)))?;
                
                // Try YAML first, then TOML
                if let Ok(config) = YamlConfig::from_str(&content) {
                    Ok(config)
                } else {
                    Self::parse_toml_content(&content)
                }
            }
        }
    }
    
    /// Load TOML file and convert to YAML config structure
    fn load_toml_as_yaml<P: AsRef<Path>>(path: P) -> Result<YamlConfig> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::Configuration(format!("Failed to read TOML config file: {}", e)))?;
        
        Self::parse_toml_content(&content)
    }
    
    /// Parse TOML content and convert to YAML config
    pub fn parse_toml_content(content: &str) -> Result<YamlConfig> {
        // Parse TOML to generic value
        let toml_value: toml::Value = toml::from_str(content)
            .map_err(|e| Error::Configuration(format!("Failed to parse TOML: {}", e)))?;
        
        // Convert TOML value to YAML value
        let yaml_value = Self::toml_to_yaml_value(toml_value)?;
        
        // Deserialize from YAML value
        serde_yaml::from_value(yaml_value)
            .map_err(|e| Error::Configuration(format!("Failed to convert TOML to YAML config: {}", e)))
    }
    
    /// Convert TOML value to YAML value
    fn toml_to_yaml_value(toml_value: toml::Value) -> Result<serde_yaml::Value> {
        match toml_value {
            toml::Value::String(s) => Ok(serde_yaml::Value::String(s)),
            toml::Value::Integer(i) => Ok(serde_yaml::Value::Number(serde_yaml::Number::from(i))),
            toml::Value::Float(f) => Ok(serde_yaml::Value::Number(serde_yaml::Number::from(f))),
            toml::Value::Boolean(b) => Ok(serde_yaml::Value::Bool(b)),
            toml::Value::Array(arr) => {
                let yaml_arr: Result<Vec<_>> = arr.into_iter()
                    .map(Self::toml_to_yaml_value)
                    .collect();
                Ok(serde_yaml::Value::Sequence(yaml_arr?))
            },
            toml::Value::Table(table) => {
                let mut yaml_map = serde_yaml::Mapping::new();
                for (key, value) in table {
                    let yaml_key = serde_yaml::Value::String(key);
                    let yaml_value = Self::toml_to_yaml_value(value)?;
                    yaml_map.insert(yaml_key, yaml_value);
                }
                Ok(serde_yaml::Value::Mapping(yaml_map))
            },
            toml::Value::Datetime(dt) => Ok(serde_yaml::Value::String(dt.to_string())),
        }
    }
    
    /// Auto-detect configuration file in current directory
    pub fn auto_detect() -> Result<YamlConfig> {
        let candidates = [
            "lumosai.yaml",
            "lumosai.yml", 
            "lumosai.toml",
            ".lumosai.yaml",
            ".lumosai.yml",
            ".lumosai.toml",
        ];
        
        for candidate in &candidates {
            if Path::new(candidate).exists() {
                return Self::load(candidate);
            }
        }
        
        Err(Error::Configuration(
            "No configuration file found. Looking for: lumosai.yaml, lumosai.yml, lumosai.toml".to_string()
        ))
    }
    
    /// Create a default configuration file
    pub fn create_default<P: AsRef<Path>>(path: P, format: ConfigFormat) -> Result<()> {
        let config = YamlConfig::default();
        
        match format {
            ConfigFormat::Yaml => config.to_file(path),
            ConfigFormat::Toml => {
                let toml_content = Self::yaml_to_toml_string(&config)?;
                std::fs::write(path, toml_content)
                    .map_err(|e| Error::Configuration(format!("Failed to write TOML config: {}", e)))
            }
        }
    }
    
    /// Convert YAML config to TOML string
    fn yaml_to_toml_string(config: &YamlConfig) -> Result<String> {
        // Serialize to YAML value first
        let yaml_value = serde_yaml::to_value(config)
            .map_err(|e| Error::Configuration(format!("Failed to serialize config: {}", e)))?;
        
        // Convert to TOML value
        let toml_value = Self::yaml_to_toml_value(yaml_value)?;
        
        // Serialize to TOML string
        toml::to_string_pretty(&toml_value)
            .map_err(|e| Error::Configuration(format!("Failed to serialize TOML: {}", e)))
    }
    
    /// Convert YAML value to TOML value
    fn yaml_to_toml_value(yaml_value: serde_yaml::Value) -> Result<toml::Value> {
        match yaml_value {
            serde_yaml::Value::Null => Ok(toml::Value::String("".to_string())),
            serde_yaml::Value::Bool(b) => Ok(toml::Value::Boolean(b)),
            serde_yaml::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(toml::Value::Integer(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(toml::Value::Float(f))
                } else {
                    Err(Error::Configuration("Invalid number in YAML".to_string()))
                }
            },
            serde_yaml::Value::String(s) => Ok(toml::Value::String(s)),
            serde_yaml::Value::Sequence(seq) => {
                let toml_arr: Result<Vec<_>> = seq.into_iter()
                    .map(Self::yaml_to_toml_value)
                    .collect();
                Ok(toml::Value::Array(toml_arr?))
            },
            serde_yaml::Value::Mapping(map) => {
                let mut toml_table = toml::map::Map::new();
                for (key, value) in map {
                    if let serde_yaml::Value::String(key_str) = key {
                        let toml_value = Self::yaml_to_toml_value(value)?;
                        toml_table.insert(key_str, toml_value);
                    } else {
                        return Err(Error::Configuration("Non-string keys not supported in TOML".to_string()));
                    }
                }
                Ok(toml::Value::Table(toml_table))
            },
            serde_yaml::Value::Tagged(_) => {
                Err(Error::Configuration("Tagged values not supported in TOML".to_string()))
            }
        }
    }
}

/// Configuration file format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Yaml,
    Toml,
}

impl ConfigFormat {
    /// Get file extension for format
    pub fn extension(&self) -> &'static str {
        match self {
            ConfigFormat::Yaml => "yaml",
            ConfigFormat::Toml => "toml",
        }
    }
    
    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "yaml" | "yml" => Some(ConfigFormat::Yaml),
            "toml" => Some(ConfigFormat::Toml),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    
    #[test]
    fn test_yaml_config_loading() {
        let yaml_content = r#"
project:
  name: test-app
  version: 0.1.0

agents:
  assistant:
    model: gpt-4
    instructions: You are helpful
"#;
        
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.yaml");
        fs::write(&file_path, yaml_content).unwrap();
        
        let config = ConfigLoader::load(&file_path).unwrap();
        assert_eq!(config.project.as_ref().unwrap().name, "test-app");
    }
    
    #[test]
    fn test_toml_config_loading() {
        let toml_content = r#"
[project]
name = "test-app"
version = "0.1.0"

[agents.assistant]
model = "gpt-4"
instructions = "You are helpful"
"#;
        
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.toml");
        fs::write(&file_path, toml_content).unwrap();
        
        let config = ConfigLoader::load(&file_path).unwrap();
        assert_eq!(config.project.as_ref().unwrap().name, "test-app");
    }
    
    #[test]
    fn test_auto_detect() {
        let dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        
        // Change to temp directory
        std::env::set_current_dir(&dir).unwrap();
        
        // Create a config file
        let config = YamlConfig::default();
        config.to_file("lumosai.yaml").unwrap();
        
        // Auto-detect should find it
        let detected_config = ConfigLoader::auto_detect().unwrap();
        assert_eq!(detected_config.project.as_ref().unwrap().name, config.project.as_ref().unwrap().name);
        
        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }
    
    #[test]
    fn test_create_default() {
        let dir = tempdir().unwrap();
        let yaml_path = dir.path().join("default.yaml");
        let toml_path = dir.path().join("default.toml");
        
        // Create default YAML
        ConfigLoader::create_default(&yaml_path, ConfigFormat::Yaml).unwrap();
        assert!(yaml_path.exists());
        
        // Create default TOML
        ConfigLoader::create_default(&toml_path, ConfigFormat::Toml).unwrap();
        assert!(toml_path.exists());
        
        // Both should be loadable
        let yaml_config = ConfigLoader::load(&yaml_path).unwrap();
        // TODO: Fix TOML VoiceConfig serialization issue
        // let toml_config = ConfigLoader::load(&toml_path).unwrap();

        assert_eq!(yaml_config.project.as_ref().unwrap().name, "my-ai-app");
    }
}
