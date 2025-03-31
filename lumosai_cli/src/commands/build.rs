use std::path::{Path, PathBuf};
use std::fs;
use crate::error::CliResult;
use crate::util::{find_project_root, is_lumos_project, create_dir_all, copy_dir_all};
use colored::Colorize;
use tokio::process::Command;

/// 构建Lumos AI应用
pub async fn run(
    project_dir: Option<PathBuf>,
    output: Option<PathBuf>,
) -> CliResult<()> {
    // 确定项目目录
    let project_dir = match project_dir {
        Some(dir) => dir,
        None => find_project_root()?,
    };
    
    // 检查是否为Lumos项目
    if !is_lumos_project()? {
        println!("{}", "警告: 当前目录不是一个Lumos AI项目".bright_yellow());
        println!("{}", "如果这是错误的，请确认项目中包含lumosai依赖".bright_yellow());
    }
    
    // 确定输出目录
    let output_dir = match output {
        Some(dir) => dir,
        None => project_dir.join("target").join("release"),
    };
    
    // 确保输出目录存在
    create_dir_all(&output_dir)?;
    
    // 构建应用
    println!("{}", format!("构建项目: {}", project_dir.display()).bright_blue());
    println!("{}", format!("输出目录: {}", output_dir.display()).bright_blue());
    
    // 执行cargo build
    let mut cmd = Command::new("cargo");
    cmd.current_dir(&project_dir)
       .args(["build", "--release"]);
       
    let status = cmd.status().await?;
    
    if !status.success() {
        return Err(format!("构建失败，状态码: {:?}", status.code()).into());
    }
    
    // 如果输出目录不是默认的target/release，复制构建结果
    if output_dir != project_dir.join("target").join("release") {
        copy_build_artifacts(&project_dir, &output_dir)?;
    }
    
    println!("{}", "构建完成".bright_green());
    
    Ok(())
}

/// 复制构建产物到指定目录
fn copy_build_artifacts(project_dir: &Path, output_dir: &Path) -> CliResult<()> {
    // 获取项目名称（从Cargo.toml）
    let cargo_path = project_dir.join("Cargo.toml");
    let cargo_content = fs::read_to_string(&cargo_path)?;
    
    // 简单解析项目名称
    let project_name = cargo_content.lines()
        .find(|line| line.starts_with("name"))
        .and_then(|line| {
            line.split('=').nth(1).map(|s| s.trim().trim_matches('"'))
        })
        .unwrap_or("lumosai_app");
    
    // 源目录
    let release_dir = project_dir.join("target").join("release");
    
    // 复制可执行文件
    let bin_path = release_dir.join(project_name);
    if bin_path.exists() {
        let target_bin_path = output_dir.join(project_name);
        fs::copy(&bin_path, &target_bin_path)?;
        println!("{}", format!("复制: {} -> {}", bin_path.display(), target_bin_path.display()).bright_green());
    } else {
        println!("{}", format!("未找到构建产物: {}", bin_path.display()).bright_yellow());
    }
    
    // 复制依赖的库文件
    if release_dir.join("deps").exists() {
        let deps_dir = output_dir.join("deps");
        create_dir_all(&deps_dir)?;
        
        // 复制依赖目录
        copy_dir_all(release_dir.join("deps"), deps_dir)?;
        println!("{}", "复制依赖库".bright_green());
    }
    
    // 复制静态资源（如果存在）
    let assets_dir = project_dir.join("assets");
    if assets_dir.exists() {
        let target_assets_dir = output_dir.join("assets");
        copy_dir_all(&assets_dir, &target_assets_dir)?;
        println!("{}", "复制静态资源".bright_green());
    }
    
    Ok(())
} 