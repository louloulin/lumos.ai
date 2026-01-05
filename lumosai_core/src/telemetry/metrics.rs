//! Prometheus Metrics 集成
//!
//! 提供与 Prometheus 兼容的指标收集功能
//!
//! # 示例
//!
//! ```rust,no_run
//! use lumosai_core::telemetry::metrics::{PrometheusMetrics, MetricsConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let metrics = PrometheusMetrics::new(MetricsConfig::default());
//!
//!     // 记录指标
//!     metrics.record_latency("agent_generate", 100.0)?;
//!     metrics.increment_counter("total_requests")?;
//!
//!     // 导出指标
//!     let output = metrics.export()?;
//!     println!("{}", output);
//!
//!     Ok(())
//! }
//! ```

use prometheus::{
    Counter, CounterVec, Encoder, Gauge, GaugeVec, Histogram, HistogramOpts, HistogramVec, Registry,
    TextEncoder, Opts,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Metrics 配置
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// 服务名称（作为指标前缀）
    pub service_name: String,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            service_name: "lumosai".to_string(),
        }
    }
}

/// Prometheus Metrics 收集器
pub struct PrometheusMetrics {
    registry: Arc<Registry>,
    counters: Arc<Mutex<HashMap<String, Counter>>>,
    gauges: Arc<Mutex<HashMap<String, Gauge>>>,
    histograms: Arc<Mutex<HashMap<String, Histogram>>>,
    counter_vecs: Arc<Mutex<HashMap<String, CounterVec>>>,
    histogram_vecs: Arc<Mutex<HashMap<String, HistogramVec>>>,
}

