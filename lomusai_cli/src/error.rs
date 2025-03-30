use thiserror::Error;
use std::io;
use std::path::PathBuf;

/// CLI 工具的错误类型
#[derive(Debug, Error)]
pub enum CliError {
    #[error("IO 错误: {0}")]
    Io(#[from] io::Error),

    #[error("项目验证错误: {0}")]
    ProjectValidation(String),

    #[error("构建错误: {0}")]
    Build(String),

    #[error("运行错误: {0}")]
    Run(String),

    #[error("部署错误: {0}")]
    Deploy(String),

    #[error("配置错误: {0}")]
    Config(String),

    #[error("模板错误: {0}")]
    Template(String),

    #[error("没有找到Rust工具链, 请安装Rust: https://rustup.rs")]
    RustToolchainNotFound,

    #[error("找不到项目: {0}")]
    ProjectNotFound(PathBuf),

    #[error("无效的Lomus AI项目: {0}")]
    InvalidProject(PathBuf),

    #[error("不支持的部署目标: {0}")]
    UnsupportedDeployTarget(String),

    #[error("CLI命令失败: {0}")]
    CommandFailed(String),

    #[error("依赖错误: {0}")]
    DependencyError(String),

    #[error("交互错误: {0}")]
    Interaction(String),

    #[error("操作已取消: {0}")]
    Canceled(String),

    #[error("{0}")]
    Generic(String),
}

pub type CliResult<T> = Result<T, CliError>;

// 提供便捷的错误转换函数
impl CliError {
    pub fn project_validation<S: Into<String>>(msg: S) -> Self {
        CliError::ProjectValidation(msg.into())
    }

    pub fn build<S: Into<String>>(msg: S) -> Self {
        CliError::Build(msg.into())
    }

    pub fn run<S: Into<String>>(msg: S) -> Self {
        CliError::Run(msg.into())
    }

    pub fn deploy<S: Into<String>>(msg: S) -> Self {
        CliError::Deploy(msg.into())
    }

    pub fn config<S: Into<String>>(msg: S) -> Self {
        CliError::Config(msg.into())
    }

    pub fn template<S: Into<String>>(msg: S) -> Self {
        CliError::Template(msg.into())
    }

    pub fn canceled<S: Into<String>>(msg: S) -> Self {
        CliError::Canceled(msg.into())
    }

    pub fn generic<S: Into<String>>(msg: S) -> Self {
        CliError::Generic(msg.into())
    }
}

// 实现From特性，便于从其它错误类型转换
impl From<String> for CliError {
    fn from(s: String) -> Self {
        CliError::Generic(s)
    }
}

impl From<&str> for CliError {
    fn from(s: &str) -> Self {
        CliError::Generic(s.to_string())
    }
}

impl From<anyhow::Error> for CliError {
    fn from(err: anyhow::Error) -> Self {
        CliError::Generic(err.to_string())
    }
}

impl From<dialoguer::Error> for CliError {
    fn from(err: dialoguer::Error) -> Self {
        CliError::Interaction(err.to_string())
    }
}

impl From<reqwest::Error> for CliError {
    fn from(err: reqwest::Error) -> Self {
        CliError::Generic(err.to_string())
    }
}

impl From<toml::de::Error> for CliError {
    fn from(err: toml::de::Error) -> Self {
        CliError::Generic(err.to_string())
    }
}

impl From<toml::ser::Error> for CliError {
    fn from(err: toml::ser::Error) -> Self {
        CliError::Generic(err.to_string())
    }
}

impl From<serde_json::Error> for CliError {
    fn from(err: serde_json::Error) -> Self {
        CliError::Generic(err.to_string())
    }
} 