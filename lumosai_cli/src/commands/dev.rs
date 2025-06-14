use clap::Args;
use std::path::{Path, PathBuf};
use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use colored::Colorize;
use tokio::sync::mpsc;
use tokio::task;
use tokio::time::{sleep, Duration};
use notify::{Watcher, RecursiveMode};

use crate::error::{CliResult, CliError};
use crate::util::is_lumos_project;
use crate::server::{ui_server, api_server};

/// 开发服务器配置选项
#[derive(Args, Debug)]
pub struct DevOptions {
    /// 项目目录
    #[arg(long)]
    project_dir: Option<PathBuf>,
    
    /// API服务器端口
    #[arg(long, default_value = "4000")]
    port: u16,
    
    /// 启动UI服务器
    #[arg(long)]
    ui: bool,
    
    /// UI服务器端口
    #[arg(long, default_value = "4001")]
    ui_port: u16,
    
    /// 开发模式下的UI主题 (light/dark)
    #[arg(long, default_value = "light")]
    theme: String,
    
    /// 监视文件变更并自动重新加载
    #[arg(long)]
    watch: bool,
    
    /// 自定义工具目录
    #[arg(long)]
    tools: Option<PathBuf>,
}

impl Default for DevOptions {
    fn default() -> Self {
        Self {
            project_dir: None,
            port: 4000,
            ui: false,
            ui_port: 4001,
            theme: "light".to_string(),
            watch: false,
            tools: None,
        }
    }
}

/// 运行开发服务器
pub async fn run(options: DevOptions) -> CliResult<()> {
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

    // 检查是否是Lumosai项目
    if !is_lumos_project(&project_dir) {
        println!("{}", "警告: 当前目录不是标准的Lumosai项目目录结构".bright_yellow());
    }

    println!("{}", "启动 Lumosai 开发服务器...".bright_blue());
    println!("{}", format!("项目目录: {}", project_dir.display()).bright_blue());
    println!("{}", format!("API端口: {}", options.port).bright_blue());
    
    if options.ui {
        println!("{}", format!("UI端口: {}", options.ui_port).bright_blue());
        println!("{}", format!("UI主题: {}", options.theme).bright_blue());
    }
    
    if options.watch {
        println!("{}", "文件监视: 启用".bright_blue());
    }
    
    if let Some(tools_dir) = &options.tools {
        println!("{}", format!("自定义工具目录: {}", tools_dir.display()).bright_blue());
    }

    // 创建通道用于服务器之间的通信
    let (tx, mut rx) = mpsc::channel::<String>(100);
    
    // 设置中断处理
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        println!("{}", "\n正在关闭开发服务器...".bright_yellow());
    }).map_err(|e| CliError::internal(format!("无法设置中断处理: {}", e).as_str()))?;
    
    // 启动API服务器
    let api_handle = {
        let project_dir = project_dir.clone();
        let api_tx = tx.clone();
        let running = running.clone();
        
        task::spawn(async move {
            while running.load(Ordering::SeqCst) {
                println!("{}", "正在启动API服务器...".bright_blue());
                
                match api_server::start_server(options.port, project_dir.clone(), None).await {
                    Ok(_) => {
                        api_tx.send("api_server_stopped".to_string()).await.ok();
                        break;
                    },
                    Err(e) => {
                        println!("{}", format!("API服务器启动失败: {}，将在5秒后重试", e).bright_red());
                        sleep(Duration::from_secs(5)).await;
                    }
                }
                
                if !running.load(Ordering::SeqCst) {
                    break;
                }
            }
        })
    };
    
    // 如果启用UI，启动UI服务器
    let ui_handle = if options.ui {
        let project_dir = project_dir.clone();
        let ui_tx = tx.clone();
        let running = running.clone();
        let api_url = format!("http://localhost:{}", options.port);
        
        Some(task::spawn(async move {
            while running.load(Ordering::SeqCst) {
                println!("{}", "正在启动UI服务器...".bright_blue());
                
                // 查找UI目录
                let ui_dir = match ui_server::find_ui_dir() {
                    Ok(dir) => dir,
                    Err(e) => {
                        println!("{}", format!("查找UI目录失败: {}", e).bright_red());
                        ui_tx.send("ui_server_error".to_string()).await.ok();
                        break;
                    }
                };
                
                match ui_server::start_server(
                    ui_dir,
                    options.ui_port,
                    options.theme.clone(),
                    true, // 开发模式下总是使用dev_mode
                    Some(api_url.clone()),
                    project_dir.clone(),
                ).await {
                    Ok(_) => {
                        ui_tx.send("ui_server_stopped".to_string()).await.ok();
                        break;
                    },
                    Err(e) => {
                        println!("{}", format!("UI服务器启动失败: {}，将在5秒后重试", e).bright_red());
                        sleep(Duration::from_secs(5)).await;
                    }
                }
                
                if !running.load(Ordering::SeqCst) {
                    break;
                }
            }
        }))
    } else {
        None
    };
    
    // 如果启用文件监视，启动文件监视器
    let watcher_handle = if options.watch {
        let project_dir = project_dir.clone();
        let watcher_tx = tx.clone();
        let running = running.clone();
        
        Some(task::spawn(async move {
            start_file_watcher(&project_dir, watcher_tx, running).await
        }))
    } else {
        None
    };
    
    // 等待服务器事件或中断信号
    while running.load(Ordering::SeqCst) {
        match rx.recv().await {
            Some(msg) => {
                match msg.as_str() {
                    "api_server_stopped" => {
                        println!("{}", "API服务器已停止".bright_yellow());
                        break;
                    },
                    "ui_server_stopped" => {
                        println!("{}", "UI服务器已停止".bright_yellow());
                        if !options.watch {
                            // 如果没有启用文件监视，UI服务器停止后也停止API服务器
                            break;
                        }
                    },
                    "file_changed" => {
                        println!("{}", "检测到文件变更，重新加载...".bright_green());
                        // 这里可以添加重新加载逻辑，例如重启服务器
                    },
                    _ => {}
                }
            },
            None => break,
        }
        
        // 短暂休眠以减少CPU使用
        sleep(Duration::from_millis(100)).await;
    }
    
    // 设置终止标志
    running.store(false, Ordering::SeqCst);
    
    // 等待所有任务完成
    if let Some(handle) = ui_handle {
        let _ = handle.await;
    }
    let _ = api_handle.await;
    if let Some(handle) = watcher_handle {
        let _ = handle.await;
    }
    
    println!("{}", "开发服务器已关闭".bright_green());
    Ok(())
}

