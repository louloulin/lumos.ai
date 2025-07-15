use std::time::Duration;
use std::collections::HashMap;
use lumosai_core::agent::{
    performance::PerformanceMonitor,
    trait_def::AgentStatus,
};
use lumosai_core::monitoring::{MetricsCollector, AgentMonitor};

/// 测试性能监控基础功能
#[tokio::test]
async fn test_performance_monitor_basic() {
    let monitor = PerformanceMonitor::new();
    
    // 测试请求计时
    let timer = monitor.start_request();
    tokio::time::sleep(Duration::from_millis(10)).await;
    timer.finish_success();
    
    // 获取指标
    let metrics = monitor.get_metrics().unwrap();
    assert_eq!(metrics.total_requests, 1);
    assert_eq!(metrics.successful_requests, 1);
    assert_eq!(metrics.failed_requests, 0);
    assert!(metrics.avg_response_time > 0.0);
    
    println!("✅ Performance monitor basic test passed");
}

/// 测试Agent状态枚举
#[test]
fn test_agent_status_enum() {
    // 测试默认状态
    let default_status = AgentStatus::default();
    assert_eq!(default_status, AgentStatus::Initializing);
    
    // 测试各种状态
    let ready = AgentStatus::Ready;
    let running = AgentStatus::Running;
    let paused = AgentStatus::Paused;
    let error = AgentStatus::Error("Test error".to_string());
    let stopped = AgentStatus::Stopped;
    
    // 测试状态比较
    assert_ne!(ready, running);
    assert_ne!(running, paused);
    
    // 测试错误状态
    match error {
        AgentStatus::Error(msg) => assert_eq!(msg, "Test error"),
        _ => panic!("Expected error status"),
    }
    
    println!("✅ Agent status enum test passed");
}

/// 测试监控系统基础功能
#[tokio::test]
async fn test_monitoring_system_basic() {
    let collector = MetricsCollector::new();
    
    // 测试计数器
    collector.increment_counter("test_counter", None).unwrap();
    collector.increment_counter("test_counter", None).unwrap();
    
    // 测试仪表盘
    collector.set_gauge("test_gauge", 42.0, None).unwrap();
    
    // 测试计时器
    collector.record_timer("test_timer", Duration::from_millis(100), None).unwrap();
    
    // 获取指标
    let metrics = collector.get_metrics().unwrap();
    assert!(!metrics.is_empty(), "Should have collected metrics");
    
    // 获取统计信息
    let stats = collector.get_stats().unwrap();
    assert!(stats.total_metrics > 0, "Should have metrics");
    
    println!("✅ Monitoring system basic test passed");
}

/// 测试Agent监控器基础功能
#[tokio::test]
async fn test_agent_monitor_basic() {
    let monitor = AgentMonitor::new("test-agent".to_string());
    
    // 记录一些操作
    monitor.record_generation_request().unwrap();
    monitor.record_generation_latency(Duration::from_millis(200)).unwrap();
    monitor.record_tool_call("test_tool").unwrap();
    monitor.record_error("test_error").unwrap();
    monitor.set_active_connections(3.0).unwrap();
    
    // 获取指标
    let metrics = monitor.collector().get_metrics().unwrap();
    assert!(!metrics.is_empty(), "Should have agent metrics");
    
    // 验证有Agent标签的指标
    let agent_metrics: Vec<_> = metrics.iter()
        .filter(|m| m.labels.get("agent") == Some(&"test-agent".to_string()))
        .collect();
    
    assert!(!agent_metrics.is_empty(), "Should have metrics with agent label");
    
    println!("✅ Agent monitor basic test passed");
}

