// swagger.rs - OpenAPI 文档生成
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OpenAPI 规范
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenApiSpec {
    pub openapi: String,
    pub info: OpenApiInfo,
    pub servers: Vec<OpenApiServer>,
    pub paths: HashMap<String, HashMap<String, OpenApiPathItem>>,
    pub components: OpenApiComponents,
}

/// OpenAPI 信息
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenApiInfo {
    pub title: String,
    pub description: String,
    pub version: String,
}

/// OpenAPI 服务器
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenApiServer {
    pub url: String,
    pub description: Option<String>,
}

/// OpenAPI 路径项
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenApiPathItem {
    pub summary: Option<String>,
    pub description: Option<String>,
    pub operationId: String,
    pub parameters: Option<Vec<OpenApiParameter>>,
    pub requestBody: Option<OpenApiRequestBody>,
    pub responses: HashMap<String, OpenApiResponse>,
}

/// OpenAPI 参数
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenApiParameter {
    pub name: String,
    pub in_type: String,
    pub description: Option<String>,
    pub required: Option<bool>,
    pub schema: OpenApiSchema,
}

/// OpenAPI 请求体
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenApiRequestBody {
    pub description: Option<String>,
    pub required: Option<bool>,
    pub content: HashMap<String, OpenApiMediaType>,
}

/// OpenAPI 媒体类型
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenApiMediaType {
    pub schema: OpenApiSchema,
}

/// OpenAPI 响应
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenApiResponse {
    pub description: String,
    pub content: Option<HashMap<String, OpenApiMediaType>>,
}

/// OpenAPI 模式
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenApiSchema {
    #[serde(rename = "type")]
    pub schema_type: Option<String>,
    pub format: Option<String>,
    pub properties: Option<HashMap<String, OpenApiSchema>>,
    pub items: Option<Box<OpenApiSchema>>,
    #[serde(rename = "enum")]
    pub enum_values: Option<Vec<String>>,
    pub required: Option<Vec<String>>,
    #[serde(rename = "$ref")]
    pub ref_path: Option<String>,
}

/// OpenAPI 组件
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenApiComponents {
    pub schemas: HashMap<String, OpenApiSchema>,
}

/// 生成 OpenAPI 规范
pub fn generate_open_api_spec(server_port: u16) -> OpenApiSpec {
    let base_url = format!("http://localhost:{}", server_port);
    
    // 创建基本规范
    let mut spec = OpenApiSpec {
        openapi: "3.0.0".to_string(),
        info: OpenApiInfo {
            title: "Lumosai API".to_string(),
            description: "Lumosai AI 框架的 REST API".to_string(),
            version: "1.0.0".to_string(),
        },
        servers: vec![
            OpenApiServer {
                url: base_url,
                description: Some("本地开发服务器".to_string()),
            }
        ],
        paths: HashMap::new(),
        components: OpenApiComponents {
            schemas: HashMap::new(),
        },
    };
    
    // 添加路径
    add_agent_routes(&mut spec);
    add_tool_routes(&mut spec);
    add_workflow_routes(&mut spec);
    add_system_routes(&mut spec);
    
    // 添加组件
    add_schema_components(&mut spec);
    
    spec
}

/// 添加系统路由
fn add_system_routes(spec: &mut OpenApiSpec) {
    // GET /api - API 状态
    spec.paths.insert(
        "/api".to_string(),
        HashMap::from([(
            "get".to_string(),
            OpenApiPathItem {
                summary: Some("获取 API 状态".to_string()),
                description: Some("返回 API 的状态和版本信息".to_string()),
                operationId: "getApiStatus".to_string(),
                parameters: None,
                requestBody: None,
                responses: HashMap::from([(
                    "200".to_string(),
                    OpenApiResponse {
                        description: "API 状态".to_string(),
                        content: Some(HashMap::from([(
                            "application/json".to_string(),
                            OpenApiMediaType {
                                schema: OpenApiSchema {
                                    schema_type: Some("object".to_string()),
                                    format: None,
                                    properties: Some(HashMap::from([
                                        (
                                            "status".to_string(),
                                            OpenApiSchema {
                                                schema_type: Some("string".to_string()),
                                                format: None,
                                                properties: None,
                                                items: None,
                                                enum_values: None,
                                                required: None,
                                                ref_path: None,
                                            }
                                        ),
                                        (
                                            "version".to_string(),
                                            OpenApiSchema {
                                                schema_type: Some("string".to_string()),
                                                format: None,
                                                properties: None,
                                                items: None,
                                                enum_values: None,
                                                required: None,
                                                ref_path: None,
                                            }
                                        ),
                                    ])),
                                    items: None,
                                    enum_values: None,
                                    required: None,
                                    ref_path: None,
                                }
                            }
                        )])),
                    }
                )]),
            }
        )]),
    );
}

