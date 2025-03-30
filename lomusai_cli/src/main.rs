mod commands;
mod error;
mod config;
mod util;
mod template;

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "lomus")]
#[command(about = "Lomus AI CLI - 构建、开发和部署Lomus AI应用", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 初始化一个新的Lomus AI项目
    Init {
        /// 项目名称
        #[arg(short, long)]
        name: Option<String>,
        
        /// 模板类型 (agent, workflow, rag)
        #[arg(short, long)]
        template: Option<String>,
        
        /// 模板URL，用于从远程仓库加载模板
        #[arg(long)]
        template_url: Option<String>,
        
        /// 输出目录
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// 启动开发服务器
    Dev {
        /// 项目根目录
        #[arg(short, long)]
        dir: Option<PathBuf>,
        
        /// 开发服务器端口
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
        
        /// 启用热重载
        #[arg(short, long)]
        hot_reload: bool,
    },
    
    /// 运行Lomus AI应用
    Run {
        /// 项目根目录
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },
    
    /// 构建Lomus AI应用
    Build {
        /// 项目根目录
        #[arg(short, long)]
        dir: Option<PathBuf>,
        
        /// 输出目录
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// 部署Lomus AI应用
    Deploy {
        /// 项目根目录
        #[arg(short, long)]
        dir: Option<PathBuf>,
        
        /// 部署目标 (local, docker, aws, azure, gcp)
        #[arg(short, long, default_value = "local")]
        target: String,
    },
    
    /// 管理模板
    Template {
        #[command(subcommand)]
        action: TemplateCommands,
    },
}

#[derive(Subcommand)]
enum TemplateCommands {
    /// 列出所有可用模板
    List,
    
    /// 下载远程模板
    Download {
        /// 模板的URL，支持Git仓库或HTTP/HTTPS
        #[arg(short, long)]
        url: String,
        
        /// 模板的名称
        #[arg(short, long)]
        name: Option<String>,
    },
    
    /// 删除模板
    Remove {
        /// 要删除的模板名称
        #[arg(short, long)]
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_banner();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Init { name, template, template_url, output } => {
            commands::init::run(name, template, template_url, output).await?;
        },
        Commands::Dev { dir, port, hot_reload } => {
            commands::dev::run(dir, port, hot_reload).await?;
        },
        Commands::Run { dir } => {
            commands::run::run(dir).await?;
        },
        Commands::Build { dir, output } => {
            commands::build::run(dir, output).await?;
        },
        Commands::Deploy { dir, target } => {
            commands::deploy::run(dir, target).await?;
        },
        Commands::Template { action } => {
            match action {
                TemplateCommands::List => {
                    list_templates().await?;
                },
                TemplateCommands::Download { url, name } => {
                    download_template(url, name).await?;
                },
                TemplateCommands::Remove { name } => {
                    remove_template(name).await?;
                },
            }
        },
    }
    
    Ok(())
}

fn print_banner() {
    println!("{}", r#"
 _                                 _    ___ 
| |    ___  _ __ ___  _   _ ___  / \  |_ _|
| |   / _ \| '_ ` _ \| | | / __| \_/   | | 
| |__| (_) | | | | | | |_| \__ \  _    | | 
|_____\___/|_| |_| |_|\__,_|___/ (_)  |___|
                                           
    "#.bright_cyan());
    println!("{}", "Lomus AI CLI - 构建、开发和部署Lomus AI应用".bright_green());
    println!();
}

// 列出所有可用模板
async fn list_templates() -> Result<(), Box<dyn std::error::Error>> {
    use crate::template::TemplateManager;
    
    let template_manager = TemplateManager::new()?;
    let templates = template_manager.list_templates()?;
    
    if templates.is_empty() {
        println!("{}", "没有可用的模板".bright_yellow());
        println!("{}", "可以使用 'lomus template download --url <URL>' 下载模板".bright_blue());
        return Ok(());
    }
    
    println!("{}", "可用的模板:".bright_green());
    for template in templates {
        println!("  - {}", template);
    }
    
    Ok(())
}

// 下载远程模板
async fn download_template(url: String, name: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    use crate::template::TemplateManager;
    
    let template_manager = TemplateManager::new()?;
    
    let template_name = match name {
        Some(n) => n,
        None => {
            // 从URL提取模板名称
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
    println!("{}", format!("模板 '{}' 下载成功", template_name).bright_green());
    
    Ok(())
}

// 删除模板
async fn remove_template(name: String) -> Result<(), Box<dyn std::error::Error>> {
    use crate::template::TemplateManager;
    use std::fs;
    
    let template_manager = TemplateManager::new()?;
    let templates = template_manager.list_templates()?;
    
    if !templates.contains(&name) {
        println!("{}", format!("找不到模板 '{}'", name).bright_red());
        return Ok(());
    }
    
    // 确认删除
    use dialoguer::Confirm;
    let confirm = Confirm::new()
        .with_prompt(format!("确定要删除模板 '{}'?", name))
        .default(false)
        .interact()?;
        
    if !confirm {
        println!("{}", "操作已取消".bright_yellow());
        return Ok(());
    }
    
    // 删除模板目录
    let template_dir = template_manager.template_directory().join(&name);
    fs::remove_dir_all(&template_dir)?;
    
    println!("{}", format!("模板 '{}' 已删除", name).bright_green());
    
    Ok(())
}
