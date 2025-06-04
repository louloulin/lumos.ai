//! CLI command implementations
//! 
//! This module contains the actual implementation of all CLI commands

use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use crate::Result;
use super::{ProjectConfig, ToolConfig, ToolSource, CliUtils};
use crate::marketplace::{Marketplace, ToolCategory};

/// Command implementations
pub struct Commands;

impl Commands {
    /// Create a new Lumos.ai project
    pub async fn new_project(name: &str, template: &str, directory: Option<PathBuf>) -> Result<()> {
        let target_dir = directory.unwrap_or_else(|| PathBuf::from(name));
        
        CliUtils::progress(&format!("Creating new Lumos.ai project '{}'...", name));
        
        // Create project directory
        fs::create_dir_all(&target_dir)?;
        
        // Generate project structure based on template
        Self::create_project_structure(&target_dir, name, template).await?;
        
        // Initialize configuration
        let mut config = ProjectConfig::default();
        config.name = name.to_string();
        
        // Add default tools based on template
        Self::add_template_tools(&mut config, template).await?;
        
        // Save configuration
        let config_path = target_dir.join("lumos.toml");
        CliUtils::save_config(&config_path, &config)?;
        
        CliUtils::success(&format!("Project '{}' created successfully!", name));
        CliUtils::info(&format!("Next steps:"));
        CliUtils::info(&format!("  cd {}", name));
        CliUtils::info(&format!("  lumos dev"));
        
        Ok(())
    }

    /// Start development server
    pub async fn dev_server(hot_reload: bool, debug: bool, port: u16) -> Result<()> {
        // Check if we're in a Lumos project
        let project_root = CliUtils::find_project_root(".")
            .ok_or_else(|| crate::Error::Other("Not in a Lumos.ai project directory".to_string()))?;
        
        CliUtils::progress("Starting development server...");
        
        // Load project configuration
        let config_path = project_root.join("lumos.toml");
        let config = CliUtils::load_config(&config_path)?;
        
        CliUtils::info(&format!("Project: {} v{}", config.name, config.version));
        CliUtils::info(&format!("Server: http://localhost:{}", port));
        
        if hot_reload {
            CliUtils::info("Hot reload: enabled");
        }
        
        if debug {
            CliUtils::info("Debug mode: enabled");
        }
        
        // Start the development server
        super::dev_server::start_dev_server(port, hot_reload, debug, &project_root).await?;
        
        Ok(())
    }

    /// List available tools
    pub async fn list_tools(available: bool, category: Option<String>) -> Result<()> {
        if available {
            CliUtils::progress("Fetching available tools from marketplace...");
            
            // Connect to marketplace
            let marketplace = Marketplace::new(
                "https://marketplace.lumos.ai".to_string(),
                PathBuf::from(".lumos/cache")
            );
            
            let category_filter = category.as_ref().and_then(|c| match c.as_str() {
                "web" => Some(ToolCategory::Network),
                "file" => Some(ToolCategory::FileSystem),
                "data" => Some(ToolCategory::DataProcessing),
                "system" => Some(ToolCategory::Network), // Use Network as fallback
                "math" => Some(ToolCategory::DataProcessing), // Use DataProcessing as fallback
                _ => None,
            });
            
            let tools = marketplace.search("", category_filter).await?;
            
            println!("\n📦 Available Tools:");
            println!("┌─────────────────────────────────────────────────────────────────┐");
            
            for tool in tools {
                // Use a default icon since ToolPackage doesn't have category field
                let category_icon = "🔧";

                println!("│ {} {:<20} │ {:<35} │", category_icon, tool.name, tool.description);
            }
            
            println!("└─────────────────────────────────────────────────────────────────┘");
        } else {
            // List installed tools
            let project_root = CliUtils::find_project_root(".")
                .ok_or_else(|| crate::Error::Other("Not in a Lumos.ai project directory".to_string()))?;
            
            let config_path = project_root.join("lumos.toml");
            let config = CliUtils::load_config(&config_path)?;
            
            println!("\n🔧 Installed Tools:");
            
            if config.tools.is_empty() {
                CliUtils::info("No tools installed. Use 'lumos tools add <tool>' to add tools.");
            } else {
                println!("┌─────────────────────────────────────────────────────────────────┐");
                
                for tool in &config.tools {
                    let source_info = match &tool.source {
                        ToolSource::Marketplace { package } => format!("marketplace:{}", package),
                        ToolSource::Git { url, .. } => format!("git:{}", url),
                        ToolSource::Local { path } => format!("local:{}", path.display()),
                        ToolSource::MCP { server } => format!("mcp:{}", server),
                    };
                    
                    println!("│ 🔧 {:<20} │ v{:<10} │ {:<20} │", tool.name, tool.version, source_info);
                }
                
                println!("└─────────────────────────────────────────────────────────────────┘");
            }
        }
        
        Ok(())
    }

