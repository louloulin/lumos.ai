//! 函数调用支持模块
//!
//! 提供现代化的函数调用支持，兼容OpenAI/Anthropic的函数调用格式

use crate::error::{Error, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use super::{Tool, ToolExecutionContext};

/// 函数调用参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionParameter {
    /// 参数名称
    pub name: String,
    /// 参数类型
    pub r#type: String,
    /// 参数描述
    pub description: String,
    /// 是否必需
    pub required: bool,
    /// 枚举值（可选）
    pub enum_values: Option<Vec<String>>,
    /// 默认值（可选）
    pub default_value: Option<Value>,
}

/// 函数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// 函数名称
    pub name: String,
    /// 函数描述
    pub description: String,
    /// 参数定义
    pub parameters: Vec<FunctionParameter>,
    /// 是否为严格模式
    pub strict: Option<bool>,
}

/// 函数调用请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// 函数名称
    pub name: String,
    /// 调用参数
    pub arguments: HashMap<String, Value>,
    /// 调用ID（可选）
    pub call_id: Option<String>,
}

/// 函数调用结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionResult {
    /// 调用ID
    pub call_id: String,
    /// 函数名称
    pub name: String,
    /// 执行结果
    pub result: Value,
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果失败）
    pub error: Option<String>,
    /// 执行时间（毫秒）
    pub execution_time_ms: Option<u64>,
}

/// OpenAI格式工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIToolDefinition {
    /// 工具类型
    #[serde(rename = "type")]
    pub tool_type: String,
    /// 函数定义
    pub function: FunctionDefinition,
}

impl From<FunctionDefinition> for OpenAIToolDefinition {
    fn from(function: FunctionDefinition) -> Self {
        Self {
            tool_type: "function".to_string(),
            function,
        }
    }
}

/// Anthropic格式工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicToolDefinition {
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: String,
    /// 输入模式
    pub input_schema: serde_json::Value,
}

impl From<FunctionDefinition> for AnthropicToolDefinition {
    fn from(function: FunctionDefinition) -> Self {
        // 构建JSON Schema输入模式
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();
        
        for param in &function.parameters {
            let mut property = serde_json::Map::new();
            property.insert("type".to_string(), Value::String(param.r#type.clone()));
            property.insert("description".to_string(), Value::String(param.description.clone()));
            
            if let Some(enum_values) = &param.enum_values {
                property.insert("enum".to_string(), 
                    Value::Array(enum_values.iter().map(|v| Value::String(v.clone())).collect()));
            }
            
            if let Some(default_value) = &param.default_value {
                property.insert("default".to_string(), default_value.clone());
            }
            
            properties.insert(param.name.clone(), Value::Object(property));
            
            if param.required {
                required.push(param.name.clone());
            }
        }
        
        let input_schema = serde_json::json!({
            "type": "object",
            "properties": Value::Object(properties),
            "required": required
        });
        
        Self {
            name: function.name,
            description: function.description,
            input_schema,
        }
    }
}

/// 函数调用转换器
pub struct FunctionCallConverter;

impl FunctionCallConverter {
    /// 转换为OpenAI格式
    pub fn to_openai_tools(functions: &[FunctionDefinition]) -> Vec<OpenAIToolDefinition> {
        functions.iter().cloned().map(OpenAIToolDefinition::from).collect()
    }
    
    /// 转换为Anthropic格式
    pub fn to_anthropic_tools(functions: &[FunctionDefinition]) -> Vec<AnthropicToolDefinition> {
        functions.iter().cloned().map(AnthropicToolDefinition::from).collect()
    }
    
    /// 解析OpenAI格式的函数调用
    pub fn parse_openai_call(tool_call: &serde_json::Value) -> Result<FunctionCall> {
        let tool_call: serde_json::Value = serde_json::from_str(
            &tool_call["function"].to_string()
        ).map_err(|e| Error::Other(format!("Failed to parse OpenAI function call: {}", e)))?;
        
        let name = tool_call["name"]
            .as_str()
            .ok_or_else(|| Error::Other("Missing function name".to_string()))?;
        
        let arguments = tool_call["arguments"]
            .as_object()
            .ok_or_else(|| Error::Other("Missing function arguments".to_string()))?
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        
        Ok(FunctionCall {
            name: name.to_string(),
            arguments,
            call_id: None,
        })
    }
    
    /// 解析Anthropic格式的函数调用
    pub fn parse_anthropic_call(tool_call: &serde_json::Value) -> Result<FunctionCall> {
        let name = tool_call["name"]
            .as_str()
            .ok_or_else(|| Error::Other("Missing function name".to_string()))?;
        
        let input = tool_call.get("input")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        
        let arguments = input.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        
        Ok(FunctionCall {
            name: name.to_string(),
            arguments,
            call_id: None,
        })
    }
}

/// 函数调用执行器
pub struct FunctionCallExecutor {
    functions: Arc<RwLock<HashMap<String, Arc<dyn FunctionExecutor>>>>,
}

/// 函数执行器trait
#[async_trait]
pub trait FunctionExecutor: Send + Sync {
    /// 执行函数调用
    async fn execute(&self, call: &FunctionCall, context: &ToolExecutionContext) -> Result<FunctionResult>;
    
    /// 获取函数定义
    fn get_definition(&self) -> &FunctionDefinition;
}

/// 异步函数执行器
pub struct AsyncFunctionExecutor {
    definition: FunctionDefinition,
    executor: Arc<dyn Fn(HashMap<String, Value>, ToolExecutionContext) -> Result<Value> + Send + Sync>,
}

impl AsyncFunctionExecutor {
    pub fn new(
        definition: FunctionDefinition,
        executor: impl Fn(HashMap<String, Value>, ToolExecutionContext) -> Result<Value> + Send + Sync + 'static,
    ) -> Self {
        Self {
            definition,
            executor: Arc::new(executor),
        }
    }
}

#[async_trait]
impl FunctionExecutor for AsyncFunctionExecutor {
    async fn execute(&self, call: &FunctionCall, context: &ToolExecutionContext) -> Result<FunctionResult> {
        let start_time = std::time::Instant::now();
        
        match (self.executor)(call.arguments.clone(), context.clone()) {
            Ok(result) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                Ok(FunctionResult {
                    call_id: call.call_id.clone().unwrap_or_else(|| {
                        uuid::Uuid::new_v4().to_string()
                    }),
                    name: call.name.clone(),
                    result,
                    success: true,
                    error: None,
                    execution_time_ms: Some(execution_time),
                })
            }
            Err(e) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                Ok(FunctionResult {
                    call_id: call.call_id.clone().unwrap_or_else(|| {
                        uuid::Uuid::new_v4().to_string()
                    }),
                    name: call.name.clone(),
                    result: Value::Null,
                    success: false,
                    error: Some(e.to_string()),
                    execution_time_ms: Some(execution_time),
                })
            }
        }
    }
    
