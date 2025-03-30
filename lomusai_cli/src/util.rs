use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;
use crate::error::{CliError, CliResult};
use colored::Colorize;

/// 检查Rust工具链是否正确安装
pub fn check_rust_toolchain() -> CliResult<()> {
    // 检查cargo是否可用
    match Command::new("cargo").arg("--version").output() {
        Ok(output) => {
            if output.status.success() {
                println!("{}", "Rust工具链检查通过".bright_green());
                Ok(())
            } else {
                Err(CliError::toolchain_error("Cargo命令执行失败"))
            }
        },
        Err(_) => {
            let message = "未找到Cargo，请确认Rust工具链已正确安装。\n请访问https://www.rust-lang.org/tools/install安装Rust";
            Err(CliError::toolchain_error(message))
        }
    }
}

/// 查找项目根目录
pub fn find_project_root() -> CliResult<PathBuf> {
    let current_dir = std::env::current_dir()
        .map_err(|e| CliError::Io(e))?;
        
    let mut dir = current_dir.clone();
    
    // 向上查找直到找到Cargo.toml或到达根目录
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            return Ok(dir);
        }
        
        if !dir.pop() {
            // 已经到达根目录
            break;
        }
    }
    
    // 如果没有找到项目根目录，使用当前目录
    println!("{}", "未找到项目根目录，使用当前目录".bright_yellow());
    Ok(current_dir)
}

/// 检查当前目录是否为Lomus AI项目
pub fn is_lomus_project() -> CliResult<bool> {
    // 获取项目根目录
    let project_root = find_project_root()?;
    
    // 检查Cargo.toml文件
    let cargo_toml = project_root.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Ok(false);
    }
    
    // 读取Cargo.toml内容
    let content = fs::read_to_string(&cargo_toml)
        .map_err(|e| CliError::io_error(e, &cargo_toml))?;
        
    // 检查是否包含lomusai依赖
    let is_lomus = content.contains("lomusai = ");
    
    Ok(is_lomus)
}

/// 创建目录，包括所有父目录
pub fn create_dir_all<P: AsRef<Path>>(path: P) -> CliResult<()> {
    fs::create_dir_all(&path)
        .map_err(|e| CliError::io_error(e, &path.as_ref()))
}

/// 递归复制目录
pub fn copy_dir_all<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> CliResult<()> {
    let from_path = from.as_ref();
    let to_path = to.as_ref();
    
    // 创建目标目录
    create_dir_all(to_path)?;
    
    // 读取源目录
    for entry in fs::read_dir(from_path)
        .map_err(|e| CliError::io_error(e, from_path))? {
            
        let entry = entry.map_err(|e| CliError::io_error(e, from_path))?;
        let from_entry = entry.path();
        let to_entry = to_path.join(entry.file_name());
        
        if from_entry.is_dir() {
            // 递归复制子目录
            copy_dir_all(&from_entry, &to_entry)?;
        } else {
            // 复制文件
            fs::copy(&from_entry, &to_entry)
                .map_err(|e| CliError::io_error(e, &from_entry))?;
        }
    }
    
    Ok(())
}

/// 检查命令是否可用
pub fn check_command_available(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// 获取有效端口号
pub fn get_available_port(start_port: u16) -> Option<u16> {
    // 从起始端口开始检查
    for port in start_port..65535 {
        // 尝试绑定TCP端口
        match std::net::TcpListener::bind(("127.0.0.1", port)) {
            Ok(_) => return Some(port),
            Err(_) => continue,
        }
    }
    
    None
}

/// 读取项目配置
pub fn read_project_config(project_dir: &Path) -> CliResult<toml::Value> {
    let config_file = project_dir.join("Cargo.toml");
    
    // 检查配置文件是否存在
    if !config_file.exists() {
        return Err(CliError::Other(format!(
            "找不到项目配置文件: {}",
            config_file.display()
        )));
    }
    
    // 读取配置文件内容
    let content = fs::read_to_string(&config_file)
        .map_err(|e| CliError::io_error(e, &config_file))?;
        
    // 解析TOML内容
    let config: toml::Value = toml::from_str(&content)
        .map_err(|e| CliError::Other(format!("解析配置文件错误: {}", e)))?;
        
    Ok(config)
} 