/// 测试性能指标更新
#[tokio::test]
async fn test_performance_metrics_update() {
    let monitor = PerformanceMonitor::new();
    
    // 更新系统指标
    monitor.update_memory_usage(1024 * 1024 * 100).unwrap(); // 100MB
    monitor.update_cpu_usage(15.5).unwrap();
    monitor.update_cache_hit_rate(85.0).unwrap();
    
    // 获取指标
    let metrics = monitor.get_metrics().unwrap();
    assert_eq!(metrics.memory_usage, 1024 * 1024 * 100);
    assert_eq!(metrics.cpu_usage, 15.5);
    assert_eq!(metrics.cache_hit_rate, 85.0);
    
    println!("✅ Performance metrics update test passed");
}

/// 测试指标重置功能
#[tokio::test]
async fn test_metrics_reset() {
    let monitor = PerformanceMonitor::new();
    
    // 添加一些指标
    let timer = monitor.start_request();
    timer.finish_success();
    monitor.update_memory_usage(1024).unwrap();
    
    // 验证指标存在
    let metrics_before = monitor.get_metrics().unwrap();
    assert_eq!(metrics_before.total_requests, 1);
    assert_eq!(metrics_before.memory_usage, 1024);
    
    // 重置指标
    monitor.reset_metrics().unwrap();
    
    // 验证指标已重置
    let metrics_after = monitor.get_metrics().unwrap();
    assert_eq!(metrics_after.total_requests, 0);
    assert_eq!(metrics_after.memory_usage, 0);
    
    println!("✅ Metrics reset test passed");
}

/// 测试监控系统的时间范围查询
#[tokio::test]
async fn test_monitoring_time_range() {
    let collector = MetricsCollector::new();
    
    let start_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    // 记录指标
    collector.increment_counter("time_test", None).unwrap();
    
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    let end_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    // 查询时间范围内的指标
    let metrics_in_range = collector.get_metrics_in_range(start_time, end_time).unwrap();
    assert!(!metrics_in_range.is_empty(), "Should find metrics in time range");
    
    println!("✅ Monitoring time range test passed");
}

/// 测试监控系统的清除功能
#[tokio::test]
async fn test_monitoring_cleanup() {
    let collector = MetricsCollector::new();
    
    // 添加指标
    collector.increment_counter("cleanup_test", None).unwrap();
    collector.set_gauge("cleanup_gauge", 100.0, None).unwrap();
    
    // 验证指标存在
    let metrics_before = collector.get_metrics().unwrap();
    assert!(!metrics_before.is_empty(), "Should have metrics before cleanup");
    
    // 清除指标
    collector.clear_metrics().unwrap();
    
    // 验证指标已清除
    let metrics_after = collector.get_metrics().unwrap();
    assert!(metrics_after.is_empty(), "Should have no metrics after cleanup");
    
    println!("✅ Monitoring cleanup test passed");
}

/// 综合基础功能测试
#[tokio::test]
async fn test_comprehensive_basic_features() {
    // 创建各种监控组件
    let perf_monitor = PerformanceMonitor::new();
    let metrics_collector = MetricsCollector::new();
    let agent_monitor = AgentMonitor::new("comprehensive-test".to_string());
    
    // 执行一系列操作
    let timer = perf_monitor.start_request();
    
    // 模拟一些处理时间
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // 记录各种指标
    metrics_collector.increment_counter("operations", None).unwrap();
    agent_monitor.record_generation_request().unwrap();
    agent_monitor.record_generation_latency(Duration::from_millis(50)).unwrap();
    
    timer.finish_success();
    
    // 验证所有系统都正常工作
    let perf_metrics = perf_monitor.get_metrics().unwrap();
    let collected_metrics = metrics_collector.get_metrics().unwrap();
    let agent_metrics = agent_monitor.collector().get_metrics().unwrap();
    
    assert_eq!(perf_metrics.total_requests, 1);
    assert!(!collected_metrics.is_empty());
    assert!(!agent_metrics.is_empty());
    
    println!("✅ Comprehensive basic features test passed!");
    println!("Performance requests: {}", perf_metrics.total_requests);
    println!("Collected metrics: {}", collected_metrics.len());
    println!("Agent metrics: {}", agent_metrics.len());
}
