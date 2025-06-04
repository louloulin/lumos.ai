//! Project templates for Lumos.ai CLI
//! 
//! This module provides various project templates for different use cases

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub files: Vec<TemplateFile>,
    pub dependencies: Vec<String>,
    pub tools: Vec<String>,
}

/// Template category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateCategory {
    Basic,
    WebAgent,
    DataAgent,
    ChatBot,
    Enterprise,
    Custom,
}

/// Template file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFile {
    pub path: String,
    pub content: String,
    pub executable: bool,
}

/// Template registry
pub struct TemplateRegistry {
    templates: HashMap<String, Template>,
}

impl TemplateRegistry {
    /// Create a new template registry
    pub fn new() -> Self {
        let mut registry = Self {
            templates: HashMap::new(),
        };
        
        registry.register_builtin_templates();
        registry
    }

    /// Register built-in templates
    fn register_builtin_templates(&mut self) {
        // Basic template
        self.templates.insert("basic".to_string(), Template {
            name: "basic".to_string(),
            description: "A basic Lumos.ai agent project".to_string(),
            category: TemplateCategory::Basic,
            files: vec![],
            dependencies: vec!["lumosai".to_string()],
            tools: vec![],
        });

        // Web agent template
        self.templates.insert("web-agent".to_string(), Template {
            name: "web-agent".to_string(),
            description: "A web-enabled agent with HTTP and search capabilities".to_string(),
            category: TemplateCategory::WebAgent,
            files: vec![],
            dependencies: vec!["lumosai".to_string(), "reqwest".to_string()],
            tools: vec!["web_search".to_string(), "http_request".to_string()],
        });

        // Data agent template
        self.templates.insert("data-agent".to_string(), Template {
            name: "data-agent".to_string(),
            description: "A data processing agent with analytics capabilities".to_string(),
            category: TemplateCategory::DataAgent,
            files: vec![],
            dependencies: vec!["lumosai".to_string(), "csv".to_string(), "serde".to_string()],
            tools: vec!["csv_reader".to_string(), "data_analyzer".to_string()],
        });

        // Chatbot template
        self.templates.insert("chat-bot".to_string(), Template {
            name: "chat-bot".to_string(),
            description: "An interactive chatbot with memory".to_string(),
            category: TemplateCategory::ChatBot,
            files: vec![],
            dependencies: vec!["lumosai".to_string()],
            tools: vec!["memory".to_string(), "calculator".to_string()],
        });
    }

    /// Get a template by name
    pub fn get_template(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }

    /// List all available templates
    pub fn list_templates(&self) -> Vec<&Template> {
        self.templates.values().collect()
    }

    /// Get templates by category
    pub fn get_templates_by_category(&self, category: TemplateCategory) -> Vec<&Template> {
        self.templates.values()
            .filter(|t| std::mem::discriminant(&t.category) == std::mem::discriminant(&category))
            .collect()
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}