    /// Add a tool to the project
    pub async fn add_tool(name: &str, version: Option<String>) -> Result<()> {
        let project_root = CliUtils::find_project_root(".")
            .ok_or_else(|| crate::Error::Other("Not in a Lumos.ai project directory".to_string()))?;
        
        CliUtils::progress(&format!("Adding tool '{}'...", name));
        
        // Load current configuration
        let config_path = project_root.join("lumos.toml");
        let mut config = CliUtils::load_config(&config_path)?;
        
        // Check if tool already exists
        if config.tools.iter().any(|t| t.name == name) {
            CliUtils::warning(&format!("Tool '{}' is already installed", name));
            return Ok(());
        }
        
        // Connect to marketplace to get tool info
        let marketplace = Marketplace::new(
            "https://marketplace.lumos.ai".to_string(),
            PathBuf::from(".lumos/cache")
        );
        
        let tools = marketplace.search(name, None).await?;
        let tool_info = tools.iter().find(|t| t.name == name)
            .ok_or_else(|| crate::Error::Other(format!("Tool '{}' not found in marketplace", name)))?;
        
        // Install the tool
        marketplace.install(&tool_info.name, version.as_deref()).await?;
        
        // Add to configuration
        let tool_config = ToolConfig {
            name: name.to_string(),
            version: version.unwrap_or_else(|| tool_info.version.clone()),
            source: ToolSource::Marketplace { 
                package: tool_info.name.clone() 
            },
            config: HashMap::new(),
        };
        
        config.tools.push(tool_config);
        
        // Save configuration
        CliUtils::save_config(&config_path, &config)?;
        
        CliUtils::success(&format!("Tool '{}' added successfully!", name));
        
        Ok(())
    }

    /// Remove a tool from the project
    pub async fn remove_tool(name: &str) -> Result<()> {
        let project_root = CliUtils::find_project_root(".")
            .ok_or_else(|| crate::Error::Other("Not in a Lumos.ai project directory".to_string()))?;
        
        CliUtils::progress(&format!("Removing tool '{}'...", name));
        
        // Load current configuration
        let config_path = project_root.join("lumos.toml");
        let mut config = CliUtils::load_config(&config_path)?;
        
        // Find and remove the tool
        let initial_len = config.tools.len();
        config.tools.retain(|t| t.name != name);
        
        if config.tools.len() == initial_len {
            CliUtils::warning(&format!("Tool '{}' is not installed", name));
            return Ok(());
        }
        
        // Save configuration
        CliUtils::save_config(&config_path, &config)?;
        
        CliUtils::success(&format!("Tool '{}' removed successfully!", name));
        
        Ok(())
    }

    /// Search for tools
    pub async fn search_tools(query: &str, limit: usize) -> Result<()> {
        CliUtils::progress(&format!("Searching for tools matching '{}'...", query));
        
        let marketplace = Marketplace::new(
            "https://marketplace.lumos.ai".to_string(),
            PathBuf::from(".lumos/cache")
        );
        
        let tools = marketplace.search(query, None).await?;
        let limited_tools: Vec<_> = tools.into_iter().take(limit).collect();
        
        if limited_tools.is_empty() {
            CliUtils::info(&format!("No tools found matching '{}'", query));
            return Ok(());
        }
        
        println!("\n🔍 Search Results for '{}':", query);
        println!("┌─────────────────────────────────────────────────────────────────┐");
        
        for tool in limited_tools {
            // Use a default icon since ToolPackage doesn't have category field
            let category_icon = "🔧";

            println!("│ {} {:<20} │ v{:<8} │ {:<25} │",
                category_icon, tool.name, tool.version,
                tool.description.chars().take(25).collect::<String>()
            );
        }
        
        println!("└─────────────────────────────────────────────────────────────────┘");
        println!("\nUse 'lumos tools add <tool-name>' to install a tool.");
        
        Ok(())
    }

