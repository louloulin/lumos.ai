//! Node.js绑定模块
//! 
//! 为JavaScript/TypeScript提供Lumos.ai的完整绑定支持

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;
use crate::core::{CrossLangAgent, CrossLangAgentBuilder, CrossLangTool, CrossLangResponse};
use crate::error::BindingError;
use crate::types::*;

/// Node.js Agent包装器
#[napi]
pub struct Agent {
    inner: CrossLangAgent,
}

/// Node.js AgentBuilder包装器
#[napi]
pub struct AgentBuilder {
    inner: CrossLangAgentBuilder,
}

/// Node.js Tool包装器
#[napi]
pub struct Tool {
    inner: CrossLangTool,
}

/// Node.js Response包装器
#[napi(object)]
pub struct Response {
    /// 响应内容
    pub content: String,
    /// 响应类型
    pub response_type: String,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
    /// 工具调用结果
    pub tool_calls: Vec<ToolCallResult>,
    /// 错误信息
    pub error: Option<String>,
}

/// Node.js工具调用结果
#[napi(object)]
pub struct ToolCallResult {
    /// 工具名称
    pub tool_name: String,
    /// 调用参数
    pub parameters: serde_json::Value,
    /// 调用结果
    pub result: serde_json::Value,
    /// 执行时间（毫秒）
    pub execution_time_ms: u32,
    /// 是否成功
    pub success: bool,
    /// 错误信息
    pub error: Option<String>,
}

/// Node.js配置对象
#[napi(object)]
pub struct Config {
    /// 模型配置
    pub model: ModelConfig,
    /// 工具列表
    pub tools: Vec<String>,
    /// 运行时配置
    pub runtime: RuntimeConfig,
}

/// Node.js模型配置
#[napi(object)]
pub struct ModelConfig {
    /// 模型名称
    pub name: String,
    /// API密钥
    pub api_key: Option<String>,
    /// 基础URL
    pub base_url: Option<String>,
}

/// Node.js运行时配置
#[napi(object)]
pub struct RuntimeConfig {
    /// 超时时间（秒）
    pub timeout_seconds: u32,
    /// 最大重试次数
    pub max_retries: u32,
    /// 并发限制
    pub concurrency_limit: u32,
    /// 启用日志
    pub enable_logging: bool,
    /// 日志级别
    pub log_level: String,
}

#[napi]
impl Agent {
    /// 生成响应
    #[napi]
    pub fn generate(&self, input: String) -> Result<Response> {
        let response = self.inner.generate(&input)
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
        
        Ok(Response {
            content: response.content,
            response_type: format!("{:?}", response.response_type),
            metadata: response.metadata,
            tool_calls: response.tool_calls.into_iter().map(|tc| ToolCallResult {
                tool_name: tc.tool_name,
                parameters: tc.parameters,
                result: tc.result,
                execution_time_ms: tc.execution_time_ms as u32,
                success: tc.success,
                error: tc.error,
            }).collect(),
            error: response.error,
        })
    }
    
    /// 异步生成响应
    #[napi]
    pub async fn generate_async(&self, input: String) -> Result<Response> {
        let response = self.inner.generate_async(&input).await
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
        
        Ok(Response {
            content: response.content,
            response_type: format!("{:?}", response.response_type),
            metadata: response.metadata,
            tool_calls: response.tool_calls.into_iter().map(|tc| ToolCallResult {
                tool_name: tc.tool_name,
                parameters: tc.parameters,
                result: tc.result,
                execution_time_ms: tc.execution_time_ms as u32,
                success: tc.success,
                error: tc.error,
            }).collect(),
            error: response.error,
        })
    }
    
    /// 获取配置
    #[napi]
    pub fn get_config(&self) -> Config {
        let config = self.inner.get_config();
        
        Config {
            model: ModelConfig {
                name: config.model.name,
                api_key: config.model.api_key,
                base_url: config.model.base_url,
            },
            tools: config.tools,
            runtime: RuntimeConfig {
                timeout_seconds: config.runtime.timeout_seconds as u32,
                max_retries: config.runtime.max_retries,
                concurrency_limit: config.runtime.concurrency_limit as u32,
                enable_logging: config.runtime.enable_logging,
                log_level: config.runtime.log_level,
            },
        }
    }
}

