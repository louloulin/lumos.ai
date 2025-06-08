use clap::Parser;
use colored::Colorize;
use lumosai_cli::{Cli, Commands, error::{CliResult, CliError}, commands, template};

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