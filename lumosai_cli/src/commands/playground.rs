use clap::Args;
use std::path::PathBuf;
use std::env;
use colored::Colorize;

use crate::error::{CliResult, CliError};
use crate::server::ui_server;

/// 运行Lumosai交互式测试环境
#[derive(Args, Debug)]
pub struct PlaygroundOptions {
    /// 项目目录
    #[arg(long)]
    project_dir: Option<PathBuf>,
    
    /// 测试环境端口
    #[arg(long, default_value = "4000")]
    port: u16,
    
    /// 要测试的代理ID
    #[arg(long)]
    agent: Option<String>,
    
    /// 不保存测试历史
    #[arg(long)]
    no_save_history: bool,
    
    /// 自定义API URL
    #[arg(long)]
    api_url: Option<String>,
}

impl Default for PlaygroundOptions {
    fn default() -> Self {
        Self {
            project_dir: None,
            port: 4000,
            agent: None,
            no_save_history: false,
            api_url: None,
        }
    }
}

/// 运行Playground命令
pub async fn run(options: PlaygroundOptions) -> CliResult<()> {
    // 解析项目目录
    let project_dir = match &options.project_dir {
        Some(dir) => dir.clone(),
        None => env::current_dir().map_err(|e| CliError::io("获取当前目录失败", e))?,
    };

    // 检查项目目录是否存在
    if !project_dir.exists() {
        return Err(CliError::path_not_found(
            project_dir.to_string_lossy().to_string(),
            "项目目录不存在",
        ));
    }

    println!("{}", "启动 Lumosai 交互式测试环境...".bright_blue());
    println!("{}", format!("项目目录: {}", project_dir.display()).bright_blue());
    println!("{}", format!("端口: {}", options.port).bright_blue());
    
    if let Some(agent) = &options.agent {
        println!("{}", format!("代理ID: {}", agent).bright_blue());
    }
    
    println!("{}", format!("保存历史: {}", !options.no_save_history).bright_blue());
    
    if let Some(api_url) = &options.api_url {
        println!("{}", format!("API URL: {}", api_url).bright_blue());
    }

    // 查找Playground UI目录
    let playground_dir = ui_server::find_playground_dir()?;
    
    println!("{}", format!("Playground UI目录: {}", playground_dir.display()).bright_blue());

    // 启动Playground服务器
    ui_server::start_playground(
        playground_dir,
        options.port,
        options.agent,
        !options.no_save_history,
        options.api_url,
        project_dir,
    ).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_default_options() {
        let options = PlaygroundOptions::default();
        assert_eq!(options.port, 4000);
        assert_eq!(options.agent, None);
        assert_eq!(options.no_save_history, false);
        assert_eq!(options.api_url, None);
    }

    #[test]
    fn test_invalid_directory() {
        use tokio::runtime::Runtime;
        let rt = Runtime::new().unwrap();
        
        let invalid_dir = temp_dir().join("invalid_dir_that_doesnt_exist_12345");
        let options = PlaygroundOptions {
            project_dir: Some(invalid_dir),
            ..PlaygroundOptions::default()
        };
        
        let result = rt.block_on(run(options));
        assert!(result.is_err());
    }

    #[test]
    fn test_playground_options() {
        let options = PlaygroundOptions {
            project_dir: Some(PathBuf::from(".")),
            port: 5000,
            agent: Some("test-agent".to_string()),
            no_save_history: true,
            api_url: Some("http://localhost:8000".to_string()),
        };
        
        assert_eq!(options.port, 5000);
        assert_eq!(options.agent, Some("test-agent".to_string()));
        assert_eq!(options.no_save_history, true);
        assert_eq!(options.api_url, Some("http://localhost:8000".to_string()));
    }
} 