//! 监控与遥测演示
//! 
//! 展示如何实现全面的性能监控系统，包括：
//! - 基础遥测配置
//! - 性能指标收集
//! - SLA 监控
//! - 告警系统

use lumosai_core::prelude::*;
use lumosai_core::agent::{AgentBuilder, BasicAgent};
use lumosai_core::telemetry::{TelemetryCollector, MetricsCollector, TraceCollector};
use lumosai_core::monitoring::{PerformanceMonitor, SLAMonitor, AlertManager};
use lumosai_core::llm::{MockLlmProvider, Message, Role};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde_json::json;
use tokio;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("📊 监控与遥测演示");
    println!("==================");
    
    // 演示1: 基础遥测配置
    demo_basic_telemetry().await?;
    
    // 演示2: 性能监控
    demo_performance_monitoring().await?;
    
    // 演示3: SLA 监控
    demo_sla_monitoring().await?;
    
    // 演示4: 告警系统
    demo_alert_system().await?;
    
    Ok(())
}

/// 演示基础遥测配置
async fn demo_basic_telemetry() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示1: 基础遥测配置 ===");
    
    // 创建遥测配置
    let telemetry_config = TelemetryConfig {
        enable_metrics: true,
        enable_tracing: true,
        enable_logging: true,
        metrics_endpoint: "http://localhost:9090".to_string(),
        trace_endpoint: "http://localhost:14268".to_string(),
        log_level: "info".to_string(),
        sampling_rate: 0.1,
        export_interval_seconds: 10,
    };
    
    println!("遥测配置:");
    println!("  指标收集: {}", telemetry_config.enable_metrics);
    println!("  链路追踪: {}", telemetry_config.enable_tracing);
    println!("  日志记录: {}", telemetry_config.enable_logging);
    println!("  指标端点: {}", telemetry_config.metrics_endpoint);
    println!("  追踪端点: {}", telemetry_config.trace_endpoint);
    println!("  采样率: {}", telemetry_config.sampling_rate);
    
    // 初始化遥测收集器
    let telemetry_collector = TelemetryCollector::new(telemetry_config)?;
    telemetry_collector.start().await?;
    
    println!("✅ 遥测系统已启动");
    
    // 创建指标收集器
    let metrics_collector = MetricsCollector::new();
    
    // 记录一些基础指标
    metrics_collector.increment_counter("system_startup", &[("component", "telemetry")]).await?;
    metrics_collector.record_histogram("startup_time_ms", 1500.0, &[("phase", "initialization")]).await?;
    metrics_collector.set_gauge("active_connections", 10.0, &[("type", "websocket")]).await?;
    
    println!("✅ 基础指标已记录");
    
    // 创建追踪收集器
    let trace_collector = TraceCollector::new();
    
    // 开始一个追踪 span
    let span = trace_collector.start_span("demo_operation", &[
        ("operation", "telemetry_demo"),
        ("version", "1.0.0"),
    ]).await?;
    
    // 模拟一些操作
    tokio::time::sleep(Duration::from_millis(100)).await;
    span.add_event("operation_started", &[("step", "1")]).await?;
    
    tokio::time::sleep(Duration::from_millis(200)).await;
    span.add_event("operation_completed", &[("step", "2")]).await?;
    
    span.finish().await?;
    
    println!("✅ 追踪数据已记录");
    
    Ok(())
}

