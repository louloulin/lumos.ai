use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// 性能指标类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// 平均响应时间（毫秒）
    pub avg_response_time: f64,
    /// 最小响应时间（毫秒）
    pub min_response_time: f64,
    /// 最大响应时间（毫秒）
    pub max_response_time: f64,
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
    /// 当前并发数
    pub current_concurrency: u32,
    /// 最大并发数
    pub max_concurrency: u32,
    /// 内存使用量（字节）
    pub memory_usage: u64,
    /// CPU使用率（百分比）
    pub cpu_usage: f64,
    /// 缓存命中率（百分比）
    pub cache_hit_rate: f64,
    /// 最后更新时间
    pub last_updated: u64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            avg_response_time: 0.0,
            min_response_time: f64::MAX,
            max_response_time: 0.0,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            current_concurrency: 0,
            max_concurrency: 0,
            memory_usage: 0,
            cpu_usage: 0.0,
            cache_hit_rate: 0.0,
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }
}

/// 性能监控器
pub struct PerformanceMonitor {
    metrics: Arc<Mutex<PerformanceMetrics>>,
    response_times: Arc<Mutex<Vec<f64>>>,
    max_history_size: usize,
}

impl PerformanceMonitor {
    /// 创建新的性能监控器
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(PerformanceMetrics::default())),
            response_times: Arc::new(Mutex::new(Vec::new())),
            max_history_size: 1000, // 保留最近1000次请求的响应时间
        }
    }

    /// 记录请求开始
    pub fn start_request(&self) -> RequestTimer {
        // 增加当前并发数
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.current_concurrency += 1;
            if metrics.current_concurrency > metrics.max_concurrency {
                metrics.max_concurrency = metrics.current_concurrency;
            }
        }

        RequestTimer::new(self.metrics.clone(), self.response_times.clone())
    }

    /// 获取当前性能指标
    pub fn get_metrics(&self) -> Result<PerformanceMetrics> {
        let metrics = self
            .metrics
            .lock()
            .map_err(|e| Error::Lock(format!("Failed to lock metrics: {e}")))?;
        Ok(metrics.clone())
    }

    /// 重置性能指标
    pub fn reset_metrics(&self) -> Result<()> {
        let mut metrics = self
            .metrics
            .lock()
            .map_err(|e| Error::Lock(format!("Failed to lock metrics: {e}")))?;

        let mut response_times = self
            .response_times
            .lock()
            .map_err(|e| Error::Lock(format!("Failed to lock response times: {e}")))?;

        *metrics = PerformanceMetrics::default();
        response_times.clear();

        Ok(())
    }

    /// 更新内存使用量
    pub fn update_memory_usage(&self, memory_bytes: u64) -> Result<()> {
        let mut metrics = self
            .metrics
            .lock()
            .map_err(|e| Error::Lock(format!("Failed to lock metrics: {e}")))?;

        metrics.memory_usage = memory_bytes;
        metrics.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Ok(())
    }

    /// 更新CPU使用率
    pub fn update_cpu_usage(&self, cpu_percent: f64) -> Result<()> {
        let mut metrics = self
            .metrics
            .lock()
            .map_err(|e| Error::Lock(format!("Failed to lock metrics: {e}")))?;

        metrics.cpu_usage = cpu_percent;
        metrics.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Ok(())
    }

    /// 更新缓存命中率
    pub fn update_cache_hit_rate(&self, hit_rate: f64) -> Result<()> {
        let mut metrics = self
            .metrics
            .lock()
            .map_err(|e| Error::Lock(format!("Failed to lock metrics: {e}")))?;

        metrics.cache_hit_rate = hit_rate;
        metrics.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Ok(())
    }

    /// 计算平均响应时间
    fn calculate_avg_response_time(&self, response_times: &[f64]) -> f64 {
        if response_times.is_empty() {
            0.0
        } else {
            response_times.iter().sum::<f64>() / response_times.len() as f64
        }
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// 请求计时器
pub struct RequestTimer {
    start_time: Instant,
    metrics: Arc<Mutex<PerformanceMetrics>>,
    response_times: Arc<Mutex<Vec<f64>>>,
    finished: bool,
}

impl RequestTimer {
    fn new(metrics: Arc<Mutex<PerformanceMetrics>>, response_times: Arc<Mutex<Vec<f64>>>) -> Self {
        Self {
            start_time: Instant::now(),
            metrics,
            response_times,
            finished: false,
        }
    }

    /// 完成请求并记录成功
    pub fn finish_success(mut self) {
        self.finished = true;
        self.finish_internal(true);
    }

    /// 完成请求并记录失败
    pub fn finish_error(mut self) {
        self.finished = true;
        self.finish_internal(false);
    }

    fn finish_internal(self, success: bool) {
        let duration = self.start_time.elapsed();
        let duration_ms = duration.as_secs_f64() * 1000.0;

        // 更新指标
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.total_requests += 1;
            if success {
                metrics.successful_requests += 1;
            } else {
                metrics.failed_requests += 1;
            }

            // 减少当前并发数
            if metrics.current_concurrency > 0 {
                metrics.current_concurrency -= 1;
            }

            // 更新响应时间统计
            if duration_ms < metrics.min_response_time {
                metrics.min_response_time = duration_ms;
            }
            if duration_ms > metrics.max_response_time {
                metrics.max_response_time = duration_ms;
            }

            metrics.last_updated = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
        }

        // 更新响应时间历史
        if let Ok(mut response_times) = self.response_times.lock() {
            response_times.push(duration_ms);

            // 保持历史记录大小限制
            if response_times.len() > 1000 {
                let excess = response_times.len() - 1000;
                response_times.drain(0..excess);
            }

            // 重新计算平均响应时间
            if let Ok(mut metrics) = self.metrics.lock() {
                metrics.avg_response_time = if response_times.is_empty() {
                    0.0
                } else {
                    response_times.iter().sum::<f64>() / response_times.len() as f64
                };
            }
        }
    }
}

