use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use colored::Colorize;
use tokio::process::Command;
use crate::error::{CliResult, CliError};
use crate::util::{find_project_root, is_lumos_project, check_command_available, get_available_port};
use std::env;
use crate::server::ui_server;

/// UI服务器配置
pub struct UiServerConfig {
    /// 项目目录
    pub project_dir: PathBuf,
    /// 服务器端口
    pub port: u16,
    /// 主题设置 (light/dark)
    pub theme: String,
    /// 开发模式
    pub dev_mode: bool,
    /// API服务器URL
    pub api_url: Option<String>,
}

impl Default for UiServerConfig {
    fn default() -> Self {
        Self {
            project_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            port: 4003,
            theme: "light".to_string(),
            dev_mode: false,
            api_url: None,
        }
    }
}

/// 启动Lumos AI UI服务器
pub async fn run(
    project_dir: Option<PathBuf>,
    port: u16,
    theme: Option<String>,
    dev_mode: bool,
    api_url: Option<String>,
) -> CliResult<()> {
    let project_dir = match project_dir {
        Some(dir) => dir,
        None => env::current_dir().map_err(|e| CliError::io("获取当前目录失败", e))?,
    };

    // 检查项目目录
    if !project_dir.exists() {
        return Err(CliError::path_not_found(
            project_dir.to_string_lossy().to_string(),
            "项目目录不存在",
        ));
    }

    println!("{}", "启动Lumos AI UI服务器...".bright_cyan());
    println!("{}", format!("项目目录: {}", project_dir.display()).bright_blue());
    println!("{}", format!("端口: {}", port).bright_blue());

    if let Some(theme_value) = &theme {
        println!("{}", format!("主题: {}", theme_value).bright_blue());
    }

    if let Some(api) = &api_url {
        println!("{}", format!("API URL: {}", api).bright_blue());
    }

    if dev_mode {
        println!("{}", "开发模式: 启用".bright_yellow());
    }

    // 查找UI目录
    let ui_dir = ui_server::find_ui_dir()?;
    println!("{}", format!("UI目录: {}", ui_dir.display()).bright_blue());

    // 启动UI服务器
    ui_server::start_server(
        ui_dir,
        port,
        theme.unwrap_or_else(|| "light".to_string()),
        dev_mode,
        api_url,
        project_dir,
    ).await
}

/// 获取UI服务器路径
fn get_ui_server_path() -> CliResult<PathBuf> {
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
        if path.exists() && path.join("server.js").exists() {
            return Ok(path);
        }
    }
    
    // 如果找不到，返回默认路径并打印警告
    println!("{}", "警告: 找不到UI服务器路径，使用默认路径".bright_yellow());
    Ok(exe_dir.join("ui"))
}

