use actix::{Actor, ActorContext, AsyncContext, Running, StreamHandler};
use actix_cors::Cors;
use actix_files as fs;
use actix_web::{
    middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result as ActixResult,
};
use actix_web_actors::ws;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;
use tokio::sync::RwLock;

use crate::error::{CliError, CliResult};
use crate::util::get_available_port;
use lumosai_core::compat::{MetricsCollector, TraceCollector, MetricValue, InMemoryMetricsCollector};

// 临时结构体定义，用于监控服务器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub avg_execution_time_ms: f64,
    pub min_execution_time_ms: u64,
    pub max_execution_time_ms: u64,
    pub total_tokens_used: u64,
    pub avg_tokens_per_execution: f64,
    pub tool_call_stats: HashMap<String, u64>,
    pub time_range: TimeRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: u64,
    pub end: u64,
}

/// 监控服务器配置
#[derive(Debug, Clone)]
pub struct MonitoringServerConfig {
    /// 服务器绑定地址
    pub bind_address: String,
    /// 服务器端口
    pub port: u16,
    /// 项目目录
    pub project_dir: PathBuf,
    /// 是否启用实时监控
    pub enable_realtime: bool,
    /// 指标刷新间隔（秒）
    pub refresh_interval: u64,
}

impl Default for MonitoringServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 4001,
            project_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            enable_realtime: true,
            refresh_interval: 5,
        }
    }
}

impl MonitoringServerConfig {
    /// 创建新配置
    pub fn new(port: u16, project_dir: PathBuf) -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port,
            project_dir,
            enable_realtime: true,
            refresh_interval: 5,
        }
    }

    /// 获取完整绑定地址
    pub fn get_bind_address(&self) -> String {
        format!("{}:{}", self.bind_address, self.port)
    }
}

/// 监控应用状态
#[derive(Clone)]
pub struct MonitoringAppState {
    pub metrics_collector: Arc<dyn MetricsCollector>,
    pub trace_collector: Arc<dyn TraceCollector>,
    pub config: MonitoringServerConfig,
    pub start_time: SystemTime,
    pub websocket_manager: WebSocketManager,
}

/// 实时监控数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeMetrics {
    /// 当前时间戳
    pub timestamp: u64,
    /// 系统运行时间（秒）
    pub uptime_seconds: u64,
    /// 代理概览统计
    pub agent_overview: AgentOverview,
    /// 活跃代理列表
    pub active_agents: Vec<AgentStatus>,
    /// 系统性能指标
    pub system_metrics: SystemMetrics,
    /// 最近错误
    pub recent_errors: Vec<ErrorEvent>,
    /// 工具使用统计
    pub tool_usage: Vec<ToolUsageStats>,
    /// 内存操作统计
    pub memory_stats: MemoryStats,
}

/// 代理概览统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOverview {
    /// 总代理数
    pub total_agents: u64,
    /// 活跃代理数
    pub active_agents: u64,
    /// 总执行次数
    pub total_executions: u64,
    /// 成功执行次数
    pub successful_executions: u64,
    /// 成功率
    pub success_rate: f64,
    /// 平均响应时间（毫秒）
    pub avg_response_time: f64,
    /// 总Token使用量
    pub total_tokens: u64,
}

/// 代理状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    /// 代理名称
    pub name: String,
    /// 最后活动时间
    pub last_activity: u64,
    /// 是否活跃
    pub is_active: bool,
    /// 最近1小时执行次数
    pub executions_last_hour: u64,
    /// 平均响应时间
    pub avg_response_time: f64,
    /// 成功率
    pub success_rate: f64,
    /// 当前状态
    pub status: String,
}

/// 系统性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// CPU使用率（百分比）
    pub cpu_usage: f64,
    /// 内存使用（MB）
    pub memory_usage_mb: f64,
    /// 活跃连接数
    pub active_connections: u64,
    /// 请求处理速率（每分钟）
    pub requests_per_minute: f64,
    /// 错误率
    pub error_rate: f64,
}

/// 错误事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    /// 时间戳
    pub timestamp: u64,
    /// 代理名称
    pub agent_name: String,
    /// 错误类型
    pub error_type: String,
    /// 错误消息
    pub error_message: String,
    /// 严重程度
    pub severity: String,
}

/// 工具使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsageStats {
    /// 工具名称
    pub tool_name: String,
    /// 使用次数
    pub usage_count: u64,
    /// 平均执行时间
    pub avg_execution_time: f64,
    /// 成功率
    pub success_rate: f64,
    /// 最后使用时间
    pub last_used: u64,
}

/// 内存操作统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// 总操作次数
    pub total_operations: u64,
    /// 读操作次数
    pub read_operations: u64,
    /// 写操作次数
    pub write_operations: u64,
    /// 平均操作时间
    pub avg_operation_time: f64,
    /// 缓存命中率
    pub cache_hit_rate: f64,
    /// 存储使用量（MB）
    pub storage_usage_mb: f64,
}

/// 监控API响应
#[derive(Serialize, Deserialize)]
pub struct MonitoringApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: u64,
}

impl<T> MonitoringApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }
}

/// 检查服务器端口是否可用
async fn check_port_available(port: u16) -> bool {
    TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .is_ok()
}

/// 获取实时监控数据处理器
async fn get_realtime_metrics(data: web::Data<MonitoringAppState>) -> ActixResult<impl Responder> {
    let state = data.get_ref();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let uptime = now
        - state
            .start_time
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

    // 获取代理概览统计
    let agent_overview = get_agent_overview(&state.metrics_collector).await;

    // 获取活跃代理列表
    let active_agents = get_active_agents(&state.metrics_collector).await;

    // 获取系统性能指标
    let system_metrics = get_system_metrics().await;

    // 获取最近错误
    let recent_errors = get_recent_errors(&state.metrics_collector).await;

    // 获取工具使用统计
    let tool_usage = get_tool_usage_stats(&state.metrics_collector).await;

    // 获取内存统计
    let memory_stats = get_memory_stats(&state.metrics_collector).await;

    let metrics = RealTimeMetrics {
        timestamp: now,
        uptime_seconds: uptime / 1000,
        agent_overview,
        active_agents,
        system_metrics,
        recent_errors,
        tool_usage,
        memory_stats,
    };

    Ok(HttpResponse::Ok().json(MonitoringApiResponse::success(metrics)))
}