/// 添加智能体路由
fn add_agent_routes(spec: &mut OpenApiSpec) {
    // GET /api/agents - 获取所有智能体
    spec.paths.insert(
        "/api/agents".to_string(),
        HashMap::from([(
            "get".to_string(),
            OpenApiPathItem {
                summary: Some("获取所有智能体".to_string()),
                description: Some("返回所有注册的智能体列表".to_string()),
                operationId: "getAgents".to_string(),
                parameters: None,
                requestBody: None,
                responses: HashMap::from([(
                    "200".to_string(),
                    OpenApiResponse {
                        description: "智能体列表".to_string(),
                        content: Some(HashMap::from([(
                            "application/json".to_string(),
                            OpenApiMediaType {
                                schema: OpenApiSchema {
                                    schema_type: Some("array".to_string()),
                                    format: None,
                                    properties: None,
                                    items: Some(Box::new(OpenApiSchema {
                                        schema_type: None,
                                        format: None,
                                        properties: None,
                                        items: None,
                                        enum_values: None,
                                        required: None,
                                        ref_path: Some("#/components/schemas/Agent".to_string()),
                                    })),
                                    enum_values: None,
                                    required: None,
                                    ref_path: None,
                                }
                            }
                        )])),
                    }
                )]),
            }
        )]),
    );
    
    // POST /api/agents/:agentId/generate - 生成响应
    spec.paths.insert(
        "/api/agents/{agentId}/generate".to_string(),
        HashMap::from([(
            "post".to_string(),
            OpenApiPathItem {
                summary: Some("生成智能体响应".to_string()),
                description: Some("向指定的智能体发送请求并生成响应".to_string()),
                operationId: "generateAgentResponse".to_string(),
                parameters: Some(vec![
                    OpenApiParameter {
                        name: "agentId".to_string(),
                        in_type: "path".to_string(),
                        description: Some("智能体 ID".to_string()),
                        required: Some(true),
                        schema: OpenApiSchema {
                            schema_type: Some("string".to_string()),
                            format: None,
                            properties: None,
                            items: None,
                            enum_values: None,
                            required: None,
                            ref_path: None,
                        }
                    }
                ]),
                requestBody: Some(OpenApiRequestBody {
                    description: Some("生成请求".to_string()),
                    required: Some(true),
                    content: HashMap::from([(
                        "application/json".to_string(),
                        OpenApiMediaType {
                            schema: OpenApiSchema {
                                schema_type: None,
                                format: None,
                                properties: None,
                                items: None,
                                enum_values: None,
                                required: None,
                                ref_path: Some("#/components/schemas/GenerateRequest".to_string()),
                            }
                        }
                    )]),
                }),
                responses: HashMap::from([(
                    "200".to_string(),
                    OpenApiResponse {
                        description: "智能体响应".to_string(),
                        content: Some(HashMap::from([(
                            "application/json".to_string(),
                            OpenApiMediaType {
                                schema: OpenApiSchema {
                                    schema_type: None,
                                    format: None,
                                    properties: None,
                                    items: None,
                                    enum_values: None,
                                    required: None,
                                    ref_path: Some("#/components/schemas/GenerateResponse".to_string()),
                                }
                            }
                        )])),
                    }
                )]),
            }
        )]),
    );
}

