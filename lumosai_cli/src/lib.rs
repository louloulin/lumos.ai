use clap::{Parser, Subcommand, Args};
use std::path::PathBuf;

pub mod commands;
pub mod error;
pub mod util;
pub mod server;
pub mod template;

/// Lumosai CLI工具主要版本号
pub const LUMOSAI_CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Lumosai CLI工具的名称
pub const LUMOSAI_CLI_NAME: &str = env!("CARGO_PKG_NAME");

/// Lumosai CLI工具的描述
pub const LUMOSAI_CLI_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Lumosai CLI工具的作者
pub const LUMOSAI_CLI_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

/// Lumosai 命令行工具
///
/// 用于创建、开发、运行和部署 Lumosai AI 应用
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 创建一个新的Lumosai项目
    Create(CreateArgs),

    /// 启动开发服务器
    Dev(commands::dev::DevOptions),

    /// 启动UI服务器
    Ui(UiArgs),

    /// 启动交互式测试环境
    Playground(commands::playground::PlaygroundOptions),

    /// 生成API端点
    Api(ApiArgs),

    /// 生成可视化图表
    Visualize(commands::visualize::VisualizeOptions),

    /// 启动监控服务器
    Monitoring(commands::monitoring::MonitoringOptions),
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    /// 项目名称
    #[arg(long)]
    pub name: Option<String>,

    /// 包含的组件，以逗号分隔 (agents,tools,workflows,rag)
    #[arg(long)]
    pub components: Option<String>,

    /// LLM提供商 (openai, anthropic, gemini, local)
    #[arg(long)]
    pub llm: Option<String>,

    /// LLM API密钥
    #[arg(long = "api-key")]
    pub llm_api_key: Option<String>,

    /// 项目目录
    #[arg(long)]
    pub project_dir: Option<PathBuf>,

    /// 添加示例代码
    #[arg(long)]
    pub example: bool,
}

#[derive(Args, Debug)]
pub struct UiArgs {
    /// 项目目录
    #[arg(long)]
    pub project_dir: Option<PathBuf>,

    /// 端口号
    #[arg(long, default_value = "4003")]
    pub port: u16,

    /// 主题 (light/dark)
    #[arg(long)]
    pub theme: Option<String>,

    /// 开发模式
    #[arg(long)]
    pub dev: bool,

    /// API服务器URL
    #[arg(long)]
    pub api_url: Option<String>,
}

#[derive(Args, Debug)]
pub struct ApiArgs {
    /// 项目目录
    #[arg(long)]
    pub project_dir: Option<PathBuf>,

    /// 输出目录
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// 代理列表，以逗号分隔
    #[arg(long)]
    pub agents: Option<String>,
}

/// 获取完整的版本信息
pub fn version_info() -> String {
    format!(
        "{} v{}",
        LUMOSAI_CLI_NAME,
        LUMOSAI_CLI_VERSION
    )
}

/// 获取完整的版本字符串，包括额外信息
pub fn full_version_string() -> String {
    format!(
        "{}\n{}\n作者: {}",
        version_info(),
        LUMOSAI_CLI_DESCRIPTION,
        LUMOSAI_CLI_AUTHORS
    )
} 