use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use colored::Colorize;

use crate::error::{CliError, CliResult};
use crate::util::{check_rust_toolchain, find_project_root, is_lomus_project, get_package_manager, create_dir_all, copy_file};

/// 构建Lomus AI应用
pub async fn run(dir: Option<PathBuf>, output: Option<PathBuf>) -> CliResult<()> {
    // 检查Rust工具链
    check_rust_toolchain()?;
    
    // 找到项目根目录
    let project_root = find_project_root(dir.as_ref().map(|p| p.as_path()))?;
    
    // 验证是否为有效项目
    if !is_lomus_project(&project_root) {
        return Err(CliError::InvalidProject(project_root.clone()));
    }
    
    // 确定输出目录
    let output_dir = match output {
        Some(path) => path,
        None => project_root.join("dist"),
    };
    
    // 确保输出目录存在
    if !output_dir.exists() {
        create_dir_all(&output_dir)?;
    }
    
    println!("{}", format!("在 {} 构建Lomus AI应用", project_root.display()).bright_blue());
    println!("{}", format!("输出目录: {}", output_dir.display()).bright_blue());
    
    // 编译项目
    println!("{}", "优化编译...".bright_blue());
    
    let status = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&project_root)
        .status()
        .map_err(|e| CliError::CommandFailed(format!("无法编译项目: {}", e)))?;
        
    if !status.success() {
        return Err(CliError::build("项目编译失败"));
    }
    
    println!("{}", "编译完成".bright_green());
    
    // 复制构建产物
    println!("{}", "复制构建产物...".bright_blue());
    
    // 确定目标文件名
    let target_dir = project_root.join("target/release");
    let exe_name = project_root
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.replace('-', "_"))
        .unwrap_or_else(|| "lomusai_app".to_string());
        
    #[cfg(windows)]
    let exe_path = target_dir.join(format!("{}.exe", exe_name));
    
    #[cfg(not(windows))]
    let exe_path = target_dir.join(&exe_name);
    
    if !exe_path.exists() {
        println!("{}", format!("找不到可执行文件: {}", exe_path.display()).bright_yellow());
        
        // 尝试查找任何可执行文件
        let entries = fs::read_dir(&target_dir)
            .map_err(|e| CliError::Io(e))?;
            
        let mut found = false;
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if is_executable(&path) {
                    println!("{}", format!("找到可执行文件: {}", path.display()).bright_blue());
                    
                    // 复制到输出目录
                    let dest = output_dir.join(path.file_name().unwrap());
                    copy_file(&path, &dest)?;
                    
                    found = true;
                    break;
                }
            }
        }
        
        if !found {
            return Err(CliError::build("无法找到构建产物"));
        }
    } else {
        // 复制可执行文件到输出目录
        let dest = output_dir.join(exe_path.file_name().unwrap());
        copy_file(&exe_path, &dest)?;
    }
    
    // 检查是否有UI目录，如果有，也进行构建
    let ui_dir = project_root.join("ui");
    if ui_dir.exists() && ui_dir.is_dir() {
        // 检查是否有package.json
        let package_json = ui_dir.join("package.json");
        if package_json.exists() {
            println!("{}", "检测到UI目录，构建UI...".bright_blue());
            
            // 获取包管理器
            if let Some(pkg_manager) = get_package_manager(&ui_dir) {
                // 构建UI
                let status = Command::new(&pkg_manager)
                    .args(["run", "build"])
                    .current_dir(&ui_dir)
                    .status()
                    .map_err(|e| CliError::CommandFailed(format!("无法构建UI: {}", e)))?;
                    
                if !status.success() {
                    println!("{}", "UI构建失败，但继续进行".bright_yellow());
                } else {
                    // 复制UI构建产物
                    let ui_build_dir = ui_dir.join("dist");
                    if ui_build_dir.exists() && ui_build_dir.is_dir() {
                        let ui_dest_dir = output_dir.join("ui");
                        if !ui_dest_dir.exists() {
                            create_dir_all(&ui_dest_dir)?;
                        }
                        
                        // 递归复制UI构建产物
                        copy_dir_all(&ui_build_dir, &ui_dest_dir)?;
                        println!("{}", "UI构建产物已复制".bright_green());
                    }
                }
            } else {
                println!("{}", "未找到支持的包管理器(yarn/pnpm/npm)，跳过UI构建".bright_yellow());
            }
        }
    }
    
    // 复制配置文件
    for config_name in &[".env", "config.toml", "config.json"] {
        let config_path = project_root.join(config_name);
        if config_path.exists() {
            let dest = output_dir.join(config_name);
            copy_file(&config_path, &dest)?;
            println!("{}", format!("已复制配置文件: {}", config_name).bright_blue());
        }
    }
    
    // 创建启动脚本
    #[cfg(windows)]
    {
        let bat_content = format!("@echo off\r\n.\\{}.exe %*\r\n", exe_name);
        fs::write(output_dir.join("start.bat"), bat_content)
            .map_err(|e| CliError::Io(e))?;
    }
    
    #[cfg(not(windows))]
    {
        let sh_content = format!("#!/bin/sh\n./{} \"$@\"\n", exe_name);
        let sh_path = output_dir.join("start.sh");
        fs::write(&sh_path, sh_content)
            .map_err(|e| CliError::Io(e))?;
        
        // 设置可执行权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&sh_path)
                .map_err(|e| CliError::Io(e))?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&sh_path, perms)
                .map_err(|e| CliError::Io(e))?;
        }
    }
    
    println!("{}", format!("构建完成，输出至: {}", output_dir.display()).bright_green());
    
    Ok(())
}

// 检查文件是否为可执行文件
fn is_executable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    
    #[cfg(windows)]
    {
        // 在Windows上检查扩展名
        if let Some(ext) = path.extension() {
            return ext == "exe";
        }
    }
    
    #[cfg(unix)]
    {
        // 在Unix上检查权限
        if let Ok(metadata) = path.metadata() {
            use std::os::unix::fs::PermissionsExt;
            let mode = metadata.permissions().mode();
            return (mode & 0o111) != 0;
        }
    }
    
    false
}

/// 递归复制目录
fn copy_dir_all(src: &Path, dst: &Path) -> CliResult<()> {
    if !src.exists() {
        return Err(CliError::Generic(format!("源目录不存在: {}", src.display())));
    }
    
    if !dst.exists() {
        create_dir_all(dst)?;
    }
    
    for entry in fs::read_dir(src).map_err(CliError::Io)? {
        let entry = entry.map_err(CliError::Io)?;
        let ty = entry.file_type().map_err(CliError::Io)?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            copy_file(&src_path, &dst_path)?;
        }
    }
    
    Ok(())
} 