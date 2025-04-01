use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use std::process::Command;
use std::net::TcpListener;
use rand::Rng;
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
    let current_dir = env::current_dir()
        .map_err(|e| CliError::io("无法获取当前目录", e))?;

    // 首先检查当前目录是否为项目根目录
    if is_lumos_project(&current_dir) {
        return Ok(current_dir);
    }
    
    // 向上递归搜索
    let mut path = current_dir.clone();
    while let Some(parent) = path.parent() {
        if is_lumos_project(parent) {
            return Ok(parent.to_path_buf());
        }
        path = parent.to_path_buf();
    }
    
    // 未找到项目根目录，返回当前目录
    Ok(current_dir)
}

/// 检查目录是否为Lumosai项目
pub fn is_lumos_project(path: &Path) -> bool {
    // 检查是否存在Cargo.toml
    let cargo_toml = path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return false;
    }
    
    // 检查是否存在src目录
    let src_dir = path.join("src");
    if !src_dir.exists() || !src_dir.is_dir() {
        return false;
    }
    
    // 检查是否存在任一Lumosai特定目录
    let agents_dir = src_dir.join("agents");
    let tools_dir = src_dir.join("tools");
    let workflows_dir = src_dir.join("workflows");
    
    agents_dir.exists() || tools_dir.exists() || workflows_dir.exists()
}

/// 检查命令是否可用
pub fn check_command_available(command: &str) -> bool {
    let result = if cfg!(target_os = "windows") {
        Command::new("where")
            .arg(command)
            .output()
    } else {
        Command::new("which")
            .arg(command)
            .output()
    };
    
    match result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// 获取一个可用的端口
pub fn get_available_port(start_port: u16) -> Option<u16> {
    let mut rng = rand::thread_rng();
    
    // 尝试找一个可用端口，首先从起始端口开始尝试
    if is_port_available(start_port) {
        return Some(start_port);
    }
    
    // 然后尝试后续10个端口
    for port in (start_port + 1)..=(start_port + 10) {
        if is_port_available(port) {
            return Some(port);
        }
    }
    
    // 最后随机尝试更大范围的端口
    for _ in 0..20 {
        let port = rng.gen_range(3000..10000);
        if is_port_available(port) {
            return Some(port);
        }
    }
    
    None
}

/// 检查端口是否可用
pub fn is_port_available(port: u16) -> bool {
    TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok()
}

/// 创建目录（如果不存在）
pub fn ensure_dir_exists<P: AsRef<Path>>(path: P) -> CliResult<()> {
    let path = path.as_ref();
    if !path.exists() {
        fs::create_dir_all(path)
            .map_err(|e| CliError::io_error(e, path))?;
    }
    Ok(())
}

/// 复制文件或目录（递归）
pub fn copy_recursively<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> CliResult<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    
    if !src.exists() {
        return Err(CliError::path_not_found(src, "源路径不存在"));
    }
    
    if src.is_dir() {
        if !dst.exists() {
            fs::create_dir_all(dst)
                .map_err(|e| CliError::io_error(e, dst))?;
        }
        
        for entry in fs::read_dir(src)
            .map_err(|e| CliError::io_error(e, src))?
        {
            let entry = entry.map_err(|e| CliError::io_error(e, src))?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            
            if src_path.is_dir() {
                copy_recursively(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)
                    .map_err(|e| CliError::io_error(e, format!("{} -> {}", src_path.display(), dst_path.display())))?;
            }
        }
    } else {
        // 确保目标目录存在
        if let Some(parent) = dst.parent() {
            ensure_dir_exists(parent)?;
        }
        
        fs::copy(src, dst)
            .map_err(|e| CliError::io_error(e, format!("{} -> {}", src.display(), dst.display())))?;
    }
    
    Ok(())
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