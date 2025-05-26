use clap::{Parser, Subcommand, Args};
use colored::Colorize;
use std::path::PathBuf;

mod commands;
mod error;
mod util;
mod server;
mod template;

use error::{CliResult, CliError};

/// Lumosai 命令行工具
///
/// 用于创建、开发、运行和部署 Lumosai AI 应用
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
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
struct CreateArgs {
    /// 项目名称
    #[arg(long)]
    name: Option<String>,
    
    /// 包含的组件，以逗号分隔 (agents,tools,workflows,rag)
    #[arg(long)]
    components: Option<String>,
    
    /// LLM提供商 (openai, anthropic, gemini, local)
    #[arg(long)]
    llm: Option<String>,
    
    /// LLM API密钥
    #[arg(long = "api-key")]
    llm_api_key: Option<String>,
    
    /// 项目目录
    #[arg(long)]
    project_dir: Option<PathBuf>,
    
    /// 添加示例代码
    #[arg(long)]
    example: bool,
}

#[derive(Args, Debug)]
struct UiArgs {
    /// 项目目录
    #[arg(long)]
    project_dir: Option<PathBuf>,
    
    /// 端口号
    #[arg(long, default_value = "4003")]
    port: u16,
    
    /// 主题 (light/dark)
    #[arg(long)]
    theme: Option<String>,
    
    /// 开发模式
    #[arg(long)]
    dev: bool,
    
    /// API服务器URL
    #[arg(long)]
    api_url: Option<String>,
}

#[derive(Args, Debug)]
struct ApiArgs {
    /// 项目目录
    #[arg(long)]
    project_dir: Option<PathBuf>,
    
    /// 输出目录
    #[arg(long)]
    output: Option<PathBuf>,
    
    /// 代理列表，以逗号分隔
    #[arg(long)]
    agents: Option<String>,
}

#[tokio::main]
async fn main() -> CliResult<()> {
    // 显示欢迎信息
    let version = env!("CARGO_PKG_VERSION");
    println!("{}", format!("Lumosai CLI v{}", version).bright_cyan());
    println!("{}", "用于创建、开发和部署 Lumosai AI 应用".bright_cyan());
    println!();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Create(args) => {
            // 将逗号分隔的字符串转换为 Vec<String>
            let components = args.components.map(|s| {
                s.split(',').map(|s| s.trim().to_string()).collect()
            });
            
            // 创建name的副本以避免所有权问题
            let name_clone = args.name.clone();
            
            commands::create::run(
                name_clone,
                components,
                args.llm,
                args.llm_api_key,
                args.name.or_else(|| args.project_dir.and_then(|p| p.file_name().and_then(|n| n.to_str().map(|s| s.to_string())))),
                args.example,
            ).await
        },
        Commands::Dev(options) => {
            commands::dev::run(options).await
        },
        Commands::Ui(args) => {
            commands::ui::run(
                args.project_dir,
                args.port,
                args.theme,
                args.dev,
                args.api_url,
            ).await
        },
        Commands::Playground(options) => {
            commands::playground::run(options).await
        },
        Commands::Api(args) => {
            // 将逗号分隔的字符串转换为 Vec<String>
            let agents = args.agents.map(|s| {
                s.split(',').map(|s| s.trim().to_string()).collect()
            });
            
            commands::api::run(
                args.project_dir,
                args.output,
                agents,
            ).await
        },
        Commands::Visualize(options) => {
            commands::visualize::run(options).await
        },
        Commands::Monitoring(options) => {
            commands::monitoring::run(options).await
        },
    }
}

/// 列出可用模板
async fn list_templates() -> CliResult<()> {
    let template_manager = template::TemplateManager::new()?;
    let templates = template_manager.list_templates()?;
    
    if templates.is_empty() {
        println!("{}", "没有可用模板".bright_yellow());
        println!("{}", "使用以下命令下载模板:".bright_blue());
        println!("  lumos template download --url <URL> [--name <n>]");
    } else {
        println!("{}", "可用模板:".bright_blue());
        for (name, description) in templates {
            println!("  {} - {}", name.bright_green(), description);
        }
    }
    
    Ok(())
}

/// 下载模板
async fn download_template(url: String, name: Option<String>) -> CliResult<()> {
    let template_manager = template::TemplateManager::new()?;
    
    let template_name = name.unwrap_or_else(|| {
        // 尝试从URL提取名称
        let parts: Vec<&str> = url.split('/').collect();
        parts.last().unwrap_or(&"custom-template").to_string()
    });
    
    println!("{}", format!("下载模板 '{}' 从 {}", template_name, url).bright_blue());
    
    template_manager.download_template(&url, &template_name)?;
    
    println!("{}", format!("模板 '{}' 已下载", template_name).bright_green());
    
    Ok(())
}

/// 删除模板
async fn remove_template(name: String, force: bool) -> CliResult<()> {
    let template_manager = template::TemplateManager::new()?;
    
    if !force {
        use dialoguer::Confirm;
        
        let confirm = Confirm::new()
            .with_prompt(format!("确定要删除模板'{}'吗?", name))
            .default(false)
            .interact()?;
            
        if !confirm {
            return Err(CliError::canceled("删除已取消"));
        }
    }
    
    println!("{}", format!("删除模板 '{}'...", name).bright_blue());
    
    template_manager.remove_template(&name)?;
    
    println!("{}", format!("模板 '{}' 已删除", name).bright_green());
    
    Ok(())
} 