use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::collections::HashMap;
use lumosai_core::error::Error;
use lumosai_core::agent::config_validator::ConfigValidator;
use lumosai_core::monitoring::{MetricsCollector, AgentMonitor, MetricType};
use serde_json::json;

/// 测试配置验证功能
#[tokio::test]
async fn test_config_validation_functionality() {
    let validator = ConfigValidator::new();
    
    // 测试有效配置
    let valid_config = json!({
        "name": "test-agent",
        "model": "gpt-4",
        "temperature": 0.7,
        "max_tokens": 1000
    });
    
    let result = validator.validate_json(&valid_config);
    assert!(result.is_ok(), "Valid configuration should pass validation");
    
    // 测试无效配置 - 缺少必需字段
    let invalid_config = json!({
        "model": "gpt-4"
        // missing "name"
    });
    
    let result = validator.validate_json(&invalid_config);
    assert!(result.is_err(), "Invalid configuration should fail validation");
    assert!(result.unwrap_err().to_string().contains("Missing required field: name"));
    
    // 测试无效配置 - 错误的温度值
    let invalid_temp_config = json!({
        "name": "test-agent",
        "model": "gpt-4",
        "temperature": 3.0  // invalid: > 2.0
    });
    
    let result = validator.validate_json(&invalid_temp_config);
    assert!(result.is_err(), "Invalid temperature should fail validation");
    assert!(result.unwrap_err().to_string().contains("Temperature must be between 0.0 and 2.0"));
    
    // 测试验证报告
    let config_with_issues = json!({
        "name": "test-agent",
        "model": "gpt-4",
        "temperature": 3.0,  // invalid
        "unknown_field": "value"  // unknown
    });
    
    let report = validator.validate_with_report(&config_with_issues);
    assert!(!report.is_valid(), "Configuration with issues should not be valid");
    assert!(!report.errors.is_empty(), "Should have validation errors");
    assert!(!report.warnings.is_empty(), "Should have warnings for unknown fields");
    
    println!("Validation report: {}", report.summary());
}

/// 测试监控功能
#[tokio::test]
async fn test_monitoring_functionality() {
    let collector = MetricsCollector::new();
    
    // 测试计数器
    collector.increment_counter("test_requests", None).unwrap();
    collector.increment_counter("test_requests", None).unwrap();
    collector.increment_counter("test_requests", None).unwrap();
    
    // 测试带标签的计数器
    let labels = HashMap::from([
        ("service".to_string(), "agent".to_string()),
        ("version".to_string(), "1.0".to_string()),
    ]);
    collector.increment_counter("labeled_requests", Some(labels)).unwrap();
    
    // 测试仪表盘
    collector.set_gauge("memory_usage", 75.5, None).unwrap();
    collector.set_gauge("cpu_usage", 45.2, None).unwrap();
    
    // 测试计时器
    collector.record_timer("request_duration", Duration::from_millis(250), None).unwrap();
    collector.record_timer("request_duration", Duration::from_millis(180), None).unwrap();
    
    // 测试直方图
    let response_times = vec![100.0, 150.0, 200.0, 120.0, 180.0];
    collector.record_histogram("response_times", response_times, None).unwrap();
    
    // 验证指标收集
    let metrics = collector.get_metrics().unwrap();
    assert!(!metrics.is_empty(), "Should have collected metrics");
    
    // 验证统计信息
    let stats = collector.get_stats().unwrap();
    assert!(stats.total_metrics > 0, "Should have metrics");
    assert!(stats.total_counters > 0, "Should have counters");
    assert!(stats.total_gauges > 0, "Should have gauges");
    
    println!("Collected {} metrics", stats.total_metrics);
    println!("Counters: {}, Gauges: {}", stats.total_counters, stats.total_gauges);
}

/// 测试Agent监控器
#[tokio::test]
async fn test_agent_monitor() {
    let monitor = AgentMonitor::new("test-agent".to_string());
    
    // 记录各种Agent操作
    monitor.record_generation_request().unwrap();
    monitor.record_generation_request().unwrap();
    monitor.record_generation_latency(Duration::from_millis(500)).unwrap();
    monitor.record_generation_latency(Duration::from_millis(750)).unwrap();
    
    monitor.record_tool_call("calculator").unwrap();
    monitor.record_tool_call("web_search").unwrap();
    monitor.record_tool_call("calculator").unwrap();
    
    monitor.record_error("timeout").unwrap();
    monitor.record_error("rate_limit").unwrap();
    
    monitor.set_active_connections(5.0).unwrap();
    monitor.set_active_connections(8.0).unwrap();
    
    // 验证指标收集
    let metrics = monitor.collector().get_metrics().unwrap();
    assert!(!metrics.is_empty(), "Should have collected agent metrics");
    
    // 验证指标类型
    let has_counter = metrics.iter().any(|m| matches!(m.metric_type, MetricType::Counter));
    let has_gauge = metrics.iter().any(|m| matches!(m.metric_type, MetricType::Gauge));
    let has_timer = metrics.iter().any(|m| matches!(m.metric_type, MetricType::Timer));
    
    assert!(has_counter, "Should have counter metrics");
    assert!(has_gauge, "Should have gauge metrics");
    assert!(has_timer, "Should have timer metrics");
    
    // 验证标签
    let agent_metrics: Vec<_> = metrics.iter()
        .filter(|m| m.labels.get("agent") == Some(&"test-agent".to_string()))
        .collect();
    
    assert!(!agent_metrics.is_empty(), "Should have metrics with agent label");
    
    println!("Agent monitor collected {} metrics", metrics.len());
}

