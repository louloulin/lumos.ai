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
    
    /// 其他错误
    Other(String),
}

impl CliError {
    /// 创建IO错误
    pub fn io_error<P: AsRef<Path>>(error: io::Error, path: P) -> Self {
        let path_display = path.as_ref().display().to_string();
        CliError::Other(format!("I/O错误: {} (路径: {})", error, path_display))
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

impl From<dialoguer::Error> for CliError {
    fn from(err: dialoguer::Error) -> Self {
        CliError::Interaction(err.to_string())
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