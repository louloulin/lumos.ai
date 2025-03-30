use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use colored::Colorize;
use tempfile::TempDir;

use crate::error::{CliError, CliResult};
use crate::util::{check_rust_toolchain, find_project_root, is_lomus_project, command_exists, copy_file, create_dir_all};
use crate::commands::build;

/// 部署Lomus AI应用
pub async fn run(dir: Option<PathBuf>, target: String) -> CliResult<()> {
    // 检查Rust工具链
    check_rust_toolchain()?;
    
    // 找到项目根目录
    let project_root = find_project_root(dir.as_ref().map(|p| p.as_path()))?;
    
    // 验证是否为有效项目
    if !is_lomus_project(&project_root) {
        return Err(CliError::InvalidProject(project_root.clone()));
    }
    
    println!("{}", format!("准备部署Lomus AI项目到 {}", target).bright_blue());
    
    // 创建临时构建目录
    let temp_dir = TempDir::new()
        .map_err(|e| CliError::Io(e))?;
    let build_dir = temp_dir.path().to_path_buf();
    
    // 先构建项目
    println!("{}", "构建项目...".bright_blue());
    build::run(Some(project_root.clone()), Some(build_dir.clone())).await?;
    
    // 根据目标平台部署
    match target.to_lowercase().as_str() {
        "local" => deploy_local(&build_dir).await?,
        "docker" => deploy_docker(&project_root, &build_dir).await?,
        "aws" => deploy_aws(&build_dir).await?,
        "azure" => deploy_azure(&build_dir).await?,
        "gcp" => deploy_gcp(&build_dir).await?,
        _ => {
            return Err(CliError::UnsupportedDeployTarget(target));
        }
    }
    
    println!("{}", "部署完成".bright_green());
    
    Ok(())
}

// 本地部署
async fn deploy_local(build_dir: &Path) -> CliResult<()> {
    println!("{}", "执行本地部署...".bright_blue());
    
    // 确定部署目录
    let deploy_dir = dirs::home_dir()
        .ok_or_else(|| CliError::Generic("无法确定用户主目录".to_string()))?
        .join(".lomusai/deploy");
        
    // 确保目录存在
    if !deploy_dir.exists() {
        create_dir_all(&deploy_dir)?;
    }
    
    // 复制构建产物到部署目录
    copy_dir_all(build_dir, &deploy_dir)?;
    
    println!("{}", format!("本地部署完成，应用已部署到: {}", deploy_dir.display()).bright_green());
    
    Ok(())
}

// Docker部署
async fn deploy_docker(project_root: &Path, build_dir: &Path) -> CliResult<()> {
    println!("{}", "执行Docker部署...".bright_blue());
    
    // 检查Docker是否安装
    if !command_exists("docker") {
        return Err(CliError::DependencyError("找不到docker命令，请确保已安装Docker".to_string()));
    }
    
    // 创建Dockerfile
    let dockerfile_path = build_dir.join("Dockerfile");
    let dockerfile_content = r#"FROM alpine:latest

WORKDIR /app

COPY . .

RUN chmod +x /app/start.sh

CMD ["/app/start.sh"]
"#;
    
    fs::write(&dockerfile_path, dockerfile_content)
        .map_err(|e| CliError::Io(e))?;
    
    // 获取项目名称作为镜像名
    let image_name = project_root
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.replace(' ', "_").to_lowercase())
        .unwrap_or_else(|| "lomusai_app".to_string());
    
    // 构建Docker镜像
    println!("{}", format!("构建Docker镜像: {}...", image_name).bright_blue());
    
    let status = Command::new("docker")
        .args(["build", "-t", &image_name, "."])
        .current_dir(build_dir)
        .status()
        .map_err(|e| CliError::CommandFailed(format!("Docker构建失败: {}", e)))?;
        
    if !status.success() {
        return Err(CliError::deploy("Docker构建失败"));
    }
    
    println!("{}", format!("Docker镜像 {} 构建成功", image_name).bright_green());
    println!("{}", "要运行Docker容器，请执行:".bright_blue());
    println!("{}", format!("  docker run -p 3000:3000 {}", image_name).bright_cyan());
    
    Ok(())
}

// AWS部署
async fn deploy_aws(_build_dir: &Path) -> CliResult<()> {
    println!("{}", "执行AWS部署...".bright_blue());
    println!("{}", "AWS部署功能尚未完全实现，这是一个占位实现".bright_yellow());
    
    // 检查AWS CLI是否安装
    if !command_exists("aws") {
        return Err(CliError::DependencyError("找不到aws命令，请确保已安装AWS CLI".to_string()));
    }
    
    // 这里应该实现实际的AWS部署逻辑
    // 例如使用AWS S3、AWS Lambda或AWS ECS
    
    println!("{}", "AWS部署占位实现完成".bright_green());
    
    Ok(())
}

// Azure部署
async fn deploy_azure(_build_dir: &Path) -> CliResult<()> {
    println!("{}", "执行Azure部署...".bright_blue());
    println!("{}", "Azure部署功能尚未完全实现，这是一个占位实现".bright_yellow());
    
    // 检查Azure CLI是否安装
    if !command_exists("az") {
        return Err(CliError::DependencyError("找不到az命令，请确保已安装Azure CLI".to_string()));
    }
    
    // 这里应该实现实际的Azure部署逻辑
    // 例如使用Azure App Service或Azure Functions
    
    println!("{}", "Azure部署占位实现完成".bright_green());
    
    Ok(())
}

// GCP部署
async fn deploy_gcp(_build_dir: &Path) -> CliResult<()> {
    println!("{}", "执行GCP部署...".bright_blue());
    println!("{}", "GCP部署功能尚未完全实现，这是一个占位实现".bright_yellow());
    
    // 检查GCP CLI是否安装
    if !command_exists("gcloud") {
        return Err(CliError::DependencyError("找不到gcloud命令，请确保已安装Google Cloud SDK".to_string()));
    }
    
    // 这里应该实现实际的GCP部署逻辑
    // 例如使用Google Cloud Run或Google App Engine
    
    println!("{}", "GCP部署占位实现完成".bright_green());
    
    Ok(())
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