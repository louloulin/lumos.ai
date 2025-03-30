use std::path::PathBuf;
use crate::error::CliResult;
use crate::util::{find_project_root, is_lomus_project, check_command_available, get_available_port};
use colored::Colorize;
use tokio::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// 启动开发服务器
pub async fn run(
    project_dir: Option<PathBuf>,
    port: u16,
    hot_reload: bool,
) -> CliResult<()> {
    // 确定项目目录
    let project_dir = match project_dir {
        Some(dir) => dir,
        None => find_project_root()?,
    };
    
    // 检查是否为Lomus项目
    if !is_lomus_project()? {
        println!("{}", "警告: 当前目录不是一个Lomus AI项目".bright_yellow());
        println!("{}", "如果这是错误的，请确认项目中包含lomusai依赖".bright_yellow());
    }
    
    // 获取可用端口
    let actual_port = get_available_port(port).unwrap_or(port);
    if actual_port != port {
        println!("{}", format!("端口 {} 已被占用，使用端口: {}", port, actual_port).bright_yellow());
    }
    
    // 启动开发服务器
    println!("{}", format!("启动开发服务器，目录: {}", project_dir.display()).bright_blue());
    println!("{}", format!("访问: http://localhost:{}", actual_port).bright_green());
    
    if hot_reload {
        // 检查cargo-watch
        if !check_command_available("cargo-watch") {
            println!("{}", "没有找到cargo-watch，正在尝试安装...".bright_yellow());
            
            // 安装cargo-watch
            let mut install_cmd = Command::new("cargo");
            install_cmd.args(["install", "cargo-watch"]);
            
            let status = install_cmd.status().await?;
            if !status.success() {
                return Err("安装cargo-watch失败，请手动安装: cargo install cargo-watch".into());
            }
            
            println!("{}", "cargo-watch安装成功".bright_green());
        }
        
        // 使用cargo-watch启动热重载开发服务器
        println!("{}", "启用热重载模式".bright_blue());
        
        // 设置中断处理
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();
        
        ctrlc::set_handler(move || {
            r.store(false, Ordering::SeqCst);
            println!("{}", "\n停止开发服务器...".bright_yellow());
        })
        .expect("无法设置Ctrl-C处理函数");
        
        // 启动带热重载的服务器
        let mut cmd = Command::new("cargo");
        cmd.current_dir(&project_dir)
           .args([
               "watch",
               "-x", "run",
               "-w", "src",
               "-w", "Cargo.toml",
           ])
           .env("LOMUS_DEV", "1")
           .env("LOMUS_PORT", actual_port.to_string());
           
        let mut child = cmd.spawn()?;
        
        while running.load(Ordering::SeqCst) {
            if let Ok(Some(status)) = child.try_wait() {
                if !status.success() {
                    println!("{}", format!("开发服务器异常退出，状态码: {:?}", status.code()).bright_red());
                }
                break;
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        // 确保进程被终止
        let _ = child.kill().await;
    } else {
        // 不使用热重载，直接运行项目
        let mut cmd = Command::new("cargo");
        cmd.current_dir(&project_dir)
           .args(["run"])
           .env("LOMUS_DEV", "1")
           .env("LOMUS_PORT", actual_port.to_string());
           
        let status = cmd.status().await?;
        
        if !status.success() {
            return Err(format!("开发服务器异常退出，状态码: {:?}", status.code()).into());
        }
    }
    
    Ok(())
} 