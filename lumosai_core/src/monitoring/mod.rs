use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use serde::{Serialize, Deserialize};
use crate::error::{Error, Result};

/// 监控指标类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Timer,
}

/// 监控指标值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
    Timer(Duration),
}

/// 监控指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub metric_type: MetricType,
    pub value: MetricValue,
    pub labels: HashMap<String, String>,
    pub timestamp: u64,
}

impl Metric {
    pub fn new(name: String, metric_type: MetricType, value: MetricValue) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        Self {
            name,
            metric_type,
            value,
            labels: HashMap::new(),
            timestamp,
        }
    }
    
    pub fn with_label(mut self, key: String, value: String) -> Self {
        self.labels.insert(key, value);
        self
    }
    
    pub fn with_labels(mut self, labels: HashMap<String, String>) -> Self {
        self.labels.extend(labels);
        self
    }
}

/// 基础监控收集器
pub struct MetricsCollector {
    metrics: Arc<Mutex<Vec<Metric>>>,
    counters: Arc<Mutex<HashMap<String, u64>>>,
    gauges: Arc<Mutex<HashMap<String, f64>>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(Vec::new())),
            counters: Arc::new(Mutex::new(HashMap::new())),
            gauges: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// 增加计数器
    pub fn increment_counter(&self, name: &str, labels: Option<HashMap<String, String>>) -> Result<()> {
        let mut counters = self.counters.lock()
            .map_err(|e| Error::Lock(format!("Failed to lock counters: {}", e)))?;

        let labels_ref = labels.as_ref().map(|l| l.clone()).unwrap_or_default();
        let key = self.build_metric_key(name, &labels_ref);
        let count = counters.entry(key.clone()).or_insert(0);
        *count += 1;

        let metric = Metric::new(
            name.to_string(),
            MetricType::Counter,
            MetricValue::Counter(*count),
        ).with_labels(labels.unwrap_or_default());

        self.record_metric(metric)?;
        Ok(())
    }
    
    /// 设置仪表盘值
    pub fn set_gauge(&self, name: &str, value: f64, labels: Option<HashMap<String, String>>) -> Result<()> {
        let mut gauges = self.gauges.lock()
            .map_err(|e| Error::Lock(format!("Failed to lock gauges: {}", e)))?;

        let labels_ref = labels.as_ref().map(|l| l.clone()).unwrap_or_default();
        let key = self.build_metric_key(name, &labels_ref);
        gauges.insert(key, value);

        let metric = Metric::new(
            name.to_string(),
            MetricType::Gauge,
            MetricValue::Gauge(value),
        ).with_labels(labels.unwrap_or_default());

        self.record_metric(metric)?;
        Ok(())
    }
    
    /// 记录计时器
    pub fn record_timer(&self, name: &str, duration: Duration, labels: Option<HashMap<String, String>>) -> Result<()> {
        let metric = Metric::new(
            name.to_string(),
            MetricType::Timer,
            MetricValue::Timer(duration),
        ).with_labels(labels.unwrap_or_default());
        
        self.record_metric(metric)?;
        Ok(())
    }
    
    /// 记录直方图
    pub fn record_histogram(&self, name: &str, values: Vec<f64>, labels: Option<HashMap<String, String>>) -> Result<()> {
        let metric = Metric::new(
            name.to_string(),
            MetricType::Histogram,
            MetricValue::Histogram(values),
        ).with_labels(labels.unwrap_or_default());
        
        self.record_metric(metric)?;
        Ok(())
    }
    
    /// 记录指标
    fn record_metric(&self, metric: Metric) -> Result<()> {
        let mut metrics = self.metrics.lock()
            .map_err(|e| Error::Lock(format!("Failed to lock metrics: {}", e)))?;
        
        metrics.push(metric);

        // 保持最近1000个指标
        if metrics.len() > 1000 {
            let excess = metrics.len() - 1000;
            metrics.drain(0..excess);
        }
        
        Ok(())
    }
    
    /// 获取所有指标
    pub fn get_metrics(&self) -> Result<Vec<Metric>> {
        let metrics = self.metrics.lock()
            .map_err(|e| Error::Lock(format!("Failed to lock metrics: {}", e)))?;
        
        Ok(metrics.clone())
    }
    
    /// 获取指定时间范围内的指标
    pub fn get_metrics_in_range(&self, start: u64, end: u64) -> Result<Vec<Metric>> {
        let metrics = self.metrics.lock()
            .map_err(|e| Error::Lock(format!("Failed to lock metrics: {}", e)))?;
        
        let filtered: Vec<Metric> = metrics
            .iter()
            .filter(|m| m.timestamp >= start && m.timestamp <= end)
            .cloned()
            .collect();
        
        Ok(filtered)
    }
    
    /// 清除所有指标
    pub fn clear_metrics(&self) -> Result<()> {
        let mut metrics = self.metrics.lock()
            .map_err(|e| Error::Lock(format!("Failed to lock metrics: {}", e)))?;
        
        let mut counters = self.counters.lock()
            .map_err(|e| Error::Lock(format!("Failed to lock counters: {}", e)))?;
        
        let mut gauges = self.gauges.lock()
            .map_err(|e| Error::Lock(format!("Failed to lock gauges: {}", e)))?;
        
        metrics.clear();
        counters.clear();
        gauges.clear();
        
        Ok(())
    }
    
    /// 构建指标键
    fn build_metric_key(&self, name: &str, labels: &HashMap<String, String>) -> String {
        if labels.is_empty() {
            name.to_string()
        } else {
            let mut key = name.to_string();
            let mut sorted_labels: Vec<_> = labels.iter().collect();
            sorted_labels.sort_by_key(|(k, _)| *k);
            
            for (k, v) in sorted_labels {
                key.push_str(&format!("{}={}", k, v));
            }
            key
        }
    }
    
    /// 获取监控统计信息
    pub fn get_stats(&self) -> Result<MonitoringStats> {
        let metrics = self.metrics.lock()
            .map_err(|e| Error::Lock(format!("Failed to lock metrics: {}", e)))?;
        
        let counters = self.counters.lock()
            .map_err(|e| Error::Lock(format!("Failed to lock counters: {}", e)))?;
        
        let gauges = self.gauges.lock()
            .map_err(|e| Error::Lock(format!("Failed to lock gauges: {}", e)))?;
        
        Ok(MonitoringStats {
            total_metrics: metrics.len(),
            total_counters: counters.len(),
            total_gauges: gauges.len(),
            oldest_metric_timestamp: metrics.first().map(|m| m.timestamp),
            newest_metric_timestamp: metrics.last().map(|m| m.timestamp),
        })
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// 监控统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringStats {
    pub total_metrics: usize,
    pub total_counters: usize,
    pub total_gauges: usize,
    pub oldest_metric_timestamp: Option<u64>,
    pub newest_metric_timestamp: Option<u64>,
}

