mod commands;
mod error;
mod config;
mod util;
mod template;

use clap::{Parser, Subcommand, Args};
use colored::Colorize;
use std::path::PathBuf;
use error::{CliResult, CliError};

/// Lomus AI 命令行工具
///
/// 用于创建、开发、运行和部署 Lomus AI 应用
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 初始化一个新的Lomus AI项目
    #[clap(name = "init")]
    Init(InitArgs),
    
    /// 启动开发服务器
    #[clap(name = "dev")]
    Dev(DevArgs),
    
    /// 运行Lomus AI应用
    #[clap(name = "run")]
    Run(RunArgs),
    
    /// 构建Lomus AI应用
    #[clap(name = "build")]
    Build(BuildArgs),
    
    /// 部署Lomus AI应用
    #[clap(name = "deploy")]
    Deploy(DeployArgs),
    
    /// 模板管理
    #[clap(name = "template", subcommand)]
    Template(TemplateCommands),
}

#[derive(Args, Debug)]
struct InitArgs {
    /// 项目名称
    #[clap(short, long)]
    name: Option<String>,
    
    /// 模板类型（agent, workflow, rag 或自定义名称）
    #[clap(short, long)]
    template: Option<String>,
    
    /// 从URL下载模板
    #[clap(long)]
    template_url: Option<String>,
    
    /// 输出目录
    #[clap(short, long)]
    output: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct DevArgs {
    /// 项目目录
    #[clap(short = 'd', long)]
    project_dir: Option<PathBuf>,
    
    /// 端口号
    #[clap(short, long, default_value = "3000")]
    port: u16,
    
    /// 启用热重载（使用-r而不是-h避免与帮助选项冲突）
    #[clap(short = 'r', long)]
    hot_reload: bool,
}

#[derive(Args, Debug)]
struct RunArgs {
    /// 项目目录
    #[clap(short = 'd', long)]
    project_dir: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct BuildArgs {
    /// 项目目录
    #[clap(short = 'd', long)]
    project_dir: Option<PathBuf>,
    
    /// 输出目录
    #[clap(short, long)]
    output: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct DeployArgs {
    /// 项目目录
    #[clap(short = 'd', long)]
    project_dir: Option<PathBuf>,
    
    /// 部署目标（local, docker, aws, azure, gcp）
    #[clap(short, long, default_value = "local")]
    target: String,
}

#[derive(Subcommand, Debug)]
enum TemplateCommands {
    /// 列出可用模板
    List,
    
    /// 下载模板
    Download(TemplateDownloadArgs),
    
    /// 删除模板
    Remove(TemplateRemoveArgs),
}

#[derive(Args, Debug)]
struct TemplateDownloadArgs {
    /// 模板URL
    #[clap(short, long)]
    url: String,
    
    /// 模板名称
    #[clap(short, long)]
    name: Option<String>,
}

#[derive(Args, Debug)]
struct TemplateRemoveArgs {
    /// 模板名称
    #[clap(short, long)]
    name: String,
    
    /// 不确认删除
    #[clap(short, long)]
    force: bool,
}

#[tokio::main]
async fn main() -> CliResult<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Init(args) => {
            commands::init::run(
                args.name,
                args.template,
                args.template_url,
                args.output,
            ).await
        },
        Commands::Dev(args) => {
            commands::dev::run(
                args.project_dir,
                args.port,
                args.hot_reload,
            ).await
        },
        Commands::Run(args) => {
            commands::run::run(
                args.project_dir,
            ).await
        },
        Commands::Build(args) => {
            commands::build::run(
                args.project_dir,
                args.output,
            ).await
        },
        Commands::Deploy(args) => {
            commands::deploy::run(
                args.project_dir,
                &args.target,
            ).await
        },
        Commands::Template(template_cmd) => {
            match template_cmd {
                TemplateCommands::List => {
                    list_templates().await
                },
                TemplateCommands::Download(args) => {
                    download_template(args.url, args.name).await
                },
                TemplateCommands::Remove(args) => {
                    remove_template(args.name, args.force).await
                },
            }
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
        println!("  lomus template download --url <URL> [--name <NAME>]");
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
    
    // 如果没有提供名称，从URL提取
    let template_name = match name {
        Some(n) => n,
        None => {
            // 从URL提取模板名称，例如从GitHub仓库名或URL的最后一部分
            if url.contains("github.com") {
                // 从GitHub URL中提取仓库名
                url.split('/')
                   .last()
                   .unwrap_or("custom-template")
                   .replace(".git", "")
            } else {
                // 从其他URL中提取最后一部分
                url.split('/')
                   .last()
                   .unwrap_or("custom-template")
                   .split('.')
                   .next()
                   .unwrap_or("custom-template")
                   .to_string()
            }
        }
    };
    
    template_manager.download_template(&url, &template_name)?;
    
    Ok(())
}

/// 删除模板
async fn remove_template(name: String, force: bool) -> CliResult<()> {
    let template_manager = template::TemplateManager::new()?;
    
    // 如果不是强制删除，先确认
    if !force {
        let confirm = match dialoguer::Confirm::new()
            .with_prompt(format!("确定要删除模板 {}?", name))
            .default(false)
            .interact() {
                Ok(result) => result,
                Err(e) => return Err(CliError::Interaction(e.to_string()))
            };
            
        if !confirm {
            println!("{}", "操作已取消".bright_yellow());
            return Ok(());
        }
    }
    
    template_manager.remove_template(&name)?;
    
    Ok(())
} 