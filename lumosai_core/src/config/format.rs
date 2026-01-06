//! Configuration format utilities

/// Supported configuration serialization formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Yaml,
    Toml,
    Json,
}

impl ConfigFormat {
    /// File extension associated with the format
    pub fn extension(self) -> &'static str {
        match self {
            ConfigFormat::Yaml => "yaml",
            ConfigFormat::Toml => "toml",
            ConfigFormat::Json => "json",
        }
    }

    /// Detect format from file extension
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_lowercase().as_str() {
            "yaml" | "yml" => Some(ConfigFormat::Yaml),
            "toml" => Some(ConfigFormat::Toml),
            "json" => Some(ConfigFormat::Json),
            _ => None,
        }
    }
}