/// 获取代理概览统计
async fn get_agent_overview(collector: &Arc<dyn MetricsCollector>) -> AgentOverview {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let twenty_four_hours_ago = now - (24 * 60 * 60 * 1000);

    // 临时实现 - 创建默认的指标摘要
    let summary = MetricsSummary {
        total_executions: 0,
        successful_executions: 0,
        failed_executions: 0,
        avg_execution_time_ms: 0.0,
        min_execution_time_ms: 0,
        max_execution_time_ms: 0,
        total_tokens_used: 0,
        avg_tokens_per_execution: 0.0,
        tool_call_stats: HashMap::new(),
        time_range: TimeRange {
            start: twenty_four_hours_ago,
            end: now,
        },
    };

    let success_rate = if summary.total_executions > 0 {
        summary.successful_executions as f64 / summary.total_executions as f64
    } else {
        0.0
    };

    AgentOverview {
        total_agents: 5,  // 示例数据，实际应从配置获取
        active_agents: 3, // 示例数据
        total_executions: summary.total_executions,
        successful_executions: summary.successful_executions,
        success_rate,
        avg_response_time: summary.avg_execution_time_ms,
        total_tokens: summary.total_tokens_used,
    }
}

/// 获取活跃代理列表
async fn get_active_agents(collector: &Arc<dyn MetricsCollector>) -> Vec<AgentStatus> {
    // 示例数据，实际应从collector获取真实数据
    vec![
        AgentStatus {
            name: "chat_agent".to_string(),
            last_activity: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            is_active: true,
            executions_last_hour: 15,
            avg_response_time: 1200.0,
            success_rate: 0.95,
            status: "healthy".to_string(),
        },
        AgentStatus {
            name: "research_agent".to_string(),
            last_activity: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64
                - 300000, // 5分钟前
            is_active: true,
            executions_last_hour: 8,
            avg_response_time: 2800.0,
            success_rate: 0.92,
            status: "healthy".to_string(),
        },
    ]
}

/// 获取系统性能指标
async fn get_system_metrics() -> SystemMetrics {
    // 示例数据，实际应从系统获取真实指标
    SystemMetrics {
        cpu_usage: 45.2,
        memory_usage_mb: 512.0,
        active_connections: 25,
        requests_per_minute: 120.0,
        error_rate: 0.02,
    }
}

/// 获取最近错误
async fn get_recent_errors(collector: &Arc<dyn MetricsCollector>) -> Vec<ErrorEvent> {
    // 示例数据，实际应从collector获取真实错误数据
    vec![ErrorEvent {
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
            - 120000, // 2分钟前
        agent_name: "chat_agent".to_string(),
        error_type: "timeout".to_string(),
        error_message: "Request timeout after 30 seconds".to_string(),
        severity: "warning".to_string(),
    }]
}

/// 获取工具使用统计
async fn get_tool_usage_stats(collector: &Arc<dyn MetricsCollector>) -> Vec<ToolUsageStats> {
    // 示例数据，实际应从collector获取真实工具使用数据
    vec![
        ToolUsageStats {
            tool_name: "web_search".to_string(),
            usage_count: 45,
            avg_execution_time: 2500.0,
            success_rate: 0.93,
            last_used: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        },
        ToolUsageStats {
            tool_name: "file_reader".to_string(),
            usage_count: 28,
            avg_execution_time: 800.0,
            success_rate: 0.98,
            last_used: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64
                - 180000, // 3分钟前
        },
    ]
}

/// 获取内存统计
async fn get_memory_stats(collector: &Arc<dyn MetricsCollector>) -> MemoryStats {
    // 示例数据，实际应从collector获取真实内存统计
    MemoryStats {
        total_operations: 1250,
        read_operations: 850,
        write_operations: 400,
        avg_operation_time: 25.0,
        cache_hit_rate: 0.87,
        storage_usage_mb: 128.0,
    }
}

/// 获取代理性能统计处理器
async fn get_agent_performance(
    path: web::Path<String>,
    data: web::Data<MonitoringAppState>,
) -> ActixResult<impl Responder> {
    let agent_name = path.into_inner();
    let state = data.get_ref();

    // 临时实现 - 返回空的性能数据
    let performance: HashMap<String, f64> = HashMap::new();
    Ok(HttpResponse::Ok().json(MonitoringApiResponse::success(performance)))
}

/// 获取指标摘要处理器
async fn get_metrics_summary(
    query: web::Query<HashMap<String, String>>,
    data: web::Data<MonitoringAppState>,
) -> ActixResult<impl Responder> {
    let state = data.get_ref();

    let agent_name = query.get("agent").map(|s| s.as_str());
    let from_time = query.get("from").and_then(|s| s.parse::<u64>().ok());
    let to_time = query.get("to").and_then(|s| s.parse::<u64>().ok());

    // 临时实现 - 创建默认的指标摘要
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
    let start_time = from_time.unwrap_or(now - (24 * 60 * 60 * 1000));
    let end_time = to_time.unwrap_or(now);

    let summary = MetricsSummary {
        total_executions: 0,
        successful_executions: 0,
        failed_executions: 0,
        avg_execution_time_ms: 0.0,
        min_execution_time_ms: 0,
        max_execution_time_ms: 0,
        total_tokens_used: 0,
        avg_tokens_per_execution: 0.0,
        tool_call_stats: HashMap::new(),
        time_range: TimeRange {
            start: start_time,
            end: end_time,
        },
    };

    Ok(HttpResponse::Ok().json(MonitoringApiResponse::success(summary)))
}