/// 添加工具路由
fn add_tool_routes(spec: &mut OpenApiSpec) {
    // GET /api/tools - 获取所有工具
    spec.paths.insert(
        "/api/tools".to_string(),
        HashMap::from([(
            "get".to_string(),
            OpenApiPathItem {
                summary: Some("获取所有工具".to_string()),
                description: Some("返回所有注册的工具列表".to_string()),
                operationId: "getTools".to_string(),
                parameters: None,
                requestBody: None,
                responses: HashMap::from([(
                    "200".to_string(),
                    OpenApiResponse {
                        description: "工具列表".to_string(),
                        content: Some(HashMap::from([(
                            "application/json".to_string(),
                            OpenApiMediaType {
                                schema: OpenApiSchema {
                                    schema_type: Some("array".to_string()),
                                    format: None,
                                    properties: None,
                                    items: Some(Box::new(OpenApiSchema {
                                        schema_type: None,
                                        format: None,
                                        properties: None,
                                        items: None,
                                        enum_values: None,
                                        required: None,
                                        ref_path: Some("#/components/schemas/Tool".to_string()),
                                    })),
                                    enum_values: None,
                                    required: None,
                                    ref_path: None,
                                }
                            }
                        )])),
                    }
                )]),
            }
        )]),
    );
    
    // POST /api/tools/:toolId/execute - 执行工具
    spec.paths.insert(
        "/api/tools/{toolId}/execute".to_string(),
        HashMap::from([(
            "post".to_string(),
            OpenApiPathItem {
                summary: Some("执行工具".to_string()),
                description: Some("执行指定的工具并返回结果".to_string()),
                operationId: "executeTool".to_string(),
                parameters: Some(vec![
                    OpenApiParameter {
                        name: "toolId".to_string(),
                        in_type: "path".to_string(),
                        description: Some("工具 ID".to_string()),
                        required: Some(true),
                        schema: OpenApiSchema {
                            schema_type: Some("string".to_string()),
                            format: None,
                            properties: None,
                            items: None,
                            enum_values: None,
                            required: None,
                            ref_path: None,
                        }
                    }
                ]),
                requestBody: Some(OpenApiRequestBody {
                    description: Some("工具执行请求".to_string()),
                    required: Some(true),
                    content: HashMap::from([(
                        "application/json".to_string(),
                        OpenApiMediaType {
                            schema: OpenApiSchema {
                                schema_type: None,
                                format: None,
                                properties: None,
                                items: None,
                                enum_values: None,
                                required: None,
                                ref_path: Some("#/components/schemas/ToolExecuteRequest".to_string()),
                            }
                        }
                    )]),
                }),
                responses: HashMap::from([(
                    "200".to_string(),
                    OpenApiResponse {
                        description: "工具执行结果".to_string(),
                        content: Some(HashMap::from([(
                            "application/json".to_string(),
                            OpenApiMediaType {
                                schema: OpenApiSchema {
                                    schema_type: None,
                                    format: None,
                                    properties: None,
                                    items: None,
                                    enum_values: None,
                                    required: None,
                                    ref_path: Some("#/components/schemas/ToolExecuteResponse".to_string()),
                                }
                            }
                        )])),
                    }
                )]),
            }
        )]),
    );
}

