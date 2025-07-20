//! 性能监控系统示例 - 展示LumosAI的性能监控和优化
//! 
//! 这个示例展示了如何监控Agent性能、工作流执行时间、内存使用等指标。
//! 
//! 运行方式:
//! ```bash
//! cargo run --example performance_monitoring
//! ```

use lumosai_core::prelude::*;
use lumosai_core::llm::MockLlmProvider;
use lumosai_core::agent::AgentTrait; // 正确的Agent trait导入
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use anyhow::Result;
use tracing::{info, warn, error};
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;

/// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub agent_id: String,
    pub operation: String,
    pub duration: Duration,
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub success: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

/// 性能监控器
#[derive(Debug)]
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<Vec<PerformanceMetrics>>>,
    thresholds: PerformanceThresholds,
    alerts: Arc<RwLock<Vec<PerformanceAlert>>>,
}

/// 性能阈值配置
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    pub max_response_time: Duration,
    pub max_memory_usage: u64,
    pub max_cpu_usage: f64,
    pub min_success_rate: f64,
}

/// 性能警报
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub alert_type: AlertType,
    pub message: String,
    pub severity: AlertSeverity,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub agent_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    HighResponseTime,
    HighMemoryUsage,
    HighCpuUsage,
    LowSuccessRate,
    SystemOverload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
            thresholds: PerformanceThresholds {
                max_response_time: Duration::from_secs(5),
                max_memory_usage: 1024 * 1024 * 100, // 100MB
                max_cpu_usage: 80.0,
                min_success_rate: 0.95,
            },
            alerts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 记录性能指标
    pub async fn record_metric(&self, metric: PerformanceMetrics) {
        // 检查是否超过阈值
        self.check_thresholds(&metric).await;
        
        // 存储指标
        let mut metrics = self.metrics.write().await;
        metrics.push(metric);
        
        // 保持最近1000条记录
        if metrics.len() > 1000 {
            let excess = metrics.len() - 1000;
            metrics.drain(0..excess);
        }
    }

    /// 检查性能阈值
    async fn check_thresholds(&self, metric: &PerformanceMetrics) {
        let mut alerts = Vec::new();

        // 检查响应时间
        if metric.duration > self.thresholds.max_response_time {
            alerts.push(PerformanceAlert {
                alert_type: AlertType::HighResponseTime,
                message: format!("Agent {} 响应时间过长: {:?}", metric.agent_id, metric.duration),
                severity: AlertSeverity::Warning,
                timestamp: chrono::Utc::now(),
                agent_id: metric.agent_id.clone(),
            });
        }

        // 检查内存使用
        if metric.memory_usage > self.thresholds.max_memory_usage {
            alerts.push(PerformanceAlert {
                alert_type: AlertType::HighMemoryUsage,
                message: format!("Agent {} 内存使用过高: {} MB", 
                    metric.agent_id, metric.memory_usage / 1024 / 1024),
                severity: AlertSeverity::Critical,
                timestamp: chrono::Utc::now(),
                agent_id: metric.agent_id.clone(),
            });
        }

        // 检查CPU使用
        if metric.cpu_usage > self.thresholds.max_cpu_usage {
            alerts.push(PerformanceAlert {
                alert_type: AlertType::HighCpuUsage,
                message: format!("Agent {} CPU使用过高: {:.1}%", metric.agent_id, metric.cpu_usage),
                severity: AlertSeverity::Warning,
                timestamp: chrono::Utc::now(),
                agent_id: metric.agent_id.clone(),
            });
        }

        // 存储警报
        if !alerts.is_empty() {
            let mut alert_storage = self.alerts.write().await;
            alert_storage.extend(alerts);
        }
    }

    /// 获取性能统计
    pub async fn get_performance_stats(&self, agent_id: Option<&str>) -> PerformanceStats {
        let metrics = self.metrics.read().await;
        let filtered_metrics: Vec<_> = metrics.iter()
            .filter(|m| agent_id.map_or(true, |id| m.agent_id == id))
            .collect();

        if filtered_metrics.is_empty() {
            return PerformanceStats::default();
        }

        let total_requests = filtered_metrics.len();
        let successful_requests = filtered_metrics.iter().filter(|m| m.success).count();
        let success_rate = successful_requests as f64 / total_requests as f64;

        let avg_response_time = filtered_metrics.iter()
            .map(|m| m.duration.as_millis() as f64)
            .sum::<f64>() / total_requests as f64;

        let avg_memory_usage = filtered_metrics.iter()
            .map(|m| m.memory_usage as f64)
            .sum::<f64>() / total_requests as f64;

        let avg_cpu_usage = filtered_metrics.iter()
            .map(|m| m.cpu_usage)
            .sum::<f64>() / total_requests as f64;

        PerformanceStats {
            total_requests,
            successful_requests,
            success_rate,
            avg_response_time_ms: avg_response_time,
            avg_memory_usage_mb: avg_memory_usage / 1024.0 / 1024.0,
            avg_cpu_usage_percent: avg_cpu_usage,
        }
    }

    /// 获取最近的警报
    pub async fn get_recent_alerts(&self, limit: usize) -> Vec<PerformanceAlert> {
        let alerts = self.alerts.read().await;
        alerts.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }
}

