use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};
use crate::error::{Error, Result};
use crate::agent::trait_def::Agent;

/// API文档生成器
pub struct ApiDocumentationGenerator {
    output_dir: String,
    format: DocumentationFormat,
    include_examples: bool,
    include_schemas: bool,
}

/// 文档格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentationFormat {
    Markdown,
    Html,
    Json,
    OpenApi,
}

/// API文档结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDocumentation {
    pub title: String,
    pub version: String,
    pub description: String,
    pub base_url: String,
    pub endpoints: Vec<ApiEndpoint>,
    pub schemas: HashMap<String, ApiSchema>,
    pub examples: Vec<ApiExample>,
}

/// API端点文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoint {
    pub path: String,
    pub method: String,
    pub summary: String,
    pub description: String,
    pub parameters: Vec<ApiParameter>,
    pub request_body: Option<ApiRequestBody>,
    pub responses: HashMap<String, ApiResponse>,
    pub examples: Vec<ApiExample>,
}

/// API参数文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiParameter {
    pub name: String,
    pub location: ParameterLocation,
    pub description: String,
    pub required: bool,
    pub schema: ApiSchema,
    pub example: Option<serde_json::Value>,
}

/// 参数位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterLocation {
    Query,
    Path,
    Header,
    Cookie,
}

/// API请求体文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequestBody {
    pub description: String,
    pub required: bool,
    pub content: HashMap<String, ApiMediaType>,
}

/// API响应文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub description: String,
    pub headers: HashMap<String, ApiHeader>,
    pub content: HashMap<String, ApiMediaType>,
}

/// API媒体类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMediaType {
    pub schema: ApiSchema,
    pub example: Option<serde_json::Value>,
    pub examples: HashMap<String, ApiExample>,
}

/// API头部文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiHeader {
    pub description: String,
    pub required: bool,
    pub schema: ApiSchema,
}

/// API模式文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSchema {
    pub schema_type: String,
    pub format: Option<String>,
    pub description: Option<String>,
    pub properties: HashMap<String, ApiSchema>,
    pub required: Vec<String>,
    pub items: Option<Box<ApiSchema>>,
    pub example: Option<serde_json::Value>,
}

/// API示例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiExample {
    pub name: String,
    pub summary: String,
    pub description: String,
    pub value: serde_json::Value,
}

impl ApiDocumentationGenerator {
    /// 创建新的文档生成器
    pub fn new(output_dir: String, format: DocumentationFormat) -> Self {
        Self {
            output_dir,
            format,
            include_examples: true,
            include_schemas: true,
        }
    }
    
    /// 设置是否包含示例
    pub fn with_examples(mut self, include: bool) -> Self {
        self.include_examples = include;
        self
    }
    
    /// 设置是否包含模式
    pub fn with_schemas(mut self, include: bool) -> Self {
        self.include_schemas = include;
        self
    }
    
    /// 生成Agent API文档
    pub async fn generate_agent_documentation<T: Agent>(&self, agent: &T) -> Result<ApiDocumentation> {
        let mut doc = ApiDocumentation {
            title: format!("{} API Documentation", agent.get_name()),
            version: "1.0.0".to_string(),
            description: agent.get_instructions().to_string(),
            base_url: "http://localhost:8080".to_string(),
            endpoints: Vec::new(),
            schemas: HashMap::new(),
            examples: Vec::new(),
        };
        
        // 生成核心端点
        self.generate_core_endpoints(&mut doc, agent).await?;
        
        // 生成工具端点
        self.generate_tool_endpoints(&mut doc, agent)?;
        
        // 生成监控端点
        self.generate_monitoring_endpoints(&mut doc, agent).await?;
        
        // 生成模式定义
        if self.include_schemas {
            self.generate_schemas(&mut doc)?;
        }
        
        // 生成示例
        if self.include_examples {
            self.generate_examples(&mut doc, agent).await?;
        }
        
        Ok(doc)
    }
    
