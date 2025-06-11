//! YAML configuration support for LumosAI
//! 
//! This module provides YAML configuration parsing and loading,
//! extending the existing TOML configuration support.

use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::{Result, Error};

/// YAML configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlConfig {
    pub project: Option<ProjectConfig>,
    pub agents: Option<HashMap<String, AgentConfig>>,
    pub workflows: Option<HashMap<String, WorkflowConfig>>,
    pub rag: Option<RagConfig>,
    pub deployment: Option<DeploymentConfig>,
    pub tools: Option<HashMap<String, ToolConfig>>,
}

/// Project configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub model: String,
    pub instructions: String,
    pub tools: Option<Vec<String>>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub timeout: Option<u64>,
    pub memory: Option<MemoryConfig>,
    pub voice: Option<VoiceConfig>,
}

/// Memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub enabled: Option<bool>,
    pub max_capacity: Option<usize>,
    pub persistence: Option<String>,
}

/// Voice configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub enabled: Option<bool>,
    pub provider: Option<String>,
    pub voice_id: Option<String>,
}

/// Workflow configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub trigger: Option<String>,
    pub steps: Vec<WorkflowStepConfig>,
    pub timeout: Option<u64>,
    pub max_retries: Option<u32>,
}

/// Workflow step configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStepConfig {
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub agent: Option<String>,
    pub tool: Option<String>,
    pub workflow: Option<String>,
    pub condition: Option<String>,
    pub input: Option<serde_json::Value>,
    pub timeout: Option<u64>,
    pub retries: Option<u32>,
}

/// RAG configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagConfig {
    pub vector_store: Option<String>,
    pub embeddings: Option<String>,
    pub chunk_size: Option<usize>,
    pub chunk_overlap: Option<usize>,
    pub documents: Option<Vec<String>>,
    pub index_name: Option<String>,
}

/// Deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub platform: Option<String>,
    pub environment: Option<String>,
    pub vercel: Option<VercelConfig>,
    pub aws: Option<AwsConfig>,
    pub docker: Option<DockerConfig>,
}

/// Vercel deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VercelConfig {
    pub functions: Option<Vec<String>>,
    pub environment: Option<HashMap<String, String>>,
}

/// AWS deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsConfig {
    pub runtime: Option<String>,
    pub memory: Option<u32>,
    pub timeout: Option<u32>,
    pub region: Option<String>,
}

/// Docker deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    pub base_image: Option<String>,
    pub port: Option<u16>,
    pub optimize: Option<bool>,
}

/// Tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub enabled: Option<bool>,
    pub config: Option<HashMap<String, serde_yaml::Value>>,
}

impl YamlConfig {
    /// Load configuration from YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::Configuration(format!("Failed to read YAML config file: {}", e)))?;
        
