//! 多语言绑定错误处理
//! 
//! 提供跨语言的统一错误处理机制

use std::fmt;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 绑定错误类型
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum BindingError {
    /// 核心错误
    #[error("Core error: {message}")]
    Core { message: String },
    
    /// 序列化错误
    #[error("Serialization error: {message}")]
    Serialization { message: String },
    
    /// 运行时错误
    #[error("Runtime error: {message}")]
    Runtime { message: String },
    
    /// 配置错误
    #[error("Configuration error: {field}: {message}")]
    Configuration { field: String, message: String },
    
    /// 工具错误
    #[error("Tool error: {tool_name}: {message}")]
    Tool { tool_name: String, message: String },
    
    /// 模型错误
    #[error("Model error: {model_name}: {message}")]
    Model { model_name: String, message: String },
    
    /// 网络错误
    #[error("Network error: {message}")]
    Network { message: String },
    
    /// 超时错误
    #[error("Timeout error: operation timed out after {timeout_seconds}s")]
    Timeout { timeout_seconds: u64 },
    
    /// 权限错误
    #[error("Permission error: {message}")]
    Permission { message: String },
    
    /// 资源不足错误
    #[error("Resource exhausted: {resource}: {message}")]
    ResourceExhausted { resource: String, message: String },
    
    /// 不支持的操作
    #[error("Unsupported operation: {operation}")]
    UnsupportedOperation { operation: String },
    
    /// 无效参数
    #[error("Invalid parameter: {parameter}: {message}")]
    InvalidParameter { parameter: String, message: String },
}

/// 错误上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// 错误代码
    pub error_code: String,
    
    /// 错误消息
    pub message: String,
    
    /// 详细信息
    pub details: Option<String>,
    
    /// 建议操作
    pub suggestions: Vec<String>,
    
    /// 错误分类
    pub category: ErrorCategory,
    
    /// 严重程度
    pub severity: ErrorSeverity,
    
    /// 是否可重试
    pub retryable: bool,
    
    /// 相关文档链接
    pub documentation_url: Option<String>,
}

/// 错误分类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// 用户错误
    User,
    /// 系统错误
    System,
    /// 网络错误
    Network,
    /// 配置错误
    Configuration,
    /// 外部服务错误
    ExternalService,
}

/// 错误严重程度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
    /// 严重
    Critical,
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, BindingError>;

impl BindingError {
    /// 创建核心错误
    pub fn core(message: impl Into<String>) -> Self {
        Self::Core {
            message: message.into(),
        }
    }
    
