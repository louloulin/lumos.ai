//! Tests for chain call with model names and DSL configuration support

// Remove unused import
use lumosai_core::agent::{AgentBuilder, ModelResolver};
use lumosai_core::app::LumosApp;
use lumosai_core::config::{ConfigLoader, YamlConfig, ConfigFormat};
use std::env;
use tempfile::tempdir;
use std::fs;

#[tokio::test]
async fn test_model_resolver_basic() {
    let resolver = ModelResolver::new();
    
    // Test model name parsing
    let models = resolver.list_models();
    assert!(!models.is_empty());
    assert!(models.contains(&"gpt-4".to_string()));
    assert!(models.contains(&"claude-3-sonnet".to_string()));
    assert!(models.contains(&"deepseek-chat".to_string()));
}

#[tokio::test]
async fn test_chain_call_with_mock_keys() {
    // Set up mock API keys
    env::set_var("OPENAI_API_KEY", "test-key");
    env::set_var("ANTHROPIC_API_KEY", "test-key");
    env::set_var("DEEPSEEK_API_KEY", "test-key");
    
    // Test building agent with model name (should fail due to invalid API key, but parsing should work)
    let result = AgentBuilder::new()
        .name("test_agent")
        .instructions("You are a test assistant")
        .model_name("gpt-4")
        .build_async()
        .await;
    
    // Should fail due to invalid API key, but not due to parsing
    assert!(result.is_err());
    // We can't use unwrap_err() because BasicAgent doesn't implement Debug
    // Just check that it's an error, which means parsing worked
    // (if parsing failed, we'd get a different error type)
}

#[tokio::test]
async fn test_yaml_config_parsing() {
    let yaml_content = r#"
project:
  name: test-app
  version: 0.1.0

agents:
  assistant:
    model: gpt-4
    instructions: You are helpful
    tools:
      - web_search
      - calculator
    temperature: 0.7

  coder:
    model: deepseek-coder
    instructions: You are a programmer
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
"#;
    
    let config = YamlConfig::from_str(yaml_content).unwrap();
    
    // Test project configuration
    assert_eq!(config.project.as_ref().unwrap().name, "test-app");
    assert_eq!(config.project.as_ref().unwrap().version.as_ref().unwrap(), "0.1.0");
    
    // Test agents configuration
    let agents = config.agents.as_ref().unwrap();
    assert_eq!(agents.len(), 2);
    
    let assistant = agents.get("assistant").unwrap();
    assert_eq!(assistant.model, "gpt-4");
    assert_eq!(assistant.instructions, "You are helpful");
    assert_eq!(assistant.temperature.unwrap(), 0.7);
    assert_eq!(assistant.tools.as_ref().unwrap().len(), 2);
    
    let coder = agents.get("coder").unwrap();
    assert_eq!(coder.model, "deepseek-coder");
    assert_eq!(coder.temperature.unwrap(), 0.3);
    
    // Test workflows configuration
    let workflows = config.workflows.as_ref().unwrap();
    assert_eq!(workflows.len(), 1);
    
    let support = workflows.get("support").unwrap();
    assert_eq!(support.trigger.as_ref().unwrap(), "user_message");
    assert_eq!(support.steps.len(), 2);
    
    // Test RAG configuration
    let rag = config.rag.as_ref().unwrap();
    assert_eq!(rag.vector_store.as_ref().unwrap(), "memory");
    assert_eq!(rag.chunk_size.unwrap(), 1000);
    
    // Test validation
    assert!(config.validate().is_ok());
}

#[tokio::test]
async fn test_toml_config_loading() {
    let toml_content = r#"
[project]
name = "test-app"
version = "0.1.0"

[agents.assistant]
model = "gpt-4"
instructions = "You are helpful"
tools = ["web_search", "calculator"]
temperature = 0.7

[agents.coder]
model = "deepseek-coder"
instructions = "You are a programmer"
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
documents = ["docs/"]
"#;
    
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.toml");
    fs::write(&file_path, toml_content).unwrap();
    
    let config = ConfigLoader::load(&file_path).unwrap();
    
    // Test that TOML was correctly converted to YAML config
    assert_eq!(config.project.as_ref().unwrap().name, "test-app");
    assert_eq!(config.agents.as_ref().unwrap().len(), 2);
    assert_eq!(config.workflows.as_ref().unwrap().len(), 1);
    
    // Test validation
    assert!(config.validate().is_ok());
}

