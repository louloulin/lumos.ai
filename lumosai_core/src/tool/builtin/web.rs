//! Web-related tools inspired by Mastra's web tools
//! 
//! This module provides HTTP request, web scraping, and API interaction tools

use crate::tool::{Tool, ToolSchema, ParameterSchema, FunctionTool, ToolExecutionContext, ToolExecutionOptions};
use serde_json::{Value, json};
use std::collections::HashMap;
use async_trait::async_trait;
use crate::{Result, Error};
use crate::base::Base;

/// Create an HTTP request tool
/// Similar to Mastra's fetch tool
pub fn create_http_request_tool() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "url".to_string(),
            description: "The URL to request".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "method".to_string(),
            description: "HTTP method (GET, POST, PUT, DELETE)".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: Some(json!("GET")),
        },
        ParameterSchema {
            name: "headers".to_string(),
            description: "HTTP headers as JSON object".to_string(),
            r#type: "object".to_string(),
            required: false,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "body".to_string(),
            description: "Request body for POST/PUT requests".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: None,
        },
    ]);

    FunctionTool::new(
        "http_request",
        "Make HTTP requests to web APIs and websites",
        schema,
        |params| {
            let url = params.get("url")
                .and_then(|v| v.as_str())
                .ok_or("URL is required")?;
            
            let method = params.get("method")
                .and_then(|v| v.as_str())
                .unwrap_or("GET");

            // For now, return a mock response
            // In a real implementation, this would use reqwest or similar
            Ok(json!({
                "status": 200,
                "url": url,
                "method": method,
                "body": format!("Mock response for {} {}", method, url),
                "headers": {
                    "content-type": "application/json"
                }
            }))
        },
    )
}

/// Create a web scraper tool
/// Similar to Mastra's scraping capabilities
pub fn create_web_scraper_tool() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "url".to_string(),
            description: "The URL to scrape".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "selector".to_string(),
            description: "CSS selector to extract specific elements".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "extract_text".to_string(),
            description: "Whether to extract only text content".to_string(),
            r#type: "boolean".to_string(),
            required: false,
            properties: None,
            default: Some(json!(true)),
        },
    ]);

    FunctionTool::new(
        "web_scraper",
        "Scrape content from web pages",
        schema,
        |params| {
            let url = params.get("url")
                .and_then(|v| v.as_str())
                .ok_or("URL is required")?;
            
            let selector = params.get("selector")
                .and_then(|v| v.as_str());
            
            let extract_text = params.get("extract_text")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            // Mock scraping result
            Ok(json!({
                "url": url,
                "selector": selector,
                "extract_text": extract_text,
                "content": format!("Mock scraped content from {}", url),
                "elements_found": 5,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        },
    )
}

/// Create a JSON API tool
/// Specialized tool for JSON API interactions
pub fn create_json_api_tool() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "url".to_string(),
            description: "The API endpoint URL".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "method".to_string(),
            description: "HTTP method".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: Some(json!("GET")),
        },
        ParameterSchema {
            name: "data".to_string(),
            description: "JSON data to send".to_string(),
            r#type: "object".to_string(),
            required: false,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "auth_token".to_string(),
            description: "Bearer token for authentication".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: None,
        },
    ]);

    FunctionTool::new(
        "json_api",
        "Make JSON API requests with automatic parsing",
        schema,
        |params| {
            let url = params.get("url")
                .and_then(|v| v.as_str())
                .ok_or("URL is required")?;
            
            let method = params.get("method")
                .and_then(|v| v.as_str())
                .unwrap_or("GET");
            
            let data = params.get("data");
            let auth_token = params.get("auth_token").and_then(|v| v.as_str());

            // Mock API response
            Ok(json!({
                "success": true,
                "url": url,
                "method": method,
                "data": data,
                "has_auth": auth_token.is_some(),
                "response": {
                    "message": "Mock API response",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }
            }))
        },
    )
}

/// Create a URL validator tool
pub fn create_url_validator_tool() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "url".to_string(),
            description: "The URL to validate".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
    ]);

    FunctionTool::new(
        "url_validator",
        "Validate URL format and accessibility",
        schema,
        |params| {
            let url = params.get("url")
                .and_then(|v| v.as_str())
                .ok_or("URL is required")?;

            // Basic URL validation
            let is_valid = url.starts_with("http://") || url.starts_with("https://");
            let has_domain = url.contains(".");

            Ok(json!({
                "url": url,
                "is_valid": is_valid && has_domain,
                "has_protocol": url.starts_with("http"),
                "has_domain": has_domain,
                "validation_details": {
                    "protocol": if url.starts_with("https://") { "https" } else if url.starts_with("http://") { "http" } else { "none" },
                    "length": url.len()
                }
            }))
        },
    )
}

/// Web search tool for searching the internet
#[derive(Clone)]
pub struct WebSearchTool {
    base: crate::base::BaseComponent,
    id: String,
    description: String,
    schema: ToolSchema,
}

