//! 多语言绑定类型定义
//! 
//! 提供跨语言的统一类型定义和转换

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// 跨语言消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// 消息ID
    pub id: String,
    
    /// 消息内容
    pub content: String,
    
    /// 消息类型
    pub message_type: MessageType,
    
    /// 发送者
    pub sender: String,
    
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// 用户消息
    User,
    /// 助手消息
    Assistant,
    /// 系统消息
    System,
    /// 工具调用
    ToolCall,
    /// 工具结果
    ToolResult,
}

/// 跨语言会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    /// 会话ID
    pub id: String,
    
    /// 消息列表
    pub messages: Vec<Message>,
    
    /// 会话状态
    pub status: ConversationStatus,
    
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    
    /// 会话元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 会话状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversationStatus {
    /// 活跃
    Active,
    /// 暂停
    Paused,
    /// 已结束
    Ended,
    /// 错误
    Error,
}

/// 跨语言工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// 工具名称
    pub name: String,
    
    /// 工具描述
    pub description: String,
    
    /// 参数模式
    pub parameters: ParameterSchema,
    
    /// 返回值模式
    pub returns: Option<ParameterSchema>,
    
    /// 工具类型
    pub tool_type: ToolType,
    
    /// 是否异步
    pub is_async: bool,
    
    /// 工具标签
    pub tags: Vec<String>,
    
    /// 工具版本
    pub version: String,
}

/// 参数模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSchema {
    /// 参数类型
    pub param_type: ParameterType,
    
    /// 参数描述
    pub description: Option<String>,
    
    /// 是否必需
    pub required: bool,
    
    /// 默认值
    pub default: Option<serde_json::Value>,
    
    /// 枚举值
    pub enum_values: Option<Vec<serde_json::Value>>,
    
    /// 子参数（对象类型）
    pub properties: Option<HashMap<String, ParameterSchema>>,
    
    /// 数组项类型
    pub items: Option<Box<ParameterSchema>>,
}

/// 参数类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    /// 字符串
    String,
    /// 数字
    Number,
    /// 整数
    Integer,
    /// 布尔值
    Boolean,
    /// 对象
    Object,
    /// 数组
    Array,
    /// 空值
    Null,
}

/// 工具类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolType {
    /// 内置工具
    Builtin,
    /// 自定义工具
    Custom,
    /// 外部API
    ExternalApi,
    /// 数据库工具
    Database,
    /// 文件工具
    File,
    /// 网络工具
    Network,
    /// 计算工具
    Compute,
}

/// 跨语言执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// 执行ID
    pub execution_id: String,
    
    /// 是否成功
    pub success: bool,
    
    /// 结果数据
    pub data: serde_json::Value,
    
    /// 错误信息
    pub error: Option<String>,
    
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
    
    /// 内存使用（字节）
    pub memory_usage_bytes: Option<u64>,
    
    /// 执行统计
    pub stats: ExecutionStats,
}

/// 执行统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStats {
    /// 开始时间
    pub start_time: DateTime<Utc>,
    
    /// 结束时间
    pub end_time: DateTime<Utc>,
    
    /// CPU使用率
    pub cpu_usage_percent: Option<f64>,
    
    /// 网络请求数
    pub network_requests: u32,
    
    /// 缓存命中数
    pub cache_hits: u32,
    
    /// 缓存未命中数
    pub cache_misses: u32,
}

/// 跨语言配置选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOptions {
    /// 语言特定配置
    pub language_config: LanguageConfig,
    
    /// 性能配置
    pub performance: PerformanceConfig,
    
    /// 安全配置
    pub security: SecurityConfig,
    
    /// 日志配置
    pub logging: LoggingConfig,
}

/// 语言配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    /// 目标语言
    pub target_language: TargetLanguage,
    
    /// 语言版本
    pub language_version: String,
    
    /// 特定选项
    pub language_options: HashMap<String, serde_json::Value>,
}

/// 目标语言
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetLanguage {
    /// Python
    Python,
    /// JavaScript/Node.js
    JavaScript,
    /// TypeScript
    TypeScript,
    /// Go
    Go,
    /// Java
    Java,
    /// C#
    CSharp,
    /// C++
    Cpp,
    /// WebAssembly
    WebAssembly,
}

/// 性能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// 线程池大小
    pub thread_pool_size: Option<usize>,
    
    /// 内存限制（字节）
    pub memory_limit_bytes: Option<u64>,
    
    /// 超时时间（秒）
    pub timeout_seconds: u64,
    
    /// 启用缓存
    pub enable_cache: bool,
    
    /// 缓存大小
    pub cache_size: Option<usize>,
}

/// 安全配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// 启用沙箱
    pub enable_sandbox: bool,
    
    /// 允许的域名
    pub allowed_domains: Vec<String>,
    
    /// 禁止的操作
    pub forbidden_operations: Vec<String>,
    
    /// API密钥验证
    pub require_api_key: bool,
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: LogLevel,
    
    /// 日志格式
    pub format: LogFormat,
    
    /// 输出目标
    pub output: LogOutput,
    
    /// 启用结构化日志
    pub structured: bool,
}

/// 日志级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    /// 跟踪
    Trace,
    /// 调试
    Debug,
    /// 信息
    Info,
    /// 警告
    Warn,
    /// 错误
    Error,
}

/// 日志格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    /// 纯文本
    Text,
    /// JSON
    Json,
    /// 紧凑格式
    Compact,
}

/// 日志输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    /// 标准输出
    Stdout,
    /// 标准错误
    Stderr,
    /// 文件
    File { path: String },
    /// 网络
    Network { endpoint: String },
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            thread_pool_size: None,
            memory_limit_bytes: None,
            timeout_seconds: 30,
            enable_cache: true,
            cache_size: Some(1000),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_sandbox: true,
            allowed_domains: Vec::new(),
            forbidden_operations: Vec::new(),
            require_api_key: false,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            format: LogFormat::Text,
            output: LogOutput::Stdout,
            structured: false,
        }
    }
}

/// 类型转换工具
pub mod conversion {
    use super::*;
    
    /// 将Rust值转换为跨语言值
    pub fn to_cross_lang_value(value: &serde_json::Value) -> serde_json::Value {
        value.clone()
    }
    
    /// 将跨语言值转换为Rust值
    pub fn from_cross_lang_value(value: &serde_json::Value) -> serde_json::Value {
        value.clone()
    }
    
    /// 验证参数类型
    pub fn validate_parameter(value: &serde_json::Value, schema: &ParameterSchema) -> bool {
        match (&schema.param_type, value) {
            (ParameterType::String, serde_json::Value::String(_)) => true,
            (ParameterType::Number, serde_json::Value::Number(_)) => true,
            (ParameterType::Integer, serde_json::Value::Number(n)) => n.is_i64(),
            (ParameterType::Boolean, serde_json::Value::Bool(_)) => true,
            (ParameterType::Object, serde_json::Value::Object(_)) => true,
            (ParameterType::Array, serde_json::Value::Array(_)) => true,
            (ParameterType::Null, serde_json::Value::Null) => true,
            _ => false,
        }
    }
}
