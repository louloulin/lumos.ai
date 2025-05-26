use std::fmt;
use std::io;
use std::path::Path;

/// CLI命令结果类型
pub type CliResult<T> = Result<T, CliError>;

/// CLI命令错误类型
#[derive(Debug)]
pub enum CliError {
    /// 输入/输出错误
    Io(io::Error),
    
    /// JSON解析错误
    JsonParse(serde_json::Error),
    
    /// 模板错误
    TemplateNotFound(String),
    
    /// 模板配置未找到
    TemplateConfigNotFound(String),
    
    /// 工具链错误
    ToolchainError(String),
    
    /// 用户交互错误
    Interaction(String),
    
    /// 命令取消
    Canceled(String),
    
    /// 路径未找到
    PathNotFound(String, String),
    
    /// 依赖项未找到
    DependencyMissing(String, String),
    
    /// 服务器错误
    ServerError(String),
    
    /// 内部错误
    Internal(String),
    
    /// 操作失败
    Failed(String, Option<Box<dyn std::error::Error + Send + Sync>>),
    
    /// 其他错误
    Other(String),
}

impl CliError {
    /// 创建IO错误
    pub fn io_error<P: AsRef<Path>>(error: io::Error, path: P) -> Self {
        let path_display = path.as_ref().display().to_string();
        CliError::Other(format!("I/O错误: {} (路径: {})", error, path_display))
    }
    
    /// 创建IO错误，自定义消息
    pub fn io(message: &str, error: io::Error) -> Self {
        CliError::Other(format!("{}: {}", message, error))
    }
    
    /// 创建IO错误，自定义消息 (String版本)
    pub fn io_string(message: String, error: io::Error) -> Self {
        CliError::Other(format!("{}: {}", message, error))
    }
    
    /// 创建模板未找到错误
    pub fn template_not_found(template_name: &str) -> Self {
        CliError::TemplateNotFound(template_name.to_string())
    }
    
    /// 创建模板配置未找到错误
    pub fn template_config_not_found<P: AsRef<Path>>(path: P) -> Self {
        let path_display = path.as_ref().display().to_string();
        CliError::TemplateConfigNotFound(path_display)
    }
    
    /// 创建模板解析错误
    pub fn template_parse_error(error: serde_json::Error) -> Self {
        CliError::JsonParse(error)
    }
    
    /// 创建工具链错误
    pub fn toolchain_error(message: &str) -> Self {
        CliError::ToolchainError(message.to_string())
    }
    
    /// 创建取消错误
    pub fn canceled(message: &str) -> Self {
        CliError::Canceled(message.to_string())
    }
    
    /// 创建路径未找到错误
    pub fn path_not_found<P: AsRef<Path>>(path: P, message: impl ToString) -> Self {
        let path_display = path.as_ref().display().to_string();
        CliError::PathNotFound(path_display, message.to_string())
    }
    
    /// 创建依赖项未找到错误
    pub fn dependency(dependency: &str, message: &str) -> Self {
        CliError::DependencyMissing(dependency.to_string(), message.to_string())
    }
    
    /// 创建服务器错误
    pub fn server(message: &str) -> Self {
        CliError::ServerError(message.to_string())
    }
    
    /// 创建内部错误
    pub fn internal(message: &str) -> Self {
        CliError::Internal(message.to_string())
    }
    
    /// 创建操作失败错误
    pub fn failed<E>(message: &str, error: Option<E>) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        let boxed_error = error.map(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>);
        CliError::Failed(message.to_string(), boxed_error)
    }
    
    /// 创建无效输入错误
    pub fn invalid_input(message: &str) -> Self {
        CliError::Other(format!("无效输入: {}", message))
    }
    
    /// 创建无效输入错误 (接受String)
    pub fn invalid_input_string(message: String) -> Self {
        CliError::Other(format!("无效输入: {}", message))
    }
    
    /// 创建其他错误
    pub fn other(message: String) -> Self {
        CliError::Other(message)
    }
    
    /// 创建失败错误 (接受String消息)
    pub fn failed_string<E>(message: String, error: Option<E>) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        let boxed_error = error.map(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>);
        CliError::Failed(message, boxed_error)
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CliError::Io(err) => write!(f, "I/O错误: {}", err),
            CliError::JsonParse(err) => write!(f, "JSON解析错误: {}", err),
            CliError::TemplateNotFound(name) => write!(f, "模板未找到: {}", name),
            CliError::TemplateConfigNotFound(path) => write!(f, "模板配置未找到: {}", path),
            CliError::ToolchainError(msg) => write!(f, "工具链错误: {}", msg),
            CliError::Interaction(msg) => write!(f, "交互错误: {}", msg),
            CliError::Canceled(msg) => write!(f, "{}", msg),
            CliError::PathNotFound(path, msg) => write!(f, "{}: {}", msg, path),
            CliError::DependencyMissing(dep, msg) => write!(f, "缺少依赖 {}: {}", dep, msg),
            CliError::ServerError(msg) => write!(f, "服务器错误: {}", msg),
            CliError::Internal(msg) => write!(f, "内部错误: {}", msg),
            CliError::Failed(msg, Some(err)) => write!(f, "{}: {}", msg, err),
            CliError::Failed(msg, None) => write!(f, "{}", msg),
            CliError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for CliError {}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> Self {
        CliError::Io(err)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(err: serde_json::Error) -> Self {
        CliError::JsonParse(err)
    }
}

impl From<String> for CliError {
    fn from(err: String) -> Self {
        CliError::Other(err)
    }
}

impl From<&str> for CliError {
    fn from(err: &str) -> Self {
        CliError::Other(err.to_string())
    }
}

impl From<dialoguer::Error> for CliError {
    fn from(err: dialoguer::Error) -> Self {
        CliError::Other(format!("交互错误: {}", err))
    }
}

impl From<ctrlc::Error> for CliError {
    fn from(err: ctrlc::Error) -> Self {
        CliError::Other(format!("Ctrl+C处理错误: {}", err))
    }
}

impl From<reqwest::Error> for CliError {
    fn from(err: reqwest::Error) -> Self {
        CliError::Other(format!("HTTP请求错误: {}", err))
    }
} 