/// 获取系统健康状态处理器
async fn get_health_status(data: web::Data<MonitoringAppState>) -> ActixResult<impl Responder> {
    let state = data.get_ref();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let uptime = now
        - state
            .start_time
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

    let health_status = serde_json::json!({
        "status": "healthy",
        "uptime_ms": uptime,
        "timestamp": now,
        "services": {
            "metrics_collector": "healthy",
            "trace_collector": "healthy",
            "api_server": "healthy"
        },
        "version": env!("CARGO_PKG_VERSION")
    });

    Ok(HttpResponse::Ok().json(MonitoringApiResponse::success(health_status)))
}

/// 获取监控配置处理器
async fn get_monitoring_config(data: web::Data<MonitoringAppState>) -> ActixResult<impl Responder> {
    let state = data.get_ref();

    let config_info = serde_json::json!({
        "port": state.config.port,
        "bind_address": state.config.bind_address,
        "enable_realtime": state.config.enable_realtime,
        "refresh_interval": state.config.refresh_interval,
        "project_dir": state.config.project_dir.display().to_string()
    });

    Ok(HttpResponse::Ok().json(MonitoringApiResponse::success(config_info)))
}

/// 获取静态资源目录
fn get_static_dir() -> PathBuf {
    // 尝试多个可能的路径
    let possible_paths = [
        PathBuf::from("static/monitoring"),
        PathBuf::from("lumosai_cli/static/monitoring"),
        PathBuf::from("../lumosai_cli/static/monitoring"),
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|p| p.join("static/monitoring")))
            .unwrap_or_else(|| PathBuf::from("static/monitoring")),
    ];

    for path in &possible_paths {
        if path.exists() && path.join("index.html").exists() {
            return path.clone();
        }
    }

    // 如果找不到，返回默认路径（稍后会创建）
    PathBuf::from("static/monitoring")
}

