use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

pub mod commands;
pub mod error;
pub mod server;
pub mod template;
pub mod util;

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
    New(CreateArgs),

    /// 启动开发服务器（热重载）
    Dev(commands::dev::DevOptions),

    /// 运行测试
    Test(TestArgs),

    /// 构建项目
    Build(BuildArgs),

    /// 部署项目到云端
    Deploy(DeployArgs),

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

    /// 创建项目（兼容性别名）
    #[clap(hide = true)]
    Create(CreateArgs),
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    /// 项目名称
    pub name: Option<String>,

    /// 项目模板 (hello-world, chatbot, rag-system, multi-agent, research-assistant, workflow-automation)
    #[arg(long, short)]
    pub template: Option<String>,

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
pub struct TestArgs {
    /// 项目目录
    #[arg(long)]
    pub project_dir: Option<PathBuf>,

    /// 运行特定测试
    #[arg(long)]
    pub test: Option<String>,

    /// 显示详细输出
    #[arg(long, short)]
    pub verbose: bool,

    /// 运行性能测试
    #[arg(long)]
    pub bench: bool,
}

#[derive(Args, Debug)]
pub struct BuildArgs {
    /// 项目目录
    #[arg(long)]
    pub project_dir: Option<PathBuf>,

    /// 构建模式 (debug, release)
    #[arg(long, default_value = "release")]
    pub mode: String,

    /// 目标平台
    #[arg(long)]
    pub target: Option<String>,

    /// 输出目录
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct DeployArgs {
    /// 项目目录
    #[arg(long)]
    pub project_dir: Option<PathBuf>,

    /// 部署环境 (dev, staging, production)
    #[arg(long, default_value = "dev")]
    pub env: String,

    /// 部署配置文件
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// 强制部署（跳过确认）
    #[arg(long)]
    pub force: bool,
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
    format!("{} v{}", LUMOSAI_CLI_NAME, LUMOSAI_CLI_VERSION)
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