impl WebSearchTool {
    /// Create a new web search tool
    pub fn new() -> Self {
        let schema = ToolSchema::new(vec![
            ParameterSchema {
                name: "query".to_string(),
                description: "Search query to execute".to_string(),
                r#type: "string".to_string(),
                required: true,
                properties: None,
                default: None,
            },
            ParameterSchema {
                name: "max_results".to_string(),
                description: "Maximum number of results to return".to_string(),
                r#type: "integer".to_string(),
                required: false,
                properties: None,
                default: Some(json!(10)),
            },
        ]);

        Self {
            base: crate::base::BaseComponent::new_with_name(
                "web_search".to_string(),
                crate::logger::Component::Tool
            ),
            id: "web_search".to_string(),
            description: "Search the web for information".to_string(),
            schema,
        }
    }
}

impl std::fmt::Debug for WebSearchTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebSearchTool")
            .field("id", &self.id)
            .field("description", &self.description)
            .finish()
    }
}

impl Base for WebSearchTool {
    fn name(&self) -> Option<&str> {
        self.base.name()
    }

    fn component(&self) -> crate::logger::Component {
        self.base.component()
    }

    fn logger(&self) -> std::sync::Arc<dyn crate::logger::Logger> {
        self.base.logger()
    }

    fn set_logger(&mut self, logger: std::sync::Arc<dyn crate::logger::Logger>) {
        self.base.set_logger(logger);
    }

    fn telemetry(&self) -> Option<std::sync::Arc<dyn crate::telemetry::TelemetrySink>> {
        self.base.telemetry()
    }

    fn set_telemetry(&mut self, telemetry: std::sync::Arc<dyn crate::telemetry::TelemetrySink>) {
        self.base.set_telemetry(telemetry);
    }
}

#[async_trait]
impl Tool for WebSearchTool {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn schema(&self) -> ToolSchema {
        self.schema.clone()
    }

    async fn execute(
        &self,
        params: Value,
        _context: ToolExecutionContext,
        _options: &ToolExecutionOptions
    ) -> Result<Value> {
        let query = params.get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Tool("Query parameter is required".to_string()))?;

        let max_results = params.get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(10);

        // Mock implementation - in a real implementation, this would call a search API
        let results = vec![
            json!({
                "title": format!("Search result 1 for '{}'", query),
                "url": "https://example.com/1",
                "snippet": "This is a mock search result snippet..."
            }),
            json!({
                "title": format!("Search result 2 for '{}'", query),
                "url": "https://example.com/2",
                "snippet": "Another mock search result snippet..."
            }),
        ];

        Ok(json!({
            "query": query,
            "results": results[..std::cmp::min(results.len(), max_results as usize)],
            "total_results": results.len()
        }))
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }
}

impl Default for WebSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a web search tool function
pub fn create_web_search_tool() -> WebSearchTool {
    WebSearchTool::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_request_tool() {
        let tool = create_http_request_tool();
        
        let mut params = HashMap::new();
        params.insert("url".to_string(), json!("https://api.example.com/data"));
        params.insert("method".to_string(), json!("GET"));

        let context = crate::tool::context::ToolExecutionContext::new();
        let options = crate::tool::schema::ToolExecutionOptions::new();
        let result = tool.execute(serde_json::to_value(&params).unwrap(), context, &options).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response["status"], 200);
        assert_eq!(response["url"], "https://api.example.com/data");
        assert_eq!(response["method"], "GET");
    }

    #[tokio::test]
    async fn test_web_scraper_tool() {
        let tool = create_web_scraper_tool();
        
        let mut params = HashMap::new();
        params.insert("url".to_string(), json!("https://example.com"));
        params.insert("selector".to_string(), json!("h1"));

        let context = crate::tool::context::ToolExecutionContext::new();
        let options = crate::tool::schema::ToolExecutionOptions::new();
        let result = tool.execute(serde_json::to_value(&params).unwrap(), context, &options).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response["url"], "https://example.com");
        assert_eq!(response["selector"], "h1");
    }

    #[tokio::test]
    async fn test_url_validator_tool() {
        let tool = create_url_validator_tool();
        
        let mut params = HashMap::new();
        params.insert("url".to_string(), json!("https://example.com"));

        let context = crate::tool::context::ToolExecutionContext::new();
        let options = crate::tool::schema::ToolExecutionOptions::new();
        let result = tool.execute(serde_json::to_value(&params).unwrap(), context, &options).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response["is_valid"], true);
        assert_eq!(response["has_protocol"], true);
    }

    #[tokio::test]
    async fn test_web_search_tool() {
        let tool = WebSearchTool::new();

        assert_eq!(tool.name(), Some("web_search"));
        assert_eq!(tool.description(), "Search the web for information");

        let params = json!({
            "query": "rust programming",
            "max_results": 5
        });

        let context = crate::tool::ToolExecutionContext::default();
        let options = crate::tool::ToolExecutionOptions::default();

        let result = tool.execute(params, context, &options).await.unwrap();
        assert!(result.get("query").is_some());
        assert!(result.get("results").is_some());
        assert!(result.get("total_results").is_some());
    }

    #[tokio::test]
    async fn test_web_search_tool_missing_query() {
        let tool = WebSearchTool::new();

        let params = json!({
            "max_results": 5
        });

        let context = crate::tool::ToolExecutionContext::default();
        let options = crate::tool::ToolExecutionOptions::default();

        let result = tool.execute(params, context, &options).await;
        assert!(result.is_err());
    }
}
