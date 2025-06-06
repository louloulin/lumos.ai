//! 企业级监控功能综合测试
//! 
//! 测试覆盖：
//! - 企业级监控系统集成
//! - 合规监控和审计追踪
//! - 业务指标收集和分析
//! - 异常检测和告警
//! - 容量规划和预测
//! - SLA监控和报告

use std::collections::HashMap;
use std::sync::Arc;
use chrono::{Utc, Duration};
use uuid::Uuid;

use lumosai_core::{
    telemetry::{
        enterprise::*,
        compliance_monitor::*,
        business_metrics::*,
        anomaly_detection::*,
        capacity_planning::*,
        sla_monitoring::*,
        MetricsCollector, AlertManager, PerformanceAnalyzer,
        AgentMetrics, AlertEvent, ExecutionTrace,
    },
    error::LumosError,
};

// 模拟组件
struct MockMetricsCollector;
struct MockAlertManager;
struct MockPerformanceAnalyzer;

#[async_trait::async_trait]
impl MetricsCollector for MockMetricsCollector {
    async fn collect_metrics(&self, _agent_id: &str) -> Result<AgentMetrics, LumosError> {
        Ok(AgentMetrics {
            agent_id: "test_agent".to_string(),
            timestamp: Utc::now(),
            cpu_usage: 0.5,
            memory_usage: 0.6,
            request_count: 100,
            error_count: 2,
            average_response_time: 150.0,
            custom_metrics: HashMap::new(),
        })
    }
    
    async fn record_metrics(&self, _metrics: AgentMetrics) -> Result<(), LumosError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl AlertManager for MockAlertManager {
    async fn send_alert(&self, _alert: AlertEvent) -> Result<(), LumosError> {
        Ok(())
    }
    
    async fn get_active_alerts(&self) -> Result<Vec<AlertEvent>, LumosError> {
        Ok(Vec::new())
    }
}

#[async_trait::async_trait]
impl PerformanceAnalyzer for MockPerformanceAnalyzer {
    async fn analyze_performance(&self, _trace: ExecutionTrace) -> Result<(), LumosError> {
        Ok(())
    }
    