        Self::from_str(&content)
    }
    
    /// Parse configuration from YAML string
    pub fn from_str(content: &str) -> Result<Self> {
        serde_yaml::from_str(content)
            .map_err(|e| Error::Configuration(format!("Failed to parse YAML config: {}", e)))
    }
    
    /// Save configuration to YAML file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = self.to_string()?;
        std::fs::write(path, content)
            .map_err(|e| Error::Configuration(format!("Failed to write YAML config file: {}", e)))
    }
    
    /// Convert configuration to YAML string
    pub fn to_string(&self) -> Result<String> {
        serde_yaml::to_string(self)
            .map_err(|e| Error::Configuration(format!("Failed to serialize YAML config: {}", e)))
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate project configuration
        if let Some(project) = &self.project {
            if project.name.is_empty() {
                return Err(Error::Configuration("Project name cannot be empty".to_string()));
            }
        }
        
        // Validate agents
        if let Some(agents) = &self.agents {
            for (name, agent) in agents {
                if name.is_empty() {
                    return Err(Error::Configuration("Agent name cannot be empty".to_string()));
                }
                if agent.model.is_empty() {
                    return Err(Error::Configuration(format!("Agent '{}' must have a model", name)));
                }
                if agent.instructions.is_empty() {
                    return Err(Error::Configuration(format!("Agent '{}' must have instructions", name)));
                }
            }
        }
        
        // Validate workflows
        if let Some(workflows) = &self.workflows {
            for (name, workflow) in workflows {
                if name.is_empty() {
                    return Err(Error::Configuration("Workflow name cannot be empty".to_string()));
                }
                if workflow.steps.is_empty() {
                    return Err(Error::Configuration(format!("Workflow '{}' must have at least one step", name)));
                }

                // Validate workflow steps
                for (i, step) in workflow.steps.iter().enumerate() {
                    // At least one of agent, tool, or workflow must be specified
                    if step.agent.is_none() && step.tool.is_none() && step.workflow.is_none() {
                        return Err(Error::Configuration(format!(
                            "Workflow '{}' step {} must specify an agent, tool, or workflow", name, i
                        )));
                    }

                    // Validate agent reference if specified
                    if let Some(agent_name) = &step.agent {
                        if agent_name.is_empty() {
                            return Err(Error::Configuration(format!(
                                "Workflow '{}' step {} agent name cannot be empty", name, i
                            )));
                        }
                    }

                    // Validate tool reference if specified
                    if let Some(tool_name) = &step.tool {
                        if tool_name.is_empty() {
                            return Err(Error::Configuration(format!(
                                "Workflow '{}' step {} tool name cannot be empty", name, i
                            )));
                        }
                    }

                    // Validate workflow reference if specified
                    if let Some(workflow_id) = &step.workflow {
                        if workflow_id.is_empty() {
                            return Err(Error::Configuration(format!(
                                "Workflow '{}' step {} workflow ID cannot be empty", name, i
                            )));
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get agent configuration by name
    pub fn get_agent(&self, name: &str) -> Option<&AgentConfig> {
        self.agents.as_ref()?.get(name)
    }
    
    /// Get workflow configuration by name
    pub fn get_workflow(&self, name: &str) -> Option<&WorkflowConfig> {
        self.workflows.as_ref()?.get(name)
    }
    
    /// List all agent names
    pub fn list_agents(&self) -> Vec<String> {
        self.agents
            .as_ref()
            .map(|agents| agents.keys().cloned().collect())
            .unwrap_or_default()
    }
    
    /// List all workflow names
    pub fn list_workflows(&self) -> Vec<String> {
        self.workflows
            .as_ref()
            .map(|workflows| workflows.keys().cloned().collect())
            .unwrap_or_default()
    }
}

impl Default for YamlConfig {
    fn default() -> Self {
        Self {
            project: Some(ProjectConfig {
                name: "my-ai-app".to_string(),
                version: Some("0.1.0".to_string()),
                description: None,
                author: None,
            }),
            agents: Some({
                let mut agents = HashMap::new();
                agents.insert("assistant".to_string(), AgentConfig {
                    model: "gpt-4".to_string(),
                    instructions: "You are a helpful assistant".to_string(),
                    tools: Some(vec!["web_search".to_string(), "calculator".to_string()]),
                    temperature: Some(0.7),
                    max_tokens: Some(2000),
                    timeout: Some(30),
                    memory: Some(MemoryConfig {
                        enabled: Some(true),
                        max_capacity: Some(100),
                        persistence: Some("memory".to_string()),
                    }),
                    voice: None,
                });
                agents
            }),
            workflows: None,
            rag: Some(RagConfig {
                vector_store: Some("memory".to_string()),
                embeddings: Some("openai".to_string()),
                chunk_size: Some(1000),
                chunk_overlap: Some(200),
                documents: Some(vec!["docs/".to_string()]),
                index_name: Some("default".to_string()),
            }),
            deployment: Some(DeploymentConfig {
                platform: Some("auto".to_string()),
                environment: Some("development".to_string()),
                vercel: None,
                aws: None,
                docker: Some(DockerConfig {
                    base_image: Some("alpine".to_string()),
                    port: Some(8080),
                    optimize: Some(true),
                }),
            }),
            tools: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_yaml_config_parsing() {
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

workflows:
  support:
    id: support_workflow
    name: Support Workflow
    trigger: user_message
    steps:
      - id: step1
        name: Assistant Step
        agent: assistant
        condition: general_query
"#;
        
        let config = YamlConfig::from_str(yaml_content).unwrap();
        
        assert_eq!(config.project.as_ref().unwrap().name, "test-app");
        assert_eq!(config.get_agent("assistant").unwrap().model, "gpt-4");
        assert_eq!(config.get_workflow("support").unwrap().steps.len(), 1);
    }
    
    #[test]
    fn test_yaml_config_validation() {
        let mut config = YamlConfig::default();
        
        // Valid configuration should pass
        assert!(config.validate().is_ok());
        
        // Invalid agent model should fail
        if let Some(agents) = &mut config.agents {
            if let Some(agent) = agents.get_mut("assistant") {
                agent.model = "".to_string();
            }
        }
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_yaml_config_serialization() {
        let config = YamlConfig::default();
        let yaml_string = config.to_string().unwrap();
        
        // Should be able to parse back
        let parsed_config = YamlConfig::from_str(&yaml_string).unwrap();
        assert_eq!(parsed_config.project.as_ref().unwrap().name, config.project.as_ref().unwrap().name);
    }
}