    /// Build project for production
    pub async fn build_project(target: &str, output: Option<PathBuf>) -> Result<()> {
        let project_root = CliUtils::find_project_root(".")
            .ok_or_else(|| crate::Error::Other("Not in a Lumos.ai project directory".to_string()))?;
        
        CliUtils::progress(&format!("Building project for {}...", target));
        
        // Load configuration
        let config_path = project_root.join("lumos.toml");
        let config = CliUtils::load_config(&config_path)?;
        
        // Determine output directory
        let output_dir = output.unwrap_or_else(|| project_root.join("dist"));
        fs::create_dir_all(&output_dir)?;
        
        // Build based on target
        match target {
            "release" => {
                CliUtils::execute_command("cargo", &["build", "--release"], Some(&project_root)).await?;
            }
            "wasm" => {
                CliUtils::execute_command("cargo", &["build", "--target", "wasm32-unknown-unknown", "--release"], Some(&project_root)).await?;
            }
            _ => {
                return Err(crate::Error::Other(format!("Unknown build target: {}", target)));
            }
        }
        
        CliUtils::success(&format!("Project built successfully for {}", target));
        CliUtils::info(&format!("Output: {}", output_dir.display()));
        
        Ok(())
    }

    /// Create project structure based on template
    async fn create_project_structure(dir: &Path, name: &str, template: &str) -> Result<()> {
        match template {
            "basic" => Self::create_basic_template(dir, name).await,
            "web-agent" => Self::create_web_agent_template(dir, name).await,
            "data-agent" => Self::create_data_agent_template(dir, name).await,
            "chat-bot" => Self::create_chatbot_template(dir, name).await,
            _ => Err(crate::Error::Other(format!("Unknown template: {}", template))),
        }
    }