    /// 生成核心端点文档
    async fn generate_core_endpoints<T: Agent>(&self, doc: &mut ApiDocumentation, agent: &T) -> Result<()> {
        // Generate endpoint
        doc.endpoints.push(ApiEndpoint {
            path: "/api/v1/generate".to_string(),
            method: "POST".to_string(),
            summary: "Generate response".to_string(),
            description: "Generate a response from the agent based on input messages".to_string(),
            parameters: vec![],
            request_body: Some(ApiRequestBody {
                description: "Generation request".to_string(),
                required: true,
                content: HashMap::from([
                    ("application/json".to_string(), ApiMediaType {
                        schema: self.create_generate_request_schema(),
                        example: Some(serde_json::json!({
                            "messages": [
                                {
                                    "role": "user",
                                    "content": "Hello, how are you?"
                                }
                            ],
                            "options": {
                                "temperature": 0.7,
                                "max_tokens": 1000
                            }
                        })),
                        examples: HashMap::new(),
                    })
                ]),
            }),
            responses: HashMap::from([
                ("200".to_string(), ApiResponse {
                    description: "Successful response".to_string(),
                    headers: HashMap::new(),
                    content: HashMap::from([
                        ("application/json".to_string(), ApiMediaType {
                            schema: self.create_generate_response_schema(),
                            example: Some(serde_json::json!({
                                "response": "Hello! I'm doing well, thank you for asking.",
                                "usage": {
                                    "prompt_tokens": 10,
                                    "completion_tokens": 15,
                                    "total_tokens": 25
                                }
                            })),
                            examples: HashMap::new(),
                        })
                    ]),
                })
            ]),
            examples: vec![],
        });
        
        // Stream endpoint
        doc.endpoints.push(ApiEndpoint {
            path: "/api/v1/stream".to_string(),
            method: "POST".to_string(),
            summary: "Stream response".to_string(),
            description: "Generate a streaming response from the agent".to_string(),
            parameters: vec![],
            request_body: Some(ApiRequestBody {
                description: "Streaming request".to_string(),
                required: true,
                content: HashMap::from([
                    ("application/json".to_string(), ApiMediaType {
                        schema: self.create_stream_request_schema(),
                        example: Some(serde_json::json!({
                            "messages": [
                                {
                                    "role": "user",
                                    "content": "Tell me a story"
                                }
                            ],
                            "options": {
                                "stream": true
                            }
                        })),
                        examples: HashMap::new(),
                    })
                ]),
            }),
            responses: HashMap::from([
                ("200".to_string(), ApiResponse {
                    description: "Streaming response".to_string(),
                    headers: HashMap::new(),
                    content: HashMap::from([
                        ("text/event-stream".to_string(), ApiMediaType {
                            schema: self.create_stream_response_schema(),
                            example: Some(serde_json::json!("data: {\"chunk\": \"Once upon a time...\"}\n\n")),
                            examples: HashMap::new(),
                        })
                    ]),
                })
            ]),
            examples: vec![],
        });
        
        Ok(())
    }
    
    /// 生成工具端点文档
    fn generate_tool_endpoints<T: Agent>(&self, doc: &mut ApiDocumentation, agent: &T) -> Result<()> {
        let tools = agent.get_tools();
        
        for (tool_name, tool) in tools {
            doc.endpoints.push(ApiEndpoint {
                path: format!("/api/v1/tools/{}", tool_name),
                method: "POST".to_string(),
                summary: format!("Execute {} tool", tool_name),
                description: format!("Tool: {}", tool_name),
                parameters: vec![],
                request_body: Some(ApiRequestBody {
                    description: "Tool execution request".to_string(),
                    required: true,
                    content: HashMap::from([
                        ("application/json".to_string(), ApiMediaType {
                            schema: self.create_tool_request_schema(),
                            example: Some(serde_json::json!({
                                "parameters": {}
                            })),
                            examples: HashMap::new(),
                        })
                    ]),
                }),
                responses: HashMap::from([
                    ("200".to_string(), ApiResponse {
                        description: "Tool execution result".to_string(),
                        headers: HashMap::new(),
                        content: HashMap::from([
                            ("application/json".to_string(), ApiMediaType {
                                schema: self.create_tool_response_schema(),
                                example: Some(serde_json::json!({
                                    "result": {},
                                    "success": true
                                })),
                                examples: HashMap::new(),
                            })
                        ]),
                    })
                ]),
                examples: vec![],
            });
        }
        
        Ok(())
    }
    
