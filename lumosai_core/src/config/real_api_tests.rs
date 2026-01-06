//! Real API tests for Config module
//!
//! Tests configuration loading, validation, and error handling

#[cfg(test)]
mod tests {
    use crate::config::{
        AgentConfig, ConfigLoader, MemoryConfig, ProjectConfig, WorkflowConfig, WorkflowStepConfig,
        YamlConfig,
    };
    use std::collections::HashMap;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_yaml_config_empty_project_name() {
        let mut config = YamlConfig::default();
        if let Some(project) = &mut config.project {
            project.name = "".to_string();
        }

        let result = config.validate();
        assert!(result.is_err(), "Empty project name should fail validation");
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Project name cannot be empty"));
    }

    #[test]
    fn test_yaml_config_empty_agent_name() {
        let mut config = YamlConfig::default();
        if let Some(agents) = &mut config.agents {
            // Add an agent with empty name
            agents.insert(
                "".to_string(),
                AgentConfig {
                    model: "gpt-4".to_string(),
                    instructions: "Test".to_string(),
                    tools: None,
                    temperature: None,
                    max_tokens: None,
                    timeout: None,
                    memory: None,
                    voice: None,
                },
            );
        }

        let result = config.validate();
        assert!(result.is_err(), "Empty agent name should fail validation");
    }

    #[test]
    fn test_yaml_config_empty_agent_model() {
        let mut config = YamlConfig::default();
        if let Some(agents) = &mut config.agents {
            if let Some(agent) = agents.get_mut("assistant") {
                agent.model = "".to_string();
            }
        }

        let result = config.validate();
        assert!(result.is_err(), "Empty agent model should fail validation");
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must have a model"));
    }

    #[test]
    fn test_yaml_config_empty_agent_instructions() {
        let mut config = YamlConfig::default();
        if let Some(agents) = &mut config.agents {
            if let Some(agent) = agents.get_mut("assistant") {
                agent.instructions = "".to_string();
            }
        }

        let result = config.validate();
        assert!(
            result.is_err(),
            "Empty agent instructions should fail validation"
        );
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must have instructions"));
    }

    #[test]
    fn test_yaml_config_workflow_validation() {
        // Create config with workflows
        let mut config = YamlConfig::default();
        let mut workflows = HashMap::new();

        // Test empty workflow name
        workflows.insert(
            "".to_string(),
            WorkflowConfig {
                id: Some("test".to_string()),
                name: Some("Test".to_string()),
                description: None,
                trigger: Some("manual".to_string()),
                steps: vec![WorkflowStepConfig {
                    id: Some("step1".to_string()),
                    name: Some("Step 1".to_string()),
                    description: None,
                    agent: Some("assistant".to_string()),
                    tool: None,
                    workflow: None,
                    condition: None,
                    input: None,
                    timeout: None,
                    retries: None,
                }],
                timeout: None,
                max_retries: None,
            },
        );
        config.workflows = Some(workflows.clone());

        let result = config.validate();
        assert!(
            result.is_err(),
            "Empty workflow name should fail validation"
        );

        // Test empty workflow steps
        workflows.clear();
        workflows.insert(
            "empty_workflow".to_string(),
            WorkflowConfig {
                id: Some("empty".to_string()),
                name: Some("Empty Workflow".to_string()),
                description: None,
                trigger: Some("manual".to_string()),
                steps: vec![],
                timeout: None,
                max_retries: None,
            },
        );
        config.workflows = Some(workflows.clone());

        let result = config.validate();
        assert!(
            result.is_err(),
            "Workflow with no steps should fail validation"
        );

        // Test workflow step without target
        workflows.clear();
        workflows.insert(
            "invalid_workflow".to_string(),
            WorkflowConfig {
                id: Some("invalid".to_string()),
                name: Some("Invalid Workflow".to_string()),
                description: None,
                trigger: Some("manual".to_string()),
                steps: vec![WorkflowStepConfig {
                    id: Some("step1".to_string()),
                    name: Some("Step 1".to_string()),
                    description: None,
                    agent: None,
                    tool: None,
                    workflow: None,
                    condition: None,
                    input: None,
                    timeout: None,
                    retries: None,
                }],
                timeout: None,
                max_retries: None,
            },
        );
        config.workflows = Some(workflows);

        let result = config.validate();
        assert!(
            result.is_err(),
            "Workflow step without agent/tool/workflow should fail validation"
        );
    }

