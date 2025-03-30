use std::path::PathBuf;
use std::process::{Command, Stdio};
use colored::Colorize;

use crate::error::{CliError, CliResult};
use crate::util::{check_rust_toolchain, find_project_root, is_lomus_project};

/// 运行Lomus AI应用
pub async fn run(dir: Option<PathBuf>) -> CliResult<()> {
    // 检查Rust工具链
    check_rust_toolchain()?;
    
    // 找到项目根目录
    let project_root = find_project_root(dir.as_ref().map(|p| p.as_path()))?;
    
    // 验证是否为有效项目
    if !is_lomus_project(&project_root) {
        return Err(CliError::InvalidProject(project_root.clone()));
    }
    
    println!("{}", format!("在 {} 运行Lomus AI应用", project_root.display()).bright_blue());
    
    // 编译项目
    println!("{}", "编译项目...".bright_blue());
    
    let status = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&project_root)
        .status()
        .map_err(|e| CliError::CommandFailed(format!("无法编译项目: {}", e)))?;
        
    if !status.success() {
        return Err(CliError::build("项目编译失败"));
    }
    
    println!("{}", "编译完成".bright_green());
    println!("{}", "运行应用...".bright_blue());
    
    // 运行应用
    let mut process = Command::new("cargo")
        .args(["run", "--release"])
        .current_dir(&project_root)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| CliError::CommandFailed(format!("无法启动应用: {}", e)))?;
        
    // 等待进程完成
    let result = process.wait()
        .map_err(|e| CliError::CommandFailed(format!("无法等待应用完成: {}", e)))?;
        
    if result.success() {
        println!("{}", "应用运行完成".bright_green());
        Ok(())
    } else {
        println!("{}", "应用运行失败".bright_red());
        Err(CliError::run("应用运行失败"))
    }
} 