    fn get_definition(&self) -> &FunctionDefinition {
        &self.definition
    }
}

/// 同步函数执行器
pub struct SyncFunctionExecutor {
    definition: FunctionDefinition,
    executor: Arc<dyn Fn(HashMap<String, Value>, ToolExecutionContext) -> Result<Value> + Send + Sync>,
}

impl SyncFunctionExecutor {
    pub fn new(
        definition: FunctionDefinition,
        executor: impl Fn(HashMap<String, Value>, ToolExecutionContext) -> Result<Value> + Send + Sync + 'static,
    ) -> Self {
        Self {
            definition,
            executor: Arc::new(executor),
        }
    }
}

#[async_trait]
impl FunctionExecutor for SyncFunctionExecutor {
    async fn execute(&self, call: &FunctionCall, context: &ToolExecutionContext) -> Result<FunctionResult> {
        let start_time = std::time::Instant::now();
        
        match (self.executor)(call.arguments.clone(), context.clone()) {
            Ok(result) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                Ok(FunctionResult {
                    call_id: call.call_id.clone().unwrap_or_else(|| {
                        uuid::Uuid::new_v4().to_string()
                    }),
                    name: call.name.clone(),
                    result,
                    success: true,
                    error: None,
                    execution_time_ms: Some(execution_time),
                })
            }
            Err(e) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                Ok(FunctionResult {
                    call_id: call.call_id.clone().unwrap_or_else(|| {
                        uuid::Uuid::new_v4().to_string()
                    }),
                    name: call.name.clone(),
                    result: Value::Null,
                    success: false,
                    error: Some(e.to_string()),
                    execution_time_ms: Some(execution_time),
                })
            }
        }
    }
    
    fn get_definition(&self) -> &FunctionDefinition {
        &self.definition
    }
}

use tokio::sync::RwLock;