/// 添加工作流路由
fn add_workflow_routes(spec: &mut OpenApiSpec) {
    // GET /api/workflows - 获取所有工作流
    spec.paths.insert(
        "/api/workflows".to_string(),
        HashMap::from([(
            "get".to_string(),
            OpenApiPathItem {
                summary: Some("获取所有工作流".to_string()),
                description: Some("返回所有注册的工作流列表".to_string()),
                operationId: "getWorkflows".to_string(),
                parameters: None,
                requestBody: None,
                responses: HashMap::from([(
                    "200".to_string(),
                    OpenApiResponse {
                        description: "工作流列表".to_string(),
                        content: Some(HashMap::from([(
                            "application/json".to_string(),
                            OpenApiMediaType {
                                schema: OpenApiSchema {
                                    schema_type: Some("array".to_string()),
                                    format: None,
                                    properties: None,
                                    items: Some(Box::new(OpenApiSchema {
                                        schema_type: None,
                                        format: None,
                                        properties: None,
                                        items: None,
                                        enum_values: None,
                                        required: None,
                                        ref_path: Some("#/components/schemas/Workflow".to_string()),
                                    })),
                                    enum_values: None,
                                    required: None,
                                    ref_path: None,
                                }
                            }
                        )])),
                    }
                )]),
            }
        )]),
    );
    
    // POST /api/workflows/:workflowId/start - 开始工作流
    spec.paths.insert(
        "/api/workflows/{workflowId}/start".to_string(),
        HashMap::from([(
            "post".to_string(),
            OpenApiPathItem {
                summary: Some("开始工作流".to_string()),
                description: Some("启动指定的工作流并返回实例 ID".to_string()),
                operationId: "startWorkflow".to_string(),
                parameters: Some(vec![
                    OpenApiParameter {
                        name: "workflowId".to_string(),
                        in_type: "path".to_string(),
                        description: Some("工作流 ID".to_string()),
                        required: Some(true),
                        schema: OpenApiSchema {
                            schema_type: Some("string".to_string()),
                            format: None,
                            properties: None,
                            items: None,
                            enum_values: None,
                            required: None,
                            ref_path: None,
                        }
                    }
                ]),
                requestBody: Some(OpenApiRequestBody {
                    description: Some("工作流启动请求".to_string()),
                    required: Some(true),
                    content: HashMap::from([(
                        "application/json".to_string(),
                        OpenApiMediaType {
                            schema: OpenApiSchema {
                                schema_type: None,
                                format: None,
                                properties: None,
                                items: None,
                                enum_values: None,
                                required: None,
                                ref_path: Some("#/components/schemas/WorkflowStartRequest".to_string()),
                            }
                        }
                    )]),
                }),
                responses: HashMap::from([(
                    "200".to_string(),
                    OpenApiResponse {
                        description: "工作流启动结果".to_string(),
                        content: Some(HashMap::from([(
                            "application/json".to_string(),
                            OpenApiMediaType {
                                schema: OpenApiSchema {
                                    schema_type: None,
                                    format: None,
                                    properties: None,
                                    items: None,
                                    enum_values: None,
                                    required: None,
                                    ref_path: Some("#/components/schemas/WorkflowStartResponse".to_string()),
                                }
                            }
                        )])),
                    }
                )]),
            }
        )]),
    );
}