    /// 创建序列化错误
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::Serialization {
            message: message.into(),
        }
    }
    
    /// 创建运行时错误
    pub fn runtime(message: impl Into<String>) -> Self {
        Self::Runtime {
            message: message.into(),
        }
    }
    
    /// 创建配置错误
    pub fn configuration(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Configuration {
            field: field.into(),
            message: message.into(),
        }
    }
    
    /// 创建工具错误
    pub fn tool(tool_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Tool {
            tool_name: tool_name.into(),
            message: message.into(),
        }
    }
    
    /// 创建模型错误
    pub fn model(model_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Model {
            model_name: model_name.into(),
            message: message.into(),
        }
    }
    
    /// 创建网络错误
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
        }
    }
    
    /// 创建超时错误
    pub fn timeout(timeout_seconds: u64) -> Self {
        Self::Timeout { timeout_seconds }
    }
    
    /// 创建权限错误
    pub fn permission(message: impl Into<String>) -> Self {
        Self::Permission {
            message: message.into(),
        }
    }
    
    /// 创建资源不足错误
    pub fn resource_exhausted(resource: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ResourceExhausted {
            resource: resource.into(),
            message: message.into(),
        }
    }
    
    /// 创建不支持操作错误
    pub fn unsupported_operation(operation: impl Into<String>) -> Self {
        Self::UnsupportedOperation {
            operation: operation.into(),
        }
    }
    
    /// 创建无效参数错误
    pub fn invalid_parameter(parameter: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidParameter {
            parameter: parameter.into(),
            message: message.into(),
        }
    }
    
    /// 获取错误上下文
    pub fn context(&self) -> ErrorContext {
        match self {
            Self::Core { message } => ErrorContext {
                error_code: "CORE_ERROR".to_string(),
                message: message.clone(),
                details: None,
                suggestions: vec![
                    "检查Lumos.ai核心库是否正确安装".to_string(),
                    "查看详细日志获取更多信息".to_string(),
                ],
                category: ErrorCategory::System,
                severity: ErrorSeverity::High,
                retryable: false,
                documentation_url: Some("https://docs.lumosai.com/troubleshooting/core-errors".to_string()),
            },
            Self::Serialization { message } => ErrorContext {
                error_code: "SERIALIZATION_ERROR".to_string(),
                message: message.clone(),
                details: None,
                suggestions: vec![
                    "检查输入数据格式是否正确".to_string(),
                    "确认数据类型匹配".to_string(),
                ],
                category: ErrorCategory::User,
                severity: ErrorSeverity::Medium,
                retryable: false,
                documentation_url: Some("https://docs.lumosai.com/api/data-formats".to_string()),
            },
            Self::Runtime { message } => ErrorContext {
                error_code: "RUNTIME_ERROR".to_string(),
                message: message.clone(),
                details: None,
                suggestions: vec![
                    "重试操作".to_string(),
                    "检查系统资源是否充足".to_string(),
                ],
                category: ErrorCategory::System,
                severity: ErrorSeverity::Medium,
                retryable: true,
                documentation_url: Some("https://docs.lumosai.com/troubleshooting/runtime-errors".to_string()),
            },
            Self::Configuration { field, message } => ErrorContext {
                error_code: "CONFIGURATION_ERROR".to_string(),
                message: format!("配置字段 '{}': {}", field, message),
                details: None,
                suggestions: vec![
                    format!("检查配置字段 '{}' 的值", field),
                    "参考配置文档确认正确格式".to_string(),
                ],
                category: ErrorCategory::Configuration,
                severity: ErrorSeverity::High,
                retryable: false,
                documentation_url: Some("https://docs.lumosai.com/configuration".to_string()),
            },
            Self::Tool { tool_name, message } => ErrorContext {
                error_code: "TOOL_ERROR".to_string(),
                message: format!("工具 '{}': {}", tool_name, message),
                details: None,
                suggestions: vec![
                    format!("检查工具 '{}' 是否正确配置", tool_name),
                    "验证工具参数是否正确".to_string(),
                ],
                category: ErrorCategory::User,
                severity: ErrorSeverity::Medium,
                retryable: true,
                documentation_url: Some(format!("https://docs.lumosai.com/tools/{}", tool_name)),
            },
            Self::Network { message } => ErrorContext {
                error_code: "NETWORK_ERROR".to_string(),
                message: message.clone(),
                details: None,
                suggestions: vec![
                    "检查网络连接".to_string(),
                    "验证API端点是否可访问".to_string(),
                    "稍后重试".to_string(),
                ],
                category: ErrorCategory::Network,
                severity: ErrorSeverity::Medium,
                retryable: true,
                documentation_url: Some("https://docs.lumosai.com/troubleshooting/network-issues".to_string()),
            },
            Self::Timeout { timeout_seconds } => ErrorContext {
                error_code: "TIMEOUT_ERROR".to_string(),
                message: format!("操作超时 ({}秒)", timeout_seconds),
                details: None,
                suggestions: vec![
                    "增加超时时间".to_string(),
                    "检查网络连接速度".to_string(),
                    "重试操作".to_string(),
                ],
                category: ErrorCategory::Network,
                severity: ErrorSeverity::Medium,
                retryable: true,
                documentation_url: Some("https://docs.lumosai.com/configuration/timeouts".to_string()),
            },
            _ => ErrorContext {
                error_code: "UNKNOWN_ERROR".to_string(),
                message: self.to_string(),
                details: None,
                suggestions: vec!["联系技术支持".to_string()],
                category: ErrorCategory::System,
                severity: ErrorSeverity::High,
                retryable: false,
                documentation_url: Some("https://docs.lumosai.com/support".to_string()),
            },
        }
    }
    
    /// 获取错误代码
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Core { .. } => "CORE_ERROR",
            Self::Serialization { .. } => "SERIALIZATION_ERROR",
            Self::Runtime { .. } => "RUNTIME_ERROR",
            Self::Configuration { .. } => "CONFIGURATION_ERROR",
            Self::Tool { .. } => "TOOL_ERROR",
            Self::Model { .. } => "MODEL_ERROR",
            Self::Network { .. } => "NETWORK_ERROR",
            Self::Timeout { .. } => "TIMEOUT_ERROR",
            Self::Permission { .. } => "PERMISSION_ERROR",
            Self::ResourceExhausted { .. } => "RESOURCE_EXHAUSTED",
            Self::UnsupportedOperation { .. } => "UNSUPPORTED_OPERATION",
            Self::InvalidParameter { .. } => "INVALID_PARAMETER",
        }
    }
    
    /// 是否可重试
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Runtime { .. } |
            Self::Network { .. } |
            Self::Timeout { .. } |
            Self::Tool { .. }
        )
    }
}

// 从lumosai_core错误转换
impl From<lumosai_core::error::LumosError> for BindingError {
    fn from(err: lumosai_core::error::LumosError) -> Self {
        Self::Core {
            message: err.to_string(),
        }
    }
}

// 从serde_json错误转换
impl From<serde_json::Error> for BindingError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization {
            message: err.to_string(),
        }
    }
}

// 从tokio错误转换
impl From<tokio::task::JoinError> for BindingError {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::Runtime {
            message: err.to_string(),
        }
    }
}