    /// Create basic project template
    async fn create_basic_template(dir: &Path, name: &str) -> Result<()> {
        // Create directory structure
        fs::create_dir_all(dir.join("src"))?;
        fs::create_dir_all(dir.join("tests"))?;
        fs::create_dir_all(dir.join("examples"))?;
        
        // Create main.rs
        let main_rs = format!(r#"//! {} - A Lumos.ai Agent
//! 
//! This is a basic Lumos.ai agent project.

use lumosai::{{agent, tools, Result}};

#[tokio::main]
async fn main() -> Result<()> {{
    // Create a simple agent
    let agent = agent!{{
        name: "{}",
        instructions: "You are a helpful assistant.",
        model: "gpt-4",
        tools: []
    }};

    // Example interaction
    let response = agent.generate("Hello! How can you help me?").await?;
    println!("Agent: {{}}", response);

    Ok(())
}}
"#, name, name);
        
        fs::write(dir.join("src/main.rs"), main_rs)?;
        
        // Create Cargo.toml
        let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
lumosai = {{ path = "../lumosai_core" }}
tokio = {{ version = "1.0", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
"#, name);
        
        fs::write(dir.join("Cargo.toml"), cargo_toml)?;
        
        Ok(())
    }

    /// Create web agent template
    async fn create_web_agent_template(dir: &Path, name: &str) -> Result<()> {
        Self::create_basic_template(dir, name).await?;
        
        // Override main.rs with web-specific content
        let main_rs = format!(r#"//! {} - A Web-Enabled Lumos.ai Agent

use lumosai::{{agent, tools, Result}};

#[tokio::main]
async fn main() -> Result<()> {{
    // Create a web-enabled agent
    let agent = agent!{{
        name: "{}",
        instructions: "You are a web-savvy assistant that can search and fetch web content.",
        model: "gpt-4",
        tools: ["web_search", "http_request", "url_extract"]
    }};

    // Example web interaction
    let response = agent.generate("Search for the latest news about AI").await?;
    println!("Agent: {{}}", response);

    Ok(())
}}
"#, name, name);
        
        fs::write(dir.join("src/main.rs"), main_rs)?;
        
        Ok(())
    }

    /// Create data agent template
    async fn create_data_agent_template(dir: &Path, name: &str) -> Result<()> {
        Self::create_basic_template(dir, name).await?;
        
        // Override main.rs with data-specific content
        let main_rs = format!(r#"//! {} - A Data Processing Lumos.ai Agent

use lumosai::{{agent, tools, Result}};

#[tokio::main]
async fn main() -> Result<()> {{
    // Create a data processing agent
    let agent = agent!{{
        name: "{}",
        instructions: "You are a data analyst that can process and analyze various data formats.",
        model: "gpt-4",
        tools: ["csv_reader", "json_parser", "data_analyzer", "chart_generator"]
    }};

    // Example data interaction
    let response = agent.generate("Analyze the sales data in data/sales.csv").await?;
    println!("Agent: {{}}", response);

    Ok(())
}}
"#, name, name);
        
        fs::write(dir.join("src/main.rs"), main_rs)?;
        
        // Create sample data directory
        fs::create_dir_all(dir.join("data"))?;
        
        Ok(())
    }

    /// Create chatbot template
    async fn create_chatbot_template(dir: &Path, name: &str) -> Result<()> {
        Self::create_basic_template(dir, name).await?;
        
        // Override main.rs with chatbot-specific content
        let main_rs = format!(r#"//! {} - An Interactive Chatbot

use lumosai::{{agent, tools, Result}};
use std::io::{{self, Write}};

#[tokio::main]
async fn main() -> Result<()> {{
    // Create an interactive chatbot
    let agent = agent!{{
        name: "{}",
        instructions: "You are a friendly and helpful chatbot. Engage in natural conversation.",
        model: "gpt-4",
        tools: ["memory", "time", "calculator"]
    }};

    println!("🤖 {} Chatbot");
    println!("Type 'quit' to exit\n");

    loop {{
        print!("You: ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        if input.eq_ignore_ascii_case("quit") {{
            break;
        }}
        
        match agent.generate(input).await {{
            Ok(response) => println!("🤖: {{}}\n", response),
            Err(e) => println!("Error: {{}}\n", e),
        }}
    }}

    println!("Goodbye! 👋");
    Ok(())
}}
"#, name, name, name);
        
        fs::write(dir.join("src/main.rs"), main_rs)?;
        
        Ok(())
    }

    /// Add template-specific tools to configuration
    async fn add_template_tools(config: &mut ProjectConfig, template: &str) -> Result<()> {
        match template {
            "web-agent" => {
                config.tools.extend(vec![
                    ToolConfig {
                        name: "web_search".to_string(),
                        version: "1.0.0".to_string(),
                        source: ToolSource::Marketplace { package: "web-search".to_string() },
                        config: HashMap::new(),
                    },
                    ToolConfig {
                        name: "http_request".to_string(),
                        version: "1.0.0".to_string(),
                        source: ToolSource::Marketplace { package: "http-request".to_string() },
                        config: HashMap::new(),
                    },
                ]);
            }
            "data-agent" => {
                config.tools.extend(vec![
                    ToolConfig {
                        name: "csv_reader".to_string(),
                        version: "1.0.0".to_string(),
                        source: ToolSource::Marketplace { package: "csv-reader".to_string() },
                        config: HashMap::new(),
                    },
                    ToolConfig {
                        name: "data_analyzer".to_string(),
                        version: "1.0.0".to_string(),
                        source: ToolSource::Marketplace { package: "data-analyzer".to_string() },
                        config: HashMap::new(),
                    },
                ]);
            }
            "chat-bot" => {
                config.tools.extend(vec![
                    ToolConfig {
                        name: "memory".to_string(),
                        version: "1.0.0".to_string(),
                        source: ToolSource::Marketplace { package: "memory".to_string() },
                        config: HashMap::new(),
                    },
                    ToolConfig {
                        name: "calculator".to_string(),
                        version: "1.0.0".to_string(),
                        source: ToolSource::Marketplace { package: "calculator".to_string() },
                        config: HashMap::new(),
                    },
                ]);
            }
            _ => {} // Basic template has no default tools
        }
        
        Ok(())
    }
}