/// 添加模式组件
fn add_schema_components(spec: &mut OpenApiSpec) {
    // Agent 模式
    spec.components.schemas.insert(
        "Agent".to_string(),
        OpenApiSchema {
            schema_type: Some("object".to_string()),
            format: None,
            properties: Some(HashMap::from([
                (
                    "id".to_string(),
                    OpenApiSchema {
                        schema_type: Some("string".to_string()),
                        format: None,
                        properties: None,
                        items: None,
                        enum_values: None,
                        required: None,
                        ref_path: None,
                    }
                ),
                (
                    "name".to_string(),
                    OpenApiSchema {
                        schema_type: Some("string".to_string()),
                        format: None,
                        properties: None,
                        items: None,
                        enum_values: None,
                        required: None,
                        ref_path: None,
                    }
                ),
                (
                    "instructions".to_string(),
                    OpenApiSchema {
                        schema_type: Some("string".to_string()),
                        format: None,
                        properties: None,
                        items: None,
                        enum_values: None,
                        required: None,
                        ref_path: None,
                    }
                ),
                (
                    "model".to_string(),
                    OpenApiSchema {
                        schema_type: Some("string".to_string()),
                        format: None,
                        properties: None,
                        items: None,
                        enum_values: None,
                        required: None,
                        ref_path: None,
                    }
                ),
            ])),
            items: None,
            enum_values: None,
            required: Some(vec!["id".to_string(), "name".to_string()]),
            ref_path: None,
        }
    );
    
    // Tool 模式
    spec.components.schemas.insert(
        "Tool".to_string(),
        OpenApiSchema {
            schema_type: Some("object".to_string()),
            format: None,
            properties: Some(HashMap::from([
                (
                    "id".to_string(),
                    OpenApiSchema {
                        schema_type: Some("string".to_string()),
                        format: None,
                        properties: None,
                        items: None,
                        enum_values: None,
                        required: None,
                        ref_path: None,
                    }
                ),
                (
                    "name".to_string(),
                    OpenApiSchema {
                        schema_type: Some("string".to_string()),
                        format: None,
                        properties: None,
                        items: None,
                        enum_values: None,
                        required: None,
                        ref_path: None,
                    }
                ),
                (
                    "description".to_string(),
                    OpenApiSchema {
                        schema_type: Some("string".to_string()),
                        format: None,
                        properties: None,
                        items: None,
                        enum_values: None,
                        required: None,
                        ref_path: None,
                    }
                ),
            ])),
            items: None,
            enum_values: None,
            required: Some(vec!["id".to_string()]),
            ref_path: None,
        }
    );
    
    // Workflow 模式
    spec.components.schemas.insert(
        "Workflow".to_string(),
        OpenApiSchema {
            schema_type: Some("object".to_string()),
            format: None,
            properties: Some(HashMap::from([
                (
                    "id".to_string(),
                    OpenApiSchema {
                        schema_type: Some("string".to_string()),
                        format: None,
                        properties: None,
                        items: None,
                        enum_values: None,
                        required: None,
                        ref_path: None,
                    }
                ),
                (
                    "name".to_string(),
                    OpenApiSchema {
                        schema_type: Some("string".to_string()),
                        format: None,
                        properties: None,
                        items: None,
                        enum_values: None,
                        required: None,
                        ref_path: None,
                    }
                ),
            ])),
            items: None,
            enum_values: None,
            required: Some(vec!["id".to_string()]),
            ref_path: None,
        }
    );
    
    // Message 模式
    spec.components.schemas.insert(
        "Message".to_string(),
        OpenApiSchema {
            schema_type: Some("object".to_string()),
            format: None,
            properties: Some(HashMap::from([
                (
                    "role".to_string(),
                    OpenApiSchema {
                        schema_type: Some("string".to_string()),
                        format: None,
                        properties: None,
                        items: None,
                        enum_values: Some(vec![
                            "system".to_string(),
                            "user".to_string(),
                            "assistant".to_string(),
                            "tool".to_string(),
                        ]),
                        required: None,
                        ref_path: None,
                    }
                ),
                (
                    "content".to_string(),
                    OpenApiSchema {
                        schema_type: Some("string".to_string()),
                        format: None,
                        properties: None,
                        items: None,
                        enum_values: None,
                        required: None,
                        ref_path: None,
                    }
                ),
            ])),
            items: None,
            enum_values: None,
            required: Some(vec!["role".to_string(), "content".to_string()]),
            ref_path: None,
        }
    );
    
    // GenerateRequest 模式
    spec.components.schemas.insert(
        "GenerateRequest".to_string(),
        OpenApiSchema {
            schema_type: Some("object".to_string()),
            format: None,
            properties: Some(HashMap::from([
                (
                    "messages".to_string(),
                    OpenApiSchema {
                        schema_type: Some("array".to_string()),
                        format: None,
                        properties: None,
                        items: Some(Box::new(OpenApiSchema {
                            schema_type: None,
                            format: None,
                            properties: None,
                            items: None,
                            enum_values: None,
                            required: None,
                            ref_path: Some("#/components/schemas/Message".to_string()),
                        })),
                        enum_values: None,
                        required: None,
                        ref_path: None,
                    }
                ),
                (
                    "options".to_string(),
                    OpenApiSchema {
                        schema_type: Some("object".to_string()),
                        format: None,
                        properties: Some(HashMap::new()),
                        items: None,
                        enum_values: None,
                        required: None,
                        ref_path: None,
                    }
                ),
            ])),
            items: None,
            enum_values: None,
            required: Some(vec!["messages".to_string()]),
            ref_path: None,
        }
    );
    
    // GenerateResponse 模式
    spec.components.schemas.insert(
        "GenerateResponse".to_string(),
        OpenApiSchema {
            schema_type: Some("object".to_string()),
            format: None,
            properties: Some(HashMap::from([
                (
                    "text".to_string(),
                    OpenApiSchema {
                        schema_type: Some("string".to_string()),
                        format: None,
                        properties: None,
                        items: None,
                        enum_values: None,
                        required: None,
                        ref_path: None,
                    }
                ),
                (
                    "message".to_string(),
                    OpenApiSchema {
                        schema_type: None,
                        format: None,
                        properties: None,
                        items: None,
                        enum_values: None,
                        required: None,
                        ref_path: Some("#/components/schemas/Message".to_string()),
                    }
                ),
            ])),
            items: None,
            enum_values: None,
            required: None,
            ref_path: None,
        }
    );
    
    // ToolExecuteRequest 模式
    spec.components.schemas.insert(
        "ToolExecuteRequest".to_string(),
        OpenApiSchema {
            schema_type: Some("object".to_string()),
            format: None,
            properties: Some(HashMap::from([
                (
                    "input".to_string(),
                    OpenApiSchema {
                        schema_type: Some("object".to_string()),
                        format: None,
                        properties: Some(HashMap::new()),
                        items: None,
                        enum_values: None,
                        required: None,
                        ref_path: None,
                    }
                ),
            ])),
            items: None,
            enum_values: None,
            required: Some(vec!["input".to_string()]),
            ref_path: None,
        }
    );
    
    // ToolExecuteResponse 模式
    spec.components.schemas.insert(
        "ToolExecuteResponse".to_string(),
        OpenApiSchema {
            schema_type: Some("object".to_string()),
            format: None,
            properties: Some(HashMap::from([
                (
                    "output".to_string(),
                    OpenApiSchema {
                        schema_type: Some("object".to_string()),
                        format: None,
                        properties: Some(HashMap::new()),
                        items: None,
                        enum_values: None,
                        required: None,
                        ref_path: None,
                    }
                ),
            ])),
            items: None,
            enum_values: None,
            required: None,
            ref_path: None,
        }
    );
    
    // WorkflowStartRequest 模式
    spec.components.schemas.insert(
        "WorkflowStartRequest".to_string(),
        OpenApiSchema {
            schema_type: Some("object".to_string()),
            format: None,
            properties: Some(HashMap::from([
                (
                    "input".to_string(),
                    OpenApiSchema {
                        schema_type: Some("object".to_string()),
                        format: None,
                        properties: Some(HashMap::new()),
                        items: None,
                        enum_values: None,
                        required: None,
                        ref_path: None,
                    }
                ),
            ])),
            items: None,
            enum_values: None,
            required: None,
            ref_path: None,
        }
    );
    
    // WorkflowStartResponse 模式
    spec.components.schemas.insert(
        "WorkflowStartResponse".to_string(),
        OpenApiSchema {
            schema_type: Some("object".to_string()),
            format: None,
            properties: Some(HashMap::from([
                (
                    "workflowId".to_string(),
                    OpenApiSchema {
                        schema_type: Some("string".to_string()),
                        format: None,
                        properties: None,
                        items: None,
                        enum_values: None,
                        required: None,
                        ref_path: None,
                    }
                ),
                (
                    "instanceId".to_string(),
                    OpenApiSchema {
                        schema_type: Some("string".to_string()),
                        format: None,
                        properties: None,
                        items: None,
                        enum_values: None,
                        required: None,
                        ref_path: None,
                    }
                ),
            ])),
            items: None,
            enum_values: None,
            required: None,
            ref_path: None,
        }
    );
}

