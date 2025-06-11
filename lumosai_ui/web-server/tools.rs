/*!
# Tools Module

工具调用系统，实现AI工具的注册、管理和执行。

## 功能特性

- **工具注册**: 动态注册和管理工具
- **工具调用**: 支持AI模型调用工具
- **内置工具**: 提供常用的内置工具集
- **权限控制**: 工具执行的安全控制

## 支持的工具类型

- **Web工具**: 网络搜索、HTTP请求、URL验证
- **文件工具**: 文件读写、目录操作
- **数据工具**: JSON处理、计算器、时间工具
- **系统工具**: 环境变量、系统信息
*/

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use thiserror::Error;

/// 工具错误类型
#[derive(Debug, Error)]
pub enum ToolError {
    #[error("工具未找到: {0}")]
    ToolNotFound(String),
    #[error("参数错误: {0}")]
    InvalidParameters(String),
    #[error("执行错误: {0}")]
    ExecutionError(String),
    #[error("权限不足: {0}")]
    PermissionDenied(String),
    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// 工具参数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub r#type: String,
    pub description: String,
    pub required: bool,
    pub default: Option<Value>,
}

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
    pub category: String,
    pub enabled: bool,
}

/// 工具执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

/// 工具执行上下文
#[derive(Debug, Clone)]
pub struct ToolContext {
    pub user_id: i64,
    pub conversation_id: i64,
    pub permissions: Vec<String>,
}

/// 工具特征
pub trait Tool: Send + Sync + std::fmt::Debug {
    /// 获取工具定义
    fn definition(&self) -> ToolDefinition;

    /// 执行工具
    fn execute(&self, params: Value, context: &ToolContext) -> Result<ToolResult, ToolError>;

    /// 克隆工具
    fn clone_box(&self) -> Box<dyn Tool>;
}

impl Clone for Box<dyn Tool> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// 计算器工具
#[derive(Debug, Clone)]
pub struct CalculatorTool;

impl Tool for CalculatorTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "calculator".to_string(),
            description: "执行基本的数学计算".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "expression".to_string(),
                    r#type: "string".to_string(),
                    description: "要计算的数学表达式".to_string(),
                    required: true,
                    default: None,
                },
            ],
            category: "数学".to_string(),
            enabled: true,
        }
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }
    
    fn execute(&self, params: Value, _context: &ToolContext) -> Result<ToolResult, ToolError> {
        let start_time = std::time::Instant::now();
        
        let expression = params.get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("缺少expression参数".to_string()))?;
        
        // 简单的计算器实现
        let result = match self.evaluate_expression(expression) {
            Ok(value) => ToolResult {
                success: true,
                result: Some(json!(value)),
                error: None,
                execution_time_ms: start_time.elapsed().as_millis() as u64,
            },
            Err(e) => ToolResult {
                success: false,
                result: None,
                error: Some(e),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
            },
        };
        
        Ok(result)
    }
}

impl CalculatorTool {
    fn evaluate_expression(&self, expr: &str) -> Result<f64, String> {
        // 简化的表达式计算器
        let expr = expr.replace(" ", "");
        
        // 支持基本的四则运算
        if let Some(pos) = expr.find('+') {
            let (left, right) = expr.split_at(pos);
            let right = &right[1..];
            let left_val = self.parse_number(left)?;
            let right_val = self.parse_number(right)?;
            return Ok(left_val + right_val);
        }
        
        if let Some(pos) = expr.find('-') {
            let (left, right) = expr.split_at(pos);
            let right = &right[1..];
            let left_val = self.parse_number(left)?;
            let right_val = self.parse_number(right)?;
            return Ok(left_val - right_val);
        }
        
        if let Some(pos) = expr.find('*') {
            let (left, right) = expr.split_at(pos);
            let right = &right[1..];
            let left_val = self.parse_number(left)?;
            let right_val = self.parse_number(right)?;
            return Ok(left_val * right_val);
        }
        
        if let Some(pos) = expr.find('/') {
            let (left, right) = expr.split_at(pos);
            let right = &right[1..];
            let left_val = self.parse_number(left)?;
            let right_val = self.parse_number(right)?;
            if right_val == 0.0 {
                return Err("除零错误".to_string());
            }
            return Ok(left_val / right_val);
        }
        
        // 如果没有运算符，直接解析数字
        self.parse_number(&expr)
    }
    