/// 启动文件监视器
async fn start_file_watcher(
    project_dir: &Path,
    tx: mpsc::Sender<String>,
    running: Arc<AtomicBool>,
) -> CliResult<()> {
    println!("{}", "启动文件监视器...".bright_blue());
    
    // 确定要监视的目录
    let src_dir = project_dir.join("src");
    if !src_dir.exists() {
        return Err(CliError::path_not_found(
            src_dir.to_string_lossy().to_string(),
            "源代码目录不存在",
        ));
    }
    
    // 使用通道在线程间通信
    let (watcher_tx, mut watcher_rx) = mpsc::channel(100);
    
    // 配置监视器
    let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
        match res {
            Ok(event) => {
                // 只关注修改和创建事件
                if event.kind.is_modify() || event.kind.is_create() {
                    let _ = watcher_tx.blocking_send(());
                }
            },
            Err(e) => println!("{}", format!("文件监视错误: {}", e).bright_red()),
        }
    }).map_err(|e| CliError::internal(format!("创建文件监视器失败: {}", e).as_str()))?;
    
    // 添加要监视的目录
    watcher.watch(&src_dir, RecursiveMode::Recursive)
        .map_err(|e| CliError::internal(format!("监视目录失败: {}", e).as_str()))?;
    
    // 可能的其他目录，如果存在也添加到监视
    let dirs_to_watch = [
        project_dir.join("config"),
        project_dir.join("assets"),
    ];
    
    for dir in &dirs_to_watch {
        if dir.exists() && dir.is_dir() {
            watcher.watch(dir, RecursiveMode::Recursive)
                .map_err(|e| CliError::internal(format!("监视目录失败: {}", e).as_str()))?;
        }
    }
    
    println!("{}", format!("正在监视目录: {}", src_dir.display()).bright_blue());
    
    // 防抖动计时器 - 避免过于频繁的重新加载
    let mut last_reload = tokio::time::Instant::now();
    let debounce_duration = Duration::from_secs(2);
    
    // 监视循环
    while running.load(Ordering::SeqCst) {
        match tokio::time::timeout(Duration::from_secs(1), watcher_rx.recv()).await {
            Ok(Some(_)) => {
                let now = tokio::time::Instant::now();
                if now.duration_since(last_reload) >= debounce_duration {
                    // 发送文件变更通知
                    let _ = tx.send("file_changed".to_string()).await;
                    last_reload = now;
                }
            },
            Ok(None) => break,
            Err(_) => {} // 超时，继续循环
        }
    }
    
    println!("{}", "文件监视器已停止".bright_yellow());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::fs;
    
    #[test]
    fn test_default_options() {
        let options = DevOptions::default();
        assert_eq!(options.port, 4000);
        assert_eq!(options.ui_port, 4001);
        assert_eq!(options.theme, "light");
        assert_eq!(options.ui, false);
        assert_eq!(options.watch, false);
        assert_eq!(options.tools, None);
    }
    
    #[test]
    fn test_custom_options() {
        let options = DevOptions {
            project_dir: Some(PathBuf::from("/tmp")),
            port: 5000,
            ui: true,
            ui_port: 5001,
            theme: "dark".to_string(),
            watch: true,
            tools: Some(PathBuf::from("/tmp/tools")),
        };
        
        assert_eq!(options.port, 5000);
        assert_eq!(options.ui_port, 5001);
        assert_eq!(options.theme, "dark");
        assert_eq!(options.ui, true);
        assert_eq!(options.watch, true);
        assert_eq!(options.tools, Some(PathBuf::from("/tmp/tools")));
    }
    
    #[test]
    fn test_invalid_directory() {
        use tokio::runtime::Runtime;
        let rt = Runtime::new().unwrap();
        
        let invalid_dir = temp_dir().join("invalid_dir_that_doesnt_exist_12345");
        let options = DevOptions {
            project_dir: Some(invalid_dir),
            ..DevOptions::default()
        };
        
        let result = rt.block_on(run(options));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_create_minimal_project_structure() -> CliResult<()> {
        // 创建临时项目目录
        let temp_dir = temp_dir().join("lumosai_test_project");
        let src_dir = temp_dir.join("src");
        let agents_dir = src_dir.join("agents");
        
        // 清理可能存在的旧目录
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        // 创建目录结构
        fs::create_dir_all(&agents_dir).unwrap();
        
        // 创建Cargo.toml
        fs::write(
            temp_dir.join("Cargo.toml"),
            r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2021"

[dependencies]
lumosai = "0.1.0"
"#,
        ).unwrap();
        
        // 测试is_lumos_project函数
        assert!(is_lumos_project(&temp_dir));
        
        // 清理
        fs::remove_dir_all(&temp_dir).unwrap();
    
    Ok(())
    }
} 