/// 演示性能监控
async fn demo_performance_monitoring() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示2: 性能监控 ===");
    
    // 创建性能监控器
    let performance_monitor = PerformanceMonitor::new(PerformanceConfig {
        enable_real_time_monitoring: true,
        enable_historical_analysis: true,
        metrics_retention_days: 30,
        alert_thresholds: AlertThresholds {
            response_time_ms: 5000.0,
            error_rate_percent: 5.0,
            cpu_usage_percent: 80.0,
            memory_usage_percent: 85.0,
            throughput_rps: 100.0,
        },
    })?;
    
    performance_monitor.start().await?;
    println!("性能监控器已启动");
    
    // 创建被监控的 Agent
    let monitored_responses = vec![
        "这是一个快速响应的示例。".to_string(),
        "这是一个中等复杂度的响应，需要更多处理时间。".to_string(),
        "这是一个复杂的响应，涉及多步骤处理和分析。".to_string(),
    ];
    
    let llm_provider = Arc::new(MockLlmProvider::new(monitored_responses));
    
    let monitored_agent = AgentBuilder::new()
        .name("monitored_agent")
        .instructions("你是一个被监控的助手")
        .model(llm_provider)
        .enable_performance_monitoring(true)
        .build()?;
    
    // 模拟不同类型的请求
    let test_scenarios = vec![
        ("快速响应测试", "你好", 100),
        ("中等复杂度测试", "请解释什么是机器学习", 500),
        ("复杂任务测试", "请详细分析人工智能的发展历史", 1000),
    ];
    
    println!("\n执行性能测试场景:");
    
    for (scenario_name, input, expected_delay_ms) in test_scenarios {
        println!("  执行场景: {}", scenario_name);
        
        let start_time = Instant::now();
        
        // 模拟处理延迟
        tokio::time::sleep(Duration::from_millis(expected_delay_ms)).await;
        
        match monitored_agent.generate(input).await {
            Ok(response) => {
                let duration = start_time.elapsed();
                println!("    ✅ 成功 - 耗时: {:?}", duration);
                println!("    📝 响应长度: {} 字符", response.content.len());
                
                // 记录性能指标
                performance_monitor.record_request_metrics(RequestMetrics {
                    agent_name: "monitored_agent".to_string(),
                    operation: scenario_name.to_string(),
                    duration,
                    success: true,
                    response_size: response.content.len(),
                    error: None,
                }).await?;
            }
            Err(e) => {
                let duration = start_time.elapsed();
                println!("    ❌ 失败 - 耗时: {:?}, 错误: {}", duration, e);
                
                // 记录错误指标
                performance_monitor.record_request_metrics(RequestMetrics {
                    agent_name: "monitored_agent".to_string(),
                    operation: scenario_name.to_string(),
                    duration,
                    success: false,
                    response_size: 0,
                    error: Some(e.to_string()),
                }).await?;
            }
        }
        
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    
    // 获取性能统计
    let performance_stats = performance_monitor.get_statistics().await?;
    
    println!("\n📊 性能统计:");
    println!("  总请求数: {}", performance_stats.total_requests);
    println!("  成功率: {:.2}%", performance_stats.success_rate * 100.0);
    println!("  平均响应时间: {:.2}ms", performance_stats.avg_response_time_ms);
    println!("  P95 响应时间: {:.2}ms", performance_stats.p95_response_time_ms);
    println!("  P99 响应时间: {:.2}ms", performance_stats.p99_response_time_ms);
    println!("  吞吐量: {:.2} RPS", performance_stats.throughput_rps);
    
    Ok(())
}

/// 演示 SLA 监控
async fn demo_sla_monitoring() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示3: SLA 监控 ===");
    
    // 创建 SLA 监控器
    let mut sla_monitor = SLAMonitor::new(SLAConfig {
        real_time_monitoring: true,
        violation_alerting: true,
        report_generation: true,
        retention_days: 90,
    });
    
    // 定义 SLA 指标
    let response_time_sla = ServiceLevelAgreement {
        id: "agent_response_time".to_string(),
        name: "Agent 响应时间 SLA".to_string(),
        description: "Agent 必须在 3 秒内响应".to_string(),
        metrics: vec![
            SLAMetric {
                name: "response_time_ms".to_string(),
                threshold: 3000.0,
                operator: ThresholdOperator::LessThan,
                target_percentage: 95.0, // 95% 的请求
            }
        ],
        measurement_window: Duration::from_secs(300), // 5分钟窗口
        evaluation_frequency: Duration::from_secs(60), // 每分钟评估
    };
    
    let availability_sla = ServiceLevelAgreement {
        id: "system_availability".to_string(),
        name: "系统可用性 SLA".to_string(),
        description: "系统可用性必须达到 99.9%".to_string(),
        metrics: vec![
            SLAMetric {
                name: "availability_percent".to_string(),
                threshold: 99.9,
                operator: ThresholdOperator::GreaterThanOrEqual,
                target_percentage: 100.0,
            }
        ],
        measurement_window: Duration::from_secs(3600), // 1小时窗口
        evaluation_frequency: Duration::from_secs(300), // 每5分钟评估
    };
    
    sla_monitor.add_sla(response_time_sla).await?;
    sla_monitor.add_sla(availability_sla).await?;
    
    println!("SLA 监控配置:");
    println!("  响应时间 SLA: 95% 请求 < 3秒");
    println!("  可用性 SLA: 99.9% 可用性");
    
    // 模拟 SLA 数据收集
    println!("\n模拟 SLA 数据收集...");
    
    let test_data = vec![
        (1500.0, true),  // 快速响应，成功
        (2800.0, true),  // 正常响应，成功
        (4200.0, true),  // 慢响应，成功（违反SLA）
        (1200.0, true),  // 快速响应，成功
        (0.0, false),    // 失败请求
        (2100.0, true),  // 正常响应，成功
        (5500.0, true),  // 很慢响应，成功（违反SLA）
        (1800.0, true),  // 快速响应，成功
    ];
    
    for (i, (response_time, success)) in test_data.iter().enumerate() {
        // 记录响应时间指标
        sla_monitor.record_metric("agent_response_time", "response_time_ms", *response_time).await?;
        
        // 记录可用性指标
        let availability = if *success { 100.0 } else { 0.0 };
        sla_monitor.record_metric("system_availability", "availability_percent", availability).await?;
        
        println!("  记录数据点 {}: 响应时间 {:.0}ms, 成功: {}", 
            i + 1, response_time, success);
        
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // 评估 SLA 合规性
    let sla_results = sla_monitor.evaluate_all_slas().await?;
    
    println!("\n📋 SLA 评估结果:");
    for result in sla_results {
        let status_icon = if result.compliant { "✅" } else { "❌" };
        println!("  {} {}: {:.2}% 合规", 
            status_icon, result.sla_name, result.compliance_percentage);
        
        if !result.compliant {
            println!("    ⚠️  违反阈值: {} 个指标", result.violations.len());
            for violation in &result.violations {
                println!("      - {}: 实际值 {:.2}, 阈值 {:.2}", 
                    violation.metric_name, violation.actual_value, violation.threshold);
            }
        }
    }
    
    Ok(())
}

