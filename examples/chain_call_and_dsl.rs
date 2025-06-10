//! Example demonstrating chain call with model names and DSL configuration
//! 
//! This example shows the new features:
//! 1. Chain calls with model name strings
//! 2. YAML/TOML configuration support
//! 3. Configuration-driven agent creation

use lumosai_core::{Result, Error};
use lumosai_core::agent::{AgentBuilder, ModelResolver};
use lumosai_core::app::LumosApp;
use lumosai_core::config::{ConfigLoader, YamlConfig, ConfigFormat};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 LumosAI Chain Call and DSL Configuration Example");
    
    // Example 1: Chain call with model names
    println!("\n📋 Example 1: Chain Call with Model Names");
    chain_call_example().await?;
    
    // Example 2: YAML configuration
    println!("\n📋 Example 2: YAML Configuration");
    yaml_config_example().await?;
    
    // Example 3: TOML configuration
    println!("\n📋 Example 3: TOML Configuration");
    toml_config_example().await?;
    
    // Example 4: Configuration-driven app
    println!("\n📋 Example 4: Configuration-driven App");
    config_driven_app_example().await?;
    
    Ok(())
}

/// Example 1: Chain call with model names
async fn chain_call_example() -> Result<()> {
    println!("Creating agents with model name strings...");
    
    // Set up environment variables for testing
    env::set_var("OPENAI_API_KEY", "test-key");
    env::set_var("ANTHROPIC_API_KEY", "test-key");
    env::set_var("DEEPSEEK_API_KEY", "test-key");
    
    // Example 1.1: Direct model name
    println!("1. Creating agent with 'gpt-4' model name:");
    let result = AgentBuilder::new()
        .name("assistant")
        .instructions("You are a helpful assistant")
        .model_name("gpt-4")  // Automatically resolves to OpenAI provider
        .build_async()
        .await;
    
    match result {
        Ok(_agent) => println!("   ✅ Successfully created agent with gpt-4"),
        Err(e) => println!("   ❌ Failed to create agent: {}", e),
    }
    
    // Example 1.2: Explicit provider specification
    println!("2. Creating agent with 'anthropic/claude-3-sonnet' model name:");
    let result = AgentBuilder::new()
        .name("claude_assistant")
        .instructions("You are Claude, a helpful AI assistant")
        .model_name("anthropic/claude-3-sonnet")  // Explicit provider
        .build_async()
        .await;
    
    match result {
        Ok(_agent) => println!("   ✅ Successfully created agent with Claude"),
        Err(e) => println!("   ❌ Failed to create agent: {}", e),
    }
    
    // Example 1.3: DeepSeek model
    println!("3. Creating agent with 'deepseek-chat' model name:");
    let result = AgentBuilder::new()
        .name("deepseek_assistant")
        .instructions("You are a DeepSeek AI assistant")
        .model_name("deepseek-chat")  // Auto-resolves to DeepSeek
        .build_async()
        .await;
    
    match result {
        Ok(_agent) => println!("   ✅ Successfully created agent with DeepSeek"),
        Err(e) => println!("   ❌ Failed to create agent: {}", e),
    }
    
    // Example 1.4: List supported models
    println!("4. Supported models:");
    let resolver = ModelResolver::new();
    let models = resolver.list_models();
    for model in models.iter().take(5) {
        println!("   - {}", model);
    }
    println!("   ... and {} more", models.len().saturating_sub(5));
    
    Ok(())
}

/// Example 2: YAML configuration
async fn yaml_config_example() -> Result<()> {
    println!("Creating and loading YAML configuration...");
    
    // Create a sample YAML configuration
    let yaml_content = r#"
project:
  name: my-ai-app
  version: 0.1.0
  description: Example AI application

agents:
  assistant:
    model: gpt-4
    instructions: You are a helpful assistant
    tools:
      - web_search
      - calculator
    temperature: 0.7
    max_tokens: 2000
    
  coder:
    model: deepseek-coder
    instructions: You are an expert programmer
    tools:
      - code_executor
      - file_manager
    temperature: 0.3

workflows:
  support:
    trigger: user_message
    steps:
      - agent: assistant
        condition: general_query
      - agent: coder
        condition: code_related

rag:
  vector_store: memory
  embeddings: openai
  chunk_size: 1000
  documents:
    - docs/
    - knowledge/

deployment:
  platform: auto
  docker:
    base_image: alpine
    port: 8080
    optimize: true
"#;
    
    // Parse YAML configuration
    match YamlConfig::from_str(yaml_content) {
        Ok(config) => {
            println!("   ✅ Successfully parsed YAML configuration");
            println!("   📋 Project: {}", config.project.as_ref().unwrap().name);
            println!("   🤖 Agents: {}", config.list_agents().join(", "));
            println!("   🔄 Workflows: {}", config.list_workflows().join(", "));
            
            // Validate configuration
            match config.validate() {
                Ok(()) => println!("   ✅ Configuration is valid"),
                Err(e) => println!("   ❌ Configuration validation failed: {}", e),
            }
        },
        Err(e) => println!("   ❌ Failed to parse YAML: {}", e),
    }
    
    Ok(())
}

/// Example 3: TOML configuration
async fn toml_config_example() -> Result<()> {
    println!("Creating and loading TOML configuration...");
    
    // Create a sample TOML configuration
    let toml_content = r#"
[project]
name = "my-ai-app"
version = "0.1.0"
description = "Example AI application"

[agents.assistant]
model = "gpt-4"
instructions = "You are a helpful assistant"
tools = ["web_search", "calculator"]
temperature = 0.7
max_tokens = 2000

[agents.coder]
model = "deepseek-coder"
instructions = "You are an expert programmer"
tools = ["code_executor", "file_manager"]
temperature = 0.3

[workflows.support]
trigger = "user_message"
steps = [
  { agent = "assistant", condition = "general_query" },
  { agent = "coder", condition = "code_related" }
]

[rag]
vector_store = "memory"
embeddings = "openai"
chunk_size = 1000
documents = ["docs/", "knowledge/"]

[deployment]
platform = "auto"

[deployment.docker]
base_image = "alpine"
port = 8080
optimize = true
"#;
    
    // Parse TOML as YAML config
    match ConfigLoader::parse_toml_content(toml_content) {
        Ok(config) => {
            println!("   ✅ Successfully parsed TOML configuration");
            println!("   📋 Project: {}", config.project.as_ref().unwrap().name);
            println!("   🤖 Agents: {}", config.list_agents().join(", "));
            
            // Validate configuration
            match config.validate() {
                Ok(()) => println!("   ✅ Configuration is valid"),
                Err(e) => println!("   ❌ Configuration validation failed: {}", e),
            }
        },
        Err(e) => println!("   ❌ Failed to parse TOML: {}", e),
    }
    
    Ok(())
}

/// Example 4: Configuration-driven app
async fn config_driven_app_example() -> Result<()> {
    println!("Creating app from configuration...");
    
    // Create a default configuration
    let config = YamlConfig::default();
    
    // Try to create app from configuration
    match LumosApp::from_yaml_config(config).await {
        Ok(app) => {
            println!("   ✅ Successfully created app from configuration");
            println!("   📋 App name: {}", app.name());
            println!("   🤖 Available agents: {}", app.agents().len());
            
            // Try to get an agent
            match app.agent("assistant") {
                Ok(_agent) => println!("   ✅ Successfully retrieved 'assistant' agent"),
                Err(e) => println!("   ❌ Failed to get agent: {}", e),
            }
        },
        Err(e) => println!("   ❌ Failed to create app from config: {}", e),
    }
    
    Ok(())
}