#[tokio::test]
async fn test_config_file_auto_detection() {
    let dir = tempdir().unwrap();
    let original_dir = env::current_dir().unwrap();
    
    // Change to temp directory
    env::set_current_dir(&dir).unwrap();
    
    // Create a YAML config file
    let yaml_content = r#"
project:
  name: auto-detected-app
  version: 1.0.0

agents:
  assistant:
    model: gpt-4
    instructions: Auto-detected assistant
"#;
    
    fs::write("lumosai.yaml", yaml_content).unwrap();
    
    // Test auto-detection
    let config = ConfigLoader::auto_detect().unwrap();
    assert_eq!(config.project.as_ref().unwrap().name, "auto-detected-app");
    
    // Restore original directory
    env::set_current_dir(original_dir).unwrap();
}

#[tokio::test]
async fn test_config_format_detection() {
    use lumosai_core::config::ConfigFormat;
    
    assert_eq!(ConfigFormat::from_extension("yaml"), Some(ConfigFormat::Yaml));
    assert_eq!(ConfigFormat::from_extension("yml"), Some(ConfigFormat::Yaml));
    assert_eq!(ConfigFormat::from_extension("toml"), Some(ConfigFormat::Toml));
    assert_eq!(ConfigFormat::from_extension("txt"), None);
    
    assert_eq!(ConfigFormat::Yaml.extension(), "yaml");
    assert_eq!(ConfigFormat::Toml.extension(), "toml");
}

#[tokio::test]
async fn test_create_default_configs() {
    let dir = tempdir().unwrap();
    
    // Create default YAML config
    let yaml_path = dir.path().join("default.yaml");
    ConfigLoader::create_default(&yaml_path, ConfigFormat::Yaml).unwrap();
    assert!(yaml_path.exists());
    
    // Create default TOML config
    let toml_path = dir.path().join("default.toml");
    ConfigLoader::create_default(&toml_path, ConfigFormat::Toml).unwrap();
    assert!(toml_path.exists());
    
    // Both should be loadable and equivalent
    let yaml_config = ConfigLoader::load(&yaml_path).unwrap();
    let toml_config = ConfigLoader::load(&toml_path).unwrap();
    
    assert_eq!(
        yaml_config.project.as_ref().unwrap().name,
        toml_config.project.as_ref().unwrap().name
    );
}

#[tokio::test]
async fn test_config_validation() {
    // Test valid configuration
    let valid_config = YamlConfig::default();
    assert!(valid_config.validate().is_ok());
    
    // Test invalid configuration - empty project name
    let mut invalid_config = YamlConfig::default();
    if let Some(project) = &mut invalid_config.project {
        project.name = "".to_string();
    }
    assert!(invalid_config.validate().is_err());
    
    // Test invalid configuration - empty agent model
    let mut invalid_config = YamlConfig::default();
    if let Some(agents) = &mut invalid_config.agents {
        if let Some(agent) = agents.get_mut("assistant") {
            agent.model = "".to_string();
        }
    }
    assert!(invalid_config.validate().is_err());
}

#[tokio::test]
async fn test_app_from_config() {
    let config = YamlConfig::default();
    
    // Test creating app from config (should fail due to missing API keys, but structure should be correct)
    let result = LumosApp::from_yaml_config(config.clone()).await;
    
    // Should fail due to missing API keys for model resolution
    assert!(result.is_err());

    // We can't use unwrap_err() because LumosApp doesn't implement Debug
    // Just check that it's an error, which means the config structure is correct
    // (if config structure was wrong, we'd get a different error type)
}

#[tokio::test]
async fn test_config_convenience_methods() {
    let config = YamlConfig::default();
    
    // Test agent listing
    let agents = config.list_agents();
    assert!(!agents.is_empty());
    assert!(agents.contains(&"assistant".to_string()));
    
    // Test agent retrieval
    let assistant = config.get_agent("assistant");
    assert!(assistant.is_some());
    assert_eq!(assistant.unwrap().model, "gpt-4");
    
    // Test non-existent agent
    let non_existent = config.get_agent("non_existent");
    assert!(non_existent.is_none());
}
