use std::path::PathBuf;
use crate::error::CliResult;
use crate::util::{find_project_root, is_lumos_project};
use colored::Colorize;
use tokio::process::Command;

/// 运行Lumos AI应用
pub async fn run(
    project_dir: Option<PathBuf>,
) -> CliResult<()> {
    // 确定项目目录
    let project_dir = match project_dir {
        Some(dir) => dir,
        None => find_project_root()?,
    };
    
    // 检查是否为Lumos项目
    if !is_lumos_project(&project_dir) {
        println!("{}", "警告: 当前目录不是一个Lumos AI项目".bright_yellow());
        println!("{}", "如果这是错误的，请确认项目中包含lumosai依赖".bright_yellow());
    }
    
    // 构建运行命令
    println!("{}", format!("运行项目: {}", project_dir.display()).bright_blue());
    
    // 执行cargo run
    let mut cmd = Command::new("cargo");
    cmd.current_dir(&project_dir)
       .args(["run", "--release"]);
       
    // 运行命令
    let status = cmd.status().await?;
    
    if !status.success() {
        return Err(format!("应用运行失败，状态码: {:?}", status.code()).into());
    }
    
    Ok(())
} 