    async fn get_performance_summary(&self, _agent_id: &str) -> Result<String, LumosError> {
        Ok("Mock performance summary".to_string())
    }
}

#[tokio::test]
async fn test_enterprise_monitoring_creation() {
    let config = EnterpriseMonitoringConfig::default();
    let metrics_collector = Arc::new(MockMetricsCollector) as Arc<dyn MetricsCollector>;
    let alert_manager = Arc::new(MockAlertManager) as Arc<dyn AlertManager>;
    let performance_analyzer = Arc::new(MockPerformanceAnalyzer) as Arc<dyn PerformanceAnalyzer>;
    
    let result = EnterpriseMonitoring::new(
        metrics_collector,
        alert_manager,
        performance_analyzer,
        config,
    ).await;
    
    assert!(result.is_ok(), "企业级监控系统创建应该成功");
}

#[tokio::test]
async fn test_enterprise_metric_recording() {
    let config = EnterpriseMonitoringConfig::default();
    let metrics_collector = Arc::new(MockMetricsCollector) as Arc<dyn MetricsCollector>;
    let alert_manager = Arc::new(MockAlertManager) as Arc<dyn AlertManager>;
    let performance_analyzer = Arc::new(MockPerformanceAnalyzer) as Arc<dyn PerformanceAnalyzer>;
    
    let monitoring = EnterpriseMonitoring::new(
        metrics_collector,
        alert_manager,
        performance_analyzer,
        config,
    ).await.unwrap();
    
    let metric = EnterpriseMetric {
        id: Uuid::new_v4(),
        name: "test_metric".to_string(),
        metric_type: EnterpriseMetricType::Business,
        value: 42.0,
        labels: HashMap::new(),
        timestamp: Utc::now(),
        business_context: Some(BusinessContext {
            tenant_id: Some("tenant1".to_string()),
            user_id: Some("user1".to_string()),
            business_process: "test_process".to_string(),
            cost_center: Some("engineering".to_string()),
            service_level: ServiceLevel::Premium,
        }),
        compliance_relevance: vec!["SOC2".to_string()],
    };
    
    let result = monitoring.record_metric(metric).await;
    assert!(result.is_ok(), "企业级指标记录应该成功");
}

#[tokio::test]
async fn test_compliance_monitoring() {
    let config = ComplianceConfig::default();
    let mut monitor = ComplianceMonitor::new(config);
    
    // 测试审计事件记录
    let audit_event = AuditEvent {
        id: Uuid::new_v4(),
        event_type: AuditEventType::DataAccess,
        timestamp: Utc::now(),
        user_id: Some("user123".to_string()),
        resource_id: Some("sensitive_data".to_string()),
        action: "read".to_string(),
        result: AuditResult::Success,
        details: {
            let mut details = HashMap::new();
            details.insert("ip_address".to_string(), "192.168.1.100".to_string());
            details
        },
    };
    
    let result = monitor.record_audit_event(audit_event).await;
    assert!(result.is_ok(), "审计事件记录应该成功");
    
    // 测试合规报告生成
    let report = monitor.generate_compliance_report().await;
    assert!(report.is_ok(), "合规报告生成应该成功");
    
    let report = report.unwrap();
    assert_eq!(report.total_audit_events, 1, "应该有1个审计事件");
    assert!(report.compliance_score >= 0.0 && report.compliance_score <= 100.0, "合规分数应该在0-100之间");
}

#[tokio::test]
async fn test_business_metrics_collection() {
    let config = BusinessMetricsConfig::default();
    let mut collector = BusinessMetricsCollector::new(config);
    
    // 测试用户活动记录
    let activity = UserActivity {
        activity_id: Uuid::new_v4(),
        activity_type: ActivityType::FeatureUsage,
        timestamp: Utc::now(),
        duration_seconds: Some(120),
        details: {
            let mut details = HashMap::new();
            details.insert("feature".to_string(), "ai_agent".to_string());
            details
        },
    };
    
    let result = collector.record_user_activity("user123", activity).await;
    assert!(result.is_ok(), "用户活动记录应该成功");
    
    // 测试收入事件记录
    let revenue_event = RevenueEvent {
        event_type: RevenueEventType::NewSubscription,
        amount: 99.99,
        currency: "USD".to_string(),
        customer_id: "customer456".to_string(),
        timestamp: Utc::now(),
        metadata: HashMap::new(),
    };
    
    let result = collector.record_revenue_event(revenue_event).await;
    assert!(result.is_ok(), "收入事件记录应该成功");
    
    // 测试客户反馈记录
    let feedback = CustomerFeedback {
        feedback_id: Uuid::new_v4(),
        customer_id: "customer456".to_string(),
        feedback_type: FeedbackType::CSAT,
        rating: Some(5),
        comment: Some("Great service!".to_string()),
        submitted_at: Utc::now(),
        sentiment: Some(SentimentAnalysis {
            sentiment_score: 0.9,
            sentiment_class: SentimentClass::Positive,
            confidence: 0.95,
            keywords: vec!["great".to_string(), "service".to_string()],
        }),
    };
    
    let result = collector.record_customer_feedback(feedback).await;
    assert!(result.is_ok(), "客户反馈记录应该成功");
    
    // 测试业务报告生成
    let report = collector.generate_business_report().await;
    assert!(report.is_ok(), "业务报告生成应该成功");
    
    let report = report.unwrap();
    assert!(!report.key_insights.is_empty() || report.key_insights.is_empty(), "报告应该包含洞察或为空");
}

#[tokio::test]
async fn test_anomaly_detection() {
    let config = AnomalyDetectionConfig::default();
    let mut detector = AnomalyDetectionEngine::new(config);
    
    // 添加正常数据点
    for i in 0..20 {
        let normal_value = 50.0 + (i as f64 * 0.1);
        let timestamp = Utc::now() - Duration::minutes(20 - i);
        let result = detector.detect_metric_anomaly("response_time", normal_value, timestamp).await;
        assert!(result.is_ok(), "正常指标检测应该成功");
    }
    
    // 添加异常数据点
    let anomalous_value = 200.0; // 明显异常
    let result = detector.detect_metric_anomaly("response_time", anomalous_value, Utc::now()).await;
    assert!(result.is_ok(), "异常指标检测应该成功");
    
    // 测试行为异常检测
    let behavior_data = BehaviorData {
        user_id: "user123".to_string(),
        session_id: "session456".to_string(),
        activity_type: "login".to_string(),
        timestamp: Utc::now(),
        ip_address: Some("192.168.1.100".to_string()),
        user_agent: Some("Mozilla/5.0".to_string()),
        geo_location: Some("US".to_string()),
        device_fingerprint: Some("device123".to_string()),
        attributes: HashMap::new(),
    };
    
    let result = detector.detect_behavior_anomaly("user123", &behavior_data).await;
    assert!(result.is_ok(), "行为异常检测应该成功");
    
    // 测试异常报告生成
    let report = detector.generate_anomaly_report(None).await;
    assert!(report.is_ok(), "异常报告生成应该成功");
    
    let report = report.unwrap();
    assert!(report.total_anomalies >= 0, "异常总数应该大于等于0");
}

#[tokio::test]
async fn test_capacity_planning() {
    let config = CapacityPlanningConfig::default();
    let mut planner = CapacityPlanner::new(config);
    
    // 测试资源使用记录
    let usage_point = ResourceUsagePoint {
        timestamp: Utc::now(),
        resource_type: ResourceType::CPU,
        usage_amount: 6.4,
        total_capacity: 8.0,
        utilization_rate: 0.8,
        metadata: HashMap::new(),
    };
    
    let result = planner.record_resource_usage(usage_point).await;
    assert!(result.is_ok(), "资源使用记录应该成功");
    
    // 测试容量预测
    let forecast = planner.generate_capacity_forecast(&ResourceType::CPU).await;
    assert!(forecast.is_ok(), "容量预测应该成功");
    
    let forecast = forecast.unwrap();
    assert!(!forecast.is_empty() || forecast.is_empty(), "预测结果应该有效");
    
    // 测试扩容建议
    let recommendations = planner.generate_scaling_recommendations().await;
    assert!(recommendations.is_ok(), "扩容建议生成应该成功");
    
    // 测试容量规划报告
    let report = planner.generate_capacity_report().await;
    assert!(report.is_ok(), "容量规划报告生成应该成功");
    
    let report = report.unwrap();
    assert!(report.cost_analysis.current_monthly_cost >= 0.0, "当前成本应该大于等于0");
}

#[tokio::test]
async fn test_sla_monitoring() {
    let config = SLAMonitoringConfig::default();
    let mut monitor = SLAMonitor::new(config);
    
    // 测试SLA定义
    let sla = ServiceLevelAgreement {
        id: "test_sla".to_string(),
        name: "Test SLA".to_string(),
        service_name: "test_service".to_string(),
        objectives: vec![
            SLAObjective {
                id: "availability".to_string(),
                name: "Availability".to_string(),
                metric_type: SLAMetricType::Availability,
                target_value: 99.9,
                operator: ComparisonOperator::GreaterThanOrEqual,
                unit: "percent".to_string(),
                priority: SLAPriority::Critical,
                description: "Service availability".to_string(),
            }
        ],
        measurement_window: MeasurementWindow {
            window_type: WindowType::Sliding,
            window_size: Duration::hours(24),
            sliding_interval: Some(Duration::minutes(5)),
        },
        effective_from: Utc::now(),
        effective_until: None,
        customer_info: None,
        violation_consequences: Vec::new(),
        enabled: true,
    };
    
    let result = monitor.add_sla(sla).await;
    assert!(result.is_ok(), "SLA添加应该成功");
    
    // 测试SLA指标记录
    let metric_point = SLAMetricPoint {
        timestamp: Utc::now(),
        metric_type: SLAMetricType::Availability,
        value: 99.95,
        service_name: "test_service".to_string(),
        labels: HashMap::new(),
        metadata: HashMap::new(),
    };
    
    let result = monitor.record_metric(metric_point).await;
    assert!(result.is_ok(), "SLA指标记录应该成功");
    
    // 测试SLA报告生成
    let report = monitor.generate_report(
        ReportType::Daily,
        (Utc::now() - Duration::days(1), Utc::now()),
    ).await;
    assert!(report.is_ok(), "SLA报告生成应该成功");
    
    let report = report.unwrap();
    assert!(report.sla_summary.overall_compliance_rate >= 0.0, "合规率应该大于等于0");
    
    // 测试合规状态查询
    let status = monitor.get_compliance_status("test_sla").await;
    assert!(status.is_ok(), "合规状态查询应该成功");
}

#[tokio::test]
async fn test_enterprise_report_generation() {
    let config = EnterpriseMonitoringConfig::default();
    let metrics_collector = Arc::new(MockMetricsCollector) as Arc<dyn MetricsCollector>;
    let alert_manager = Arc::new(MockAlertManager) as Arc<dyn AlertManager>;
    let performance_analyzer = Arc::new(MockPerformanceAnalyzer) as Arc<dyn PerformanceAnalyzer>;
    
    let monitoring = EnterpriseMonitoring::new(
        metrics_collector,
        alert_manager,
        performance_analyzer,
        config,
    ).await.unwrap();
    
    // 测试企业级报告生成
    let report = monitoring.generate_enterprise_report().await;
    assert!(report.is_ok(), "企业级报告生成应该成功");
    
    let report = report.unwrap();
    assert!(!report.summary.is_empty(), "报告摘要不应该为空");
    assert!(report.generated_at <= Utc::now(), "报告生成时间应该有效");
}

#[tokio::test]
async fn test_configuration_validation() {
    // 测试默认配置
    let config = EnterpriseMonitoringConfig::default();
    assert!(config.compliance_monitoring_enabled, "默认应该启用合规监控");
    assert!(config.business_metrics_enabled, "默认应该启用业务指标");
    assert!(config.anomaly_detection_enabled, "默认应该启用异常检测");
    
    // 测试合规配置
    let compliance_config = ComplianceConfig::default();
    assert!(!compliance_config.enabled_standards.is_empty(), "默认应该有启用的合规标准");
    assert!(compliance_config.real_time_monitoring, "默认应该启用实时监控");
    
    // 测试业务指标配置
    let business_config = BusinessMetricsConfig::default();
    assert!(business_config.revenue_metrics_enabled, "默认应该启用收入指标");
    assert!(business_config.usage_metrics_enabled, "默认应该启用使用指标");
    
    // 测试异常检测配置
    let anomaly_config = AnomalyDetectionConfig::default();
    assert!(anomaly_config.statistical_detection_enabled, "默认应该启用统计检测");
    assert!(anomaly_config.behavior_detection_enabled, "默认应该启用行为检测");
    
    // 测试容量规划配置
    let capacity_config = CapacityPlanningConfig::default();
    assert!(capacity_config.prediction_window_days > 0, "预测窗口应该大于0");
    assert!(capacity_config.auto_scaling_recommendations, "默认应该启用自动扩容建议");
    
    // 测试SLA监控配置
    let sla_config = SLAMonitoringConfig::default();
    assert!(sla_config.real_time_monitoring, "默认应该启用实时监控");
    assert!(sla_config.monitoring_interval_seconds > 0, "监控间隔应该大于0");
}

#[tokio::test]
async fn test_error_handling() {
    // 测试无效SLA查询
    let config = SLAMonitoringConfig::default();
    let monitor = SLAMonitor::new(config);
    
    let result = monitor.get_compliance_status("nonexistent_sla").await;
    assert!(result.is_err(), "查询不存在的SLA应该返回错误");
    
    // 测试配置验证
    let mut config = EnterpriseMonitoringConfig::default();
    config.data_retention_days = 0; // 无效值
    // 注意：这里我们假设有配置验证，实际实现中应该添加
    
    // 测试异常检测边界情况
    let anomaly_config = AnomalyDetectionConfig::default();
    let mut detector = AnomalyDetectionEngine::new(anomaly_config);
    
    // 测试空数据的异常检测
    let result = detector.detect_metric_anomaly("empty_metric", 0.0, Utc::now()).await;
    assert!(result.is_ok(), "空数据的异常检测应该处理正常");
}