/// 创建默认的dashboard HTML（如果静态文件不存在）
async fn serve_dashboard() -> ActixResult<impl Responder> {
    let static_dir = get_static_dir();
    let dashboard_path = static_dir.join("index.html");

    if dashboard_path.exists() {
        match tokio::fs::read_to_string(&dashboard_path).await {
            Ok(content) => {
                return Ok(HttpResponse::Ok().content_type("text/html").body(content));
            }
            Err(_) => {
                // 如果读取失败，使用内嵌的HTML
            }
        }
    }

    // 使用内嵌的HTML内容
    let dashboard_html = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Lumosai 监控仪表板</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #f5f7fa;
            color: #2d3748;
            line-height: 1.6;
        }

        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 2rem 0;
            text-align: center;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
        }

        .header h1 {
            font-size: 2.5rem;
            margin-bottom: 0.5rem;
            font-weight: 600;
        }

        .header p {
            font-size: 1.1rem;
            opacity: 0.9;
        }

        .container {
            max-width: 1200px;
            margin: 0 auto;
            padding: 2rem;
        }

        .dashboard {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 2rem;
            margin-top: 2rem;
        }

        .card {
            background: white;
            border-radius: 12px;
            padding: 1.5rem;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.05);
            border: 1px solid #e2e8f0;
            transition: transform 0.2s, box-shadow 0.2s;
        }

        .card:hover {
            transform: translateY(-2px);
            box-shadow: 0 8px 25px rgba(0, 0, 0, 0.1);
        }

        .card-header {
            display: flex;
            align-items: center;
            margin-bottom: 1rem;
        }

        .card-icon {
            width: 40px;
            height: 40px;
            border-radius: 8px;
            margin-right: 1rem;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 1.2rem;
        }

        .card-title {
            font-size: 1.1rem;
            font-weight: 600;
            color: #2d3748;
        }

        .metric {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 0.75rem;
            padding: 0.5rem 0;
            border-bottom: 1px solid #f1f5f9;
        }

        .metric:last-child {
            border-bottom: none;
        }

        .metric-label {
            color: #64748b;
            font-size: 0.9rem;
        }

        .metric-value {
            font-weight: 600;
            font-size: 1rem;
        }

        .status-indicator {
            display: inline-block;
            width: 8px;
            height: 8px;
            border-radius: 50%;
            margin-right: 0.5rem;
        }

        .status-healthy {
            background: #10b981;
        }

        .status-warning {
            background: #f59e0b;
        }

        .status-error {
            background: #ef4444;
        }

        .error-message {
            background: #fef2f2;
            color: #dc2626;
            padding: 1rem;
            border-radius: 8px;
            border: 1px solid #fecaca;
            text-align: center;
            margin: 2rem 0;
        }

        .loading {
            text-align: center;
            padding: 2rem;
            color: #64748b;
        }

        .loading::after {
            content: '';
            display: inline-block;
            width: 20px;
            height: 20px;
            border: 2px solid #e2e8f0;
            border-left-color: #667eea;
            border-radius: 50%;
            animation: spin 1s linear infinite;
            margin-left: 0.5rem;
        }

        @keyframes spin {
            to {
                transform: rotate(360deg);
            }
        }

        .refresh-button {
            position: fixed;
            bottom: 2rem;
            right: 2rem;
            background: #667eea;
            color: white;
            border: none;
            border-radius: 50%;
            width: 56px;
            height: 56px;
            font-size: 1.5rem;
            cursor: pointer;
            box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
            transition: all 0.2s;
        }

        .refresh-button:hover {
            transform: scale(1.1);
            box-shadow: 0 6px 20px rgba(102, 126, 234, 0.6);
        }

        .agent-list {
            margin-top: 1rem;
        }

        .agent-item {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 0.75rem;
            background: #f8fafc;
            border-radius: 8px;
            margin-bottom: 0.5rem;
        }

        .agent-name {
            font-weight: 500;
            color: #374151;
        }

        .agent-status {
            font-size: 0.8rem;
            padding: 0.25rem 0.5rem;
            border-radius: 12px;
            background: #dcfce7;
            color: #166534;
        }

        .timestamp {
            text-align: center;
            color: #64748b;
            font-size: 0.9rem;
            margin-top: 2rem;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>🤖 Lumosai 监控仪表板</h1>
        <p>实时监控AI代理的性能和健康状态</p>
    </div>

    <div class="container">
        <div id="loading" class="loading">
            正在加载监控数据...
        </div>

        <div id="error-container" style="display: none;"></div>

        <div id="dashboard" class="dashboard" style="display: none;">
            <!-- 系统概览 -->
            <div class="card">
                <div class="card-header">
                    <div class="card-icon" style="background: #dbeafe; color: #2563eb;">
                        📊
                    </div>
                    <div class="card-title">系统概览</div>
                </div>
                <div class="metric">
                    <span class="metric-label">运行时间</span>
                    <span class="metric-value" id="uptime">-</span>
                </div>
                <div class="metric">
                    <span class="metric-label">总代理数</span>
                    <span class="metric-value" id="total-agents">-</span>
                </div>
                <div class="metric">
                    <span class="metric-label">活跃代理</span>
                    <span class="metric-value" id="active-agents">-</span>
                </div>
                <div class="metric">
                    <span class="metric-label">总执行次数</span>
                    <span class="metric-value" id="total-executions">-</span>
                </div>
            </div>

            <!-- 性能指标 -->
            <div class="card">
                <div class="card-header">
                    <div class="card-icon" style="background: #dcfce7; color: #059669;">
                        ⚡
                    </div>
                    <div class="card-title">性能指标</div>
                </div>
                <div class="metric">
                    <span class="metric-label">成功率</span>
                    <span class="metric-value" id="success-rate">-</span>
                </div>
                <div class="metric">
                    <span class="metric-label">平均响应时间</span>
                    <span class="metric-value" id="avg-response-time">-</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Token使用量</span>
                    <span class="metric-value" id="total-tokens">-</span>
                </div>
                <div class="metric">
                    <span class="metric-label">错误率</span>
                    <span class="metric-value" id="error-rate">-</span>
                </div>
            </div>

            <!-- 系统资源 -->
            <div class="card">
                <div class="card-header">
                    <div class="card-icon" style="background: #fef3c7; color: #d97706;">
                        💾
                    </div>
                    <div class="card-title">系统资源</div>
                </div>
                <div class="metric">
                    <span class="metric-label">CPU使用率</span>
                    <span class="metric-value" id="cpu-usage">-</span>
                </div>
                <div class="metric">
                    <span class="metric-label">内存使用</span>
                    <span class="metric-value" id="memory-usage">-</span>
                </div>
                <div class="metric">
                    <span class="metric-label">活跃连接</span>
                    <span class="metric-value" id="active-connections">-</span>
                </div>
                <div class="metric">
                    <span class="metric-label">请求速率</span>
                    <span class="metric-value" id="requests-per-minute">-</span>
                </div>
            </div>

            <!-- 活跃代理 -->
            <div class="card">
                <div class="card-header">
                    <div class="card-icon" style="background: #f3e8ff; color: #7c3aed;">
                        🤖
                    </div>
                    <div class="card-title">活跃代理</div>
                </div>
                <div id="agent-list" class="agent-list">
                    <!-- 代理列表将在这里动态生成 -->
                </div>
            </div>

            <!-- 内存操作 -->
            <div class="card">
                <div class="card-header">
                    <div class="card-icon" style="background: #ecfdf5; color: #059669;">
                        🧠
                    </div>
                    <div class="card-title">内存操作</div>
                </div>
                <div class="metric">
                    <span class="metric-label">总操作数</span>
                    <span class="metric-value" id="memory-total-ops">-</span>
                </div>
                <div class="metric">
                    <span class="metric-label">读操作</span>
                    <span class="metric-value" id="memory-read-ops">-</span>
                </div>
                <div class="metric">
                    <span class="metric-label">写操作</span>
                    <span class="metric-value" id="memory-write-ops">-</span>
                </div>
                <div class="metric">
                    <span class="metric-label">缓存命中率</span>
                    <span class="metric-value" id="cache-hit-rate">-</span>
                </div>
            </div>

            <!-- 最近错误 -->
            <div class="card">
                <div class="card-header">
                    <div class="card-icon" style="background: #fef2f2; color: #dc2626;">
                        ⚠️
                    </div>
                    <div class="card-title">最近错误</div>
                </div>
                <div id="recent-errors">
                    <!-- 错误列表将在这里动态生成 -->
                </div>
            </div>
        </div>

        <div class="timestamp" id="last-update">
            最后更新: -
        </div>
    </div>

    <button class="refresh-button" onclick="loadDashboardData()" title="刷新数据">
        🔄
    </button>

    <script>
        const API_BASE = window.location.origin;
        const WS_URL = `ws://${window.location.host}/ws/monitoring`;
        let refreshInterval;
        let websocket = null;
        let wsConnectionStatus = 'disconnected';
        let fallbackToHttp = false;

        // WebSocket连接管理
        function connectWebSocket() {
            if (websocket && websocket.readyState === WebSocket.OPEN) {
                return;
            }

            try {
                websocket = new WebSocket(WS_URL);

                websocket.onopen = function(event) {
                    console.log('WebSocket连接已建立');
                    wsConnectionStatus = 'connected';
                    fallbackToHttp = false;
                    updateConnectionStatus();

                    // 停止HTTP轮询
                    if (refreshInterval) {
                        clearInterval(refreshInterval);
                        refreshInterval = null;
                    }
                };

                websocket.onmessage = function(event) {
                    try {
                        const message = JSON.parse(event.data);
                        handleWebSocketMessage(message);
                    } catch (error) {
                        console.error('解析WebSocket消息失败:', error);
                    }
                };

                websocket.onclose = function(event) {
                    console.log('WebSocket连接已断开');
                    wsConnectionStatus = 'disconnected';
                    updateConnectionStatus();

                    // 启用HTTP轮询作为备用
                    fallbackToHttp = true;
                    startHttpPolling();

                    // 尝试重连
                    setTimeout(connectWebSocket, 5000);
                };

                websocket.onerror = function(error) {
                    console.error('WebSocket错误:', error);
                    wsConnectionStatus = 'error';
                    updateConnectionStatus();
                };

            } catch (error) {
                console.error('创建WebSocket连接失败:', error);
                fallbackToHttp = true;
                startHttpPolling();
            }
        }

        // 处理WebSocket消息
        function handleWebSocketMessage(message) {
            switch (message.type) {
                case 'RealTimeMetrics':
                    updateDashboard(message.data);
                    showDashboard();
                    break;
                case 'AgentStatus':
                    updateAgentStatus(message.data);
                    break;
                case 'ErrorEvent':
                    showErrorNotification(message.data);
                    break;
                case 'Alert':
                    showAlert(message.data);
                    break;
                case 'Connected':
                    console.log('WebSocket连接确认:', message.data.client_id);
                    break;
                case 'Ping':
                    // 回复Pong
                    if (websocket && websocket.readyState === WebSocket.OPEN) {
                        websocket.send(JSON.stringify({
                            type: 'Pong',
                            data: { timestamp: message.data.timestamp }
                        }));
                    }
                    break;
                default:
                    console.log('未知WebSocket消息类型:', message.type);
            }
        }

        // 更新连接状态指示器
        function updateConnectionStatus() {
            // 在页面上添加连接状态指示器（如果不存在）
            let statusIndicator = document.getElementById('connection-status');
            if (!statusIndicator) {
                statusIndicator = document.createElement('div');
                statusIndicator.id = 'connection-status';
                statusIndicator.style.cssText = `
                    position: fixed;
                    top: 10px;
                    right: 10px;
                    padding: 8px 12px;
                    border-radius: 20px;
                    font-size: 12px;
                    font-weight: 500;
                    z-index: 1000;
                    transition: all 0.3s ease;
                `;
                document.body.appendChild(statusIndicator);
            }

            switch (wsConnectionStatus) {
                case 'connected':
                    statusIndicator.textContent = '🟢 实时连接';
                    statusIndicator.style.background = '#dcfce7';
                    statusIndicator.style.color = '#166534';
                    break;
                case 'disconnected':
                    statusIndicator.textContent = fallbackToHttp ? '🟡 HTTP轮询' : '🔴 已断开';
                    statusIndicator.style.background = fallbackToHttp ? '#fef3c7' : '#fef2f2';
                    statusIndicator.style.color = fallbackToHttp ? '#d97706' : '#dc2626';
                    break;
                case 'error':
                    statusIndicator.textContent = '🔴 连接错误';
                    statusIndicator.style.background = '#fef2f2';
                    statusIndicator.style.color = '#dc2626';
                    break;
            }
        }

        // 显示告警通知
        function showAlert(alert) {
            const alertContainer = document.createElement('div');
            alertContainer.style.cssText = `
                position: fixed;
                top: 60px;
                right: 10px;
                padding: 12px 16px;
                border-radius: 8px;
                max-width: 300px;
                z-index: 1001;
                animation: slideIn 0.3s ease;
            `;

            const bgColor = alert.level === 'error' ? '#fef2f2' :
                          alert.level === 'warning' ? '#fef3c7' : '#ecfdf5';
            const textColor = alert.level === 'error' ? '#dc2626' :
                            alert.level === 'warning' ? '#d97706' : '#059669';

            alertContainer.style.background = bgColor;
            alertContainer.style.color = textColor;
            alertContainer.innerHTML = `
                <div style="font-weight: 500; margin-bottom: 4px;">${alert.level.toUpperCase()}</div>
                <div style="font-size: 14px;">${alert.message}</div>
                <div style="font-size: 12px; opacity: 0.7; margin-top: 4px;">
                    ${new Date(alert.timestamp).toLocaleTimeString()}
                </div>
            `;

            document.body.appendChild(alertContainer);

            // 5秒后自动移除
            setTimeout(() => {
                if (alertContainer.parentNode) {
                    alertContainer.style.animation = 'slideOut 0.3s ease';
                    setTimeout(() => alertContainer.remove(), 300);
                }
            }, 5000);
        }

        // 启动HTTP轮询作为WebSocket的备用方案
        function startHttpPolling() {
            if (refreshInterval) {
                clearInterval(refreshInterval);
            }

            refreshInterval = setInterval(() => {
                if (!fallbackToHttp) return;
                loadDashboardData();
            }, 5000);
        }

        // 格式化时间
        function formatDuration(seconds) {
            const hours = Math.floor(seconds / 3600);
            const minutes = Math.floor((seconds % 3600) / 60);
            const secs = seconds % 60;

            if (hours > 0) {
                return `${hours}小时 ${minutes}分钟`;
            } else if (minutes > 0) {
                return `${minutes}分钟 ${secs}秒`;
            } else {
                return `${secs}秒`;
            }
        }

        // 格式化百分比
        function formatPercentage(value) {
            return `${(value * 100).toFixed(1)}%`;
        }

        // 格式化时间（毫秒）
        function formatTime(ms) {
            if (ms >= 1000) {
                return `${(ms / 1000).toFixed(2)}秒`;
            } else {
                return `${ms.toFixed(0)}毫秒`;
            }
        }

        // 加载仪表板数据（HTTP备用方案）
        async function loadDashboardData() {
            try {
                console.log('Loading dashboard data via HTTP...');
                const response = await fetch(`${API_BASE}/api/monitoring/realtime`);

                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                }

                const data = await response.json();
                console.log('Received data:', data);

                if (!data.success) {
                    throw new Error(data.error || '获取数据失败');
                }

                updateDashboard(data.data);
                showDashboard();

            } catch (error) {
                console.error('Error loading dashboard data:', error);
                showError(`加载监控数据失败: ${error.message}`);
            }
        }

        // 更新仪表板
        function updateDashboard(metrics) {
            // 系统概览
            document.getElementById('uptime').textContent = formatDuration(metrics.uptime_seconds);
            document.getElementById('total-agents').textContent = metrics.agent_overview.total_agents;
            document.getElementById('active-agents').textContent = metrics.agent_overview.active_agents;
            document.getElementById('total-executions').textContent = metrics.agent_overview.total_executions;

            // 性能指标
            document.getElementById('success-rate').textContent = formatPercentage(metrics.agent_overview.success_rate);
            document.getElementById('avg-response-time').textContent = formatTime(metrics.agent_overview.avg_response_time);
            document.getElementById('total-tokens').textContent = metrics.agent_overview.total_tokens.toLocaleString();
            document.getElementById('error-rate').textContent = formatPercentage(metrics.system_metrics.error_rate);

            // 系统资源
            document.getElementById('cpu-usage').textContent = `${metrics.system_metrics.cpu_usage.toFixed(1)}%`;
            document.getElementById('memory-usage').textContent = `${metrics.system_metrics.memory_usage_mb.toFixed(1)} MB`;
            document.getElementById('active-connections').textContent = metrics.system_metrics.active_connections;
            document.getElementById('requests-per-minute').textContent = `${metrics.system_metrics.requests_per_minute.toFixed(1)}/分钟`;

            // 活跃代理
            const agentList = document.getElementById('agent-list');
            agentList.innerHTML = '';

            if (metrics.active_agents.length === 0) {
                agentList.innerHTML = '<div style="text-align: center; color: #64748b; padding: 1rem;">暂无活跃代理</div>';
            } else {
                metrics.active_agents.forEach(agent => {
                    const agentItem = document.createElement('div');
                    agentItem.className = 'agent-item';
                    agentItem.innerHTML = `
                        <div class="agent-name">
                            <span class="status-indicator status-${agent.is_active ? 'healthy' : 'warning'}"></span>
                            ${agent.name}
                        </div>
                        <div class="agent-status">${agent.status}</div>
                    `;
                    agentList.appendChild(agentItem);
                });
            }

            // 内存操作
            document.getElementById('memory-total-ops').textContent = metrics.memory_stats.total_operations.toLocaleString();
            document.getElementById('memory-read-ops').textContent = metrics.memory_stats.read_operations.toLocaleString();
            document.getElementById('memory-write-ops').textContent = metrics.memory_stats.write_operations.toLocaleString();
            document.getElementById('cache-hit-rate').textContent = formatPercentage(metrics.memory_stats.cache_hit_rate);

            // 最近错误
            const errorsContainer = document.getElementById('recent-errors');
            errorsContainer.innerHTML = '';

            if (metrics.recent_errors.length === 0) {
                errorsContainer.innerHTML = '<div style="text-align: center; color: #10b981; padding: 1rem;">✅ 无最近错误</div>';
            } else {
                metrics.recent_errors.forEach(error => {
                    const errorItem = document.createElement('div');
                    errorItem.className = 'metric';
                    errorItem.innerHTML = `
                        <div>
                            <div style="font-weight: 500; color: #dc2626;">${error.error_type}</div>
                            <div style="font-size: 0.8rem; color: #64748b;">${error.agent_name}</div>
                        </div>
                        <div style="font-size: 0.8rem; color: #64748b;">
                            ${new Date(error.timestamp).toLocaleTimeString()}
                        </div>
                    `;
                    errorsContainer.appendChild(errorItem);
                });
            }

            // 更新时间戳
            document.getElementById('last-update').textContent =
                `最后更新: ${new Date().toLocaleTimeString()}`;
        }

        // 显示仪表板
        function showDashboard() {
            document.getElementById('loading').style.display = 'none';
            document.getElementById('error-container').style.display = 'none';
            document.getElementById('dashboard').style.display = 'grid';
        }

        // 显示错误
        function showError(message) {
            document.getElementById('loading').style.display = 'none';
            document.getElementById('dashboard').style.display = 'none';

            const errorContainer = document.getElementById('error-container');
            errorContainer.innerHTML = `
                <div class="error-message">
                    ${message}
                    <br><br>
                    <button onclick="loadDashboardData()" style="background: #dc2626; color: white; border: none; padding: 0.5rem 1rem; border-radius: 4px; cursor: pointer;">
                        重试
                    </button>
                </div>
            `;
            errorContainer.style.display = 'block';
        }

        // 启动自动刷新
        function startAutoRefresh() {
            if (refreshInterval) {
                clearInterval(refreshInterval);
            }

            refreshInterval = setInterval(() => {
                loadDashboardData();
            }, 5000); // 每5秒刷新一次
        }

        // 页面加载时初始化
        window.addEventListener('load', () => {
            loadDashboardData();
            startAutoRefresh();
        });

        // 页面隐藏时停止刷新，显示时恢复
        document.addEventListener('visibilitychange', () => {
            if (document.hidden) {
                if (refreshInterval) {
                    clearInterval(refreshInterval);
                    refreshInterval = null;
                }
            } else {
                startAutoRefresh();
                loadDashboardData();
            }
        });
    </script>
