use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::fs;
use std::env;
use tokio::net::TcpListener;
use tokio::process::Command;
use tokio::time::sleep;
use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware, error, Error};
use actix_files as fs_web;
use actix_cors::Cors;
use serde::Serialize;
use colored::Colorize;
use std::time::Duration;
use ctrlc;

use crate::error::{CliResult, CliError};
use crate::util::get_available_port;

// UI服务器配置
#[derive(Debug, Clone)]
pub struct UiServerConfig {
    // UI资源所在目录
    pub ui_dir: PathBuf,
    // 服务器绑定地址
    pub bind_address: String,
    // 服务器端口
    pub port: u16,
    // 主题 (light/dark)
    pub theme: String,
    // 开发模式
    pub dev_mode: bool,
    // API服务器URL
    pub api_url: Option<String>,
    // 项目目录
    pub project_dir: PathBuf,
    // 是否为playground模式
    pub is_playground: bool,
    // playground相关：代理ID
    pub agent_id: Option<String>,
    // playground相关：保存历史
    pub save_history: bool,
}

impl Default for UiServerConfig {
    fn default() -> Self {
        Self {
            ui_dir: PathBuf::from("./ui"),
            bind_address: "127.0.0.1".to_string(),
            port: 4003,
            theme: "light".to_string(),
            dev_mode: false,
            api_url: None,
            project_dir: env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            is_playground: false,
            agent_id: None,
            save_history: true,
        }
    }
}

impl UiServerConfig {
    // 创建新配置
    pub fn new(
        ui_dir: PathBuf,
        port: u16,
        theme: String,
        dev_mode: bool,
        api_url: Option<String>,
        project_dir: PathBuf,
    ) -> Self {
        Self {
            ui_dir,
            bind_address: "127.0.0.1".to_string(),
            port,
            theme,
            dev_mode,
            api_url,
            project_dir,
            is_playground: false,
            agent_id: None,
            save_history: true,
        }
    }

    // 创建新的playground配置
    pub fn new_playground(
        ui_dir: PathBuf,
        port: u16,
        agent_id: Option<String>,
        save_history: bool,
        api_url: Option<String>,
        project_dir: PathBuf,
    ) -> Self {
        Self {
            ui_dir,
            bind_address: "127.0.0.1".to_string(),
            port,
            theme: "light".to_string(),  // playground默认使用light主题
            dev_mode: false,
            api_url,
            project_dir,
            is_playground: true,
            agent_id,
            save_history,
        }
    }

    // 获取完整绑定地址
    pub fn get_bind_address(&self) -> String {
        format!("{}:{}", self.bind_address, self.port)
    }
}

// API端点响应
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

// 服务器信息
#[derive(Serialize)]
struct ServerInfo {
    version: String,
    project_dir: String,
    theme: String,
    dev_mode: bool,
    api_url: Option<String>,
    is_playground: bool,
    agent_id: Option<String>,
    save_history: Option<bool>,
}

/// 查找UI目录
pub fn find_ui_dir() -> CliResult<PathBuf> {
    // 首先尝试从环境变量获取
    if let Ok(path) = env::var("LUMOSAI_UI_PATH") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Ok(path);
        }
    }
    
    // 否则使用相对于二进制的路径
    let exe_path = env::current_exe()
        .map_err(|e| CliError::io("获取可执行文件路径失败", e))?;
    
    let exe_dir = exe_path.parent()
        .ok_or_else(|| CliError::internal("无法获取可执行文件目录"))?;
    
    // 检查几个可能的位置
    let possible_paths = vec![
        exe_dir.join("ui"),
        exe_dir.join("../ui"),
        exe_dir.join("../../ui"),
        PathBuf::from("/usr/local/share/lumosai/ui"),
        PathBuf::from("/usr/share/lumosai/ui"),
    ];
    
    for path in possible_paths {
        if path.exists() && (path.join("dist").exists() || path.join("public").exists()) {
            return Ok(path);
        }
    }
    
    // 如果找不到，返回默认路径并打印警告
    println!("{}", "警告: 找不到UI目录，使用默认路径".bright_yellow());
    Ok(exe_dir.join("ui"))
}

/// 查找Playground目录
pub fn find_playground_dir() -> CliResult<PathBuf> {
    // 首先尝试从环境变量获取
    if let Ok(path) = env::var("LUMOSAI_PLAYGROUND_PATH") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Ok(path);
        }
    }
    
    // 否则使用相对于二进制的路径
    let exe_path = env::current_exe()
        .map_err(|e| CliError::io("获取可执行文件路径失败", e))?;
    
    let exe_dir = exe_path.parent()
        .ok_or_else(|| CliError::internal("无法获取可执行文件目录"))?;
    
    // 检查几个可能的位置
    let possible_paths = vec![
        exe_dir.join("playground"),
        exe_dir.join("../playground"),
        exe_dir.join("../../playground"),
        PathBuf::from("/usr/local/share/lumosai/playground"),
        PathBuf::from("/usr/share/lumosai/playground"),
    ];
    
    for path in possible_paths {
        if path.exists() && (path.join("dist").exists() || path.join("public").exists()) {
            return Ok(path);
        }
    }
    
    // 如果找不到，返回默认路径并打印警告
    println!("{}", "警告: 找不到Playground目录，使用默认路径".bright_yellow());
    Ok(exe_dir.join("playground"))
}