impl FunctionCallExecutor {
    pub fn new() -> Self {
        Self {
            functions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 注册函数
    pub async fn register_function(&self, executor: Arc<dyn FunctionExecutor>) -> Result<()> {
        let name = executor.get_definition().name.clone();
        let mut functions = self.functions.write().await;
        functions.insert(name, executor);
        Ok(())
    }
    
    /// 执行函数调用
    pub async fn execute_call(&self, call: &FunctionCall, context: &ToolExecutionContext) -> Result<FunctionResult> {
        let functions = self.functions.read().await;
        
        let executor = functions.get(&call.name)
            .ok_or_else(|| Error::Other(format!("Function '{}' not found", call.name)))?;
        
        executor.execute(call, context).await
    }
    
    /// 批量执行函数调用
    pub async fn execute_calls(
        &self,
        calls: &[FunctionCall],
        context: &ToolExecutionContext,
    ) -> Result<Vec<FunctionResult>> {
        let mut results = Vec::new();
        
        for call in calls {
            let result = self.execute_call(call, context).await?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// 获取所有函数定义
    pub async fn get_function_definitions(&self) -> Vec<FunctionDefinition> {
        let functions = self.functions.read().await;
        functions.values()
            .map(|executor| executor.get_definition().clone())
            .collect()
    }
    
    /// 移除函数
    pub async fn remove_function(&self, name: &str) -> Result<bool> {
        let mut functions = self.functions.write().await;
        Ok(functions.remove(name).is_some())
    }
}

/// 预定义函数
pub mod builtin_functions {
    use super::*;
    use std::collections::HashMap;
    
    /// 获取当前时间
    pub fn create_get_current_time() -> Arc<dyn FunctionExecutor> {
        let definition = FunctionDefinition {
            name: "get_current_time".to_string(),
            description: "Get the current time in ISO 8601 format".to_string(),
            parameters: vec![],
            strict: Some(false),
        };
        
        Arc::new(AsyncFunctionExecutor::new(definition, |_args, _context| {
            let now = chrono::Utc::now();
            Ok(serde_json::json!({
                "timestamp": now.to_rfc3339(),
                "unix_timestamp": now.timestamp(),
                "timezone": now.timezone().to_string()
            }))
        }))
    }
    
    /// 简单计算器
    pub fn create_calculator() -> Arc<dyn FunctionExecutor> {
        let definition = FunctionDefinition {
            name: "calculator".to_string(),
            description: "Perform basic mathematical calculations".to_string(),
            parameters: vec![
                FunctionParameter {
                    name: "expression".to_string(),
                    r#type: "string".to_string(),
                    description: "Mathematical expression to evaluate".to_string(),
                    required: true,
                    enum_values: None,
                    default_value: None,
                }
            ],
            strict: Some(false),
        };
        
        Arc::new(AsyncFunctionExecutor::new(definition, |args, _context| {
            let expression = args.get("expression")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Other("Missing expression parameter".to_string()))?;
            
            // 简单的表达式解析器
            let result = evaluate_simple_expression(expression)?;
            
            Ok(serde_json::json!({
                "expression": expression,
                "result": result
            }))
        }))
    }
    
    /// 简单表达式评估
    fn evaluate_simple_expression(expr: &str) -> Result<f64> {
        // 移除空格
        let expr = expr.trim();
        
        // 支持基本运算符
        if let Some((left, right)) = expr.split_once('+') {
            let left_val = evaluate_simple_expression(left.trim())?;
            let right_val = evaluate_simple_expression(right.trim())?;
            return Ok(left_val + right_val);
        }
        
        if let Some((left, right)) = expr.split_once('-') {
            let left_val = evaluate_simple_expression(left.trim())?;
            let right_val = evaluate_simple_expression(right.trim())?;
            return Ok(left_val - right_val);
        }
        
        if let Some((left, right)) = expr.split_once('*') {
            let left_val = evaluate_simple_expression(left.trim())?;
            let right_val = evaluate_simple_expression(right.trim())?;
            return Ok(left_val * right_val);
        }
        
        if let Some((left, right)) = expr.split_once('/') {
            let left_val = evaluate_simple_expression(left.trim())?;
            let right_val = evaluate_simple_expression(right.trim())?;
            if right_val == 0.0 {
                return Err(Error::Other("Division by zero".to_string()));
            }
            return Ok(left_val / right_val);
        }
        
        // 解析为数字
        expr.parse::<f64>()
            .map_err(|e| Error::Other(format!("Invalid expression: {}", e)))
    }
    
    /// JSON验证器
    pub fn create_json_validator() -> Arc<dyn FunctionExecutor> {
        let definition = FunctionDefinition {
            name: "validate_json".to_string(),
            description: "Validate if a string is valid JSON".to_string(),
            parameters: vec![
                FunctionParameter {
                    name: "json_string".to_string(),
                    r#type: "string".to_string(),
                    description: "JSON string to validate".to_string(),
                    required: true,
                    enum_values: None,
                    default_value: None,
                }
            ],
            strict: Some(false),
        };
        
        Arc::new(AsyncFunctionExecutor::new(definition, |args, _context| {
            let json_str = args.get("json_string")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Other("Missing json_string parameter".to_string()))?;
            
            match serde_json::from_str::<serde_json::Value>(json_str) {
                Ok(_) => Ok(serde_json::json!({
                    "valid": true,
                    "parsed": json_str
                })),
                Err(_) => Ok(serde_json::json!({
                    "valid": false,
                    "error": "Invalid JSON format"
                }))
            }
        }))
    }
    
    /// 注册所有内置函数
    pub async fn register_builtin_functions(executor: &FunctionCallExecutor) -> Result<()> {
        executor.register_function(create_get_current_time()).await?;
        executor.register_function(create_calculator()).await?;
        executor.register_function(create_json_validator()).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use builtin_functions::*;
    
    #[tokio::test]
    async fn test_function_call_executor() {
        let executor = FunctionCallExecutor::new();
        
        // 注册计算器函数
        executor.register_function(create_calculator()).await.unwrap();
        
        let context = ToolExecutionContext::new();
        
        // 测试函数调用
        let call = FunctionCall {
            name: "calculator".to_string(),
            arguments: {
                let mut args = HashMap::new();
                args.insert("expression".to_string(), serde_json::Value::String("2 + 3".to_string()));
                args
            },
            call_id: Some("test-call-1".to_string()),
        };
        
        let result = executor.execute_call(&call, &context).await.unwrap();
        
        assert_eq!(result.call_id, "test-call-1");
        assert_eq!(result.name, "calculator");
        assert!(result.success);
        
        let result_data = &result.result;
        assert_eq!(result_data["expression"], "2 + 3");
        assert_eq!(result_data["result"], 5.0);
    }
    
    #[tokio::test]
    async fn test_function_converters() {
        let functions = vec![
            FunctionDefinition {
                name: "test_func".to_string(),
                description: "Test function".to_string(),
                parameters: vec![
                    FunctionParameter {
                        name: "param1".to_string(),
                        r#type: "string".to_string(),
                        description: "First parameter".to_string(),
                        required: true,
                        enum_values: None,
                        default_value: None,
                    }
                ],
                strict: Some(false),
            }
        ];
        
        // 测试OpenAI格式转换
        let openai_tools = FunctionCallConverter::to_openai_tools(&functions);
        assert_eq!(openai_tools.len(), 1);
        assert_eq!(openai_tools[0].function.name, "test_func");
        
        // 测试Anthropic格式转换
        let anthropic_tools = FunctionCallConverter::to_anthropic_tools(&functions);
        assert_eq!(anthropic_tools.len(), 1);
        assert_eq!(anthropic_tools[0].name, "test_func");
    }
    
    #[tokio::test]
    async fn test_builtin_functions() {
        let executor = FunctionCallExecutor::new();
        builtin_functions::register_builtin_functions(&executor).await.unwrap();
        
        let definitions = executor.get_function_definitions().await;
        assert!(definitions.len() >= 3); // 至少有3个内置函数
        
        // 测试获取当前时间
        let context = ToolExecutionContext::new();
        let time_result = executor.execute_call(&FunctionCall {
            name: "get_current_time".to_string(),
            arguments: HashMap::new(),
            call_id: None,
        }, &context).await.unwrap();
        
        assert!(time_result.success);
        assert!(time_result.result["timestamp"].is_string());
        
        // 测试JSON验证器
        let valid_json_result = executor.execute_call(&FunctionCall {
            name: "validate_json".to_string(),
            arguments: {
                let mut args = HashMap::new();
                args.insert("json_string".to_string(), 
                    serde_json::Value::String(r#"{"key": "value"}"#.to_string()));
                args
            },
            call_id: None,
        }, &context).await.unwrap();
        
        assert!(valid_json_result.success);
        assert_eq!(valid_json_result.result["valid"], true);
        
        // 测试无效JSON
        let invalid_json_result = executor.execute_call(&FunctionCall {
            name: "validate_json".to_string(),
            arguments: {
                let mut args = HashMap::new();
                args.insert("json_string".to_string(), 
                    serde_json::Value::String(r#"{"key": "value""#.to_string()));
                args
            },
            call_id: None,
        }, &context).await.unwrap();
        
        assert!(!invalid_json_result.success);
        assert_eq!(invalid_json_result.result["valid"], false);
    }
}