    /// 生成监控端点文档
    async fn generate_monitoring_endpoints<T: Agent>(&self, doc: &mut ApiDocumentation, agent: &T) -> Result<()> {
        // Health check endpoint
        doc.endpoints.push(ApiEndpoint {
            path: "/api/v1/health".to_string(),
            method: "GET".to_string(),
            summary: "Health check".to_string(),
            description: "Check the health status of the agent".to_string(),
            parameters: vec![],
            request_body: None,
            responses: HashMap::from([
                ("200".to_string(), ApiResponse {
                    description: "Health status".to_string(),
                    headers: HashMap::new(),
                    content: HashMap::from([
                        ("application/json".to_string(), ApiMediaType {
                            schema: self.create_health_response_schema(),
                            example: Some(serde_json::json!({
                                "status": "healthy",
                                "name": agent.get_name(),
                                "has_memory": agent.has_own_memory(),
                                "tools_count": agent.get_tools().len()
                            })),
                            examples: HashMap::new(),
                        })
                    ]),
                })
            ]),
            examples: vec![],
        });
        
        // Metrics endpoint
        doc.endpoints.push(ApiEndpoint {
            path: "/api/v1/metrics".to_string(),
            method: "GET".to_string(),
            summary: "Get metrics".to_string(),
            description: "Get performance metrics for the agent".to_string(),
            parameters: vec![],
            request_body: None,
            responses: HashMap::from([
                ("200".to_string(), ApiResponse {
                    description: "Performance metrics".to_string(),
                    headers: HashMap::new(),
                    content: HashMap::from([
                        ("application/json".to_string(), ApiMediaType {
                            schema: self.create_metrics_response_schema(),
                            example: Some(serde_json::json!({})),
                            examples: HashMap::new(),
                        })
                    ]),
                })
            ]),
            examples: vec![],
        });
        
        Ok(())
    }
    
    /// 生成模式定义
    fn generate_schemas(&self, doc: &mut ApiDocumentation) -> Result<()> {
        doc.schemas.insert("Message".to_string(), self.create_message_schema());
        doc.schemas.insert("GenerateRequest".to_string(), self.create_generate_request_schema());
        doc.schemas.insert("GenerateResponse".to_string(), self.create_generate_response_schema());
        doc.schemas.insert("StreamRequest".to_string(), self.create_stream_request_schema());
        doc.schemas.insert("StreamResponse".to_string(), self.create_stream_response_schema());
        doc.schemas.insert("ToolRequest".to_string(), self.create_tool_request_schema());
        doc.schemas.insert("ToolResponse".to_string(), self.create_tool_response_schema());
        doc.schemas.insert("HealthResponse".to_string(), self.create_health_response_schema());
        doc.schemas.insert("MetricsResponse".to_string(), self.create_metrics_response_schema());
        
        Ok(())
    }
    
    /// 生成示例
    async fn generate_examples<T: Agent>(&self, doc: &mut ApiDocumentation, agent: &T) -> Result<()> {
        doc.examples.push(ApiExample {
            name: "basic_generation".to_string(),
            summary: "Basic text generation".to_string(),
            description: "A simple example of generating text with the agent".to_string(),
            value: serde_json::json!({
                "request": {
                    "messages": [
                        {
                            "role": "user",
                            "content": "What is artificial intelligence?"
                        }
                    ]
                },
                "response": {
                    "response": "Artificial intelligence (AI) is a branch of computer science..."
                }
            }),
        });
        
        Ok(())
    }
    
    // Schema creation helper methods
    fn create_message_schema(&self) -> ApiSchema {
        ApiSchema {
            schema_type: "object".to_string(),
            format: None,
            description: Some("A message in the conversation".to_string()),
            properties: HashMap::from([
                ("role".to_string(), ApiSchema {
                    schema_type: "string".to_string(),
                    format: None,
                    description: Some("The role of the message sender".to_string()),
                    properties: HashMap::new(),
                    required: vec![],
                    items: None,
                    example: Some(serde_json::json!("user")),
                }),
                ("content".to_string(), ApiSchema {
                    schema_type: "string".to_string(),
                    format: None,
                    description: Some("The content of the message".to_string()),
                    properties: HashMap::new(),
                    required: vec![],
                    items: None,
                    example: Some(serde_json::json!("Hello, how are you?")),
                }),
            ]),
            required: vec!["role".to_string(), "content".to_string()],
            items: None,
            example: None,
        }
    }
    
    fn create_generate_request_schema(&self) -> ApiSchema {
        ApiSchema {
            schema_type: "object".to_string(),
            format: None,
            description: Some("Request for text generation".to_string()),
            properties: HashMap::from([
                ("messages".to_string(), ApiSchema {
                    schema_type: "array".to_string(),
                    format: None,
                    description: Some("Array of messages".to_string()),
                    properties: HashMap::new(),
                    required: vec![],
                    items: Some(Box::new(self.create_message_schema())),
                    example: None,
                }),
            ]),
            required: vec!["messages".to_string()],
            items: None,
            example: None,
        }
    }
    