/// 性能统计
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub total_requests: usize,
    pub successful_requests: usize,
    pub success_rate: f64,
    pub avg_response_time_ms: f64,
    pub avg_memory_usage_mb: f64,
    pub avg_cpu_usage_percent: f64,
}

/// 性能监控装饰器
pub struct MonitoredAgent {
    agent: Box<dyn AgentTrait>,
    monitor: Arc<PerformanceMonitor>,
    agent_id: String,
}

impl MonitoredAgent {
    pub fn new(agent: Box<dyn AgentTrait>, monitor: Arc<PerformanceMonitor>) -> Self {
        let agent_id = agent.get_name().to_string();
        Self {
            agent,
            monitor,
            agent_id,
        }
    }
}

impl MonitoredAgent {
    /// 监控Agent的generate方法
    pub async fn generate_monitored(&self, input: &str) -> Result<String> {
        let start_time = Instant::now();
        let start_memory = get_memory_usage();
        let start_cpu = get_cpu_usage();

        let result = self.agent.generate_simple(input).await;

        let duration = start_time.elapsed();
        let end_memory = get_memory_usage();
        let end_cpu = get_cpu_usage();

        let metric = PerformanceMetrics {
            agent_id: self.agent_id.clone(),
            operation: "generate".to_string(),
            duration,
            memory_usage: end_memory.saturating_sub(start_memory),
            cpu_usage: (end_cpu - start_cpu).max(0.0),
            success: result.is_ok(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        self.monitor.record_metric(metric).await;

        result.map_err(|e| anyhow::Error::new(e))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("📊 LumosAI 性能监控示例");
    println!("========================");

    // 创建LLM提供者
    let llm = Arc::new(MockLlmProvider::new(vec![
        "我正在处理您的请求...".to_string(),
        "计算完成，结果如下...".to_string(),
        "数据分析已完成...".to_string(),
        "任务执行成功...".to_string(),
    ]));

    // 1. 创建性能监控器
    println!("\n1️⃣ 创建性能监控器");
    println!("------------------");

    let monitor = Arc::new(PerformanceMonitor::new());
    info!("✅ 性能监控器创建成功");

    // 2. 创建被监控的Agent
    println!("\n2️⃣ 创建被监控的Agent");
    println!("--------------------");

    let base_agent = quick_agent("performance_test", "性能测试助手")
        .model(llm.clone())
        .tools(vec![calculator(), time_tool(), web_search()])
        .build()?;

    let monitored_agent = MonitoredAgent::new(Box::new(base_agent), monitor.clone());
    info!("✅ 监控Agent创建成功");

    // 3. 执行性能测试
    println!("\n3️⃣ 执行性能测试");
    println!("----------------");

    let test_queries = vec![
        "计算 123 + 456",
        "获取当前时间",
        "搜索人工智能相关信息",
        "执行复杂计算任务",
        "处理大量数据",
    ];

    for (i, query) in test_queries.iter().enumerate() {
        info!("🔄 执行测试 {}/{}: {}", i + 1, test_queries.len(), query);
        
        match monitored_agent.generate_monitored(query).await {
            Ok(response) => {
                info!("✅ 测试成功: {}", response);
            }
            Err(e) => {
                error!("❌ 测试失败: {}", e);
            }
        }

        // 添加一些延迟来模拟真实使用
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // 4. 查看性能统计
    println!("\n4️⃣ 性能统计分析");
    println!("----------------");

    let stats = monitor.get_performance_stats(Some("performance_test")).await;
    info!("📊 性能统计:");
    info!("   - 总请求数: {}", stats.total_requests);
    info!("   - 成功请求数: {}", stats.successful_requests);
    info!("   - 成功率: {:.2}%", stats.success_rate * 100.0);
    info!("   - 平均响应时间: {:.2}ms", stats.avg_response_time_ms);
    info!("   - 平均内存使用: {:.2}MB", stats.avg_memory_usage_mb);
    info!("   - 平均CPU使用: {:.2}%", stats.avg_cpu_usage_percent);

    // 5. 检查性能警报
    println!("\n5️⃣ 性能警报检查");
    println!("----------------");

    let alerts = monitor.get_recent_alerts(10).await;
    if alerts.is_empty() {
        info!("✅ 没有性能警报");
    } else {
        warn!("⚠️ 发现{}个性能警报:", alerts.len());
        for alert in &alerts {
            let severity_icon = match alert.severity {
                AlertSeverity::Info => "ℹ️",
                AlertSeverity::Warning => "⚠️",
                AlertSeverity::Critical => "🚨",
            };
            info!("   {} {}: {}", severity_icon, alert.agent_id, alert.message);
        }
    }

    // 6. 压力测试
    println!("\n6️⃣ 压力测试");
    println!("------------");

    info!("🔥 开始压力测试...");
    let stress_start = Instant::now();
    
    let mut handles = Vec::new();
    for i in 0..10 {
        let agent = MonitoredAgent::new(
            Box::new(quick_agent(&format!("stress_test_{}", i), "压力测试助手")
                .model(llm.clone())
                .tools(vec![calculator()])
                .build()?),
            monitor.clone()
        );
        
        let handle = tokio::spawn(async move {
            for j in 0..5 {
                let query = format!("计算 {} * {}", i, j);
                let _ = agent.generate_monitored(&query).await;
            }
        });
        
        handles.push(handle);
    }

    // 等待所有任务完成
    for handle in handles {
        handle.await?;
    }

    let stress_duration = stress_start.elapsed();
    info!("✅ 压力测试完成，耗时: {:?}", stress_duration);

    // 7. 最终统计
    println!("\n7️⃣ 最终统计");
    println!("------------");

    let final_stats = monitor.get_performance_stats(None).await;
    info!("📈 最终性能统计:");
    info!("   - 总请求数: {}", final_stats.total_requests);
    info!("   - 成功率: {:.2}%", final_stats.success_rate * 100.0);
    info!("   - 平均响应时间: {:.2}ms", final_stats.avg_response_time_ms);

    let final_alerts = monitor.get_recent_alerts(20).await;
    info!("🚨 总警报数: {}", final_alerts.len());

    // 8. 性能优化建议
    println!("\n8️⃣ 性能优化建议");
    println!("----------------");

    generate_optimization_suggestions(&final_stats, &final_alerts);

    println!("\n🎉 性能监控示例完成!");
    println!("\n📚 下一步学习:");
    println!("   - examples/04_production/deployment.rs - 部署指南");
    println!("   - docs/best-practices/performance.md - 性能优化最佳实践");

    Ok(())
}

/// 获取内存使用量（模拟）
fn get_memory_usage() -> u64 {
    // 在实际实现中，这里会获取真实的内存使用量
    use std::process;
    let pid = process::id();
    
    // 模拟内存使用量
    (pid as u64 % 100) * 1024 * 1024 // 0-99MB
}

/// 获取CPU使用率（模拟）
fn get_cpu_usage() -> f64 {
    // 在实际实现中，这里会获取真实的CPU使用率
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    
    // 模拟CPU使用率
    (now.as_millis() % 100) as f64
}

/// 生成性能优化建议
fn generate_optimization_suggestions(stats: &PerformanceStats, alerts: &[PerformanceAlert]) {
    info!("💡 性能优化建议:");

    if stats.avg_response_time_ms > 1000.0 {
        info!("   - 响应时间较长，建议优化Agent逻辑或增加缓存");
    }

    if stats.avg_memory_usage_mb > 50.0 {
        info!("   - 内存使用较高，建议优化内存管理或增加内存限制");
    }

    if stats.success_rate < 0.95 {
        info!("   - 成功率较低，建议检查错误处理和重试机制");
    }

    let critical_alerts = alerts.iter()
        .filter(|a| matches!(a.severity, AlertSeverity::Critical))
        .count();
    
    if critical_alerts > 0 {
        info!("   - 发现{}个严重警报，需要立即处理", critical_alerts);
    }

    if stats.total_requests > 100 {
        info!("   - 请求量较大，建议考虑负载均衡和水平扩展");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new();
        
        let metric = PerformanceMetrics {
            agent_id: "test_agent".to_string(),
            operation: "test".to_string(),
            duration: Duration::from_millis(100),
            memory_usage: 1024 * 1024, // 1MB
            cpu_usage: 10.0,
            success: true,
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        monitor.record_metric(metric).await;
        
        let stats = monitor.get_performance_stats(Some("test_agent")).await;
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.successful_requests, 1);
        assert_eq!(stats.success_rate, 1.0);
    }

    #[tokio::test]
    async fn test_monitored_agent() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Test response".to_string()]));
        let monitor = Arc::new(PerformanceMonitor::new());
        
        let base_agent = quick_agent("test", "Test agent")
            .model(llm)
            .build()
            .unwrap();
        
        let monitored_agent = MonitoredAgent::new(Box::new(base_agent), monitor.clone());
        
        let response = monitored_agent.generate("test query").await;
        assert!(response.is_ok());
        
        let stats = monitor.get_performance_stats(Some("test")).await;
        assert_eq!(stats.total_requests, 1);
    }
}
