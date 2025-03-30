use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::time::sleep;
use notify::{Watcher, RecursiveMode, Event};
use colored::Colorize;

use crate::error::{CliError, CliResult};
use crate::util::{check_rust_toolchain, find_project_root, is_lomus_project, get_package_manager};

/// 开发服务器结构体
struct DevServer {
    /// 项目根目录
    project_root: PathBuf,
    
    /// 服务器端口
    port: u16,
    
    /// 后端进程
    backend_process: Option<Child>,
    
    /// UI进程
    ui_process: Option<Child>,
    
    /// 是否启用热重载
    hot_reload: bool,
}

impl DevServer {
    /// 创建开发服务器
    fn new(project_root: PathBuf, port: u16, hot_reload: bool) -> Self {
        DevServer {
            project_root,
            port,
            backend_process: None,
            ui_process: None,
            hot_reload,
        }
    }
    
    /// 启动服务器
    async fn start(&mut self) -> CliResult<()> {
        println!("{}", "启动Lomus开发服务器...".bright_cyan());
        
        // 验证项目
        if !is_lomus_project(&self.project_root) {
            return Err(CliError::InvalidProject(self.project_root.clone()));
        }
        
        // 在单独的线程中启动后端服务器
        self.start_backend_server()?;
        
        // 检查并启动UI服务器(如果有UI)
        self.start_ui_server()?;
        
        // 如果启用了热重载，设置文件监视器
        if self.hot_reload {
            let project_root = self.project_root.clone();
            let (tx, mut rx) = mpsc::channel(1);
            
            // 创建一个共享引用以便在通道中传递
            let server = Arc::new(Mutex::new(self.clone()));
            
            // 设置文件监视器
            let _watcher = self.setup_watcher(project_root, tx)?;
            
            println!("{}", "热重载已启用 - 监视文件变更...".bright_green());
            
            // 监听中断信号
            let server_clone = Arc::clone(&server);
            tokio::spawn(async move {
                tokio::signal::ctrl_c().await.unwrap();
                println!("\n{}", "收到终止信号，正在关闭...".bright_yellow());
                
                let mut server = server_clone.lock().unwrap();
                server.shutdown();
                
                std::process::exit(0);
            });
            
            // 监听文件变更事件
            while let Some(_) = rx.recv().await {
                println!("{}", "检测到文件变更，重启服务器...".bright_blue());
                
                let mut server = server.lock().unwrap();
                
                // 停止并重启后端
                server.stop_backend();
                sleep(Duration::from_millis(500)).await;
                match server.start_backend_server() {
                    Ok(_) => println!("{}", "后端服务器已重启".bright_green()),
                    Err(e) => println!("{} {}", "重启后端服务器失败:".bright_red(), e),
                }
            }
        } else {
            // 如果没有启用热重载，只需等待终止信号
            println!("{}", "开发服务器已启动 (按Ctrl+C停止)".bright_green());
            tokio::signal::ctrl_c().await.unwrap();
            println!("\n{}", "收到终止信号，正在关闭...".bright_yellow());
            self.shutdown();
        }
        
        Ok(())
    }
    
    /// 启动后端服务器
    fn start_backend_server(&mut self) -> CliResult<()> {
        println!("{}", "编译项目...".bright_blue());
        
        // 首先编译项目
        let compile_status = Command::new("cargo")
            .args(["build"])
            .current_dir(&self.project_root)
            .status()
            .map_err(|e| CliError::CommandFailed(format!("无法运行cargo build: {}", e)))?;
            
        if !compile_status.success() {
            return Err(CliError::build("项目编译失败"));
        }
        
        println!("{}", "启动后端服务器...".bright_blue());
        
        // 运行后端服务器
        let mut child = Command::new("cargo")
            .args(["run"])
            .current_dir(&self.project_root)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| CliError::CommandFailed(format!("无法启动后端服务器: {}", e)))?;
            
        // 等待一会以确保服务器启动
        std::thread::sleep(Duration::from_secs(2));
        