</body>
</html>"#;
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(dashboard_html))
}

/// WebSocket消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketMessage {
    /// 实时指标数据
    RealTimeMetrics(RealTimeMetrics),
    /// 代理状态更新
    AgentStatus {
        agent_name: String,
        status: AgentStatus,
    },
    /// 错误事件
    ErrorEvent(ErrorEvent),
    /// 告警信息
    Alert {
        level: String,
        message: String,
        timestamp: u64,
    },
    /// 连接确认
    Connected { client_id: String, timestamp: u64 },
    /// 心跳检测
    Ping { timestamp: u64 },
    /// 心跳响应
    Pong { timestamp: u64 },
}

impl actix::Message for WebSocketMessage {
    type Result = ();
}

/// WebSocket连接信息
#[derive(Debug, Clone)]
pub struct WebSocketConnectionInfo {
    pub client_id: String,
    pub connected_at: u64,
    pub last_ping: u64,
    pub addr: actix::Addr<MonitoringWebSocket>,
}

/// WebSocket连接管理器
#[derive(Debug, Clone, Default)]
pub struct WebSocketManager {
    connections: Arc<RwLock<HashMap<String, WebSocketConnectionInfo>>>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 添加新连接
    pub async fn add_connection(&self, client_id: String, addr: actix::Addr<MonitoringWebSocket>) {
        let connection_info = WebSocketConnectionInfo {
            client_id: client_id.clone(),
            connected_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            last_ping: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            addr,
        };

        self.connections
            .write()
            .await
            .insert(client_id, connection_info);
    }