/// 演示告警系统
async fn demo_alert_system() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示4: 告警系统 ===");
    
    // 创建告警管理器
    let alert_manager = AlertManager::new(AlertConfig {
        enable_real_time_alerts: true,
        enable_escalation: true,
        notification_channels: vec![
            NotificationChannel::Email("admin@company.com".to_string()),
            NotificationChannel::Slack("#alerts".to_string()),
            NotificationChannel::Webhook("https://hooks.slack.com/webhook".to_string()),
        ],
        escalation_rules: vec![
            EscalationRule {
                severity: AlertSeverity::Critical,
                escalation_delay: Duration::from_secs(300), // 5分钟
                escalation_channels: vec![
                    NotificationChannel::PagerDuty("service-key".to_string()),
                ],
            },
        ],
    })?;
    
    alert_manager.start().await?;
    println!("告警管理器已启动");
    
    // 定义告警规则
    let alert_rules = vec![
        AlertRule {
            id: "high_response_time".to_string(),
            name: "高响应时间告警".to_string(),
            description: "当响应时间超过阈值时触发".to_string(),
            condition: "avg(response_time_ms) > 5000".to_string(),
            severity: AlertSeverity::Warning,
            evaluation_interval: Duration::from_secs(60),
            for_duration: Duration::from_secs(120),
        },
        AlertRule {
            id: "high_error_rate".to_string(),
            name: "高错误率告警".to_string(),
            description: "当错误率超过阈值时触发".to_string(),
            condition: "rate(errors_total) > 0.05".to_string(),
            severity: AlertSeverity::Critical,
            evaluation_interval: Duration::from_secs(30),
            for_duration: Duration::from_secs(60),
        },
        AlertRule {
            id: "system_overload".to_string(),
            name: "系统过载告警".to_string(),
            description: "当系统负载过高时触发".to_string(),
            condition: "cpu_usage > 90 OR memory_usage > 95".to_string(),
            severity: AlertSeverity::Critical,
            evaluation_interval: Duration::from_secs(30),
            for_duration: Duration::from_secs(180),
        },
    ];
    
    for rule in alert_rules {
        alert_manager.add_rule(rule).await?;
    }
    
    println!("告警规则已配置: {} 个规则", 3);
    
    // 模拟告警触发场景
    println!("\n模拟告警场景:");
    
    // 场景1: 高响应时间
    println!("  场景1: 模拟高响应时间");
    alert_manager.evaluate_metric("response_time_ms", 6500.0).await?;
    
    // 场景2: 高错误率
    println!("  场景2: 模拟高错误率");
    alert_manager.evaluate_metric("error_rate", 0.08).await?;
    
    // 场景3: 系统过载
    println!("  场景3: 模拟系统过载");
    alert_manager.evaluate_metric("cpu_usage", 95.0).await?;
    alert_manager.evaluate_metric("memory_usage", 97.0).await?;
    
    // 等待告警处理
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // 获取活跃告警
    let active_alerts = alert_manager.get_active_alerts().await?;
    
    println!("\n🚨 活跃告警:");
    if active_alerts.is_empty() {
        println!("  无活跃告警");
    } else {
        for alert in active_alerts {
            let severity_icon = match alert.severity {
                AlertSeverity::Critical => "🔴",
                AlertSeverity::Warning => "🟡",
                AlertSeverity::Info => "🔵",
            };
            
            println!("  {} {} - {}", 
                severity_icon, alert.rule_name, alert.description);
            println!("    触发时间: {:?}", alert.triggered_at);
            println!("    当前值: {:.2}", alert.current_value);
        }
    }
    
    // 获取告警统计
    let alert_stats = alert_manager.get_statistics().await?;
    
    println!("\n📊 告警统计:");
    println!("  总告警数: {}", alert_stats.total_alerts);
    println!("  活跃告警: {}", alert_stats.active_alerts);
    println!("  已解决告警: {}", alert_stats.resolved_alerts);
    println!("  平均解决时间: {:?}", alert_stats.avg_resolution_time);
    
    Ok(())
}