impl PrometheusMetrics {
    /// 创建新的 Metrics 收集器
    pub fn new(config: MetricsConfig) -> Self {
        let registry = Arc::new(
            Registry::new_custom(Some(config.service_name.clone()), None).unwrap()
        );

        Self {
            registry,
            counters: Arc::new(Mutex::new(HashMap::new())),
            gauges: Arc::new(Mutex::new(HashMap::new())),
            histograms: Arc::new(Mutex::new(HashMap::new())),
            counter_vecs: Arc::new(Mutex::new(HashMap::new())),
            histogram_vecs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 注册 Counter
    pub fn register_counter(&self, name: &str, help: &str) -> Result<(), String> {
        let counter = Counter::with_opts(Opts::new(name, help))
            .map_err(|e| format!("Failed to create counter {}: {}", name, e))?;

        self.registry
            .register(Box::new(counter.clone()))
            .map_err(|e| format!("Failed to register counter {}: {}", name, e))?;

        self.counters.lock().unwrap().insert(name.to_string(), counter);
        Ok(())
    }

    /// 注册 CounterVec (带标签)
    pub fn register_counter_vec(
        &self,
        name: &str,
        help: &str,
        labels: &[&str],
    ) -> Result<(), String> {
        let counter_vec = CounterVec::new(Opts::new(name, help), labels)
            .map_err(|e| format!("Failed to create counter vec {}: {}", name, e))?;

        self.registry
            .register(Box::new(counter_vec.clone()))
            .map_err(|e| format!("Failed to register counter vec {}: {}", name, e))?;

        self.counter_vecs.lock().unwrap().insert(name.to_string(), counter_vec);
        Ok(())
    }

    /// 注册 Gauge
    pub fn register_gauge(&self, name: &str, help: &str) -> Result<(), String> {
        let gauge = Gauge::with_opts(Opts::new(name, help))
            .map_err(|e| format!("Failed to create gauge {}: {}", name, e))?;

        self.registry
            .register(Box::new(gauge.clone()))
            .map_err(|e| format!("Failed to register gauge {}: {}", name, e))?;

        self.gauges.lock().unwrap().insert(name.to_string(), gauge);
        Ok(())
    }

    /// 注册 Histogram
    pub fn register_histogram(&self, name: &str, help: &str) -> Result<(), String> {
        let histogram = Histogram::with_opts(
            HistogramOpts::new(name, help)
                .buckets(vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
        )
        .map_err(|e| format!("Failed to create histogram {}: {}", name, e))?;

        self.registry
            .register(Box::new(histogram.clone()))
            .map_err(|e| format!("Failed to register histogram {}: {}", name, e))?;

        self.histograms.lock().unwrap().insert(name.to_string(), histogram);
        Ok(())
    }

    /// 注册 HistogramVec (带标签)
    pub fn register_histogram_vec(
        &self,
        name: &str,
        help: &str,
        labels: &[&str],
    ) -> Result<(), String> {
        let histogram_vec = HistogramVec::new(
            HistogramOpts::new(name, help)
                .buckets(vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
            labels,
        )
        .map_err(|e| format!("Failed to create histogram vec {}: {}", name, e))?;

        self.registry
            .register(Box::new(histogram_vec.clone()))
            .map_err(|e| format!("Failed to register histogram vec {}: {}", name, e))?;

        self.histogram_vecs.lock().unwrap().insert(name.to_string(), histogram_vec);
        Ok(())
    }

    /// 增加 Counter
    pub fn increment_counter(&self, name: &str) -> Result<(), String> {
        let counters = self.counters.lock().unwrap();
        let counter = counters.get(name).ok_or_else(|| format!("Counter not found: {}", name))?;
        counter.inc();
        Ok(())
    }

    /// Counter 增加指定值
    pub fn increment_counter_by(&self, name: &str, value: f64) -> Result<(), String> {
        let counters = self.counters.lock().unwrap();
        let counter = counters.get(name).ok_or_else(|| format!("Counter not found: {}", name))?;
        counter.inc_by(value);
        Ok(())
    }

    /// 设置 Gauge 值
    pub fn set_gauge(&self, name: &str, value: f64) -> Result<(), String> {
        let gauges = self.gauges.lock().unwrap();
        let gauge = gauges.get(name).ok_or_else(|| format!("Gauge not found: {}", name))?;
        gauge.set(value);
        Ok(())
    }

    /// 记录延迟 (Histogram)
    pub fn record_latency(&self, name: &str, value: f64) -> Result<(), String> {
        let histograms = self.histograms.lock().unwrap();
        let histogram = histograms.get(name).ok_or_else(|| format!("Histogram not found: {}", name))?;
        histogram.observe(value);
        Ok(())
    }

    /// CounterVec 增加指定标签的值
    pub fn increment_counter_vec(&self, name: &str, labels: &[&str]) -> Result<(), String> {
        let counter_vecs = self.counter_vecs.lock().unwrap();
        let counter_vec = counter_vecs.get(name).ok_or_else(|| format!("CounterVec not found: {}", name))?;
        counter_vec.with_label_values(labels).inc();
        Ok(())
    }

    /// HistogramVec 记录指定标签的值
    pub fn record_histogram_vec(&self, name: &str, value: f64, labels: &[&str]) -> Result<(), String> {
        let histogram_vecs = self.histogram_vecs.lock().unwrap();
        let histogram_vec = histogram_vecs.get(name).ok_or_else(|| format!("HistogramVec not found: {}", name))?;
        histogram_vec.with_label_values(labels).observe(value);
        Ok(())
    }

    /// 收集所有指标
    pub fn gather(&self) -> Vec<prometheus::proto::MetricFamily> {
        self.registry.gather()
    }

    /// 导出为 Prometheus 文本格式
    pub fn export(&self) -> Result<String, String> {
        let metric_families = self.gather();
        let encoder = TextEncoder::new();
        let mut buffer = Vec::new();

        encoder
            .encode(&metric_families, &mut buffer)
            .map_err(|e| format!("Failed to encode metrics: {}", e))?;

        String::from_utf8(buffer).map_err(|e| format!("Failed to convert to string: {}", e))
    }
}

/// 预定义的标准指标
impl PrometheusMetrics {
    /// 注册标准 Agent 指标
    pub fn register_agent_metrics(&self) -> Result<(), String> {
        // Counter: 总请求数
        self.register_counter("lumosai_agent_requests_total", "Total number of agent requests")?;

        // Histogram: 请求延迟
        self.register_histogram("lumosai_agent_latency_seconds", "Agent request latency in seconds")?;

        // Gauge: 当前活跃 Agent 数
        self.register_gauge("lumosai_agent_active", "Number of active agents")?;

        // Counter: 错误总数
        self.register_counter("lumosai_agent_errors_total", "Total number of agent errors")?;

        Ok(())
    }

    /// 注册 LLM 指标
    pub fn register_llm_metrics(&self) -> Result<(), String> {
        // CounterVec: LLM 请求数 (按提供商)
        self.register_counter_vec(
            "lumosai_llm_requests_total",
            "Total LLM requests",
            &["provider", "model"],
        )?;

        // HistogramVec: LLM 延迟 (按提供商)
        self.register_histogram_vec(
            "lumosai_llm_latency_seconds",
            "LLM request latency",
            &["provider", "model"],
        )?;

        // Gauge: Token 使用数
        self.register_gauge("lumosai_llm_tokens_used", "Total tokens used")?;

        Ok(())
    }

    /// 注册 RAG 指标
    pub fn register_rag_metrics(&self) -> Result<(), String> {
        // Counter: RAG 查询数
        self.register_counter("lumosai_rag_queries_total", "Total RAG queries")?;

        // Histogram: RAG 延迟
        self.register_histogram("lumosai_rag_latency_seconds", "RAG query latency")?;

        // Gauge: 检索到的文档数
        self.register_gauge("lumosai_rag_retrieved_docs", "Number of retrieved documents")?;

        Ok(())
    }

    /// 注册 Vector DB 指标
    pub fn register_vector_db_metrics(&self) -> Result<(), String> {
        // CounterVec: 向量查询数
        self.register_counter_vec(
            "lumosai_vector_db_queries_total",
            "Total vector DB queries",
            &["operation"],
        )?;

        // HistogramVec: 向量查询延迟
        self.register_histogram_vec(
            "lumosai_vector_db_latency_seconds",
            "Vector DB query latency",
            &["operation"],
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_increment_counter() {
        let metrics = PrometheusMetrics::new(MetricsConfig::default());
        metrics.register_counter("test_counter", "A test counter").unwrap();

        metrics.increment_counter("test_counter").unwrap();
        metrics.increment_counter_by("test_counter", 5.0).unwrap();

        let exported = metrics.export().unwrap();
        assert!(exported.contains("test_counter"));
    }

    #[test]
    fn test_register_and_set_gauge() {
        let metrics = PrometheusMetrics::new(MetricsConfig::default());
        metrics.register_gauge("test_gauge", "A test gauge").unwrap();

        metrics.set_gauge("test_gauge", 42.0).unwrap();

        let exported = metrics.export().unwrap();
        assert!(exported.contains("test_gauge 42"));
    }

    #[test]
    fn test_register_and_record_histogram() {
        let metrics = PrometheusMetrics::new(MetricsConfig::default());
        metrics.register_histogram("test_histogram", "A test histogram").unwrap();

        metrics.record_latency("test_histogram", 0.1).unwrap();
        metrics.record_latency("test_histogram", 0.5).unwrap();

        let exported = metrics.export().unwrap();
        assert!(exported.contains("test_histogram"));
    }

    #[test]
    fn test_counter_vec() {
        let metrics = PrometheusMetrics::new(MetricsConfig::default());
        metrics
            .register_counter_vec("test_counter_vec", "A test counter vec", &["label"])
            .unwrap();

        metrics
            .increment_counter_vec("test_counter_vec", &["value1"])
            .unwrap();
        metrics
            .increment_counter_vec("test_counter_vec", &["value2"])
            .unwrap();

        let exported = metrics.export().unwrap();
        assert!(exported.contains("test_counter_vec"));
    }

    #[test]
    fn test_register_standard_metrics() {
        let metrics = PrometheusMetrics::new(MetricsConfig::default());

        metrics.register_agent_metrics().unwrap();
        metrics.register_llm_metrics().unwrap();
        metrics.register_rag_metrics().unwrap();
        metrics.register_vector_db_metrics().unwrap();

        // 验证指标已注册
        let exported = metrics.export().unwrap();
        assert!(exported.contains("lumosai_agent_requests_total"));
        assert!(exported.contains("lumosai_llm_requests_total"));
        assert!(exported.contains("lumosai_rag_queries_total"));
        assert!(exported.contains("lumosai_vector_db_queries_total"));
    }
}
