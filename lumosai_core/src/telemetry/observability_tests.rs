//! 企业级监控和可观测性系统集成测试
//! 
//! 测试智能告警引擎、性能监控器和OpenTelemetry集成的完整功能

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use super::*;
use super::alert_engine::*;
use super::performance_monitor::*;
use super::otel::*;
use super::metrics::*;
use super::alerts::*;
use super::analyzer::*;

/// 测试企业级监控和可观测性系统的完整集成
#[tokio::test]
async fn test_enterprise_observability_integration() {
    println!("🧪 测试企业级监控和可观测性系统集成");
    
    // 创建模拟的指标收集器
    let metrics_collector = Arc::new(MockMetricsCollector::new());
    
    // 创建性能分析器
    let performance_analyzer = Arc::new(MockPerformanceAnalyzer::new());
    
    // 创建自动化执行器
    let automation_executor = Arc::new(DefaultAutomationExecutor::new(
        AutomationConfig::default()
    ));
    
    // 创建智能告警引擎
    let alert_engine = SmartAlertEngine::new(
        AlertEngineConfig::default(),
        metrics_collector.clone(),
        automation_executor,
    );
    
    // 创建企业级性能监控器
    let performance_monitor = EnterprisePerformanceMonitor::new(
        PerformanceMonitorConfig::default(),
        metrics_collector.clone(),
        performance_analyzer,
    );
    
    // 创建OpenTelemetry导出器
    let otel_exporter = HttpOtlpExporter::new("http://localhost:4318".to_string())
        .with_timeout(Duration::from_secs(5));
    
    // 测试告警规则配置
    test_alert_rule_configuration(&alert_engine).await;
    
    // 测试性能监控
    test_performance_monitoring(&performance_monitor).await;
    
    // 测试OpenTelemetry集成
    test_opentelemetry_integration(&otel_exporter).await;
    
    // 测试端到端监控流程
    test_end_to_end_monitoring_flow(&alert_engine, &performance_monitor).await;
    
    println!("✅ 企业级监控和可观测性系统集成测试完成");
}

/// 测试告警规则配置
async fn test_alert_rule_configuration(alert_engine: &SmartAlertEngine) {
    println!("📋 测试告警规则配置");
    
    // 创建响应时间告警规则
    let response_time_rule = AlertRule {
        id: "response_time_alert".to_string(),
        name: "响应时间告警".to_string(),
        description: "当平均响应时间超过1秒时触发".to_string(),
        condition: AlertCondition::ResponseTime {
            threshold_ms: 1000,
            duration_minutes: 5,
        },
        severity: AlertSeverity::Warning,
        enabled: true,
        cooldown_duration: Default::default(),
        channels: vec!["email".to_string(), "slack".to_string()],
        labels: {
            let mut labels = std::collections::HashMap::new();
            labels.insert("service".to_string(), "lumos-ai".to_string());
            labels.insert("environment".to_string(), "production".to_string());
            labels
        },
        created_at: 0,
        updated_at: 0,
    };
    
    // 创建错误率告警规则
    let error_rate_rule = AlertRule {
        id: "error_rate_alert".to_string(),
        name: "错误率告警".to_string(),
        description: "当错误率超过5%时触发".to_string(),
        condition: AlertCondition::ErrorRate {
            threshold_percent: 5.0,
            duration_minutes: 3,
        },
        severity: AlertSeverity::Critical,
        enabled: true,
        channels: vec!["pagerduty".to_string()],
        labels: {
            let mut labels = std::collections::HashMap::new();
            labels.insert("service".to_string(), "lumos-ai".to_string());
            labels.insert("team".to_string(), "platform".to_string());
            labels
        },
    };
    
    // 添加告警规则
    alert_engine.add_rule(response_time_rule).await.unwrap();
    alert_engine.add_rule(error_rate_rule).await.unwrap();
    
    // 启动告警引擎
    alert_engine.start().await.unwrap();
    
    // 等待告警检查
    sleep(Duration::from_secs(2)).await;
    
    // 验证告警统计
    let stats = alert_engine.get_alert_statistics().await;
    println!("📊 告警统计: 总告警数={}, 活跃告警数={}", stats.total_alerts, stats.active_alerts);
    
    // 测试手动告警确认
    let active_alerts = alert_engine.get_active_alerts().await;
    if !active_alerts.is_empty() {
        let alert_id = &active_alerts[0].id;
        alert_engine.acknowledge_alert(alert_id).await.unwrap();
        println!("✅ 告警确认测试完成");
    }
}