    /// 移除连接
    pub async fn remove_connection(&self, client_id: &str) {
        self.connections.write().await.remove(client_id);
    }

    /// 广播消息给所有连接
    pub async fn broadcast(&self, message: WebSocketMessage) {
        let connections = self.connections.read().await;
        for connection in connections.values() {
            connection.addr.do_send(message.clone());
        }
    }

    /// 获取连接数量
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// 清理超时连接
    pub async fn cleanup_stale_connections(&self, timeout_ms: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let mut connections = self.connections.write().await;
        connections.retain(|_id, connection| now - connection.last_ping < timeout_ms);
    }
}

/// WebSocket Actor
pub struct MonitoringWebSocket {
    client_id: String,
    websocket_manager: WebSocketManager,
    last_heartbeat: std::time::Instant,
}

impl MonitoringWebSocket {
    pub fn new(client_id: String, websocket_manager: WebSocketManager) -> Self {
        Self {
            client_id,
            websocket_manager,
            last_heartbeat: std::time::Instant::now(),
        }
    }

    /// 开始心跳检测
    fn start_heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(Duration::from_secs(30), |act, ctx| {
            // 检查是否超时
            if std::time::Instant::now().duration_since(act.last_heartbeat)
                > Duration::from_secs(60)
            {
                println!("WebSocket心跳超时，断开连接: {}", act.client_id);
                ctx.stop();
                return;
            }

            // 发送心跳
            let ping_message = WebSocketMessage::Ping {
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            };

            if let Ok(text) = serde_json::to_string(&ping_message) {
                ctx.text(text);
            }
        });
    }
}

