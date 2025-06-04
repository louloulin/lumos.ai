//! Tool marketplace infrastructure inspired by Mastra's ecosystem
//! 
//! This module provides tool discovery, installation, and management capabilities

// pub mod registry;
// pub mod installer;
// pub mod validator;

// use crate::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

/// Tool package metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPackage {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<ToolCategory>,
    pub dependencies: HashMap<String, String>,
    pub lumos_version: String,
    pub manifest: ToolManifest,
    pub metadata: HashMap<String, Value>,
}

/// Tool categories for organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolCategory {
    #[serde(rename = "web")]
    Web,
    #[serde(rename = "file")]
    File,
    #[serde(rename = "data")]
    Data,
    #[serde(rename = "ai")]
    AI,
    #[serde(rename = "system")]
    System,
    #[serde(rename = "math")]
    Math,
    #[serde(rename = "crypto")]
    Crypto,
    #[serde(rename = "database")]
    Database,
    #[serde(rename = "api")]
    API,
    #[serde(rename = "utility")]
    Utility,
    #[serde(rename = "custom")]
    Custom,
}

impl ToolCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ToolCategory::Web => "web",
            ToolCategory::File => "file",
            ToolCategory::Data => "data",
            ToolCategory::AI => "ai",
            ToolCategory::System => "system",
            ToolCategory::Math => "math",
            ToolCategory::Crypto => "crypto",
            ToolCategory::Database => "database",
            ToolCategory::API => "api",
            ToolCategory::Utility => "utility",
            ToolCategory::Custom => "custom",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            ToolCategory::Web => "🌐",
            ToolCategory::File => "📁",
            ToolCategory::Data => "📊",
            ToolCategory::AI => "🤖",
            ToolCategory::System => "⚙️",
            ToolCategory::Math => "🔢",
            ToolCategory::Crypto => "🔐",
            ToolCategory::Database => "🗄️",
            ToolCategory::API => "🔌",
            ToolCategory::Utility => "🛠️",
            ToolCategory::Custom => "🎨",
        }
    }
}

/// Tool manifest describing the tool's interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManifest {
    pub tools: Vec<ToolDefinition>,
    pub entry_point: String,
    pub exports: Vec<String>,
    pub permissions: Vec<Permission>,
    pub config_schema: Option<Value>,
}

/// Individual tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ParameterDefinition>,
    pub returns: ReturnDefinition,
    pub examples: Vec<ToolExample>,
    pub tags: Vec<String>,
}

/// Parameter definition for tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDefinition {
    pub name: String,
    pub description: String,
    pub r#type: String,
    pub required: bool,
    pub default: Option<Value>,
    pub validation: Option<ValidationRule>,
}

/// Return type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnDefinition {
    pub r#type: String,
    pub description: String,
    pub schema: Option<Value>,
}

/// Tool usage example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExample {
    pub title: String,
    pub description: String,
    pub input: Value,
    pub output: Value,
}

/// Validation rules for parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub allowed_values: Option<Vec<Value>>,
}

/// Permissions required by tools
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    #[serde(rename = "network")]
    Network,
    #[serde(rename = "filesystem")]
    FileSystem,
    #[serde(rename = "environment")]
    Environment,
    #[serde(rename = "process")]
    Process,
    #[serde(rename = "system")]
    System,
}

/// Tool marketplace client
pub struct Marketplace {
    registry_url: String,
    cache_dir: PathBuf,
    installed_tools: HashMap<String, ToolPackage>,
}

impl Marketplace {
    /// Create a new marketplace client
    pub fn new(registry_url: String, cache_dir: PathBuf) -> Self {
        Self {
            registry_url,
            cache_dir,
            installed_tools: HashMap::new(),
        }
    }

    /// Get the registry URL
    pub fn get_registry_url(&self) -> &str {
        &self.registry_url
    }