impl Drop for RequestTimer {
    fn drop(&mut self) {
        // 只有在用户忘记调用finish_*方法时才自动记录
        if !self.finished {
            let duration = self.start_time.elapsed();
            let duration_ms = duration.as_secs_f64() * 1000.0;

            if let Ok(mut metrics) = self.metrics.lock() {
                metrics.total_requests += 1;
                metrics.successful_requests += 1;

                if metrics.current_concurrency > 0 {
                    metrics.current_concurrency -= 1;
                }

                if duration_ms < metrics.min_response_time {
                    metrics.min_response_time = duration_ms;
                }
                if duration_ms > metrics.max_response_time {
                    metrics.max_response_time = duration_ms;
                }

                metrics.last_updated = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
            }
        }
    }
}

/// 性能优化建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecommendation {
    pub category: String,
    pub severity: String, // "low", "medium", "high", "critical"
    pub description: String,
    pub suggestion: String,
    pub estimated_improvement: String,
}

/// 性能分析器
pub struct PerformanceAnalyzer;

impl PerformanceAnalyzer {
    /// 分析性能指标并提供优化建议
    pub fn analyze(metrics: &PerformanceMetrics) -> Vec<PerformanceRecommendation> {
        let mut recommendations = Vec::new();

        // 分析响应时间
        if metrics.avg_response_time > 5000.0 {
            recommendations.push(PerformanceRecommendation {
                category: "Response Time".to_string(),
                severity: "high".to_string(),
                description: format!("Average response time is {:.2}ms, which is quite high", metrics.avg_response_time),
                suggestion: "Consider optimizing LLM calls, implementing caching, or reducing model complexity".to_string(),
                estimated_improvement: "30-50% response time reduction".to_string(),
            });
        } else if metrics.avg_response_time > 2000.0 {
            recommendations.push(PerformanceRecommendation {
                category: "Response Time".to_string(),
                severity: "medium".to_string(),
                description: format!(
                    "Average response time is {:.2}ms",
                    metrics.avg_response_time
                ),
                suggestion: "Consider implementing response caching for common queries".to_string(),
                estimated_improvement: "15-25% response time reduction".to_string(),
            });
        }

        // 分析错误率
        let error_rate = if metrics.total_requests > 0 {
            (metrics.failed_requests as f64 / metrics.total_requests as f64) * 100.0
        } else {
            0.0
        };

        if error_rate > 10.0 {
            recommendations.push(PerformanceRecommendation {
                category: "Error Rate".to_string(),
                severity: "critical".to_string(),
                description: format!("Error rate is {error_rate:.1}%, which is very high"),
                suggestion:
                    "Investigate error causes, improve error handling, and add retry mechanisms"
                        .to_string(),
                estimated_improvement: "Significant reliability improvement".to_string(),
            });
        } else if error_rate > 5.0 {
            recommendations.push(PerformanceRecommendation {
                category: "Error Rate".to_string(),
                severity: "medium".to_string(),
                description: format!("Error rate is {error_rate:.1}%"),
                suggestion: "Review error logs and implement better error handling".to_string(),
                estimated_improvement: "Improved reliability".to_string(),
            });
        }

        // 分析内存使用
        if metrics.memory_usage > 1_000_000_000 {
            // 1GB
            recommendations.push(PerformanceRecommendation {
                category: "Memory Usage".to_string(),
                severity: "high".to_string(),
                description: format!("Memory usage is {:.2}MB", metrics.memory_usage as f64 / 1_000_000.0),
                suggestion: "Implement memory optimization, clear unused caches, or increase available memory".to_string(),
                estimated_improvement: "20-40% memory reduction".to_string(),
            });
        }

        // 分析缓存命中率
        if metrics.cache_hit_rate < 50.0 && metrics.total_requests > 100 {
            recommendations.push(PerformanceRecommendation {
                category: "Cache Performance".to_string(),
                severity: "medium".to_string(),
                description: format!(
                    "Cache hit rate is {:.1}%, which is low",
                    metrics.cache_hit_rate
                ),
                suggestion:
                    "Optimize caching strategy, increase cache size, or improve cache key design"
                        .to_string(),
                estimated_improvement: "10-30% performance improvement".to_string(),
            });
        }

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new();
        let metrics = monitor.get_metrics().unwrap();
        
        assert_eq!(metrics.total_requests, 0);
        assert_eq!(metrics.successful_requests, 0);
        assert_eq!(metrics.failed_requests, 0);
        assert_eq!(metrics.current_concurrency, 0);
        assert_eq!(metrics.max_concurrency, 0);
    }