        // 检查服务器是否仍在运行
        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(CliError::run(format!(
                    "后端服务器异常退出，状态码: {:?}",
                    status.code()
                )));
            }
            Ok(None) => {
                // 服务器仍在运行，这是正常的
                self.backend_process = Some(child);
                println!("{}", "后端服务器已启动".bright_green());
            }
            Err(e) => {
                return Err(CliError::run(format!("检查后端服务器状态失败: {}", e)));
            }
        }
        
        Ok(())
    }
    
    /// 启动UI服务器(如果存在UI目录)
    fn start_ui_server(&mut self) -> CliResult<()> {
        let ui_dir = self.project_root.join("ui");
        
        // 检查UI目录是否存在
        if !ui_dir.exists() {
            println!("{}", "没有找到UI目录，跳过UI服务器启动".bright_yellow());
            return Ok(());
        }
        
        // 检查package.json是否存在
        let package_json = ui_dir.join("package.json");
        if !package_json.exists() {
            println!("{}", "UI目录中没有找到package.json，跳过UI服务器启动".bright_yellow());
            return Ok(());
        }
        
        // 确定使用哪个包管理器
        let package_manager = match get_package_manager(&ui_dir) {
            Some(pm) => pm,
            None => {
                println!("{}", "未找到支持的包管理器(yarn/pnpm/npm)，跳过UI服务器启动".bright_yellow());
                return Ok(());
            }
        };
        
        println!("{}", format!("使用 {} 启动UI服务器...", package_manager).bright_blue());
        
        // 启动UI服务器
        let dev_cmd = match package_manager.as_str() {
            "yarn" => "yarn",
            "pnpm" => "pnpm",
            "npm" => "npm",
            _ => unreachable!(),
        };
        
        let mut child = Command::new(dev_cmd)
            .args(["run", "dev", "--", "--port", &self.port.to_string()])
            .current_dir(&ui_dir)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| CliError::CommandFailed(format!("无法启动UI服务器: {}", e)))?;
            
        // 等待一会以确保服务器启动
        std::thread::sleep(Duration::from_secs(2));
        
        // 检查服务器是否仍在运行
        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(CliError::run(format!(
                    "UI服务器异常退出，状态码: {:?}",
                    status.code()
                )));
            }
            Ok(None) => {
                // 服务器仍在运行，这是正常的
                self.ui_process = Some(child);
                println!("{}", format!("UI服务器已启动: http://localhost:{}", self.port).bright_green());
            }
            Err(e) => {
                return Err(CliError::run(format!("检查UI服务器状态失败: {}", e)));
            }
        }
        
        Ok(())
    }
    
    /// 设置文件监视器用于热重载
    fn setup_watcher(&self, path: PathBuf, tx: mpsc::Sender<()>) -> CliResult<impl Watcher> {
        // 创建一个去抖动器，防止多个事件触发多次重启
        let (debouncer_tx, mut debouncer_rx) = mpsc::channel::<()>(1);
        
        // 创建一个克隆发送器，用于事件处理
        let tx_clone = tx.clone();
        
        // 在单独的任务中处理去抖动
        tokio::spawn(async move {
            while debouncer_rx.recv().await.is_some() {
                // 等待一小段时间，合并多个更改
                sleep(Duration::from_millis(300)).await;
                
                // 消耗所有积累的消息
                while let Ok(_) = debouncer_rx.try_recv() {}
                
                // 发送单个重启信号
                let _ = tx_clone.send(()).await;
            }
        });
        
        // 创建一个文件系统监视器
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    // 仅在特定文件类型更改时触发重建
                    if let Some(path) = event.paths.first() {
                        if let Some(ext) = path.extension() {
                            if ext == "rs" || ext == "toml" {
                                let _ = debouncer_tx.blocking_send(());
                            }
                        }
                    }
                }
                Err(e) => println!("监视错误: {:?}", e),
            }
        })
        .map_err(|e| CliError::Generic(format!("无法创建文件监视器: {}", e)))?;
        
        // 监视src目录和Cargo.toml
        watcher.watch(&path.join("src"), RecursiveMode::Recursive)
            .map_err(|e| CliError::Generic(format!("无法监视src目录: {}", e)))?;
            
        watcher.watch(&path.join("Cargo.toml"), RecursiveMode::NonRecursive)
            .map_err(|e| CliError::Generic(format!("无法监视Cargo.toml: {}", e)))?;
            
        Ok(watcher)
    }
    
    /// 停止后端服务器
    fn stop_backend(&mut self) {
        if let Some(mut child) = self.backend_process.take() {
            println!("{}", "停止后端服务器...".bright_yellow());
            
            // 尝试温和地终止进程
            match child.kill() {
                Ok(_) => {
                    let _ = child.wait();
                }
                Err(e) => {
                    println!("{} {}", "无法停止后端服务器:".bright_red(), e);
                }
            }
        }
    }
    
    /// 停止UI服务器
    fn stop_ui(&mut self) {
        if let Some(mut child) = self.ui_process.take() {
            println!("{}", "停止UI服务器...".bright_yellow());
            
            // 尝试终止进程
            match child.kill() {
                Ok(_) => {
                    let _ = child.wait();
                }
                Err(e) => {
                    println!("{} {}", "无法停止UI服务器:".bright_red(), e);
                }
            }
        }
    }
    
    /// 关闭所有服务器
    fn shutdown(&mut self) {
        self.stop_backend();
        self.stop_ui();
        println!("{}", "所有服务已停止".bright_green());
    }
}

// 需要实现Clone以便可以在Arc<Mutex>中使用
impl Clone for DevServer {
    fn clone(&self) -> Self {
        // 请注意，我们不克隆子进程句柄，因为它们无法正确克隆
        DevServer {
            project_root: self.project_root.clone(),
            port: self.port,
            backend_process: None,
            ui_process: None,
            hot_reload: self.hot_reload,
        }
    }
}

/// 运行开发服务器命令
pub async fn run(
    dir: Option<PathBuf>, 
    port: u16, 
    hot_reload: bool
) -> CliResult<()> {
    // 检查Rust工具链
    check_rust_toolchain()?;
    
    // 找到项目根目录
    let project_root = find_project_root(dir.as_ref().map(|p| p.as_path()))?;
    
    // 创建并启动开发服务器
    let mut server = DevServer::new(project_root, port, hot_reload);
    server.start().await
} 