impl Actor for MonitoringWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("WebSocket连接已建立: {}", self.client_id);

        // 开始心跳检测
        self.start_heartbeat(ctx);

        // 注册连接
        let client_id = self.client_id.clone();
        let addr = ctx.address();
        let manager = self.websocket_manager.clone();

        tokio::spawn(async move {
            manager.add_connection(client_id.clone(), addr).await;
        });

        // 发送连接确认消息
        let connected_message = WebSocketMessage::Connected {
            client_id: self.client_id.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        };

        if let Ok(text) = serde_json::to_string(&connected_message) {
            ctx.text(text);
        }
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        println!("WebSocket连接正在断开: {}", self.client_id);

        // 从管理器中移除连接
        let client_id = self.client_id.clone();
        let manager = self.websocket_manager.clone();

        tokio::spawn(async move {
            manager.remove_connection(&client_id).await;
        });

        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MonitoringWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.last_heartbeat = std::time::Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.last_heartbeat = std::time::Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                self.last_heartbeat = std::time::Instant::now();

                // 处理客户端消息
                if let Ok(message) = serde_json::from_str::<WebSocketMessage>(&text) {
                    match message {
                        WebSocketMessage::Ping { timestamp } => {
                            let pong_message = WebSocketMessage::Pong { timestamp };
                            if let Ok(response) = serde_json::to_string(&pong_message) {
                                ctx.text(response);
                            }
                        }
                        WebSocketMessage::Pong { .. } => {
                            // 更新心跳时间
                        }
                        _ => {
                            // 其他消息类型暂时忽略
                        }
                    }
                }
            }
            Ok(ws::Message::Binary(_)) => {
                // 不支持二进制消息
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl actix::Handler<WebSocketMessage> for MonitoringWebSocket {
    type Result = ();

    fn handle(&mut self, msg: WebSocketMessage, ctx: &mut Self::Context) {
        if let Ok(text) = serde_json::to_string(&msg) {
            ctx.text(text);
        }
    }
}

/// WebSocket连接处理器
async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<MonitoringAppState>,
) -> ActixResult<impl Responder> {
    // 生成客户端ID
    let client_id = format!("client_{}", rand::random::<u32>());

    // 创建WebSocket Actor
    let websocket = MonitoringWebSocket::new(client_id, data.websocket_manager.clone());

    // 启动WebSocket连接
    ws::start(websocket, &req, stream)
}