// ============================================================================
// 数据结构定义
// ============================================================================

#[derive(Debug, Clone)]
struct TelemetryConfig {
    enable_metrics: bool,
    enable_tracing: bool,
    enable_logging: bool,
    metrics_endpoint: String,
    trace_endpoint: String,
    log_level: String,
    sampling_rate: f64,
    export_interval_seconds: u64,
}

#[derive(Debug, Clone)]
struct PerformanceConfig {
    enable_real_time_monitoring: bool,
    enable_historical_analysis: bool,
    metrics_retention_days: u32,
    alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone)]
struct AlertThresholds {
    response_time_ms: f64,
    error_rate_percent: f64,
    cpu_usage_percent: f64,
    memory_usage_percent: f64,
    throughput_rps: f64,
}

#[derive(Debug, Clone)]
struct RequestMetrics {
    agent_name: String,
    operation: String,
    duration: Duration,
    success: bool,
    response_size: usize,
    error: Option<String>,
}

#[derive(Debug, Clone)]
struct PerformanceStatistics {
    total_requests: u64,
    success_rate: f64,
    avg_response_time_ms: f64,
    p95_response_time_ms: f64,
    p99_response_time_ms: f64,
    throughput_rps: f64,
}

#[derive(Debug, Clone)]
struct ServiceLevelAgreement {
    id: String,
    name: String,
    description: String,
    metrics: Vec<SLAMetric>,
    measurement_window: Duration,
    evaluation_frequency: Duration,
}

#[derive(Debug, Clone)]
struct SLAMetric {
    name: String,
    threshold: f64,
    operator: ThresholdOperator,
    target_percentage: f64,
}

#[derive(Debug, Clone)]
enum ThresholdOperator {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone)]
struct SLAConfig {
    real_time_monitoring: bool,
    violation_alerting: bool,
    report_generation: bool,
    retention_days: u32,
}

#[derive(Debug, Clone)]
struct SLAEvaluationResult {
    sla_name: String,
    compliant: bool,
    compliance_percentage: f64,
    violations: Vec<SLAViolation>,
}

#[derive(Debug, Clone)]
struct SLAViolation {
    metric_name: String,
    actual_value: f64,
    threshold: f64,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
struct AlertConfig {
    enable_real_time_alerts: bool,
    enable_escalation: bool,
    notification_channels: Vec<NotificationChannel>,
    escalation_rules: Vec<EscalationRule>,
}

#[derive(Debug, Clone)]
enum NotificationChannel {
    Email(String),
    Slack(String),
    Webhook(String),
    PagerDuty(String),
}

#[derive(Debug, Clone)]
struct EscalationRule {
    severity: AlertSeverity,
    escalation_delay: Duration,
    escalation_channels: Vec<NotificationChannel>,
}

#[derive(Debug, Clone)]
struct AlertRule {
    id: String,
    name: String,
    description: String,
    condition: String,
    severity: AlertSeverity,
    evaluation_interval: Duration,
    for_duration: Duration,
}

#[derive(Debug, Clone, PartialEq)]
enum AlertSeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
struct ActiveAlert {
    rule_name: String,
    description: String,
    severity: AlertSeverity,
    triggered_at: chrono::DateTime<chrono::Utc>,
    current_value: f64,
}

#[derive(Debug, Clone)]
struct AlertStatistics {
    total_alerts: u64,
    active_alerts: u64,
    resolved_alerts: u64,
    avg_resolution_time: Duration,
}

// ============================================================================
// 模拟实现（实际项目中应该有真实的实现）
// ============================================================================

struct TelemetryCollector;
impl TelemetryCollector {
    fn new(_config: TelemetryConfig) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(Self)
    }
    
