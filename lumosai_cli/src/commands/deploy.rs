use std::path::{Path, PathBuf};
use tempfile::TempDir;
use crate::error::CliResult;
use crate::util::{find_project_root, is_lumos_project, create_dir_all, copy_dir_all, check_command_available};
use crate::commands::build;
use colored::Colorize;
use tokio::process::Command;
use std::fs;

/// 部署Lumos AI应用
pub async fn run(
    project_dir: Option<PathBuf>,
    target: &str,
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
    
    // 创建临时构建目录
    let build_dir = TempDir::new()?;
    let build_path = build_dir.path().to_path_buf();
    
    // 构建项目
    println!("{}", "构建项目...".bright_blue());
    build::run(Some(project_dir.clone()), Some(build_path.clone())).await?;
    
    // 根据不同目标部署
    match target.to_lowercase().as_str() {
        "local" => deploy_local(&project_dir, &build_path).await?,
        "docker" => deploy_docker(&project_dir, &build_path).await?,
        "aws" => deploy_aws(&project_dir, &build_path).await?,
        "azure" => deploy_azure(&project_dir, &build_path).await?,
        "gcp" => deploy_gcp(&project_dir, &build_path).await?,
        _ => {
            return Err(format!("不支持的部署目标: {}", target).into());
        }
    }
    
    println!("{}", "部署完成".bright_green());
    
    Ok(())
}

/// 本地部署
async fn deploy_local(project_dir: &Path, build_dir: &Path) -> CliResult<()> {
    println!("{}", "执行本地部署...".bright_blue());
    
    // 确定部署目录
    let deploy_dir = project_dir.join("deploy").join("local");
    
    // 确保部署目录存在
    create_dir_all(&deploy_dir)?;
    
    // 复制构建产物到部署目录
    copy_dir_all(build_dir, &deploy_dir)?;
    
    println!("{}", format!("应用已部署到: {}", deploy_dir.display()).bright_green());
    println!("{}", "可以通过以下命令运行:".bright_blue());
    
    // 获取项目名称
    let project_name = project_dir.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("lumosai_app");
        
    println!("{}", format!("  cd {} && ./{}", deploy_dir.display(), project_name).bright_cyan());
    
    Ok(())
}

/// Docker部署
async fn deploy_docker(project_dir: &Path, build_dir: &Path) -> CliResult<()> {
    println!("{}", "执行Docker部署...".bright_blue());
    
    // 检查Docker是否安装
    if !check_command_available("docker") {
        return Err("找不到Docker命令，请确认Docker已安装".into());
    }
    
    // 创建Dockerfile
    let dockerfile_path = build_dir.join("Dockerfile");
    let dockerfile_content = r#"FROM ubuntu:20.04

WORKDIR /app

COPY . /app

RUN apt-get update && apt-get install -y libssl-dev ca-certificates

# 设置环境变量
ENV RUST_LOG=info

# 暴露端口
EXPOSE 3000

# 设置入口命令
CMD ["./app"]
"#;
    
    // 写入Dockerfile
    fs::write(&dockerfile_path, dockerfile_content)?;
    
    println!("{}", format!("创建Dockerfile: {}", dockerfile_path.display()).bright_green());
    
    // 获取项目名称
    let project_name = project_dir.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("lumosai_app");
        
    // 构建Docker镜像
    println!("{}", "构建Docker镜像...".bright_blue());
    
    let image_name = format!("lumosai/{}", project_name).to_lowercase();
    let image_tag = format!("{}:latest", image_name);
    
    let mut cmd = Command::new("docker");
    cmd.current_dir(build_dir)
       .args(["build", "-t", &image_tag, "."]);
       
    let status = cmd.status().await?;
    
    if !status.success() {
        return Err("Docker镜像构建失败".into());
    }
    
    println!("{}", format!("Docker镜像构建成功: {}", image_tag).bright_green());
    println!("{}", "可以通过以下命令运行容器:".bright_blue());
    println!("{}", format!("  docker run -p 3000:3000 {}", image_tag).bright_cyan());
    
    Ok(())
}

/// AWS部署
async fn deploy_aws(project_dir: &Path, build_dir: &Path) -> CliResult<()> {
    println!("{}", "执行AWS部署...".bright_blue());
    
    // 检查AWS CLI是否安装
    if !check_command_available("aws") {
        return Err("找不到AWS CLI，请确认已安装".into());
    }
    
    // 实际AWS部署逻辑应在此处实现
    println!("{}", "AWS部署功能尚未完全实现，敬请期待".bright_yellow());
    
    Ok(())
}

/// Azure部署
async fn deploy_azure(project_dir: &Path, build_dir: &Path) -> CliResult<()> {
    println!("{}", "执行Azure部署...".bright_blue());
    
    // 检查Azure CLI是否安装
    if !check_command_available("az") {
        return Err("找不到Azure CLI，请确认已安装".into());
    }
    
    // 实际Azure部署逻辑应在此处实现
    println!("{}", "Azure部署功能尚未完全实现，敬请期待".bright_yellow());
    
    Ok(())
}

/// GCP部署
async fn deploy_gcp(project_dir: &Path, build_dir: &Path) -> CliResult<()> {
    println!("{}", "执行GCP部署...".bright_blue());
    
    // 检查GCloud CLI是否安装
    if !check_command_available("gcloud") {
        return Err("找不到Google Cloud CLI，请确认已安装".into());
    }
    
    // 实际GCP部署逻辑应在此处实现
    println!("{}", "GCP部署功能尚未完全实现，敬请期待".bright_yellow());
    
    Ok(())
}
