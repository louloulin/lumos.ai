use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use crate::error::{Error, Result};

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
        let metrics = self.metrics.lock()
            .map_err(|e| Error::Lock(format!("Failed to lock metrics: {}", e)))?;
        Ok(metrics.clone())
    }
    
    /// 重置性能指标
    pub fn reset_metrics(&self) -> Result<()> {
        let mut metrics = self.metrics.lock()
            .map_err(|e| Error::Lock(format!("Failed to lock metrics: {}", e)))?;
        
        let mut response_times = self.response_times.lock()
            .map_err(|e| Error::Lock(format!("Failed to lock response times: {}", e)))?;
        
        *metrics = PerformanceMetrics::default();
        response_times.clear();
        
        Ok(())
    }
    
    /// 更新内存使用量
    pub fn update_memory_usage(&self, memory_bytes: u64) -> Result<()> {
        let mut metrics = self.metrics.lock()
            .map_err(|e| Error::Lock(format!("Failed to lock metrics: {}", e)))?;
        
        metrics.memory_usage = memory_bytes;
        metrics.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Ok(())
    }
    
    /// 更新CPU使用率
    pub fn update_cpu_usage(&self, cpu_percent: f64) -> Result<()> {
        let mut metrics = self.metrics.lock()
            .map_err(|e| Error::Lock(format!("Failed to lock metrics: {}", e)))?;
        
        metrics.cpu_usage = cpu_percent;
        metrics.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Ok(())
    }
    
    /// 更新缓存命中率
    pub fn update_cache_hit_rate(&self, hit_rate: f64) -> Result<()> {
        let mut metrics = self.metrics.lock()
            .map_err(|e| Error::Lock(format!("Failed to lock metrics: {}", e)))?;
        
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
                description: format!("Average response time is {:.2}ms", metrics.avg_response_time),
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
                description: format!("Error rate is {:.1}%, which is very high", error_rate),
                suggestion: "Investigate error causes, improve error handling, and add retry mechanisms".to_string(),
                estimated_improvement: "Significant reliability improvement".to_string(),
            });
        } else if error_rate > 5.0 {
            recommendations.push(PerformanceRecommendation {
                category: "Error Rate".to_string(),
                severity: "medium".to_string(),
                description: format!("Error rate is {:.1}%", error_rate),
                suggestion: "Review error logs and implement better error handling".to_string(),
                estimated_improvement: "Improved reliability".to_string(),
            });
        }
        
        // 分析内存使用
        if metrics.memory_usage > 1_000_000_000 { // 1GB
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
                description: format!("Cache hit rate is {:.1}%, which is low", metrics.cache_hit_rate),
                suggestion: "Optimize caching strategy, increase cache size, or improve cache key design".to_string(),
                estimated_improvement: "10-30% performance improvement".to_string(),
            });
        }
        
        recommendations
    }
}