    /// Get the cache directory
    pub fn get_cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }

    /// Get installed tools count
    pub fn get_installed_count(&self) -> usize {
        self.installed_tools.len()
    }

    /// Search for tools
    pub async fn search(&self, query: &str, category: Option<ToolCategory>) -> crate::Result<Vec<ToolPackage>> {
        // Mock implementation - in real version would query registry
        let mut results = Vec::new();
        
        // Example search results
        if query.contains("web") || category == Some(ToolCategory::Web) {
            results.push(ToolPackage {
                name: "web-scraper-pro".to_string(),
                version: "1.2.0".to_string(),
                description: "Advanced web scraping tool with JavaScript support".to_string(),
                author: "WebTools Inc".to_string(),
                license: "MIT".to_string(),
                homepage: Some("https://github.com/webtools/scraper-pro".to_string()),
                repository: Some("https://github.com/webtools/scraper-pro".to_string()),
                keywords: vec!["web".to_string(), "scraping".to_string(), "html".to_string()],
                categories: vec![ToolCategory::Web],
                dependencies: HashMap::new(),
                lumos_version: ">=0.1.0".to_string(),
                manifest: ToolManifest {
                    tools: vec![ToolDefinition {
                        name: "scrape_page".to_string(),
                        description: "Scrape content from web pages".to_string(),
                        parameters: vec![
                            ParameterDefinition {
                                name: "url".to_string(),
                                description: "URL to scrape".to_string(),
                                r#type: "string".to_string(),
                                required: true,
                                default: None,
                                validation: Some(ValidationRule {
                                    pattern: Some(r"^https?://".to_string()),
                                    min_length: Some(10),
                                    max_length: Some(2048),
                                    min_value: None,
                                    max_value: None,
                                    allowed_values: None,
                                }),
                            }
                        ],
                        returns: ReturnDefinition {
                            r#type: "object".to_string(),
                            description: "Scraped content and metadata".to_string(),
                            schema: None,
                        },
                        examples: vec![ToolExample {
                            title: "Scrape a news article".to_string(),
                            description: "Extract content from a news website".to_string(),
                            input: serde_json::json!({"url": "https://example.com/article"}),
                            output: serde_json::json!({"title": "Article Title", "content": "..."}),
                        }],
                        tags: vec!["web".to_string(), "scraping".to_string()],
                    }],
                    entry_point: "lib.rs".to_string(),
                    exports: vec!["scrape_page".to_string()],
                    permissions: vec![Permission::Network],
                    config_schema: None,
                },
                metadata: HashMap::new(),
            });
        }
        
        Ok(results)
    }

    /// Install a tool package
    pub async fn install(&mut self, package_name: &str, version: Option<&str>) -> crate::Result<()> {
        // Mock installation
        let version = version.unwrap_or("latest");
        
        // In real implementation, would download and install the package
        println!("📦 Installing {} v{}", package_name, version);
        println!("✅ Successfully installed {}", package_name);
        
        Ok(())
    }

    /// Uninstall a tool package
    pub async fn uninstall(&mut self, package_name: &str) -> crate::Result<()> {
        if self.installed_tools.remove(package_name).is_some() {
            println!("🗑️  Uninstalled {}", package_name);
            Ok(())
        } else {
            Err(crate::Error::Tool(format!("Package '{}' is not installed", package_name)))
        }
    }

    /// List installed tools
    pub fn list_installed(&self) -> Vec<&ToolPackage> {
        self.installed_tools.values().collect()
    }

    /// Get tool package info
    pub async fn get_package_info(&self, package_name: &str) -> crate::Result<ToolPackage> {
        // Mock package info retrieval
        if package_name == "web-scraper-pro" {
            let search_results = self.search("web", Some(ToolCategory::Web)).await?;
            if let Some(package) = search_results.into_iter().next() {
                return Ok(package);
            }
        }
        
        Err(crate::Error::Tool(format!("Package '{}' not found", package_name)))
    }

    /// Validate tool package
    pub fn validate_package(&self, package: &ToolPackage) -> crate::Result<ValidationReport> {
        let mut report = ValidationReport::new();
        
        // Validate package metadata
        if package.name.is_empty() {
            report.add_error("Package name cannot be empty");
        }
        
        if package.version.is_empty() {
            report.add_error("Package version cannot be empty");
        }
        
        if package.description.is_empty() {
            report.add_warning("Package description is empty");
        }
        
        // Validate tools
        for tool in &package.manifest.tools {
            if tool.name.is_empty() {
                report.add_error(&format!("Tool name cannot be empty"));
            }
            
            if tool.description.is_empty() {
                report.add_warning(&format!("Tool '{}' has no description", tool.name));
            }
            
            // Validate parameters
            for param in &tool.parameters {
                if param.name.is_empty() {
                    report.add_error(&format!("Parameter name cannot be empty in tool '{}'", tool.name));
                }
            }
        }
        
        Ok(report)
    }

    /// Update package cache
    pub async fn update_cache(&self) -> crate::Result<()> {
        println!("🔄 Updating package cache...");
        // Mock cache update
        println!("✅ Package cache updated");
        Ok(())
    }
}