/// 获取UI静态资源目录
fn get_static_dir(ui_dir: &Path) -> PathBuf {
    let dist_dir = ui_dir.join("dist");
    let public_dir = ui_dir.join("public");
    
    if dist_dir.exists() {
        dist_dir
    } else if public_dir.exists() {
        public_dir
    } else {
        ui_dir.to_path_buf()
    }
}

/// 检查服务器端口是否可用
async fn check_port_available(port: u16) -> bool {
    TcpListener::bind(format!("127.0.0.1:{}", port)).await.is_ok()
}

/// UI服务器信息API处理器
async fn server_info(data: web::Data<UiServerConfig>) -> impl Responder {
    let config = data.get_ref();
    
    let info = ServerInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        project_dir: config.project_dir.to_string_lossy().to_string(),
        theme: config.theme.clone(),
        dev_mode: config.dev_mode,
        api_url: config.api_url.clone(),
        is_playground: config.is_playground,
        agent_id: config.agent_id.clone(),
        save_history: if config.is_playground { Some(config.save_history) } else { None },
    };
    
    HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(info),
        error: None,
    })
}

/// 启动开发服务器（使用npm/pnpm启动开发模式）
async fn start_dev_server(config: UiServerConfig) -> CliResult<()> {
    println!("{}", "正在开发模式下启动UI服务器...".bright_blue());
    
    // 检查是否存在package.json文件
    let package_json_path = config.ui_dir.join("package.json");
    if !package_json_path.exists() {
        return Err(CliError::path_not_found(
            package_json_path.to_string_lossy().to_string(),
            "UI目录中没有找到package.json文件",
        ));
    }
    
    // 检查npm或pnpm是否可用
    let use_pnpm = fs::read_to_string(&package_json_path)
        .map_err(|e| CliError::io_error(e, &package_json_path))?
        .contains("\"packageManager\": \"pnpm");
    
    let cmd = if use_pnpm { "pnpm" } else { "npm" };
    
    // 设置中断处理
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        println!("{}", "\n正在停止UI开发服务器...".bright_yellow());
    }).map_err(|e| CliError::internal(format!("无法设置中断处理程序: {}", e).as_str()))?;
    
    // 准备环境变量
    let mut cmd_env = std::collections::HashMap::new();
    
    // 设置主题
    cmd_env.insert("LUMOS_UI_THEME".to_string(), config.theme.clone());
    
    // 设置API URL
    if let Some(api_url) = &config.api_url {
        cmd_env.insert("LUMOS_API_URL".to_string(), api_url.clone());
    }
    
    // 设置项目目录
    cmd_env.insert("LUMOS_PROJECT_DIR".to_string(), config.project_dir.to_string_lossy().to_string());
    
    // 设置端口
    cmd_env.insert("PORT".to_string(), config.port.to_string());
    
    // 设置playground相关变量
    if config.is_playground {
        cmd_env.insert("LUMOS_IS_PLAYGROUND".to_string(), "true".to_string());
        
        if let Some(agent_id) = &config.agent_id {
            cmd_env.insert("LUMOS_AGENT_ID".to_string(), agent_id.clone());
        }
        
        cmd_env.insert("LUMOS_SAVE_HISTORY".to_string(), config.save_history.to_string());
    }
    
    // 启动开发服务器
    let mut dev_command = Command::new(cmd);
    dev_command
        .arg("run")
        .arg("dev")
        .current_dir(&config.ui_dir)
        .envs(cmd_env)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    
    println!("{}", format!("正在使用 {} 启动开发服务器...", cmd).bright_blue());
    println!("{}", format!("UI目录: {}", config.ui_dir.display()).bright_blue());
    println!("{}", format!("端口: {}", config.port).bright_blue());
    
    let mut child = dev_command.spawn()
        .map_err(|e| CliError::io(format!("启动UI开发服务器失败: {}", e).as_str(), e))?;
    
    // 打开浏览器
    println!("{}", "UI服务器已启动，正在打开浏览器...".bright_green());
    
    if let Err(e) = open::that(format!("http://localhost:{}", config.port)) {
        println!("{}", format!("无法自动打开浏览器: {}", e).bright_yellow());
        println!("{}", format!("请手动访问: http://localhost:{}", config.port).bright_green());
    }
    
    // 等待子进程完成或中断
    while running.load(Ordering::SeqCst) {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return Err(CliError::failed::<std::io::Error>(
                        format!("UI开发服务器异常退出，状态码: {:?}", status.code()).as_str(),
                        None,
                    ));
                }
                break;
            },
            Ok(None) => {
                // 进程仍在运行，短暂睡眠
                sleep(Duration::from_millis(100)).await;
            },
            Err(e) => {
                return Err(CliError::io("检查UI开发服务器进程状态失败", e));
            }
        }
    }
    
    // 如果被中断，确保子进程被终止
    if !running.load(Ordering::SeqCst) {
        let _ = child.kill().await;
    }
    
    Ok(())
}

