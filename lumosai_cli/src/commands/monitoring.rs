use clap::Args;
use std::path::PathBuf;
use std::env;
use std::sync::Arc;
use colored::Colorize;

use lumosai_core::telemetry::collectors::InMemoryMetricsCollector;
use lumosai_core::telemetry::metrics::MetricsCollector;
use lumosai_core::telemetry::trace::TraceCollector;

use crate::error::{CliResult, CliError};
use crate::util::is_lumos_project;
use crate::server::monitoring_server;

/// 监控服务器配置选项
#[derive(Args, Debug)]
pub struct MonitoringOptions {
    /// 监控服务器端口
    #[arg(short, long, default_value = "4001")]
    pub port: u16,

    /// 项目目录
    #[arg(short = 'd', long)]
    pub project_dir: Option<PathBuf>,

    /// 是否启用实时监控
    #[arg(long, default_value = "true")]
    pub enable_realtime: bool,

    /// 指标刷新间隔（秒）
    #[arg(long, default_value = "5")]
    pub refresh_interval: u64,

    /// 是否启用详细日志
    #[arg(short, long)]
    pub verbose: bool,
}

impl Default for MonitoringOptions {
    fn default() -> Self {
        Self {
            port: 4001,
            project_dir: None,
            enable_realtime: true,
            refresh_interval: 5,
            verbose: false,
        }
    }
}

/// 启动监控服务器
pub async fn run(options: MonitoringOptions) -> CliResult<()> {
    // 解析项目目录
    let project_dir = match &options.project_dir {
        Some(dir) => dir.clone(),
        None => env::current_dir().map_err(|e| CliError::io("获取当前目录失败", e))?,
    };

    // 检查项目目录是否存在
    if !project_dir.exists() {
        return Err(CliError::path_not_found(
            project_dir.to_string_lossy().to_string(),
            "项目目录不存在",
        ));
    }

    // 检查是否是Lumosai项目
    if !is_lumos_project(&project_dir) {
        println!("{}", "警告: 当前目录不是标准的Lumosai项目目录结构".bright_yellow());
    }

    println!("{}", "启动 Lumosai 监控服务器...".bright_blue());
    println!("{}", format!("项目目录: {}", project_dir.display()).bright_blue());
    println!("{}", format!("监控端口: {}", options.port).bright_blue());
    
    if options.enable_realtime {
        println!("{}", "实时监控: 启用".bright_blue());
        println!("{}", format!("刷新间隔: {} 秒", options.refresh_interval).bright_blue());
    } else {
        println!("{}", "实时监控: 禁用".bright_blue());
    }

    if options.verbose {
        println!("{}", "详细日志: 启用".bright_blue());
    }

    // 创建指标收集器和追踪收集器
    // 在实际应用中，这些可能会从项目配置中加载或从文件系统恢复
    let metrics_collector: Arc<dyn MetricsCollector> = Arc::new(InMemoryMetricsCollector::new());
    let trace_collector: Arc<dyn TraceCollector> = Arc::new(InMemoryMetricsCollector::new());

    // 启动监控服务器
    monitoring_server::start_monitoring_server(
        options.port,
        project_dir,
        metrics_collector,
        trace_collector,
    ).await?;

    Ok(())
}

/// 获取监控服务器的健康状态
pub async fn health_check(port: u16) -> CliResult<()> {
    let url = format!("http://localhost:{}/health", port);
    
    println!("{}", format!("检查监控服务器健康状态: {}", url).bright_blue());
    
    match reqwest::get(&url).await {
        Ok(response) => {
            if response.status().is_success() {
                let status: serde_json::Value = response.json().await
                    .map_err(|e| CliError::from(e))?;
                
                println!("{}", "监控服务器健康状态:".bright_green());
                println!("{}", serde_json::to_string_pretty(&status).unwrap());
            } else {
                println!("{}", format!("监控服务器返回错误状态: {}", response.status()).bright_red());
            }
        },
        Err(e) => {
            println!("{}", format!("无法连接到监控服务器: {}", e).bright_red());
            return Err(CliError::from(e));
        }
    }
    
    Ok(())
}

/// 获取实时监控数据
pub async fn get_realtime_metrics(port: u16) -> CliResult<()> {
    let url = format!("http://localhost:{}/api/monitoring/realtime", port);
    
    println!("{}", format!("获取实时监控数据: {}", url).bright_blue());
    
    match reqwest::get(&url).await {
        Ok(response) => {
            if response.status().is_success() {
                let metrics: serde_json::Value = response.json().await
                    .map_err(|e| CliError::from(e))?;
                
                println!("{}", "实时监控数据:".bright_green());
                println!("{}", serde_json::to_string_pretty(&metrics).unwrap());
            } else {
                println!("{}", format!("监控服务器返回错误状态: {}", response.status()).bright_red());
            }
        },
        Err(e) => {
            println!("{}", format!("无法连接到监控服务器: {}", e).bright_red());
            return Err(CliError::from(e));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_monitoring_options_default() {
        let options = MonitoringOptions::default();
        assert_eq!(options.port, 4001);
        assert_eq!(options.enable_realtime, true);
        assert_eq!(options.refresh_interval, 5);
        assert_eq!(options.verbose, false);
    }

    #[test]
    fn test_monitoring_options_creation() {
        let project_dir = PathBuf::from("/test/project");
        let options = MonitoringOptions {
            port: 8080,
            project_dir: Some(project_dir.clone()),
            enable_realtime: false,
            refresh_interval: 10,
            verbose: true,
        };

        assert_eq!(options.port, 8080);
        assert_eq!(options.project_dir, Some(project_dir));
        assert_eq!(options.enable_realtime, false);
        assert_eq!(options.refresh_interval, 10);
        assert_eq!(options.verbose, true);
    }
}