#[napi]
impl AgentBuilder {
    /// 创建新的AgentBuilder
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: CrossLangAgentBuilder::new(),
        }
    }
    
    /// 设置名称
    #[napi]
    pub fn name(&mut self, name: String) -> &Self {
        self.inner = std::mem::take(&mut self.inner).name(&name);
        self
    }
    
    /// 设置指令
    #[napi]
    pub fn instructions(&mut self, instructions: String) -> &Self {
        self.inner = std::mem::take(&mut self.inner).instructions(&instructions);
        self
    }
    
    /// 设置模型
    #[napi]
    pub fn model(&mut self, model: String) -> &Self {
        self.inner = std::mem::take(&mut self.inner).model(&model);
        self
    }
    
    /// 添加工具
    #[napi]
    pub fn tool(&mut self, tool: &Tool) -> &Self {
        self.inner = std::mem::take(&mut self.inner).tool(tool.inner.clone());
        self
    }
    
    /// 添加多个工具
    #[napi]
    pub fn tools(&mut self, tools: Vec<&Tool>) -> &Self {
        for tool in tools {
            self.inner = std::mem::take(&mut self.inner).tool(tool.inner.clone());
        }
        self
    }
    
    /// 构建Agent
    #[napi]
    pub fn build(&self) -> Result<Agent> {
        let agent = self.inner.clone().build()
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
        
        Ok(Agent { inner: agent })
    }
    
    /// 异步构建Agent
    #[napi]
    pub async fn build_async(&self) -> Result<Agent> {
        let agent = self.inner.clone().build_async().await
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
        
        Ok(Agent { inner: agent })
    }
}

#[napi]
impl Tool {
    /// 获取工具元数据
    #[napi]
    pub fn metadata(&self) -> HashMap<String, serde_json::Value> {
        let metadata = self.inner.metadata();
        let mut result = HashMap::new();
        
        result.insert("name".to_string(), serde_json::Value::String(metadata.name.clone()));
        result.insert("description".to_string(), serde_json::Value::String(metadata.description.clone()));
        result.insert("tool_type".to_string(), serde_json::Value::String(metadata.tool_type.clone()));
        result.insert("is_async".to_string(), serde_json::Value::Bool(metadata.is_async));
        result.insert("parameters".to_string(), metadata.parameters.clone());
        
        result
    }
    
    /// 执行工具
    #[napi]
    pub fn execute(&self, parameters: HashMap<String, serde_json::Value>) -> Result<ToolCallResult> {
        let params = serde_json::Value::Object(
            parameters.into_iter().collect()
        );
        
        let result = self.inner.execute(params)
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
        
        Ok(ToolCallResult {
            tool_name: result.tool_name,
            parameters: result.parameters,
            result: result.result,
            execution_time_ms: result.execution_time_ms as u32,
            success: result.success,
            error: result.error,
        })
    }
}

/// 便利函数：快速创建Agent
#[napi]
pub fn quick_agent(name: String, instructions: String) -> AgentBuilder {
    let mut builder = AgentBuilder::new();
    builder.name(name);
    builder.instructions(instructions);
    builder
}

/// 便利函数：创建AgentBuilder
#[napi]
pub fn create_agent_builder() -> AgentBuilder {
    AgentBuilder::new()
}

/// 工具模块
pub mod tools {
    use super::*;
    use crate::core::ToolMetadata;
    use std::sync::Arc;
    
    /// Web搜索工具
    #[napi]
    pub fn web_search() -> Tool {
        let tool = lumosai_core::tools::web::web_search();
        let metadata = ToolMetadata {
            name: "web_search".to_string(),
            description: "搜索网络内容".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "搜索查询"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "最大结果数",
                        "default": 10
                    }
                },
                "required": ["query"]
            }),
            tool_type: "web".to_string(),
            is_async: true,
        };
        
        Tool {
            inner: CrossLangTool::new(tool, metadata),
        }
    }
    
    /// HTTP请求工具
    #[napi]
    pub fn http_request() -> Tool {
        let tool = lumosai_core::tools::web::http_request();
        let metadata = ToolMetadata {
            name: "http_request".to_string(),
            description: "发送HTTP请求".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "请求URL"
                    },
                    "method": {
                        "type": "string",
                        "description": "HTTP方法",
                        "enum": ["GET", "POST", "PUT", "DELETE"],
                        "default": "GET"
                    }
                },
                "required": ["url"]
            }),
            tool_type: "web".to_string(),
            is_async: true,
        };
        
        Tool {
            inner: CrossLangTool::new(tool, metadata),
        }
    }
    
    /// 计算器工具
    #[napi]
    pub fn calculator() -> Tool {
        let tool = lumosai_core::tools::math::calculator();
        let metadata = ToolMetadata {
            name: "calculator".to_string(),
            description: "基础数学计算".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "数学表达式"
                    }
                },
                "required": ["expression"]
            }),
            tool_type: "math".to_string(),
            is_async: false,
        };
        
        Tool {
            inner: CrossLangTool::new(tool, metadata),
        }
    }
    
    /// 文件读取工具
    #[napi]
    pub fn file_reader() -> Tool {
        let tool = lumosai_core::tools::file::file_reader();
        let metadata = ToolMetadata {
            name: "file_reader".to_string(),
            description: "读取文件内容".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "文件路径"
                    }
                },
                "required": ["path"]
            }),
            tool_type: "file".to_string(),
            is_async: true,
        };
        
        Tool {
            inner: CrossLangTool::new(tool, metadata),
        }
    }
}