    #[test]
    fn test_request_timer_success() {
        let monitor = PerformanceMonitor::new();
        let timer = monitor.start_request();
        
        // 模拟一些工作
        thread::sleep(Duration::from_millis(10));
        
        timer.finish_success();
        
        let metrics = monitor.get_metrics().unwrap();
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 1);
        assert_eq!(metrics.failed_requests, 0);
        assert_eq!(metrics.current_concurrency, 0);
        assert!(metrics.avg_response_time > 0.0);
    }

    #[test]
    fn test_request_timer_error() {
        let monitor = PerformanceMonitor::new();
        let timer = monitor.start_request();
        
        thread::sleep(Duration::from_millis(10));
        
        timer.finish_error();
        
        let metrics = monitor.get_metrics().unwrap();
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 0);
        assert_eq!(metrics.failed_requests, 1);
        assert_eq!(metrics.current_concurrency, 0);
    }

    #[test]
    fn test_concurrent_requests() {
        let monitor = PerformanceMonitor::new();
        
        // 启动多个并发请求
        let timer1 = monitor.start_request();
        let timer2 = monitor.start_request();
        let timer3 = monitor.start_request();
        
        let metrics = monitor.get_metrics().unwrap();
        assert_eq!(metrics.current_concurrency, 3);
        assert_eq!(metrics.max_concurrency, 3);
        
        timer1.finish_success();
        timer2.finish_success();
        timer3.finish_success();
        
        let metrics = monitor.get_metrics().unwrap();
        assert_eq!(metrics.current_concurrency, 0);
        assert_eq!(metrics.max_concurrency, 3);
        assert_eq!(metrics.total_requests, 3);
    }

    #[test]
    fn test_reset_metrics() {
        let monitor = PerformanceMonitor::new();
        
        let timer = monitor.start_request();
        thread::sleep(Duration::from_millis(10));
        timer.finish_success();
        
        let metrics = monitor.get_metrics().unwrap();
        assert_eq!(metrics.total_requests, 1);
        
        monitor.reset_metrics().unwrap();
        
        let metrics = monitor.get_metrics().unwrap();
        assert_eq!(metrics.total_requests, 0);
        assert_eq!(metrics.successful_requests, 0);
        assert_eq!(metrics.failed_requests, 0);
    }

    #[test]
    fn test_update_memory_usage() {
        let monitor = PerformanceMonitor::new();
        
        monitor.update_memory_usage(1024 * 1024 * 512).unwrap(); // 512MB
        
        let metrics = monitor.get_metrics().unwrap();
        assert_eq!(metrics.memory_usage, 1024 * 1024 * 512);
    }

    #[test]
    fn test_update_cpu_usage() {
        let monitor = PerformanceMonitor::new();
        
        monitor.update_cpu_usage(75.5).unwrap();
        
        let metrics = monitor.get_metrics().unwrap();
        assert_eq!(metrics.cpu_usage, 75.5);
    }

    #[test]
    fn test_update_cache_hit_rate() {
        let monitor = PerformanceMonitor::new();
        
        monitor.update_cache_hit_rate(85.0).unwrap();
        
        let metrics = monitor.get_metrics().unwrap();
        assert_eq!(metrics.cache_hit_rate, 85.0);
    }

    #[test]
    fn test_performance_analyzer_high_response_time() {
        let mut metrics = PerformanceMetrics::default();
        metrics.avg_response_time = 6000.0; // 6秒，应该触发高严重性建议
        
        let recommendations = PerformanceAnalyzer::analyze(&metrics);
        
        assert!(!recommendations.is_empty());
        let response_time_rec = recommendations.iter()
            .find(|r| r.category == "Response Time")
            .unwrap();
        assert_eq!(response_time_rec.severity, "high");
    }

    #[test]
    fn test_performance_analyzer_high_error_rate() {
        let mut metrics = PerformanceMetrics::default();
        metrics.total_requests = 100;
        metrics.failed_requests = 15; // 15% 错误率，应该触发中等严重性建议
        
        let recommendations = PerformanceAnalyzer::analyze(&metrics);
        
        assert!(!recommendations.is_empty());
        let error_rate_rec = recommendations.iter()
            .find(|r| r.category == "Error Rate")
            .unwrap();
        assert_eq!(error_rate_rec.severity, "critical");
    }

    #[test]
    fn test_performance_analyzer_high_memory_usage() {
        let mut metrics = PerformanceMetrics::default();
        metrics.memory_usage = 2_000_000_000; // 2GB，应该触发高严重性建议
        
        let recommendations = PerformanceAnalyzer::analyze(&metrics);
        
        assert!(!recommendations.is_empty());
        let memory_rec = recommendations.iter()
            .find(|r| r.category == "Memory Usage")
            .unwrap();
        assert_eq!(memory_rec.severity, "high");
    }

    #[test]
    fn test_performance_analyzer_low_cache_hit_rate() {
        let mut metrics = PerformanceMetrics::default();
        metrics.total_requests = 200;
        metrics.cache_hit_rate = 30.0; // 30% 缓存命中率，应该触发中等严重性建议
        
        let recommendations = PerformanceAnalyzer::analyze(&metrics);
        
        assert!(!recommendations.is_empty());
        let cache_rec = recommendations.iter()
            .find(|r| r.category == "Cache Performance")
            .unwrap();
        assert_eq!(cache_rec.severity, "medium");
    }

    #[test]
    fn test_performance_analyzer_good_metrics() {
        let mut metrics = PerformanceMetrics::default();
        metrics.avg_response_time = 500.0; // 500ms，正常
        metrics.total_requests = 100;
        metrics.failed_requests = 1; // 1% 错误率，正常
        metrics.memory_usage = 100_000_000; // 100MB，正常
        metrics.cache_hit_rate = 80.0; // 80% 缓存命中率，正常
        
        let recommendations = PerformanceAnalyzer::analyze(&metrics);
        
        // 所有指标都正常，不应该有建议
        assert!(recommendations.is_empty());
    }

    #[test]
    fn test_min_max_response_time_tracking() {
        let monitor = PerformanceMonitor::new();
        
        // 第一个请求：10ms
        let timer1 = monitor.start_request();
        thread::sleep(Duration::from_millis(10));
        timer1.finish_success();
        
        // 第二个请求：50ms
        let timer2 = monitor.start_request();
        thread::sleep(Duration::from_millis(50));
        timer2.finish_success();
        
        let metrics = monitor.get_metrics().unwrap();
        assert!(metrics.min_response_time >= 10.0);
        assert!(metrics.max_response_time >= 50.0);
        assert!(metrics.min_response_time < metrics.max_response_time);
    }
}
