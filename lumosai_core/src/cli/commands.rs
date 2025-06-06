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

    /// Interactive project initialization
    pub async fn init_interactive() -> Result<()> {
        use std::io::{self, Write};

        CliUtils::info("🚀 Welcome to Lumos.ai Interactive Setup!");
        println!();

        // Get project name
        print!("📝 Project name: ");
        io::stdout().flush()?;
        let mut project_name = String::new();
        io::stdin().read_line(&mut project_name)?;
        let project_name = project_name.trim();

        if project_name.is_empty() {
            return Err(crate::Error::Other("Project name cannot be empty".to_string()));
        }

        // Get template choice
        println!("\n🎨 Choose a template:");
        println!("  1. Basic Agent (simple conversational agent)");
        println!("  2. Web Agent (web search and scraping capabilities)");
        println!("  3. Data Agent (data processing and analysis)");
        println!("  4. Chat Bot (interactive conversation)");
        println!("  5. Stock Assistant (financial data analysis)");

        print!("Enter choice (1-5): ");
        io::stdout().flush()?;
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;

        let template = match choice.trim() {
            "1" => "basic",
            "2" => "web-agent",
            "3" => "data-agent",
            "4" => "chat-bot",
            "5" => "stock-assistant",
            _ => {
                CliUtils::warning("Invalid choice, using basic template");
                "basic"
            }
        };

        // Get model preference
        println!("\n🤖 Choose default AI model:");
        println!("  1. DeepSeek Chat (recommended, cost-effective)");
        println!("  2. OpenAI GPT-4 (high quality)");
        println!("  3. Anthropic Claude (balanced)");
        println!("  4. Local Ollama (privacy-focused)");

        print!("Enter choice (1-4): ");
        io::stdout().flush()?;
        let mut model_choice = String::new();
        io::stdin().read_line(&mut model_choice)?;

        let model = match model_choice.trim() {
            "1" => "deepseek-chat",
            "2" => "gpt-4",
            "3" => "claude-3-sonnet",
            "4" => "ollama/llama2",
            _ => {
                CliUtils::warning("Invalid choice, using DeepSeek Chat");
                "deepseek-chat"
            }
        };

        // Create the project
        Self::new_project(project_name, template, None).await?;

        // Update configuration with selected model
        let config_path = PathBuf::from(project_name).join("lumos.toml");
        let mut config = CliUtils::load_config(&config_path)?;
        config.default_model = Some(model.to_string());
        CliUtils::save_config(&config_path, &config)?;

        println!("\n✨ Project setup complete!");
        CliUtils::info(&format!("📁 Project created in: {}", project_name));
        CliUtils::info(&format!("🤖 Default model: {}", model));
        CliUtils::info(&format!("🎨 Template: {}", template));

        println!("\n🚀 Quick start:");
        println!("  cd {}", project_name);
        println!("  lumos dev --hot-reload");

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
                "web" => Some(ToolCategory::Web),
                "file" => Some(ToolCategory::File),
                "data" => Some(ToolCategory::Data),
                "system" => Some(ToolCategory::System),
                "math" => Some(ToolCategory::Math),
                "ai" => Some(ToolCategory::AI),
                "crypto" => Some(ToolCategory::Crypto),
                "database" => Some(ToolCategory::Database),
                "api" => Some(ToolCategory::API),
                "utility" => Some(ToolCategory::Utility),
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
        let mut marketplace = Marketplace::new(
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

    /// Add a model to the project
    pub async fn add_model(provider: &str, model_name: Option<String>, api_key: Option<String>) -> Result<()> {
        let project_root = CliUtils::find_project_root(".")
            .ok_or_else(|| crate::Error::Other("Not in a Lumos.ai project directory".to_string()))?;

        CliUtils::progress(&format!("Adding model provider '{}'...", provider));

        // Load current configuration
        let config_path = project_root.join("lumos.toml");
        let mut config = CliUtils::load_config(&config_path)?;

        // Determine model configuration based on provider
        let (model_id, requires_api_key) = match provider {
            "deepseek" => (model_name.unwrap_or_else(|| "deepseek-chat".to_string()), true),
            "openai" => (model_name.unwrap_or_else(|| "gpt-4".to_string()), true),
            "anthropic" => (model_name.unwrap_or_else(|| "claude-3-sonnet-20240229".to_string()), true),
            "ollama" => (model_name.unwrap_or_else(|| "llama2".to_string()), false),
            "groq" => (model_name.unwrap_or_else(|| "mixtral-8x7b-32768".to_string()), true),
            _ => return Err(crate::Error::Other(format!("Unsupported model provider: {}", provider))),
        };

        // Check if API key is required but not provided
        if requires_api_key && api_key.is_none() {
            CliUtils::warning(&format!("Provider '{}' requires an API key", provider));
            CliUtils::info("You can set it later using environment variables:");
            match provider {
                "deepseek" => CliUtils::info("  export DEEPSEEK_API_KEY=your_key_here"),
                "openai" => CliUtils::info("  export OPENAI_API_KEY=your_key_here"),
                "anthropic" => CliUtils::info("  export ANTHROPIC_API_KEY=your_key_here"),
                "groq" => CliUtils::info("  export GROQ_API_KEY=your_key_here"),
                _ => {}
            }
        }

        // Add model configuration
        if config.models.is_none() {
            config.models = Some(HashMap::new());
        }

        let models = config.models.as_mut().unwrap();
        models.insert(provider.to_string(), serde_json::json!({
            "model": model_id,
            "provider": provider,
            "api_key_env": format!("{}_API_KEY", provider.to_uppercase()),
        }));

        // Set as default if no default model is set
        if config.default_model.is_none() {
            config.default_model = Some(format!("{}:{}", provider, model_id));
        }

        // Save configuration
        CliUtils::save_config(&config_path, &config)?;

        CliUtils::success(&format!("Model provider '{}' added successfully!", provider));
        CliUtils::info(&format!("Model: {}", model_id));

        if requires_api_key && api_key.is_some() {
            CliUtils::info("API key configured");
        }

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
    pub async fn build_project(target: &str, output: Option<PathBuf>, optimize: bool) -> Result<()> {
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
        let mut args = vec!["build"];

        match target {
            "release" => {
                args.push("--release");
                if optimize {
                    // Add optimization flags
                    std::env::set_var("RUSTFLAGS", "-C target-cpu=native -C opt-level=3");
                }
            }
            "wasm" => {
                args.extend(&["--target", "wasm32-unknown-unknown", "--release"]);
                if optimize {
                    std::env::set_var("RUSTFLAGS", "-C opt-level=s -C lto=yes");
                }
            }
            "debug" => {
                // Debug build, no additional flags
            }
            _ => {
                return Err(crate::Error::Other(format!("Unknown build target: {}", target)));
            }
        }

        CliUtils::execute_command("cargo", &args, Some(&project_root)).await?;

        CliUtils::success(&format!("Project built successfully for {}", target));
        CliUtils::info(&format!("Output: {}", output_dir.display()));

        if optimize {
            CliUtils::info("Optimizations enabled");
        }

        Ok(())
    }

    /// Run tests with optional coverage
    pub async fn test_project(watch: bool, coverage: bool, filter: Option<String>) -> Result<()> {
        let project_root = CliUtils::find_project_root(".")
            .ok_or_else(|| crate::Error::Other("Not in a Lumos.ai project directory".to_string()))?;

        CliUtils::progress("Running tests...");

        if coverage {
            // Install cargo-tarpaulin if not available
            if CliUtils::execute_command("cargo", &["tarpaulin", "--version"], Some(&project_root)).await.is_err() {
                CliUtils::info("Installing cargo-tarpaulin for coverage...");
                CliUtils::execute_command("cargo", &["install", "cargo-tarpaulin"], Some(&project_root)).await?;
            }

            let mut tarpaulin_args = vec!["tarpaulin", "--out", "Html", "--output-dir", "coverage"];
            if let Some(ref filter) = filter {
                tarpaulin_args.extend(&["--test", filter]);
            }

            CliUtils::execute_command("cargo", &tarpaulin_args, Some(&project_root)).await?;
            CliUtils::success("Tests completed with coverage report");
            CliUtils::info("Coverage report: coverage/tarpaulin-report.html");
        } else if watch {
            // Install cargo-watch if not available
            if CliUtils::execute_command("cargo", &["watch", "--version"], Some(&project_root)).await.is_err() {
                CliUtils::info("Installing cargo-watch for test watching...");
                CliUtils::execute_command("cargo", &["install", "cargo-watch"], Some(&project_root)).await?;
            }

            let test_cmd = if let Some(ref filter) = filter {
                format!("test {}", filter)
            } else {
                "test".to_string()
            };
            let watch_args = vec!["watch", "-x", &test_cmd];

            CliUtils::execute_command("cargo", &watch_args, Some(&project_root)).await?;
        } else {
            let mut args = vec!["test"];
            if let Some(ref filter) = filter {
                args.push(filter);
            }

            CliUtils::execute_command("cargo", &args, Some(&project_root)).await?;
            CliUtils::success("Tests completed successfully");
        }

        Ok(())
    }

    /// Format code using rustfmt
    pub async fn format_project(check: bool) -> Result<()> {
        let project_root = CliUtils::find_project_root(".")
            .ok_or_else(|| crate::Error::Other("Not in a Lumos.ai project directory".to_string()))?;

        CliUtils::progress("Formatting code...");

        let mut args = vec!["fmt"];
        if check {
            args.push("--check");
        }

        match CliUtils::execute_command("cargo", &args, Some(&project_root)).await {
            Ok(_) => {
                if check {
                    CliUtils::success("Code formatting is correct");
                } else {
                    CliUtils::success("Code formatted successfully");
                }
            }
            Err(_) => {
                if check {
                    CliUtils::error("Code formatting issues found. Run 'lumos format' to fix.");
                    return Err(crate::Error::Other("Formatting check failed".to_string()));
                } else {
                    return Err(crate::Error::Other("Formatting failed".to_string()));
                }
            }
        }

        Ok(())
    }

    /// Lint code using clippy
    pub async fn lint_project(fix: bool) -> Result<()> {
        let project_root = CliUtils::find_project_root(".")
            .ok_or_else(|| crate::Error::Other("Not in a Lumos.ai project directory".to_string()))?;

        CliUtils::progress("Linting code...");

        let mut args = vec!["clippy"];
        if fix {
            args.extend(&["--fix", "--allow-dirty"]);
        } else {
            args.extend(&["--", "-D", "warnings"]);
        }

        match CliUtils::execute_command("cargo", &args, Some(&project_root)).await {
            Ok(_) => {
                if fix {
                    CliUtils::success("Code issues fixed automatically");
                } else {
                    CliUtils::success("No linting issues found");
                }
            }
            Err(_) => {
                if fix {
                    CliUtils::warning("Some issues could not be fixed automatically");
                } else {
                    CliUtils::error("Linting issues found. Run 'lumos lint --fix' to fix automatically.");
                    return Err(crate::Error::Other("Linting failed".to_string()));
                }
            }
        }

        Ok(())
    }

    /// Create project structure based on template
    async fn create_project_structure(dir: &Path, name: &str, template: &str) -> Result<()> {
        match template {
            "basic" => Self::create_basic_template(dir, name).await,
            "web-agent" => Self::create_web_agent_template(dir, name).await,
            "data-agent" => Self::create_data_agent_template(dir, name).await,
            "chat-bot" => Self::create_chatbot_template(dir, name).await,
            "stock-assistant" => Self::create_stock_assistant_template(dir, name).await,
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

    /// Create stock assistant template
    async fn create_stock_assistant_template(dir: &Path, name: &str) -> Result<()> {
        Self::create_basic_template(dir, name).await?;

        // Override main.rs with stock-specific content
        let main_rs = format!(r#"//! {} - A Stock Analysis Assistant
//!
//! This agent can analyze stock data, provide market insights, and track portfolios.

use lumosai::{{Agent, tools, Result}};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {{
    // Create a stock analysis agent using the simplified API
    let agent = Agent::quick("{}",
        "You are a professional stock market analyst. You can analyze stock data, \
         provide market insights, track portfolios, and give investment advice. \
         Always provide data-driven analysis and include risk warnings.")
        .model("deepseek-chat")
        .tools([
            tools::web_search(),
            tools::calculator(),
            tools::data_analyzer(),
        ])
        .build()?;

    println!("🏦 {} Stock Assistant");
    println!("Ask me about stocks, market analysis, or portfolio management!");
    println!("Type 'quit' to exit\n");

    // Example queries
    let example_queries = vec![
        "What's the current price of AAPL?",
        "Analyze the performance of Tesla stock this year",
        "What are the top 5 tech stocks to watch?",
        "Explain the P/E ratio and its significance",
        "Should I invest in renewable energy stocks?",
    ];

    println!("💡 Example queries:");
    for (i, query) in example_queries.iter().enumerate() {{
        println!("  {{}}. {{}}", i + 1, query);
    }}
    println!();

    // Interactive loop
    loop {{
        use std::io::{{self, Write}};

        print!("📈 You: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.eq_ignore_ascii_case("quit") {{
            break;
        }}

        if input.is_empty() {{
            continue;
        }}

        match agent.generate(input).await {{
            Ok(response) => {{
                println!("🤖 Assistant: {{}}\n", response);
            }}
            Err(e) => {{
                println!("❌ Error: {{}}\n", e);
            }}
        }}
    }}

    println!("📊 Thank you for using {} Stock Assistant! 👋");
    Ok(())
}}
"#, name, name, name, name);

        fs::write(dir.join("src/main.rs"), main_rs)?;

        // Create example configuration file
        let example_config = r#"# Stock Assistant Configuration
# Add your API keys and preferences here

[stock_data]
# Uncomment and add your API keys
# alpha_vantage_key = "your_alpha_vantage_key"
# finnhub_key = "your_finnhub_key"
# polygon_key = "your_polygon_key"

[preferences]
default_currency = "USD"
default_market = "NYSE"
risk_tolerance = "moderate"  # conservative, moderate, aggressive

[watchlist]
# Add stocks to watch
symbols = ["AAPL", "GOOGL", "MSFT", "TSLA", "AMZN"]

[alerts]
# Price change alerts (percentage)
price_change_threshold = 5.0
volume_spike_threshold = 2.0
"#;

        fs::write(dir.join("stock_config.toml"), example_config)?;

        // Create sample data directory with example files
        fs::create_dir_all(dir.join("data"))?;

        let sample_portfolio = r#"symbol,shares,purchase_price,purchase_date
AAPL,100,150.00,2023-01-15
GOOGL,50,2800.00,2023-02-01
MSFT,75,300.00,2023-01-20
TSLA,25,200.00,2023-03-01
"#;

        fs::write(dir.join("data/portfolio.csv"), sample_portfolio)?;

        // Create README with usage instructions
        let readme = format!(r#"# {} - Stock Analysis Assistant

A powerful AI-powered stock analysis assistant built with Lumos.ai.

## Features

- 📈 Real-time stock price analysis
- 📊 Portfolio tracking and management
- 🔍 Market research and insights
- 💡 Investment recommendations
- ⚠️ Risk assessment and warnings

## Quick Start

1. **Configure API Keys** (optional):
   Edit `stock_config.toml` to add your financial data API keys.

2. **Run the Assistant**:
   ```bash
   cargo run
   ```

3. **Example Queries**:
   - "What's the current price of AAPL?"
   - "Analyze my portfolio performance"
   - "What are the top tech stocks today?"
   - "Should I buy or sell Tesla stock?"

## Configuration

### API Keys (Optional)
For real-time data, you can configure API keys in `stock_config.toml`:
- Alpha Vantage: Free tier available
- Finnhub: Free tier available
- Polygon: Free tier available

### Portfolio Tracking
Add your holdings to `data/portfolio.csv` to track performance.

## Development

```bash
# Start development server with hot reload
lumos dev --hot-reload

# Run tests
lumos test

# Format code
lumos format

# Lint code
lumos lint
```

## Disclaimer

This assistant provides educational information only. Always consult with a qualified financial advisor before making investment decisions. Past performance does not guarantee future results.
"#, name);

        fs::write(dir.join("README.md"), readme)?;

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
            "stock-assistant" => {
                config.tools.extend(vec![
                    ToolConfig {
                        name: "web_search".to_string(),
                        version: "1.0.0".to_string(),
                        source: ToolSource::Marketplace { package: "web-search".to_string() },
                        config: HashMap::new(),
                    },
                    ToolConfig {
                        name: "calculator".to_string(),
                        version: "1.0.0".to_string(),
                        source: ToolSource::Marketplace { package: "calculator".to_string() },
                        config: HashMap::new(),
                    },
                    ToolConfig {
                        name: "data_analyzer".to_string(),
                        version: "1.0.0".to_string(),
                        source: ToolSource::Marketplace { package: "data-analyzer".to_string() },
                        config: HashMap::new(),
                    },
                    ToolConfig {
                        name: "csv_reader".to_string(),
                        version: "1.0.0".to_string(),
                        source: ToolSource::Marketplace { package: "csv-reader".to_string() },
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
