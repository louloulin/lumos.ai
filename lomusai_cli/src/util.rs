use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::error::{CliError, CliResult};
use console::style;

/// 打印带颜色的信息
pub fn print_info(message: &str) {
    println!("{} {}", style("[INFO]").green().bold(), message);
}

/// 打印带颜色的成功信息
pub fn print_success(message: &str) {
    println!("{} {}", style("[SUCCESS]").green().bold(), message);
}

/// 打印带颜色的警告信息
pub fn print_warning(message: &str) {
    println!("{} {}", style("[WARNING]").yellow().bold(), message);
}

/// 打印带颜色的错误信息
pub fn print_error(message: &str) {
    println!("{} {}", style("[ERROR]").red().bold(), message);
}

/// 递归复制目录
pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> CliResult<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    
    if !src.exists() {
        return Err(CliError::Generic(format!("源目录不存在: {}", src.display())));
    }
    
    if !dst.exists() {
        fs::create_dir_all(dst).map_err(CliError::Io)?;
    }
    
    for entry in fs::read_dir(src).map_err(CliError::Io)? {
        let entry = entry.map_err(CliError::Io)?;
        let ty = entry.file_type().map_err(CliError::Io)?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if ty.is_dir() {
            copy_dir_all(src_path, dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path).map_err(CliError::Io)?;
        }
    }
    
    Ok(())
}

/// 检查命令是否存在
pub fn command_exists(command: &str) -> bool {
    if cfg!(target_os = "windows") {
        Command::new("where").arg(command).output().map(|output| output.status.success()).unwrap_or(false)
    } else {
        Command::new("which").arg(command).output().map(|output| output.status.success()).unwrap_or(false)
    }
}

/// 检查Rust工具链
pub fn check_rust_toolchain() -> CliResult<()> {
    match Command::new("cargo").arg("--version").output() {
        Ok(output) if output.status.success() => Ok(()),
        _ => Err(CliError::RustToolchainNotFound),
    }
}

/// 检查项目是否为有效的Lomus AI项目
pub fn is_lomus_project(project_path: &Path) -> bool {
    // 检查Cargo.toml是否包含lomusai依赖
    let cargo_toml_path = project_path.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return false;
    }

    match fs::read_to_string(&cargo_toml_path) {
        Ok(content) => {
            content.contains("lomusai_core") || 
            content.contains("lomusai") ||
            content.contains("lomus_ai")
        },
        Err(_) => false,
    }
}

/// 从当前目录向上查找项目根目录
pub fn find_project_root(start_dir: Option<&Path>) -> CliResult<PathBuf> {
    let start = match start_dir {
        Some(dir) => dir.to_path_buf(),
        None => std::env::current_dir().map_err(CliError::Io)?,
    };

    let mut current = start.clone();
    loop {
        // 检查是否为项目根目录
        if current.join("Cargo.toml").exists() && is_lomus_project(&current) {
            return Ok(current);
        }

        // 向上一级目录
        if !current.pop() {
            break;
        }
    }

    Err(CliError::ProjectNotFound(start))
}

/// 从目录名生成项目名
pub fn generate_project_name(dir: &Path) -> String {
    dir.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.replace('-', "_").to_lowercase())
        .unwrap_or_else(|| "lomusai_project".to_string())
}

/// 获取适合当前平台的包管理器
pub fn get_package_manager(ui_dir: &Path) -> Option<String> {
    let yarn_lock = ui_dir.join("yarn.lock");
    let pnpm_lock = ui_dir.join("pnpm-lock.yaml");
    let npm_lock = ui_dir.join("package-lock.json");

    if yarn_lock.exists() && command_exists("yarn") {
        Some("yarn".to_string())
    } else if pnpm_lock.exists() && command_exists("pnpm") {
        Some("pnpm".to_string())
    } else if npm_lock.exists() && command_exists("npm") {
        Some("npm".to_string())
    } else if command_exists("pnpm") {
        Some("pnpm".to_string())
    } else if command_exists("yarn") {
        Some("yarn".to_string())
    } else if command_exists("npm") {
        Some("npm".to_string())
    } else {
        None
    }
}

/// 创建目录（递归）
pub fn create_dir_all(path: &Path) -> CliResult<()> {
    fs::create_dir_all(path).map_err(|e| {
        CliError::Io(e)
    })
}

/// 安全地拷贝文件
pub fn copy_file(src: &Path, dst: &Path) -> CliResult<()> {
    // 确保目标目录存在
    if let Some(parent) = dst.parent() {
        create_dir_all(parent)?;
    }
    
    fs::copy(src, dst).map(|_| ()).map_err(|e| CliError::Io(e))
}

/// 运行一个命令并返回结果
pub fn run_command(cmd: &str, args: &[&str], dir: Option<&Path>) -> CliResult<bool> {
    let mut command = Command::new(cmd);
    command.args(args);
    
    if let Some(path) = dir {
        command.current_dir(path);
    }
    
    match command.status() {
        Ok(status) => Ok(status.success()),
        Err(e) => Err(CliError::CommandFailed(format!("命令 {} {} 执行失败: {}", cmd, args.join(" "), e))),
    }
}

/// 运行一个命令并返回输出
pub fn run_command_output(cmd: &str, args: &[&str], dir: Option<&Path>) -> CliResult<String> {
    let mut command = Command::new(cmd);
    command.args(args);
    
    if let Some(path) = dir {
        command.current_dir(path);
    }
    
    match command.output() {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                Err(CliError::CommandFailed(format!("命令 {} {} 执行失败: {}", cmd, args.join(" "), stderr)))
            }
        },
        Err(e) => Err(CliError::CommandFailed(format!("命令 {} {} 执行失败: {}", cmd, args.join(" "), e))),
    }
} 