    fn create_generate_response_schema(&self) -> ApiSchema {
        ApiSchema {
            schema_type: "object".to_string(),
            format: None,
            description: Some("Response from text generation".to_string()),
            properties: HashMap::from([
                ("response".to_string(), ApiSchema {
                    schema_type: "string".to_string(),
                    format: None,
                    description: Some("Generated response text".to_string()),
                    properties: HashMap::new(),
                    required: vec![],
                    items: None,
                    example: Some(serde_json::json!("Hello! I'm doing well, thank you.")),
                }),
            ]),
            required: vec!["response".to_string()],
            items: None,
            example: None,
        }
    }
    
    fn create_stream_request_schema(&self) -> ApiSchema {
        self.create_generate_request_schema()
    }
    
    fn create_stream_response_schema(&self) -> ApiSchema {
        ApiSchema {
            schema_type: "string".to_string(),
            format: Some("event-stream".to_string()),
            description: Some("Server-sent events stream".to_string()),
            properties: HashMap::new(),
            required: vec![],
            items: None,
            example: None,
        }
    }
    
    fn create_tool_request_schema(&self) -> ApiSchema {
        ApiSchema {
            schema_type: "object".to_string(),
            format: None,
            description: Some("Tool execution request".to_string()),
            properties: HashMap::from([
                ("parameters".to_string(), ApiSchema {
                    schema_type: "object".to_string(),
                    format: None,
                    description: Some("Tool parameters".to_string()),
                    properties: HashMap::new(),
                    required: vec![],
                    items: None,
                    example: None,
                }),
            ]),
            required: vec!["parameters".to_string()],
            items: None,
            example: None,
        }
    }
    
    fn create_tool_response_schema(&self) -> ApiSchema {
        ApiSchema {
            schema_type: "object".to_string(),
            format: None,
            description: Some("Tool execution response".to_string()),
            properties: HashMap::from([
                ("result".to_string(), ApiSchema {
                    schema_type: "object".to_string(),
                    format: None,
                    description: Some("Tool execution result".to_string()),
                    properties: HashMap::new(),
                    required: vec![],
                    items: None,
                    example: None,
                }),
                ("success".to_string(), ApiSchema {
                    schema_type: "boolean".to_string(),
                    format: None,
                    description: Some("Whether the tool execution was successful".to_string()),
                    properties: HashMap::new(),
                    required: vec![],
                    items: None,
                    example: Some(serde_json::json!(true)),
                }),
            ]),
            required: vec!["result".to_string(), "success".to_string()],
            items: None,
            example: None,
        }
    }
    
    fn create_health_response_schema(&self) -> ApiSchema {
        ApiSchema {
            schema_type: "object".to_string(),
            format: None,
            description: Some("Health check response".to_string()),
            properties: HashMap::from([
                ("status".to_string(), ApiSchema {
                    schema_type: "string".to_string(),
                    format: None,
                    description: Some("Health status".to_string()),
                    properties: HashMap::new(),
                    required: vec![],
                    items: None,
                    example: Some(serde_json::json!("healthy")),
                }),
            ]),
            required: vec!["status".to_string()],
            items: None,
            example: None,
        }
    }
    
    fn create_metrics_response_schema(&self) -> ApiSchema {
        ApiSchema {
            schema_type: "object".to_string(),
            format: None,
            description: Some("Performance metrics response".to_string()),
            properties: HashMap::new(),
            required: vec![],
            items: None,
            example: None,
        }
    }
    
    /// 保存文档到文件
    pub async fn save_documentation(&self, doc: &ApiDocumentation) -> Result<()> {
        // 确保输出目录存在
        fs::create_dir_all(&self.output_dir)
            .map_err(|e| Error::Io(e))?;
        
        match self.format {
            DocumentationFormat::Markdown => self.save_as_markdown(doc).await,
            DocumentationFormat::Html => self.save_as_html(doc).await,
            DocumentationFormat::Json => self.save_as_json(doc).await,
            DocumentationFormat::OpenApi => self.save_as_openapi(doc).await,
        }
    }
    
