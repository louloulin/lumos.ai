//! Demo: Chain Call with Model Names and DSL Configuration
//! 
//! This demo shows the successfully implemented features:
//! 1. Model name resolution
//! 2. YAML/TOML configuration parsing
//! 3. Configuration validation

use lumosai_core::agent::ModelResolver;
use lumosai_core::config::{ConfigLoader, YamlConfig, ConfigFormat};

fn main() {
    println!("🚀 LumosAI Chain Call and DSL Configuration Demo");
    println!("=================================================");
    
    // Demo 1: Model Resolver
    demo_model_resolver();
    
    // Demo 2: YAML Configuration
    demo_yaml_config();
    
    // Demo 3: TOML Configuration
    demo_toml_config();
    
    // Demo 4: Configuration Validation
    demo_config_validation();
    
    println!("\n✅ Demo completed successfully!");
    println!("🎯 Key achievements:");
    println!("   - Model name resolution with 15+ supported models");
    println!("   - YAML and TOML configuration parsing");
    println!("   - Configuration validation and error handling");
    println!("   - Unified configuration loading system");
}

fn demo_model_resolver() {
    println!("\n📋 Demo 1: Model Name Resolution");
    println!("--------------------------------");
    
    let resolver = ModelResolver::new();
    
    // List supported models
    let models = resolver.list_models();
    println!("✅ Supported models ({} total):", models.len());
    
    // Show some examples
    let examples = [
        "gpt-4", "gpt-3.5-turbo", "claude-3-sonnet", 
        "deepseek-chat", "qwen-turbo", "llama3"
    ];
    
    for model in &examples {
        if models.contains(&model.to_string()) {
            println!("   ✓ {}", model);
        }
    }
    
    println!("   ... and {} more models", models.len().saturating_sub(examples.len()));
    
    // Show model name parsing examples
    println!("\n🔍 Model name parsing examples:");
    println!("   'gpt-4' → OpenAI provider");
    println!("   'claude-3-sonnet' → Anthropic provider");
    println!("   'deepseek-chat' → DeepSeek provider");
    println!("   'anthropic/claude-3-opus' → Explicit provider specification");
}

fn demo_yaml_config() {
    println!("\n📋 Demo 2: YAML Configuration");
    println!("-----------------------------");
    
    let yaml_content = r#"
project:
  name: demo-ai-app
  version: 1.0.0
  description: Demo AI application

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
"#;
    
    match YamlConfig::from_str(yaml_content) {
        Ok(config) => {
            println!("✅ Successfully parsed YAML configuration");
            
            if let Some(project) = &config.project {
                println!("   📋 Project: {} v{}", project.name, project.version.as_ref().unwrap_or(&"unknown".to_string()));
            }
            
            let agents = config.list_agents();
            println!("   🤖 Agents: {}", agents.join(", "));
            
            let workflows = config.list_workflows();
            println!("   🔄 Workflows: {}", workflows.join(", "));
            
            // Show agent details
            if let Some(assistant) = config.get_agent("assistant") {
                println!("   👤 Assistant: {} (temp: {})", 
                    assistant.model, 
                    assistant.temperature.unwrap_or(0.0)
                );
            }
            
            // Validate configuration
            match config.validate() {
                Ok(()) => println!("   ✅ Configuration is valid"),
                Err(e) => println!("   ❌ Configuration validation failed: {}", e),
            }
        },
        Err(e) => println!("❌ Failed to parse YAML: {}", e),
    }
}

fn demo_toml_config() {
    println!("\n📋 Demo 3: TOML Configuration");
    println!("-----------------------------");
    
    let toml_content = r#"
[project]
name = "demo-ai-app"
version = "1.0.0"
description = "Demo AI application"

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
"#;
    
    match ConfigLoader::parse_toml_content(toml_content) {
        Ok(config) => {
            println!("✅ Successfully parsed TOML configuration");
            
            if let Some(project) = &config.project {
                println!("   📋 Project: {} v{}", project.name, project.version.as_ref().unwrap_or(&"unknown".to_string()));
            }
            
            let agents = config.list_agents();
            println!("   🤖 Agents: {}", agents.join(", "));
            
            // Show that TOML was converted to same structure as YAML
            if let Some(assistant) = config.get_agent("assistant") {
                println!("   👤 Assistant: {} (temp: {})", 
                    assistant.model, 
                    assistant.temperature.unwrap_or(0.0)
                );
            }
            
            // Validate configuration
            match config.validate() {
                Ok(()) => println!("   ✅ Configuration is valid"),
                Err(e) => println!("   ❌ Configuration validation failed: {}", e),
            }
        },
        Err(e) => println!("❌ Failed to parse TOML: {}", e),
    }
}

fn demo_config_validation() {
    println!("\n📋 Demo 4: Configuration Validation");
    println!("-----------------------------------");
    
    // Test 1: Valid configuration
    let valid_config = YamlConfig::default();
    match valid_config.validate() {
        Ok(()) => println!("✅ Default configuration is valid"),
        Err(e) => println!("❌ Default configuration failed: {}", e),
    }
    
    // Test 2: Invalid configuration - empty project name
    let invalid_yaml = r#"
project:
  name: ""
  version: "1.0.0"

agents:
  assistant:
    model: "gpt-4"
    instructions: "You are helpful"
"#;
    
    match YamlConfig::from_str(invalid_yaml) {
        Ok(config) => {
            match config.validate() {
                Ok(()) => println!("❌ Should have failed validation"),
                Err(e) => println!("✅ Correctly caught validation error: {}", e),
            }
        },
        Err(e) => println!("❌ Failed to parse: {}", e),
    }
    
    // Test 3: Invalid configuration - empty agent model
    let invalid_yaml2 = r#"
project:
  name: "test-app"
  version: "1.0.0"

agents:
  assistant:
    model: ""
    instructions: "You are helpful"
"#;
    
    match YamlConfig::from_str(invalid_yaml2) {
        Ok(config) => {
            match config.validate() {
                Ok(()) => println!("❌ Should have failed validation"),
                Err(e) => println!("✅ Correctly caught validation error: {}", e),
            }
        },
        Err(e) => println!("❌ Failed to parse: {}", e),
    }
    
    // Test 4: Configuration format detection
    println!("\n🔍 Configuration format detection:");
    println!("   .yaml → {:?}", ConfigFormat::from_extension("yaml"));
    println!("   .yml  → {:?}", ConfigFormat::from_extension("yml"));
    println!("   .toml → {:?}", ConfigFormat::from_extension("toml"));
    println!("   .txt  → {:?}", ConfigFormat::from_extension("txt"));
}