/// 启动生产服务器（使用actix-web提供静态文件服务）
async fn start_production_server(config: UiServerConfig) -> CliResult<()> {
    println!("{}", "正在生产模式下启动UI服务器...".bright_blue());
    
    // 获取静态资源目录
    let static_dir = get_static_dir(&config.ui_dir);
    println!("{}", format!("静态资源目录: {}", static_dir.display()).bright_blue());
    
    // 设置中断处理
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        println!("{}", "\n正在停止UI服务器...".bright_yellow());
    }).map_err(|e| CliError::internal(format!("无法设置中断处理程序: {}", e).as_str()))?;
    
    // 创建应用数据
    let config_data = web::Data::new(config.clone());
    
    let app_factory = move || {
        let cors = Cors::permissive();
        
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .app_data(config_data.clone())
            .service(web::resource("/api/server-info").route(web::get().to(server_info)))
            .service(fs_web::Files::new("/", static_dir.clone())
                .index_file("index.html")
                .default_handler(web::route().to(serve_index)))
    };
    
    // 启动HTTP服务器
    let server = HttpServer::new(app_factory)
        .bind(config.get_bind_address())
        .map_err(|e| CliError::io("启动服务器失败", e))?
        .run();
    
    println!("{}", format!("UI服务器已启动，监听地址: {}", config.get_bind_address()).bright_green());
    println!("{}", "正在打开浏览器...".bright_green());
    
    if let Err(e) = open::that(format!("http://{}", config.get_bind_address())) {
        println!("{}", format!("无法自动打开浏览器: {}", e).bright_yellow());
        println!("{}", format!("请手动访问: http://{}", config.get_bind_address()).bright_green());
    }
    
    println!("{}", "按 Ctrl+C 停止服务器".bright_blue());
    
    // 等待中断信号
    let server_handle = server.handle();
    
    // 创建task监听ctrlc
    tokio::spawn(async move {
        while running.load(Ordering::SeqCst) {
            sleep(Duration::from_millis(100)).await;
        }
        server_handle.stop(true).await;
    });
    
    // 等待服务器结束
    server.await.map_err(|e| CliError::io("HTTP服务器错误", e))?;
    
    println!("{}", "UI服务器已停止".bright_green());
    Ok(())
}

/// 默认服务index.html
async fn serve_index(data: web::Data<UiServerConfig>) -> Result<HttpResponse, Error> {
    let static_dir = get_static_dir(&data.ui_dir);
    let index_path = static_dir.join("index.html");
    
    let content = match fs::read_to_string(&index_path) {
        Ok(content) => content,
        Err(e) => {
            return Err(error::ErrorInternalServerError(format!(
                "无法读取index.html: {}", e
            )));
        }
    };
    
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

/// 启动UI服务器，根据配置选择开发模式或生产模式
pub fn start_server(
    ui_dir: PathBuf,
    port: u16,
    theme: String,
    dev_mode: bool,
    api_url: Option<String>,
    project_dir: PathBuf,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = CliResult<()>> + Send>> {
    Box::pin(async move {
    // 检查端口是否可用
    if !check_port_available(port).await {
        let new_port = get_available_port(port).unwrap_or(port + 1);
        println!("{}", format!("端口 {} 已被占用，使用端口 {}", port, new_port).bright_yellow());
        
        return start_server(ui_dir, new_port, theme, dev_mode, api_url, project_dir).await;
    }
    
    // 创建配置
    let config = UiServerConfig::new(
        ui_dir,
        port,
        theme,
        dev_mode,
        api_url,
        project_dir,
    );
    
    // 根据模式启动服务器
    if dev_mode {
        start_dev_server(config).await
    } else {
        start_production_server(config).await
    }
    })
}

/// 启动Playground服务器
pub fn start_playground(
    playground_dir: PathBuf,
    port: u16,
    agent_id: Option<String>,
    save_history: bool,
    api_url: Option<String>,
    project_dir: PathBuf,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = CliResult<()>> + Send>> {
    Box::pin(async move {
    // 检查端口是否可用
    if !check_port_available(port).await {
        let new_port = get_available_port(port).unwrap_or(port + 1);
        println!("{}", format!("端口 {} 已被占用，使用端口 {}", port, new_port).bright_yellow());
        
        return start_playground(playground_dir, new_port, agent_id, save_history, api_url, project_dir).await;
    }
    
    // 创建配置
    let config = UiServerConfig::new_playground(
        playground_dir,
        port,
        agent_id,
        save_history,
        api_url,
        project_dir,
    );
    
    // 启动生产模式服务器
    start_production_server(config).await
    })
} 