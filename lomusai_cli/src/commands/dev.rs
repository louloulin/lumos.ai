use std::path::{Path, PathBuf};
use crate::error::CliResult;
use crate::util::{find_project_root, is_lomus_project, check_command_available, get_available_port};
use colored::Colorize;
use tokio::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use dotenv::dotenv;
use std::fs;

/// 开发服务器配置
pub struct DevServerConfig {
    /// 项目目录
    pub project_dir: PathBuf,
    /// 服务器端口
    pub port: u16,
    /// 是否启用热重载
    pub hot_reload: bool,
    /// 日志级别
    pub log_level: String,
    /// 是否显示调试信息
    pub debug_mode: bool,
    /// 监视的额外目录
    pub watch_dirs: Vec<String>,
    /// 是否生成 API 文档
    pub generate_docs: bool,
}

impl Default for DevServerConfig {
    fn default() -> Self {
        Self {
            project_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            port: 3000,
            hot_reload: true,
            log_level: "info".to_string(),
            debug_mode: false,
            watch_dirs: vec![],
            generate_docs: true,
        }
    }
}

/// 启动开发服务器
pub async fn run(
    project_dir: Option<PathBuf>,
    port: u16,
    hot_reload: bool,
    log_level: Option<String>,
    debug_mode: bool,
    watch_dirs: Option<Vec<String>>,
    generate_docs: bool,
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
    
    // 创建服务器配置
    let config = DevServerConfig {
        project_dir: project_dir.clone(),
        port: actual_port,
        hot_reload,
        log_level: log_level.unwrap_or_else(|| "info".to_string()),
        debug_mode,
        watch_dirs: watch_dirs.unwrap_or_default(),
        generate_docs,
    };
    
    // 启动开发服务器
    println!("{}", format!("启动开发服务器，目录: {}", project_dir.display()).bright_blue());
    println!("{}", format!("访问: http://localhost:{}", actual_port).bright_green());
    
    if debug_mode {
        println!("{}", "调试模式已启用".bright_cyan());
    }
    
    if generate_docs {
        println!("{}", format!("API文档: http://localhost:{}/swagger-ui", actual_port).bright_green());
    }
    
    if hot_reload {
        run_with_hot_reload(&config).await?;
    } else {
        run_without_hot_reload(&config).await?;
    }
    
    Ok(())
}

/// 使用热重载运行开发服务器
async fn run_with_hot_reload(config: &DevServerConfig) -> CliResult<()> {
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
    
    // 构建监视目录列表
    let mut watch_args = vec![
        "watch",
        "-x", "run",
        "-w", "src",
        "-w", "Cargo.toml",
    ];
    
    // 添加额外的监视目录
    for dir in &config.watch_dirs {
        watch_args.push("-w");
        watch_args.push(dir);
    }
    
    // 启动带热重载的服务器
    let mut cmd = Command::new("cargo");
    cmd.current_dir(&config.project_dir)
       .args(watch_args)
       .env("LOMUS_DEV", "1")
       .env("LOMUS_PORT", config.port.to_string())
       .env("LOMUS_LOG_LEVEL", &config.log_level)
       .env("LOMUS_DEBUG", config.debug_mode.to_string())
       .env("LOMUS_API_DOCS", config.generate_docs.to_string());
       
    let mut child = cmd.spawn()?;
    
    println!("{}", "开发服务器已启动，按 Ctrl+C 停止".bright_cyan());
    println!("{}", "路由概览:".bright_cyan());
    println!("  {}", format!("GET /api - API 状态").bright_white());
    println!("  {}", format!("GET /api/agents - 获取所有智能体").bright_white());
    println!("  {}", format!("POST /api/agents/:agentId/generate - 生成响应").bright_white());
    println!("  {}", format!("POST /api/agents/:agentId/stream - 流式生成响应").bright_white());
    println!("  {}", format!("GET /api/tools - 获取所有工具").bright_white());
    println!("  {}", format!("POST /api/tools/:toolId/execute - 执行工具").bright_white());
    println!("  {}", format!("GET /api/workflows - 获取所有工作流").bright_white());
    println!("  {}", format!("POST /api/workflows/:workflowId/start - 开始工作流").bright_white());
    
    if config.generate_docs {
        println!("  {}", format!("GET /swagger-ui - API 文档").bright_white());
        println!("  {}", format!("GET /openapi.json - OpenAPI 规范").bright_white());
    }
    
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
    
    Ok(())
}

/// 不使用热重载运行开发服务器
async fn run_without_hot_reload(config: &DevServerConfig) -> CliResult<()> {
    // 不使用热重载，直接运行项目
    let mut cmd = Command::new("cargo");
    cmd.current_dir(&config.project_dir)
       .args(["run"])
       .env("LOMUS_DEV", "1")
       .env("LOMUS_PORT", config.port.to_string())
       .env("LOMUS_LOG_LEVEL", &config.log_level)
       .env("LOMUS_DEBUG", config.debug_mode.to_string())
       .env("LOMUS_API_DOCS", config.generate_docs.to_string());
       
    let status = cmd.status().await?;
    
    if !status.success() {
        return Err(format!("开发服务器异常退出，状态码: {:?}", status.code()).into());
    }
    
    Ok(())
}

/// 生成 OpenAPI 文档
fn generate_api_docs(_config: &DevServerConfig) -> CliResult<()> {
    // TODO: 实现 OpenAPI 文档生成
    Ok(())
}

/// 设置环境变量
fn setup_environment(config: &DevServerConfig) -> CliResult<()> {
    // 设置必要的环境变量
    std::env::set_var("LOMUS_DEV", "1");
    std::env::set_var("LOMUS_PORT", config.port.to_string());
    std::env::set_var("LOMUS_LOG_LEVEL", &config.log_level);
    std::env::set_var("LOMUS_DEBUG", config.debug_mode.to_string());
    std::env::set_var("LOMUS_API_DOCS", config.generate_docs.to_string());
    
    // 加载 .env 文件
    if Path::new(".env").exists() {
        println!("{}", "加载 .env 文件".bright_cyan());
        match dotenv() {
            Ok(_) => println!("{}", ".env 文件加载成功".bright_green()),
            Err(e) => println!("{}", format!(".env 文件加载失败: {}", e).bright_red()),
        }
    }
    
    // 加载 .env.development 文件
    let env_dev_path = Path::new(".env.development");
    if env_dev_path.exists() {
        println!("{}", "加载 .env.development 文件".bright_cyan());
        match dotenv::from_path(env_dev_path) {
            Ok(_) => println!("{}", ".env.development 文件加载成功".bright_green()),
            Err(e) => println!("{}", format!(".env.development 文件加载失败: {}", e).bright_red()),
        }
    }
    
    // 日志环境变量的设置
    if std::env::var("RUST_LOG").is_err() {
        // 如果没有设置 RUST_LOG，则根据日志级别设置
        let rust_log = match config.log_level.as_str() {
            "trace" => "trace",
            "debug" => "debug",
            "info" => "info",
            "warn" => "warn",
            "error" => "error",
            _ => "info",
        };
        std::env::set_var("RUST_LOG", rust_log);
    }
    
    Ok(())
} 