    fn parse_number(&self, s: &str) -> Result<f64, String> {
        s.parse::<f64>().map_err(|_| format!("无效的数字: {}", s))
    }
}

/// 时间工具
#[derive(Debug, Clone)]
pub struct TimeTool;

impl Tool for TimeTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "current_time".to_string(),
            description: "获取当前时间信息".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "format".to_string(),
                    r#type: "string".to_string(),
                    description: "时间格式 (iso, timestamp, readable)".to_string(),
                    required: false,
                    default: Some(json!("iso")),
                },
            ],
            category: "系统".to_string(),
            enabled: true,
        }
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }
    
    fn execute(&self, params: Value, _context: &ToolContext) -> Result<ToolResult, ToolError> {
        let start_time = std::time::Instant::now();
        
        let format = params.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("iso");
        
        let now = chrono::Utc::now();
        
        let result = match format {
            "iso" => json!({
                "time": now.to_rfc3339(),
                "timezone": "UTC"
            }),
            "timestamp" => json!({
                "timestamp": now.timestamp(),
                "timestamp_ms": now.timestamp_millis()
            }),
            "readable" => json!({
                "time": now.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                "date": now.format("%Y-%m-%d").to_string(),
                "time_only": now.format("%H:%M:%S").to_string()
            }),
            _ => return Err(ToolError::InvalidParameters("无效的时间格式".to_string())),
        };
        
        Ok(ToolResult {
            success: true,
            result: Some(result),
            error: None,
            execution_time_ms: start_time.elapsed().as_millis() as u64,
        })
    }
}

/// 系统信息工具
#[derive(Debug, Clone)]
pub struct SystemInfoTool;

impl Tool for SystemInfoTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "system_info".to_string(),
            description: "获取系统信息".to_string(),
            parameters: vec![],
            category: "系统".to_string(),
            enabled: true,
        }
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }
    
    fn execute(&self, _params: Value, _context: &ToolContext) -> Result<ToolResult, ToolError> {
        let start_time = std::time::Instant::now();
        
        let result = json!({
            "platform": std::env::consts::OS,
            "architecture": std::env::consts::ARCH,
            "family": std::env::consts::FAMILY,
            "version": env!("CARGO_PKG_VERSION"),
            "name": env!("CARGO_PKG_NAME")
        });
        
        Ok(ToolResult {
            success: true,
            result: Some(result),
            error: None,
            execution_time_ms: start_time.elapsed().as_millis() as u64,
        })
    }
}

/// 工具注册表
#[derive(Clone)]
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl std::fmt::Debug for ToolRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolRegistry")
            .field("tools", &self.tools.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl ToolRegistry {
    /// 创建新的工具注册表
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };
        
        // 注册内置工具
        registry.register_builtin_tools();
        
        registry
    }
    
    /// 注册内置工具
    fn register_builtin_tools(&mut self) {
        self.register_tool(Box::new(CalculatorTool));
        self.register_tool(Box::new(TimeTool));
        self.register_tool(Box::new(SystemInfoTool));
    }
    
    /// 注册工具
    pub fn register_tool(&mut self, tool: Box<dyn Tool>) {
        let name = tool.definition().name.clone();
        self.tools.insert(name, tool);
    }
    
    /// 获取工具
    pub fn get_tool(&self, name: &str) -> Option<&Box<dyn Tool>> {
        self.tools.get(name)
    }
    
    /// 获取所有工具定义
    pub fn get_all_definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|tool| tool.definition()).collect()
    }
    
    /// 获取启用的工具定义
    pub fn get_enabled_definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .map(|tool| tool.definition())
            .filter(|def| def.enabled)
            .collect()
    }
    
    /// 执行工具
    pub fn execute_tool(
        &self,
        name: &str,
        params: Value,
        context: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let tool = self.get_tool(name)
            .ok_or_else(|| ToolError::ToolNotFound(name.to_string()))?;
        
        tool.execute(params, context)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