    /// 保存为Markdown格式
    async fn save_as_markdown(&self, doc: &ApiDocumentation) -> Result<()> {
        let mut content = String::new();
        
        // 标题和描述
        content.push_str(&format!("# {}\n\n", doc.title));
        content.push_str(&format!("Version: {}\n\n", doc.version));
        content.push_str(&format!("{}\n\n", doc.description));
        content.push_str(&format!("Base URL: `{}`\n\n", doc.base_url));
        
        // 端点文档
        content.push_str("## API Endpoints\n\n");
        for endpoint in &doc.endpoints {
            content.push_str(&format!("### {} {}\n\n", endpoint.method, endpoint.path));
            content.push_str(&format!("**Summary:** {}\n\n", endpoint.summary));
            content.push_str(&format!("{}\n\n", endpoint.description));
            
            if let Some(request_body) = &endpoint.request_body {
                content.push_str("**Request Body:**\n\n");
                content.push_str(&format!("- Required: {}\n", request_body.required));
                content.push_str(&format!("- Description: {}\n\n", request_body.description));
            }
            
            content.push_str("**Responses:**\n\n");
            for (status, response) in &endpoint.responses {
                content.push_str(&format!("- **{}**: {}\n", status, response.description));
            }
            content.push_str("\n");
        }
        
        // 保存文件
        let file_path = Path::new(&self.output_dir).join("api_documentation.md");
        fs::write(file_path, content)
            .map_err(|e| Error::Io(e))?;
        
        Ok(())
    }
    
    /// 保存为HTML格式
    async fn save_as_html(&self, doc: &ApiDocumentation) -> Result<()> {
        let mut content = String::new();
        
        content.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        content.push_str(&format!("<title>{}</title>\n", doc.title));
        content.push_str("<style>\nbody { font-family: Arial, sans-serif; margin: 40px; }\n");
        content.push_str("h1, h2, h3 { color: #333; }\n");
        content.push_str("code { background-color: #f4f4f4; padding: 2px 4px; }\n");
        content.push_str("pre { background-color: #f4f4f4; padding: 10px; overflow-x: auto; }\n");
        content.push_str("</style>\n</head>\n<body>\n");
        
        content.push_str(&format!("<h1>{}</h1>\n", doc.title));
        content.push_str(&format!("<p><strong>Version:</strong> {}</p>\n", doc.version));
        content.push_str(&format!("<p>{}</p>\n", doc.description));
        content.push_str(&format!("<p><strong>Base URL:</strong> <code>{}</code></p>\n", doc.base_url));
        
        content.push_str("<h2>API Endpoints</h2>\n");
        for endpoint in &doc.endpoints {
            content.push_str(&format!("<h3>{} {}</h3>\n", endpoint.method, endpoint.path));
            content.push_str(&format!("<p><strong>Summary:</strong> {}</p>\n", endpoint.summary));
            content.push_str(&format!("<p>{}</p>\n", endpoint.description));
        }
        
        content.push_str("</body>\n</html>");
        
        let file_path = Path::new(&self.output_dir).join("api_documentation.html");
        fs::write(file_path, content)
            .map_err(|e| Error::Io(e))?;
        
        Ok(())
    }
    
    /// 保存为JSON格式
    async fn save_as_json(&self, doc: &ApiDocumentation) -> Result<()> {
        let json_content = serde_json::to_string_pretty(doc)
            .map_err(|e| Error::Serialization(format!("Failed to serialize documentation: {}", e)))?;
        
        let file_path = Path::new(&self.output_dir).join("api_documentation.json");
        fs::write(file_path, json_content)
            .map_err(|e| Error::Io(e))?;
        
        Ok(())
    }
    
    /// 保存为OpenAPI格式
    async fn save_as_openapi(&self, doc: &ApiDocumentation) -> Result<()> {
        let openapi_doc = serde_json::json!({
            "openapi": "3.0.0",
            "info": {
                "title": doc.title,
                "version": doc.version,
                "description": doc.description
            },
            "servers": [
                {
                    "url": doc.base_url
                }
            ],
            "paths": {},
            "components": {
                "schemas": doc.schemas
            }
        });
        
        let json_content = serde_json::to_string_pretty(&openapi_doc)
            .map_err(|e| Error::Serialization(format!("Failed to serialize OpenAPI doc: {}", e)))?;
        
        let file_path = Path::new(&self.output_dir).join("openapi.json");
        fs::write(file_path, json_content)
            .map_err(|e| Error::Io(e))?;
        
        Ok(())
    }
}
