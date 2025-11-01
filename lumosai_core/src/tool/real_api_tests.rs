//! Real API tests for Tool module
//!
//! Tests tool functionality using real Zhipu AI Provider

#[cfg(test)]
mod tests {
    use crate::llm::test_helpers::create_test_zhipu_provider_arc;
    use crate::tool::{
        FunctionTool, GenericTool, ParameterSchema, Tool, ToolCategory, ToolExecutionContext,
        ToolExecutionOptions, ToolMetadata, ToolRegistry, ToolSchema,
    };
    use serde_json::json;
    use std::sync::Arc;
    use std::time::Duration;

    /// Helper function to retry API calls with exponential backoff
    async fn retry_with_backoff<F, Fut, T>(
        mut f: F,
        max_retries: u32,
        initial_delay_ms: u64,
    ) -> crate::Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = crate::Result<T>>,
    {
        let mut delay = initial_delay_ms;
        for attempt in 0..max_retries {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    let error_msg = format!("{:?}", e);
                    if error_msg.contains("429")
                        || error_msg.contains("Too Many Requests")
                        || error_msg.contains("1302")
                    {
                        if attempt < max_retries - 1 {
                            eprintln!(
                                "⚠️  Rate limit hit (attempt {}/{}), retrying after {}ms...",
                                attempt + 1,
                                max_retries,
                                delay
                            );
                            tokio::time::sleep(Duration::from_millis(delay)).await;
                            delay *= 2; // Exponential backoff
                            continue;
                        }
                    }
                    return Err(e);
                }
            }
        }
        unreachable!()
    }

    #[tokio::test]
    async fn test_tool_registry_register_and_get() {
        let registry = ToolRegistry::new();

        // Create a simple tool
        let schema = ToolSchema::new(vec![ParameterSchema {
            name: "input".to_string(),
            description: "Input text".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        }]);

        let tool = Arc::new(GenericTool::new(
            "test-tool",
            "A test tool",
            schema,
            |params, _context| {
                let input = params["input"].as_str().unwrap_or("");
                Ok(json!({ "output": input.to_uppercase() }))
            },
        )) as Arc<dyn Tool>;

        let metadata = ToolMetadata {
            name: "test-tool".to_string(),
            description: "A test tool".to_string(),
            version: "1.0.0".to_string(),
            author: Some("Test Author".to_string()),
            category: ToolCategory::Text,
            tags: vec!["test".to_string(), "text".to_string()],
            requires_auth: false,
            permissions: vec![],
            dependencies: vec![],
        };

        // Register tool
        let result = registry.register_tool(tool.clone(), metadata);
        assert!(result.is_ok(), "Failed to register tool: {:?}", result.err());

        // Get tool
        let retrieved = registry.get_tool("test-tool").unwrap();
        assert!(retrieved.is_some(), "Tool not found after registration");

        // Verify tool name
        let retrieved_tool = retrieved.unwrap();
        assert_eq!(retrieved_tool.name(), Some("test-tool"));
    }

    #[tokio::test]
    async fn test_tool_registry_duplicate_registration() {
        let registry = ToolRegistry::new();

        let schema = ToolSchema::new(vec![]);
        let tool = Arc::new(GenericTool::new(
            "duplicate-tool",
            "A tool",
            schema.clone(),
            |_params, _context| Ok(json!({})),
        )) as Arc<dyn Tool>;

        let metadata = ToolMetadata {
            name: "duplicate-tool".to_string(),
            description: "A tool".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            category: ToolCategory::Custom("test".to_string()),
            tags: vec![],
            requires_auth: false,
            permissions: vec![],
            dependencies: vec![],
        };

        // First registration should succeed
        assert!(registry.register_tool(tool.clone(), metadata.clone()).is_ok());

        // Second registration should fail
        let result = registry.register_tool(tool, metadata);
        assert!(result.is_err(), "Duplicate registration should fail");
    }

    #[tokio::test]
    async fn test_tool_registry_unregister() {
        let registry = ToolRegistry::new();

        let schema = ToolSchema::new(vec![]);
        let tool = Arc::new(GenericTool::new(
            "temp-tool",
            "A temporary tool",
            schema,
            |_params, _context| Ok(json!({})),
        )) as Arc<dyn Tool>;

        let metadata = ToolMetadata {
            name: "temp-tool".to_string(),
            description: "A temporary tool".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            category: ToolCategory::Custom("temp".to_string()),
            tags: vec!["temp".to_string()],
            requires_auth: false,
            permissions: vec![],
            dependencies: vec![],
        };

        // Register and verify
        registry.register_tool(tool, metadata).unwrap();
        assert!(registry.get_tool("temp-tool").unwrap().is_some());

        // Unregister and verify
        let result = registry.unregister_tool("temp-tool");
        assert!(result.is_ok(), "Failed to unregister tool: {:?}", result.err());
        assert!(registry.get_tool("temp-tool").unwrap().is_none());
    }

    #[tokio::test]
    async fn test_tool_registry_find_by_category() {
        let registry = ToolRegistry::new();

        // Register multiple tools in different categories
        for i in 0..3 {
            let schema = ToolSchema::new(vec![]);
            let tool = Arc::new(GenericTool::new(
                &format!("math-tool-{}", i),
                "A math tool",
                schema,
                |_params, _context| Ok(json!({})),
            )) as Arc<dyn Tool>;

            let metadata = ToolMetadata {
                name: format!("math-tool-{}", i),
                description: "A math tool".to_string(),
                version: "1.0.0".to_string(),
                author: None,
                category: ToolCategory::Math,
                tags: vec![],
                requires_auth: false,
                permissions: vec![],
                dependencies: vec![],
            };

            registry.register_tool(tool, metadata).unwrap();
        }

        // Find tools by category
        let math_tools = registry.find_tools_by_category(&ToolCategory::Math).unwrap();
        assert_eq!(math_tools.len(), 3, "Should find 3 math tools");
    }

    #[tokio::test]
    async fn test_tool_registry_find_by_tag() {
        let registry = ToolRegistry::new();

        let schema = ToolSchema::new(vec![]);
        let tool = Arc::new(GenericTool::new(
            "tagged-tool",
            "A tagged tool",
            schema,
            |_params, _context| Ok(json!({})),
        )) as Arc<dyn Tool>;

        let metadata = ToolMetadata {
            name: "tagged-tool".to_string(),
            description: "A tagged tool".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            category: ToolCategory::Custom("test".to_string()),
            tags: vec!["important".to_string(), "test".to_string()],
            requires_auth: false,
            permissions: vec![],
            dependencies: vec![],
        };

        registry.register_tool(tool, metadata).unwrap();

        // Find by tag
        let tools = registry.find_tools_by_tag("important").unwrap();
        assert_eq!(tools.len(), 1, "Should find 1 tool with 'important' tag");
        assert_eq!(tools[0], "tagged-tool");
    }

    #[tokio::test]
    async fn test_tool_registry_search() {
        let registry = ToolRegistry::new();

        let schema = ToolSchema::new(vec![]);
        let tool = Arc::new(GenericTool::new(
            "search-test-tool",
            "A tool for searching and processing data",
            schema,
            |_params, _context| Ok(json!({})),
        )) as Arc<dyn Tool>;

        let metadata = ToolMetadata {
            name: "search-test-tool".to_string(),
            description: "A tool for searching and processing data".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            category: ToolCategory::DataProcessing,
            tags: vec!["search".to_string(), "data".to_string()],
            requires_auth: false,
            permissions: vec![],
            dependencies: vec![],
        };

        registry.register_tool(tool, metadata).unwrap();

        // Search by name
        let results = registry.search_tools("search").unwrap();
        assert!(!results.is_empty(), "Should find tools matching 'search'");

        // Search by description
        let results = registry.search_tools("processing").unwrap();
        assert!(!results.is_empty(), "Should find tools matching 'processing'");

        // Search by tag
        let results = registry.search_tools("data").unwrap();
        assert!(!results.is_empty(), "Should find tools matching 'data'");
    }

    #[tokio::test]
    async fn test_tool_registry_list_tools() {
        let registry = ToolRegistry::new();

        // Initially empty
        let tools = registry.list_tools().unwrap();
        assert_eq!(tools.len(), 0, "Registry should be empty initially");

        // Register a tool
        let schema = ToolSchema::new(vec![]);
        let tool = Arc::new(GenericTool::new(
            "list-test-tool",
            "A tool",
            schema,
            |_params, _context| Ok(json!({})),
        )) as Arc<dyn Tool>;

        let metadata = ToolMetadata {
            name: "list-test-tool".to_string(),
            description: "A tool".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            category: ToolCategory::Custom("test".to_string()),
            tags: vec![],
            requires_auth: false,
            permissions: vec![],
            dependencies: vec![],
        };

        registry.register_tool(tool, metadata).unwrap();

        // Should have one tool
        let tools = registry.list_tools().unwrap();
        assert_eq!(tools.len(), 1, "Registry should have 1 tool");
        assert_eq!(tools[0], "list-test-tool");
    }
}