    #[test]
    fn test_yaml_config_from_invalid_yaml() {
        let invalid_yaml = r#"
project:
  name: test
  invalid_field: [unclosed array
"#;

        let result = YamlConfig::from_str(invalid_yaml);
        assert!(result.is_err(), "Invalid YAML should fail to parse");
    }

    #[test]
    fn test_yaml_config_file_not_found() {
        let result = YamlConfig::from_file("/nonexistent/path/config.yaml");
        assert!(result.is_err(), "Non-existent file should fail to load");
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read YAML config file"));
    }

    #[test]
    fn test_config_loader_unknown_extension() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.txt");

        // Write YAML content with .txt extension
        let yaml_content = r#"
project:
  name: test-app
  version: 0.1.0
"#;
        fs::write(&file_path, yaml_content).unwrap();

        // Should still be able to load by detecting content
        let result = ConfigLoader::load(&file_path);
        assert!(
            result.is_ok(),
            "Should auto-detect YAML content regardless of extension"
        );
        assert_eq!(result.unwrap().project.as_ref().unwrap().name, "test-app");
    }

    #[test]
    fn test_config_loader_invalid_content() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.yaml");

        // Write invalid content
        fs::write(&file_path, "not valid yaml or toml content [[[").unwrap();

        let result = ConfigLoader::load(&file_path);
        assert!(result.is_err(), "Invalid content should fail to load");
    }

    #[test]
    fn test_yaml_config_serialization_roundtrip() {
        let config = YamlConfig::default();

        // Serialize to string
        let yaml_string = config.to_string().unwrap();
        assert!(!yaml_string.is_empty(), "YAML string should not be empty");

        // Deserialize back
        let parsed_config = YamlConfig::from_str(&yaml_string).unwrap();

        // Verify key fields match
        assert_eq!(
            parsed_config.project.as_ref().unwrap().name,
            config.project.as_ref().unwrap().name
        );
        assert_eq!(
            parsed_config.agents.as_ref().unwrap().len(),
            config.agents.as_ref().unwrap().len()
        );
    }

    #[test]
    fn test_yaml_config_get_agent() {
        let config = YamlConfig::default();

        let agent = config.get_agent("assistant");
        assert!(agent.is_some(), "Should find assistant agent");
        assert_eq!(agent.unwrap().model, "gpt-4");

        let missing_agent = config.get_agent("nonexistent");
        assert!(
            missing_agent.is_none(),
            "Should return None for missing agent"
        );
    }

    #[test]
    fn test_yaml_config_get_workflow() {
        let mut config = YamlConfig::default();

        // Add a workflow
        let mut workflows = HashMap::new();
        workflows.insert(
            "test_workflow".to_string(),
            WorkflowConfig {
                id: Some("test".to_string()),
                name: Some("Test Workflow".to_string()),
                description: None,
                trigger: Some("manual".to_string()),
                steps: vec![WorkflowStepConfig {
                    id: Some("step1".to_string()),
                    name: Some("Step 1".to_string()),
                    description: None,
                    agent: Some("assistant".to_string()),
                    tool: None,
                    workflow: None,
                    condition: None,
                    input: None,
                    timeout: None,
                    retries: None,
                }],
                timeout: None,
                max_retries: None,
            },
        );
        config.workflows = Some(workflows);

        let workflow = config.get_workflow("test_workflow");
        assert!(workflow.is_some(), "Should find test_workflow");
        assert_eq!(workflow.unwrap().steps.len(), 1);

        let missing_workflow = config.get_workflow("nonexistent");
        assert!(
            missing_workflow.is_none(),
            "Should return None for missing workflow"
        );
    }
}