/// 测试性能监控
async fn test_performance_monitoring(performance_monitor: &EnterprisePerformanceMonitor) {
    println!("📈 测试性能监控");
    
    // 启动性能监控
    performance_monitor.start().await.unwrap();
    
    // 等待数据收集
    sleep(Duration::from_secs(3)).await;
    
    // 获取当前性能指标
    if let Some(current_metrics) = performance_monitor.get_current_performance_metrics().await {
        println!("📊 当前性能指标:");
        println!("   响应时间: {:.2}ms", current_metrics.response_time.avg_ms);
        println!("   吞吐量: {:.2} RPS", current_metrics.throughput.requests_per_second);
        println!("   CPU使用率: {:.1}%", current_metrics.resource_usage.cpu_usage_percent);
        println!("   内存使用率: {:.1}%", current_metrics.resource_usage.memory_usage_percent);
        println!("   错误率: {:.1}%", current_metrics.error_metrics.error_rate_percent);
    }
    
    // 获取性能预测
    if let Some(prediction) = performance_monitor.get_performance_prediction().await {
        println!("🔮 性能预测:");
        println!("   预测响应时间: {:.2}ms", prediction.predicted_response_time_ms);
        println!("   预测吞吐量: {:.2} RPS", prediction.predicted_throughput_rps);
        println!("   预测置信度: {:.1}%", prediction.confidence * 100.0);
    }
    
    // 获取优化建议
    let suggestions = performance_monitor.get_optimization_suggestions().await;
    println!("💡 优化建议数量: {}", suggestions.len());
    for suggestion in suggestions.iter().take(2) {
        println!("   - {}: {}", suggestion.title, suggestion.description);
    }
    
    // 获取性能摘要报告
    let summary = performance_monitor.get_performance_summary().await;
    println!("🏥 系统健康分数: {:.1}/100", summary.health_score);
    
    // 获取监控统计
    let stats = performance_monitor.get_monitoring_statistics().await;
    println!("📊 监控统计: 数据点={}, 运行时间={}秒", 
        stats.total_data_points, stats.uptime_seconds);
}

/// 测试OpenTelemetry集成
async fn test_opentelemetry_integration(otel_exporter: &HttpOtlpExporter) {
    println!("🔗 测试OpenTelemetry集成");
    
    // 创建测试span
    let test_spans = vec![
        OtelSpan {
            trace_id: "test-trace-123".to_string(),
            span_id: "test-span-456".to_string(),
            parent_span_id: None,
            name: "test_operation".to_string(),
            kind: SpanKind::Internal,
            start_time: 1640995200000,
            end_time: 1640995201000,
            status: SpanStatus::Ok,
            attributes: {
                let mut attrs = std::collections::HashMap::new();
                attrs.insert("service.name".to_string(), AttributeValue::String("lumos-ai".to_string()));
                attrs.insert("operation.duration".to_string(), AttributeValue::Int(1000));
                attrs
            },
            events: vec![
                SpanEvent {
                    timestamp: 1640995200500,
                    name: "processing_started".to_string(),
                    attributes: std::collections::HashMap::new(),
                }
            ],
        }
    ];
    
    // 创建测试指标
    let test_metrics = vec![
        OtelMetric {
            name: "response_time".to_string(),
            description: "Average response time".to_string(),
            unit: "ms".to_string(),
            data_points: vec![
                DataPoint {
                    timestamp: 1640995200000,
                    value: DataPointValue::Double(250.5),
                    attributes: {
                        let mut attrs = std::collections::HashMap::new();
                        attrs.insert("endpoint".to_string(), AttributeValue::String("/api/v1/agents".to_string()));
                        attrs
                    },
                }
            ],
        }
    ];
    
    // 测试span导出（模拟）
    println!("📤 测试span导出");
    match otel_exporter.export_spans(test_spans).await {
        Ok(_) => println!("✅ Span导出测试成功"),
        Err(e) => println!("⚠️  Span导出测试失败（预期，因为没有真实的OTLP端点）: {}", e),
    }
    
    // 测试指标导出（模拟）
    println!("📤 测试指标导出");
    match otel_exporter.export_metrics(test_metrics).await {
        Ok(_) => println!("✅ 指标导出测试成功"),
        Err(e) => println!("⚠️  指标导出测试失败（预期，因为没有真实的OTLP端点）: {}", e),
    }
    
    // 测试强制刷新
    otel_exporter.force_flush(Duration::from_secs(1)).await.unwrap();
    println!("✅ 强制刷新测试完成");
}