/// 生成 Swagger UI HTML
pub fn generate_swagger_ui_html(server_port: u16) -> String {
    let swagger_url = format!("http://localhost:{}/openapi.json", server_port);
    
    format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Lumosai API 文档</title>
    <link rel="stylesheet" type="text/css" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@4.5.0/swagger-ui.css" >
    <style>
        html {{
            box-sizing: border-box;
            overflow: -moz-scrollbars-vertical;
            overflow-y: scroll;
        }}
        
        *,
        *:before,
        *:after {{
            box-sizing: inherit;
        }}
        
        body {{
            margin:0;
            background: #fafafa;
        }}
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    
    <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@4.5.0/swagger-ui-bundle.js"> </script>
    <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@4.5.0/swagger-ui-standalone-preset.js"> </script>
    <script>
    window.onload = function() {{
        const ui = SwaggerUIBundle({{
            url: "{}",
            dom_id: '#swagger-ui',
            deepLinking: true,
            presets: [
                SwaggerUIBundle.presets.apis,
                SwaggerUIStandalonePreset
            ],
            plugins: [
                SwaggerUIBundle.plugins.DownloadUrl
            ],
            layout: "BaseLayout"
        }})
        
        window.ui = ui
    }}
    </script>
</body>
</html>
"#, swagger_url)
}
