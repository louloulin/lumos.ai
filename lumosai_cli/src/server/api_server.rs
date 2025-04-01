use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use tokio::net::TcpListener;
use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware, error, Error};
use actix_cors::Cors;
use serde::{Serialize, Deserialize};
use colored::Colorize;

use crate::error::{CliResult, CliError};
use crate::util::get_available_port;

// API服务器配置
#[derive(Debug, Clone)]
pub struct ApiServerConfig {
    // 服务器绑定地址
    pub bind_address: String,
    // 服务器端口
    pub port: u16,
    // 项目目录
    pub project_dir: PathBuf,
    // API模块路径
    pub api_module_path: PathBuf,
}

impl Default for ApiServerConfig {
    fn default() -> Self {
        let project_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let api_module_path = project_dir.join("src").join("api");
        
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 4000,
            project_dir,
            api_module_path,
        }
    }
}

impl ApiServerConfig {
    // 创建新配置
    pub fn new(
        port: u16,
        project_dir: PathBuf,
        api_module_path: Option<PathBuf>,
    ) -> Self {
        let api_path = api_module_path.unwrap_or_else(|| project_dir.join("src").join("api"));
        
        Self {
            bind_address: "127.0.0.1".to_string(),
            port,
            project_dir,
            api_module_path: api_path,
        }
    }

    // 获取完整绑定地址
    pub fn get_bind_address(&self) -> String {
        format!("{}:{}", self.bind_address, self.port)
    }
}

// API响应
#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

// API信息
#[derive(Serialize)]
struct ApiInfo {
    version: String,
    agents: Vec<String>,
    endpoints: Vec<String>,
}

/// 检查服务器端口是否可用
async fn check_port_available(port: u16) -> bool {
    TcpListener::bind(format!("127.0.0.1:{}", port)).await.is_ok()
}

/// 查找项目中的所有代理
fn find_agents(agents_dir: &Path) -> CliResult<Vec<String>> {
    let mut agents = Vec::new();

    if !agents_dir.exists() {
        return Ok(agents);
    }

    for entry in fs::read_dir(agents_dir)
        .map_err(|e| CliError::io(format!("无法读取代理目录: {}", agents_dir.display()), e))? {
        let entry = entry.map_err(|e| CliError::io("读取目录条目失败", e))?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    // 检查是否存在agent.rs文件来确认这是一个代理目录
                    if path.join("agent.rs").exists() || path.join("mod.rs").exists() {
                        agents.push(name_str.to_string());
                    }
                }
            }
        }
    }

    Ok(agents)
}

/// API信息处理器
async fn api_info(data: web::Data<ApiServerConfig>) -> impl Responder {
    let config = data.get_ref();
    
    // 查找代理
    let agents_dir = config.project_dir.join("src").join("agents");
    let agents = match find_agents(&agents_dir) {
        Ok(a) => a,
        Err(_) => Vec::new(),
    };
    
    // 构建可用端点列表
    let mut endpoints = vec![
        "/api".to_string(),
        "/api/info".to_string(),
    ];
    
    // 添加代理端点
    for agent in &agents {
        endpoints.push(format!("/api/agents/{}", agent));
        endpoints.push(format!("/api/agents/{}/chat", agent));
    }
    
    let info = ApiInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        agents,
        endpoints,
    };
    
    HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(info),
        error: None,
    })
}

/// 检查API模块是否存在
fn check_api_module(config: &ApiServerConfig) -> bool {
    config.api_module_path.exists() && config.api_module_path.join("mod.rs").exists()
}

/// 启动API服务器
pub async fn start_server(
    port: u16,
    project_dir: PathBuf,
    api_module_path: Option<PathBuf>,
) -> CliResult<()> {
    // 检查端口是否可用
    if !check_port_available(port).await {
        let new_port = get_available_port(port).unwrap_or(port + 1);
        println!("{}", format!("端口 {} 已被占用，使用端口 {}", port, new_port).bright_yellow());
        
        return start_server(new_port, project_dir, api_module_path).await;
    }
    
    // 创建配置
    let config = ApiServerConfig::new(
        port,
        project_dir.clone(),
        api_module_path,
    );
    
    // 检查API模块是否存在
    if !check_api_module(&config) {
        println!("{}", "警告: API模块不存在，只提供基本API信息".bright_yellow());
        println!("{}", "使用 'lumosai api' 命令生成API模块".bright_yellow());
    }
    
    println!("{}", "启动API服务器...".bright_blue());
    println!("{}", format!("项目目录: {}", project_dir.display()).bright_blue());
    println!("{}", format!("API模块路径: {}", config.api_module_path.display()).bright_blue());
    println!("{}", format!("绑定地址: {}", config.get_bind_address()).bright_blue());
    
    let config_data = web::Data::new(config.clone());
    
    // 创建并启动HTTP服务器
    let server = HttpServer::new(move || {
        // 配置CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .app_data(config_data.clone())
            .service(web::resource("/api").route(web::get().to(api_info)))
            .service(web::resource("/api/info").route(web::get().to(api_info)))
    })
    .bind(config.get_bind_address())
    .map_err(|e| CliError::io(format!("无法绑定到端口: {}", config.port), e))?
    .run();
    
    println!("{}", "API服务器已启动".bright_green());
    println!("{}", format!("访问: http://localhost:{}/api/info", config.port).bright_green());
    
    // 等待服务器结束
    server.await
        .map_err(|e| CliError::io("启动服务器时出错", e))?;
    
    Ok(())
} 