/// Validation report for tool packages
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub is_valid: bool,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            is_valid: true,
        }
    }

    pub fn add_error(&mut self, error: &str) {
        self.errors.push(error.to_string());
        self.is_valid = false;
    }

    pub fn add_warning(&mut self, warning: &str) {
        self.warnings.push(warning.to_string());
    }

    pub fn format_report(&self) -> String {
        let mut output = String::new();
        
        if self.is_valid {
            output.push_str("✅ Package validation passed\n");
        } else {
            output.push_str("❌ Package validation failed\n");
        }
        
        if !self.errors.is_empty() {
            output.push_str("\n🚨 Errors:\n");
            for error in &self.errors {
                output.push_str(&format!("  • {}\n", error));
            }
        }
        
        if !self.warnings.is_empty() {
            output.push_str("\n⚠️  Warnings:\n");
            for warning in &self.warnings {
                output.push_str(&format!("  • {}\n", warning));
            }
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_marketplace_creation() {
        let cache_dir = env::temp_dir().join("lumos_test_cache");
        let marketplace = Marketplace::new(
            "https://registry.lumos.ai".to_string(),
            cache_dir
        );
        
        assert_eq!(marketplace.registry_url, "https://registry.lumos.ai");
        assert_eq!(marketplace.installed_tools.len(), 0);
    }

    #[tokio::test]
    async fn test_tool_search() {
        let cache_dir = env::temp_dir().join("lumos_test_cache");
        let marketplace = Marketplace::new(
            "https://registry.lumos.ai".to_string(),
            cache_dir
        );
        
        let results = marketplace.search("web", Some(ToolCategory::Web)).await.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "web-scraper-pro");
    }

    #[tokio::test]
    async fn test_package_validation() {
        let cache_dir = env::temp_dir().join("lumos_test_cache");
        let marketplace = Marketplace::new(
            "https://registry.lumos.ai".to_string(),
            cache_dir
        );
        
        let package = ToolPackage {
            name: "test-tool".to_string(),
            version: "1.0.0".to_string(),
            description: "Test tool".to_string(),
            author: "Test Author".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            repository: None,
            keywords: vec![],
            categories: vec![ToolCategory::Utility],
            dependencies: HashMap::new(),
            lumos_version: ">=0.1.0".to_string(),
            manifest: ToolManifest {
                tools: vec![],
                entry_point: "lib.rs".to_string(),
                exports: vec![],
                permissions: vec![],
                config_schema: None,
            },
            metadata: HashMap::new(),
        };
        
        let report = marketplace.validate_package(&package).unwrap();
        assert!(report.is_valid);
    }

    #[test]
    fn test_tool_categories() {
        assert_eq!(ToolCategory::Web.as_str(), "web");
        assert_eq!(ToolCategory::Web.emoji(), "🌐");
        assert_eq!(ToolCategory::AI.emoji(), "🤖");
    }

    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new();
        assert!(report.is_valid);
        
        report.add_warning("Test warning");
        assert!(report.is_valid);
        
        report.add_error("Test error");
        assert!(!report.is_valid);
        
        let formatted = report.format_report();
        assert!(formatted.contains("❌"));
        assert!(formatted.contains("Test error"));
        assert!(formatted.contains("Test warning"));
    }
}