/// 背景任务：广播实时指标数据
async fn broadcast_realtime_metrics(
    websocket_manager: WebSocketManager,
    metrics_collector: Arc<dyn MetricsCollector>,
    refresh_interval: u64,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(refresh_interval));

    loop {
        interval.tick().await;

        // 检查是否有WebSocket连接
        if websocket_manager.connection_count().await == 0 {
            continue;
        }

        // 生成实时指标数据
        match generate_realtime_metrics(&metrics_collector).await {
            Ok(metrics) => {
                let message = WebSocketMessage::RealTimeMetrics(metrics);
                websocket_manager.broadcast(message).await;
            }
            Err(e) => {
                eprintln!("生成实时指标数据失败: {}", e);

                // 发送错误告警
                let alert_message = WebSocketMessage::Alert {
                    level: "error".to_string(),
                    message: format!("指标收集失败: {}", e),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                };
                websocket_manager.broadcast(alert_message).await;
            }
        }

        // 清理超时连接（5分钟超时）
        websocket_manager
            .cleanup_stale_connections(5 * 60 * 1000)
            .await;
    }
}

/// 生成实时指标数据
async fn generate_realtime_metrics(
    metrics_collector: &Arc<dyn MetricsCollector>,
) -> Result<RealTimeMetrics, Box<dyn std::error::Error + Send + Sync>> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    // 获取代理概览统计
    let agent_overview = get_agent_overview(metrics_collector).await;

    // 获取活跃代理列表
    let active_agents = get_active_agents(metrics_collector).await;

    // 获取系统性能指标
    let system_metrics = get_system_metrics().await;

    // 获取最近错误
    let recent_errors = get_recent_errors(metrics_collector).await;

    // 获取工具使用统计
    let tool_usage = get_tool_usage_stats(metrics_collector).await;

    // 获取内存统计
    let memory_stats = get_memory_stats(metrics_collector).await;

    Ok(RealTimeMetrics {
        timestamp: now,
        uptime_seconds: now / 1000, // 简化的运行时间计算
        agent_overview,
        active_agents,
        system_metrics,
        recent_errors,
        tool_usage,
        memory_stats,
    })
}

/// 启动监控服务器
pub fn start_monitoring_server(
    port: u16,
    project_dir: PathBuf,
    metrics_collector: Arc<dyn MetricsCollector>,
    trace_collector: Arc<dyn TraceCollector>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = CliResult<()>> + Send>> {
    Box::pin(async move {
        // 检查端口是否可用
        if !check_port_available(port).await {
            let new_port = get_available_port(port).unwrap_or(port + 1);
            println!(
                "{}",
                format!("端口 {} 已被占用，使用端口 {}", port, new_port).bright_yellow()
            );

            return start_monitoring_server(
                new_port,
                project_dir,
                metrics_collector,
                trace_collector,
            )
            .await;
        }

        let config = MonitoringServerConfig::new(port, project_dir.clone());
        let start_time = SystemTime::now();
        let websocket_manager = WebSocketManager::new();

        let app_state = MonitoringAppState {
            metrics_collector: metrics_collector.clone(),
            trace_collector,
            config: config.clone(),
            start_time,
            websocket_manager: websocket_manager.clone(),
        };

        println!("{}", "启动监控服务器...".bright_blue());
        println!(
            "{}",
            format!("项目目录: {}", project_dir.display()).bright_blue()
        );
        println!(
            "{}",
            format!("绑定地址: {}", config.get_bind_address()).bright_blue()
        );
        println!(
            "{}",
            format!(
                "实时监控: {}",
                if config.enable_realtime {
                    "启用"
                } else {
                    "禁用"
                }
            )
            .bright_blue()
        );

        let state_data = web::Data::new(app_state);

        // 启动背景任务来广播实时数据
        tokio::spawn(broadcast_realtime_metrics(
            websocket_manager.clone(),
            metrics_collector.clone(),
            config.refresh_interval,
        ));

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
                .app_data(state_data.clone())
                // 仪表板首页
                .service(web::resource("/").route(web::get().to(serve_dashboard)))
                .service(web::resource("/dashboard").route(web::get().to(serve_dashboard)))
                // WebSocket端点
                .service(web::resource("/ws/monitoring").route(web::get().to(websocket_handler)))
                // 健康检查端点
                .service(web::resource("/health").route(web::get().to(get_health_status)))
                // 实时监控数据端点
                .service(
                    web::resource("/api/monitoring/realtime")
                        .route(web::get().to(get_realtime_metrics)),
                )
                // 代理性能统计端点
                .service(
                    web::resource("/api/monitoring/agents/{name}/performance")
                        .route(web::get().to(get_agent_performance)),
                )
                // 指标摘要端点
                .service(
                    web::resource("/api/monitoring/metrics/summary")
                        .route(web::get().to(get_metrics_summary)),
                )
                // 监控配置端点
                .service(
                    web::resource("/api/monitoring/config")
                        .route(web::get().to(get_monitoring_config)),
                )
                // 静态资源（如果目录存在）
                .service({
                    let static_dir = get_static_dir();
                    if static_dir.exists() {
                        fs::Files::new("/static", &static_dir).show_files_listing()
                    } else {
                        fs::Files::new("/static", ".").show_files_listing() // 占位符
                    }
                })
        })
        .bind(&config.get_bind_address())
        .map_err(|e| CliError::io_string(format!("无法绑定到端口: {}", config.port), e))?
        .run();

        println!("{}", "监控服务器已启动".bright_green());
        println!(
            "{}",
            format!("仪表板: http://localhost:{}/", config.port).bright_green()
        );
        println!(
            "{}",
            format!("WebSocket: ws://localhost:{}/ws/monitoring", config.port).bright_green()
        );
        println!(
            "{}",
            format!("健康检查: http://localhost:{}/health", config.port).bright_green()
        );
        println!(
            "{}",
            format!(
                "实时监控API: http://localhost:{}/api/monitoring/realtime",
                config.port
            )
            .bright_green()
        );

        // 等待服务器结束
        server
            .await
            .map_err(|e| CliError::io("启动监控服务器时出错", e))?;

        Ok(())
    })
}