/// Agent监控器
pub struct AgentMonitor {
    collector: MetricsCollector,
    agent_name: String,
}

impl AgentMonitor {
    pub fn new(agent_name: String) -> Self {
        Self {
            collector: MetricsCollector::new(),
            agent_name,
        }
    }
    
    /// 记录Agent生成请求
    pub fn record_generation_request(&self) -> Result<()> {
        let labels = HashMap::from([
            ("agent".to_string(), self.agent_name.clone()),
            ("operation".to_string(), "generation".to_string()),
        ]);
        
        self.collector.increment_counter("agent_requests_total", Some(labels))
    }
    
    /// 记录Agent生成延迟
    pub fn record_generation_latency(&self, duration: Duration) -> Result<()> {
        let labels = HashMap::from([
            ("agent".to_string(), self.agent_name.clone()),
            ("operation".to_string(), "generation".to_string()),
        ]);
        
        self.collector.record_timer("agent_generation_duration", duration, Some(labels))
    }
    
    /// 记录工具调用
    pub fn record_tool_call(&self, tool_name: &str) -> Result<()> {
        let labels = HashMap::from([
            ("agent".to_string(), self.agent_name.clone()),
            ("tool".to_string(), tool_name.to_string()),
        ]);
        
        self.collector.increment_counter("agent_tool_calls_total", Some(labels))
    }
    
    /// 记录错误
    pub fn record_error(&self, error_type: &str) -> Result<()> {
        let labels = HashMap::from([
            ("agent".to_string(), self.agent_name.clone()),
            ("error_type".to_string(), error_type.to_string()),
        ]);
        
        self.collector.increment_counter("agent_errors_total", Some(labels))
    }
    
    /// 设置活跃连接数
    pub fn set_active_connections(&self, count: f64) -> Result<()> {
        let labels = HashMap::from([
            ("agent".to_string(), self.agent_name.clone()),
        ]);
        
        self.collector.set_gauge("agent_active_connections", count, Some(labels))
    }
    
    /// 获取收集器
    pub fn collector(&self) -> &MetricsCollector {
        &self.collector
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new();
        
        // 测试计数器
        collector.increment_counter("test_counter", None).unwrap();
        collector.increment_counter("test_counter", None).unwrap();
        
        // 测试仪表盘
        collector.set_gauge("test_gauge", 42.0, None).unwrap();
        
        // 测试计时器
        collector.record_timer("test_timer", Duration::from_millis(100), None).unwrap();
        
        let metrics = collector.get_metrics().unwrap();
        assert_eq!(metrics.len(), 4); // 2 counter + 1 gauge + 1 timer
    }
    
    #[test]
    fn test_agent_monitor() {
        let monitor = AgentMonitor::new("test-agent".to_string());
        
        monitor.record_generation_request().unwrap();
        monitor.record_generation_latency(Duration::from_millis(500)).unwrap();
        monitor.record_tool_call("calculator").unwrap();
        monitor.record_error("timeout").unwrap();
        monitor.set_active_connections(5.0).unwrap();
        
        let metrics = monitor.collector().get_metrics().unwrap();
        assert_eq!(metrics.len(), 5);
    }
}