/// 测试端到端监控流程
async fn test_end_to_end_monitoring_flow(
    alert_engine: &SmartAlertEngine,
    performance_monitor: &EnterprisePerformanceMonitor,
) {
    println!("🔄 测试端到端监控流程");
    
    // 模拟性能问题场景
    println!("🎭 模拟性能问题场景");
    
    // 等待系统收集数据
    sleep(Duration::from_secs(2)).await;
    
    // 检查是否触发了告警
    let active_alerts = alert_engine.get_active_alerts().await;
    println!("🚨 活跃告警数量: {}", active_alerts.len());
    
    // 检查是否生成了优化建议
    let suggestions = performance_monitor.get_optimization_suggestions().await;
    println!("💡 优化建议数量: {}", suggestions.len());
    
    // 获取综合健康状态
    let summary = performance_monitor.get_performance_summary().await;
    println!("🏥 系统健康状态: {:.1}/100", summary.health_score);
    
    // 模拟问题解决
    if !active_alerts.is_empty() {
        let alert_id = &active_alerts[0].id;
        alert_engine.resolve_alert(alert_id).await.unwrap();
        println!("✅ 模拟问题解决完成");
    }
    
    println!("✅ 端到端监控流程测试完成");
}

/// 模拟指标收集器
struct MockMetricsCollector {
    // 模拟数据
}

impl MockMetricsCollector {
    fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl MetricsCollector for MockMetricsCollector {
    async fn record_agent_metrics(&self, _metrics: AgentMetrics) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
    
    async fn record_tool_metrics(&self, _metrics: ToolMetrics) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
    
    async fn get_metrics_summary(
        &self,
        _agent_id: Option<&str>,
        _start_time: Option<u64>,
        _end_time: Option<u64>,
    ) -> Result<MetricsSummary, Box<dyn std::error::Error + Send + Sync>> {
        Ok(MetricsSummary {
            total_executions: 1000,
            successful_executions: 950,
            failed_executions: 50,
            avg_execution_time_ms: 1200.0, // 超过阈值以触发告警
            total_tokens_used: 50000,
            total_cost: 25.50,
            peak_memory_usage_mb: 512.0,
            cache_hit_rate: 0.85,
        })
    }
}

/// 模拟性能分析器
struct MockPerformanceAnalyzer;

impl MockPerformanceAnalyzer {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl PerformanceAnalyzer for MockPerformanceAnalyzer {
    async fn analyze_performance(
        &self,
        _metrics: &MetricsSummary,
    ) -> Result<PerformanceAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(PerformanceAnalysis {
            overall_score: 75.0,
            bottlenecks: vec![],
            anomalies: vec![],
            recommendations: vec![],
            trends: vec![],
        })
    }
}
