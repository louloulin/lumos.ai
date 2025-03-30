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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_banner();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Init { name, template, output } => {
            commands::init::run(name, template, output).await?;
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