    async fn start(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

struct MetricsCollector;
impl MetricsCollector {
    fn new() -> Self {
        Self
    }
    
    async fn increment_counter(&self, _name: &str, _labels: &[(&str, &str)]) -> std::result::Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    
    async fn record_histogram(&self, _name: &str, _value: f64, _labels: &[(&str, &str)]) -> std::result::Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    
    async fn set_gauge(&self, _name: &str, _value: f64, _labels: &[(&str, &str)]) -> std::result::Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

struct TraceCollector;
impl TraceCollector {
    fn new() -> Self {
        Self
    }
    
    async fn start_span(&self, _name: &str, _attributes: &[(&str, &str)]) -> std::result::Result<MockSpan, Box<dyn std::error::Error>> {
        Ok(MockSpan)
    }
}

struct MockSpan;
impl MockSpan {
    async fn add_event(&self, _name: &str, _attributes: &[(&str, &str)]) -> std::result::Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    
    async fn finish(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

struct PerformanceMonitor;
impl PerformanceMonitor {
    fn new(_config: PerformanceConfig) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(Self)
    }
    
    async fn start(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    
    async fn record_request_metrics(&self, _metrics: RequestMetrics) -> std::result::Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    
    async fn get_statistics(&self) -> std::result::Result<PerformanceStatistics, Box<dyn std::error::Error>> {
        Ok(PerformanceStatistics {
            total_requests: 8,
            success_rate: 0.875,
            avg_response_time_ms: 2100.0,
            p95_response_time_ms: 4200.0,
            p99_response_time_ms: 5500.0,
            throughput_rps: 2.67,
        })
    }
}

struct SLAMonitor;
impl SLAMonitor {
    fn new(_config: SLAConfig) -> Self {
        Self
    }
    
    async fn add_sla(&mut self, _sla: ServiceLevelAgreement) -> std::result::Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    
    async fn record_metric(&self, _sla_id: &str, _metric_name: &str, _value: f64) -> std::result::Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    
    async fn evaluate_all_slas(&self) -> Result<Vec<SLAEvaluationResult>, Box<dyn std::error::Error>> {
        Ok(vec![
            SLAEvaluationResult {
                sla_name: "Agent 响应时间 SLA".to_string(),
                compliant: false,
                compliance_percentage: 75.0,
                violations: vec![
                    SLAViolation {
                        metric_name: "response_time_ms".to_string(),
                        actual_value: 4200.0,
                        threshold: 3000.0,
                        timestamp: chrono::Utc::now(),
                    },
                ],
            },
            SLAEvaluationResult {
                sla_name: "系统可用性 SLA".to_string(),
                compliant: true,
                compliance_percentage: 87.5,
                violations: vec![],
            },
        ])
    }
}

struct AlertManager;
impl AlertManager {
    fn new(_config: AlertConfig) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(Self)
    }
    
    async fn start(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    
    async fn add_rule(&self, _rule: AlertRule) -> std::result::Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    
    async fn evaluate_metric(&self, _metric_name: &str, _value: f64) -> std::result::Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    
    async fn get_active_alerts(&self) -> Result<Vec<ActiveAlert>, Box<dyn std::error::Error>> {
        Ok(vec![
            ActiveAlert {
                rule_name: "高响应时间告警".to_string(),
                description: "当响应时间超过阈值时触发".to_string(),
                severity: AlertSeverity::Warning,
                triggered_at: chrono::Utc::now(),
                current_value: 6500.0,
            },
            ActiveAlert {
                rule_name: "系统过载告警".to_string(),
                description: "当系统负载过高时触发".to_string(),
                severity: AlertSeverity::Critical,
                triggered_at: chrono::Utc::now(),
                current_value: 95.0,
            },
        ])
    }
    
    async fn get_statistics(&self) -> std::result::Result<AlertStatistics, Box<dyn std::error::Error>> {
        Ok(AlertStatistics {
            total_alerts: 15,
            active_alerts: 2,
            resolved_alerts: 13,
            avg_resolution_time: Duration::from_secs(1800), // 30分钟
        })
    }
}