/// 确保UI依赖已安装
async fn ensure_ui_dependencies(config: &UiServerConfig) -> CliResult<()> {
    // 检查Node.js
    if !check_command_available("node") {
        return Err("找不到Node.js。请安装Node.js (v14+): https://nodejs.org".into());
    }
    
    // 检查npm
    if !check_command_available("npm") {
        return Err("找不到npm。请确保Node.js正确安装".into());
    }
    
    // 检查UI文件夹是否存在
    let ui_dir = config.project_dir.join(".lumos").join("ui");
    if !ui_dir.exists() {
        println!("{}", "UI文件夹不存在，正在安装...".bright_yellow());
        
        // 创建UI目录
        std::fs::create_dir_all(&ui_dir)?;
        
        // 克隆或下载UI文件
        let mut clone_cmd = Command::new("git");
        clone_cmd
            .args(["clone", "https://github.com/lumosai/lumos-ui.git", "."])
            .current_dir(&ui_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
            
        let clone_status = clone_cmd.status().await?;
        if !clone_status.success() {
            return Err("无法下载UI文件。请检查您的网络连接或手动安装".into());
        }
        
        // 安装UI依赖
        println!("{}", "正在安装UI依赖...".bright_yellow());
        let mut install_cmd = Command::new("npm");
        install_cmd
            .args(["install"])
            .current_dir(&ui_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
            
        let install_status = install_cmd.status().await?;
        if !install_status.success() {
            return Err("无法安装UI依赖。请尝试手动安装".into());
        }
        
        println!("{}", "UI安装成功".bright_green());
    }
    
    Ok(())
}

/// 启动UI服务器
async fn start_ui_server(config: &UiServerConfig) -> CliResult<()> {
    // UI文件夹路径
    let ui_dir = config.project_dir.join(".lumos").join("ui");
    
    // 设置中断处理
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        println!("{}", "\n停止UI服务器...".bright_yellow());
    })?;
    
    // 构建环境变量
    let mut env_vars = vec![
        ("PORT", config.port.to_string()),
        ("LUMOS_UI_THEME", config.theme.clone()),
    ];
    
    // 添加API URL（如果提供）
    if let Some(api_url) = &config.api_url {
        env_vars.push(("LUMOS_API_URL", api_url.clone()));
    }
    
    // 确定npm命令
    let npm_command = if config.dev_mode { "dev" } else { "start" };
    
    // 启动UI服务器
    let mut cmd = Command::new("npm");
    cmd.args([npm_command])
        .current_dir(&ui_dir);
        
    // 添加环境变量
    for (key, value) in env_vars {
        cmd.env(key, value);
    }
    
    // 启动进程
    let mut child = cmd.spawn()?;
    
    println!("{}", "UI服务器已启动，按 Ctrl+C 停止".bright_cyan());
    println!("{}", "UI功能概览:".bright_cyan());
    println!("  {}", "- 代理管理和可视化".bright_white());
    println!("  {}", "- 工作流监控和调试".bright_white());
    println!("  {}", "- 工具调用检查".bright_white());
    println!("  {}", "- 性能指标和日志".bright_white());
    
    // 自动打开浏览器
    if let Err(e) = open::that(format!("http://localhost:{}", config.port)) {
        println!("{}", format!("无法自动打开浏览器: {}", e).bright_yellow());
        println!("{}", format!("请手动访问: http://localhost:{}", config.port).bright_green());
    }
    
    // 等待服务器结束或收到中断信号
    while running.load(Ordering::SeqCst) {
        if let Ok(Some(status)) = child.try_wait() {
            if !status.success() {
                println!("{}", format!("UI服务器异常退出，状态码: {:?}", status.code()).bright_red());
            }
            break;
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    // 确保进程被终止
    let _ = child.kill().await;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_get_ui_server_path_from_env() {
        // 创建临时目录
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();
        
        // 创建server.js文件以满足条件
        fs::create_dir_all(temp_path.join("dist")).unwrap();
        fs::write(temp_path.join("dist").join("index.html"), "<html></html>").unwrap();
        
        // 设置环境变量
        std::env::set_var("LUMOSAI_UI_PATH", temp_path.to_str().unwrap());
        
        // 测试路径获取
        let result = ui_server::find_ui_dir();
        assert!(result.is_ok());
        
        let path = result.unwrap();
        assert_eq!(path, temp_path);
        
        // 清理环境变量
        std::env::remove_var("LUMOSAI_UI_PATH");
    }
    
    #[test]
    fn test_ui_server_config_default() {
        let config = UiServerConfig::default();
        
        assert_eq!(config.port, 4003);
        assert_eq!(config.theme, "light");
        assert_eq!(config.dev_mode, false);
        assert_eq!(config.api_url, None);
        
        // 当前目录应该是有效的路径
        assert!(config.project_dir.exists());
    }
    
    #[tokio::test]
    async fn test_run_with_invalid_directory() {
        let result = run(
            Some(PathBuf::from("/非常不太可能存在的路径/abcdef")),
            4003,
            None,
            false,
            None
        ).await;
        
        assert!(result.is_err());
        
        // 检查错误类型
        if let Err(e) = result {
            assert!(e.to_string().contains("项目目录不存在"));
        }
    }
} 