/// 测试时间范围查询
#[tokio::test]
async fn test_metrics_time_range_query() {
    let collector = MetricsCollector::new();
    
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    // 记录一些指标
    collector.increment_counter("test_metric", None).unwrap();
    
    // 等待一小段时间
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    collector.increment_counter("test_metric", None).unwrap();
    
    let end_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    // 查询时间范围内的指标
    let metrics_in_range = collector.get_metrics_in_range(start_time, end_time).unwrap();
    assert!(!metrics_in_range.is_empty(), "Should find metrics in time range");
    
    // 查询未来时间范围（应该为空）
    let future_start = end_time + 1000;
    let future_end = end_time + 2000;
    let future_metrics = collector.get_metrics_in_range(future_start, future_end).unwrap();
    assert!(future_metrics.is_empty(), "Should not find metrics in future time range");
}

/// 测试指标清除功能
#[tokio::test]
async fn test_metrics_cleanup() {
    let collector = MetricsCollector::new();
    
    // 添加一些指标
    collector.increment_counter("test_counter", None).unwrap();
    collector.set_gauge("test_gauge", 42.0, None).unwrap();
    
    // 验证指标存在
    let metrics_before = collector.get_metrics().unwrap();
    assert!(!metrics_before.is_empty(), "Should have metrics before cleanup");
    
    // 清除指标
    collector.clear_metrics().unwrap();
    
    // 验证指标已清除
    let metrics_after = collector.get_metrics().unwrap();
    assert!(metrics_after.is_empty(), "Should have no metrics after cleanup");
    
    let stats_after = collector.get_stats().unwrap();
    assert_eq!(stats_after.total_metrics, 0, "Should have zero metrics");
    assert_eq!(stats_after.total_counters, 0, "Should have zero counters");
    assert_eq!(stats_after.total_gauges, 0, "Should have zero gauges");
}

/// 测试配置验证器的自定义规则
#[tokio::test]
async fn test_custom_validation_rules() {
    let mut validator = ConfigValidator::new();
    
    // 添加自定义验证规则
    validator.add_rule("custom_field", Box::new(|value: &serde_json::Value| {
        if let Some(s) = value.as_str() {
            if s.starts_with("custom_") {
                Ok(())
            } else {
                Err(Error::Validation("Custom field must start with 'custom_'".to_string()))
            }
        } else {
            Err(Error::Validation("Custom field must be a string".to_string()))
        }
    }));
    
    // 添加自定义必需字段
    validator.add_required_field("custom_field");
    
    // 测试有效的自定义配置
    let valid_custom_config = json!({
        "name": "test-agent",
        "model": "gpt-4",
        "custom_field": "custom_value"
    });
    
    let result = validator.validate_json(&valid_custom_config);
    assert!(result.is_ok(), "Valid custom configuration should pass");
    
    // 测试无效的自定义配置
    let invalid_custom_config = json!({
        "name": "test-agent",
        "model": "gpt-4",
        "custom_field": "invalid_value"  // doesn't start with "custom_"
    });
    
    let result = validator.validate_json(&invalid_custom_config);
    assert!(result.is_err(), "Invalid custom configuration should fail");
    assert!(result.unwrap_err().to_string().contains("Custom field must start with 'custom_'"));
}

/// 综合集成测试
#[tokio::test]
async fn test_comprehensive_integration() {
    // 创建配置验证器
    let validator = ConfigValidator::new();
    
    // 创建Agent监控器
    let monitor = AgentMonitor::new("integration-test-agent".to_string());
    
    // 验证配置
    let config = json!({
        "name": "integration-test-agent",
        "model": "gpt-4",
        "temperature": 0.8,
        "max_tokens": 2000
    });
    
    let validation_result = validator.validate_json(&config);
    assert!(validation_result.is_ok(), "Configuration should be valid");
    
    // 记录配置验证成功
    monitor.record_generation_request().unwrap();
    
    // 模拟一些Agent操作
    let start_time = std::time::Instant::now();
    
    // 模拟处理延迟
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let processing_duration = start_time.elapsed();
    monitor.record_generation_latency(processing_duration).unwrap();
    
    // 模拟工具调用
    monitor.record_tool_call("config_validator").unwrap();
    
    // 验证监控数据
    let metrics = monitor.collector().get_metrics().unwrap();
    assert!(!metrics.is_empty(), "Should have collected integration metrics");
    
    let stats = monitor.collector().get_stats().unwrap();
    println!("Integration test collected {} metrics", stats.total_metrics);
    
    // 验证所有功能都正常工作
    assert!(validation_result.is_ok());
    assert!(stats.total_metrics > 0);
    assert!(stats.total_counters > 0);
    
    println!("✅ Comprehensive integration test passed!");
}
