//! Enhanced CLI tools for Lumos.ai development
//! 
//! This module provides comprehensive command-line tools for:
//! - Project creation and scaffolding
//! - Development server with hot reload
//! - Tool management and marketplace integration
//! - Deployment and production management
//! - Debugging and performance monitoring

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};
use clap::{Parser, Subcommand};
use tokio::process::Command;
use crate::Result;

pub mod commands;
pub mod templates;
pub mod dev_server;
pub mod deployment;

/// Main CLI application
#[derive(Parser)]
#[command(name = "lumos")]
#[command(about = "Lumos.ai CLI - Build powerful AI agents with ease")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands
#[derive(Subcommand)]
pub enum Commands {
    /// Create a new Lumos.ai project
    New {
        /// Project name
        name: String,
        /// Template to use
        #[arg(short, long, default_value = "basic")]
        template: String,
        /// Target directory
        #[arg(short, long)]
        directory: Option<PathBuf>,
    },
    /// Start development server
    Dev {
        /// Enable hot reload
        #[arg(long, default_value = "true")]
        hot_reload: bool,
        /// Enable debug mode
        #[arg(long)]
        debug: bool,
        /// Port to run on
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
    /// Tool management commands
    Tools {
        #[command(subcommand)]
        action: ToolCommands,
    },
    /// Build project for production
    Build {
        /// Build target
        #[arg(short, long, default_value = "release")]
        target: String,
        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Deploy to various platforms
    Deploy {
        /// Deployment platform
        #[arg(short, long, default_value = "local")]
        platform: String,
        /// Configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    /// Run tests
    Test {
        /// Test pattern
        #[arg(short, long)]
        pattern: Option<String>,
        /// Enable coverage
        #[arg(long)]
        coverage: bool,
    },
    /// Generate documentation
    Docs {
        /// Output format
        #[arg(short, long, default_value = "html")]
        format: String,
        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

/// Tool management subcommands
#[derive(Subcommand)]
pub enum ToolCommands {
    /// List available tools
    List {
        /// Show only available tools
        #[arg(long)]
        available: bool,
        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,
    },
    /// Add a tool to the project
    Add {
        /// Tool name or package
        name: String,
        /// Specific version
        #[arg(short, long)]
        version: Option<String>,
    },
    /// Remove a tool from the project
    Remove {
        /// Tool name
        name: String,
    },
    /// Update tools
    Update {
        /// Update all tools
        #[arg(long)]
        all: bool,
        /// Specific tool to update
        tool: Option<String>,
    },
    /// Search for tools
    Search {
        /// Search query
        query: String,
        /// Limit results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
}

/// Project configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub license: Option<String>,
    pub dependencies: HashMap<String, String>,
    pub dev_dependencies: HashMap<String, String>,
    pub tools: Vec<ToolConfig>,
    pub build: BuildConfig,
    pub deployment: DeploymentConfig,
}

/// Tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub name: String,
    pub version: String,
    pub source: ToolSource,
    pub config: HashMap<String, serde_json::Value>,
}

/// Tool source
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolSource {
    #[serde(rename = "marketplace")]
    Marketplace { package: String },
    #[serde(rename = "git")]
    Git { url: String, branch: Option<String> },
    #[serde(rename = "local")]
    Local { path: PathBuf },
    #[serde(rename = "mcp")]
    MCP { server: String },
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub target: String,
    pub optimization: String,
    pub features: Vec<String>,
    pub exclude: Vec<String>,
}

/// Deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub platforms: HashMap<String, PlatformConfig>,
    pub environment: HashMap<String, String>,
}

/// Platform-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub provider: String,
    pub region: Option<String>,
    pub instance_type: Option<String>,
    pub scaling: Option<ScalingConfig>,
    pub environment: HashMap<String, String>,
}

/// Scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingConfig {
    pub min_instances: u32,
    pub max_instances: u32,
    pub target_cpu: f32,
    pub target_memory: f32,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: "my-lumos-project".to_string(),
            version: "0.1.0".to_string(),
            description: None,
            authors: vec![],
            license: Some("MIT".to_string()),
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            tools: vec![],
            build: BuildConfig {
                target: "release".to_string(),
                optimization: "speed".to_string(),
                features: vec![],
                exclude: vec![],
            },
            deployment: DeploymentConfig {
                platforms: HashMap::new(),
                environment: HashMap::new(),
            },
        }
    }
}

/// CLI utilities
pub struct CliUtils;

impl CliUtils {
    /// Load project configuration
    pub fn load_config<P: AsRef<Path>>(path: P) -> Result<ProjectConfig> {
        let content = fs::read_to_string(path)?;
        let config: ProjectConfig = toml::from_str(&content)
            .map_err(|e| crate::Error::Other(format!("Failed to parse config: {}", e)))?;
        Ok(config)
    }

    /// Save project configuration
    pub fn save_config<P: AsRef<Path>>(path: P, config: &ProjectConfig) -> Result<()> {
        let content = toml::to_string_pretty(config)
            .map_err(|e| crate::Error::Other(format!("Failed to serialize config: {}", e)))?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Check if we're in a Lumos project
    pub fn is_lumos_project<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().join("lumos.toml").exists() || 
        path.as_ref().join("Lumos.toml").exists()
    }

    /// Find project root
    pub fn find_project_root<P: AsRef<Path>>(start: P) -> Option<PathBuf> {
        let mut current = start.as_ref().to_path_buf();
        
        loop {
            if Self::is_lumos_project(&current) {
                return Some(current);
            }
            
            if !current.pop() {
                break;
            }
        }
        
        None
    }

    /// Execute command with output
    pub async fn execute_command(cmd: &str, args: &[&str], cwd: Option<&Path>) -> Result<String> {
        let mut command = Command::new(cmd);
        command.args(args);
        
        if let Some(dir) = cwd {
            command.current_dir(dir);
        }
        
        let output = command.output().await?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(crate::Error::Other(format!("Command failed: {}", error)))
        }
    }

    /// Print success message
    pub fn success(message: &str) {
        println!("✅ {}", message);
    }

    /// Print error message
    pub fn error(message: &str) {
        eprintln!("❌ {}", message);
    }

    /// Print warning message
    pub fn warning(message: &str) {
        println!("⚠️  {}", message);
    }

    /// Print info message
    pub fn info(message: &str) {
        println!("ℹ️  {}", message);
    }

    /// Print progress message
    pub fn progress(message: &str) {
        println!("🔄 {}", message);
    }
}
