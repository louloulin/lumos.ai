//! 企业级性能监控系统 - 实时性能分析和优化建议
//!
//! 提供深度性能分析、瓶颈识别、预测性监控和自动化优化建议

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

use super::analyzer::{
    BottleneckType, OptimizationRecommendation, PerformanceAnalysis, PerformanceAnalyzer,
};
use super::metrics::{AgentMetrics, MetricsCollector, MetricsSummary, ToolMetrics};

/// 性能监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMonitorConfig {
    /// 监控间隔（秒）
    pub monitoring_interval_seconds: u64,
    /// 性能数据保留时间（小时）
    pub data_retention_hours: u64,
    /// 性能阈值配置
    pub thresholds: PerformanceThresholds,
    /// 预测配置
    pub prediction_config: PredictionConfig,
    /// 自动优化配置
    pub auto_optimization_config: AutoOptimizationConfig,
}

/// 性能阈值配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    /// 响应时间阈值（毫秒）
    pub response_time_ms: f64,
    /// CPU使用率阈值（百分比）
    pub cpu_usage_percent: f64,
    /// 内存使用率阈值（百分比）
    pub memory_usage_percent: f64,
    /// 错误率阈值（百分比）
    pub error_rate_percent: f64,
    /// 吞吐量阈值（请求/秒）
    pub throughput_rps: f64,
}

/// 预测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionConfig {
    /// 是否启用预测
    pub enabled: bool,
    /// 预测时间窗口（小时）
    pub prediction_window_hours: u64,
    /// 历史数据窗口（小时）
    pub history_window_hours: u64,
    /// 预测精度要求
    pub accuracy_threshold: f64,
}

/// 自动优化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoOptimizationConfig {
    /// 是否启用自动优化
    pub enabled: bool,
    /// 优化策略
    pub strategies: Vec<OptimizationStrategy>,
    /// 优化执行间隔（分钟）
    pub execution_interval_minutes: u64,
}

/// 优化策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationStrategy {
    /// 自动扩容
    AutoScaling,
    /// 缓存优化
    CacheOptimization,
    /// 连接池调优
    ConnectionPoolTuning,
    /// 垃圾回收优化
    GarbageCollectionTuning,
    /// 负载均衡调整
    LoadBalancingAdjustment,
}

/// 实时性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimePerformanceMetrics {
    /// 时间戳
    pub timestamp: u64,
    /// 响应时间统计
    pub response_time: ResponseTimeMetrics,
    /// 吞吐量统计
    pub throughput: ThroughputMetrics,
    /// 资源使用统计
    pub resource_usage: ResourceUsageMetrics,
    /// 错误统计
    pub error_metrics: ErrorMetrics,
    /// 性能趋势
    pub performance_trend: PerformanceTrend,
}

/// 响应时间指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeMetrics {
    /// 平均响应时间（毫秒）
    pub avg_ms: f64,
    /// 中位数响应时间（毫秒）
    pub median_ms: f64,
    /// 95分位数响应时间（毫秒）
    pub p95_ms: f64,
    /// 99分位数响应时间（毫秒）
    pub p99_ms: f64,
    /// 最大响应时间（毫秒）
    pub max_ms: f64,
    /// 最小响应时间（毫秒）
    pub min_ms: f64,
}

/// 吞吐量指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputMetrics {
    /// 每秒请求数
    pub requests_per_second: f64,
    /// 每分钟请求数
    pub requests_per_minute: f64,
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
}

/// 资源使用指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageMetrics {
    /// CPU使用率（百分比）
    pub cpu_usage_percent: f64,
    /// 内存使用率（百分比）
    pub memory_usage_percent: f64,
    /// 磁盘使用率（百分比）
    pub disk_usage_percent: f64,
    /// 网络带宽使用（MB/s）
    pub network_bandwidth_mbps: f64,
    /// 活跃连接数
    pub active_connections: u64,
}

/// 错误指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    /// 错误率（百分比）
    pub error_rate_percent: f64,
    /// 按类型分组的错误数量
    pub errors_by_type: HashMap<String, u64>,
    /// 按严重程度分组的错误数量
    pub errors_by_severity: HashMap<String, u64>,
    /// 最近错误列表
    pub recent_errors: Vec<ErrorEvent>,
}

/// 错误事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    /// 错误ID
    pub id: String,
    /// 错误类型
    pub error_type: String,
    /// 错误消息
    pub message: String,
    /// 发生时间
    pub timestamp: u64,
    /// 严重程度
    pub severity: String,
    /// 相关上下文
    pub context: HashMap<String, String>,
}

/// 性能趋势
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceTrend {
    /// 性能改善
    Improving,
    /// 性能稳定
    Stable,
    /// 性能下降
    Degrading,
    /// 性能波动
    Fluctuating,
}

/// 性能预测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePrediction {
    /// 预测时间戳
    pub prediction_timestamp: u64,
    /// 预测的响应时间
    pub predicted_response_time_ms: f64,
    /// 预测的吞吐量
    pub predicted_throughput_rps: f64,
    /// 预测的资源使用率
    pub predicted_resource_usage: ResourceUsageMetrics,
    /// 预测置信度
    pub confidence: f64,
    /// 预测基于的历史数据点数量
    pub data_points_used: usize,
}

/// 性能优化建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceOptimizationSuggestion {
    /// 建议ID
    pub id: String,
    /// 建议标题
    pub title: String,
    /// 建议描述
    pub description: String,
    /// 优化类型
    pub optimization_type: OptimizationStrategy,
    /// 预期改善程度
    pub expected_improvement: f64,
    /// 实施难度
    pub implementation_difficulty: DifficultyLevel,
    /// 实施步骤
    pub implementation_steps: Vec<String>,
    /// 风险评估
    pub risk_assessment: RiskLevel,
}

/// 难度级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DifficultyLevel {
    /// 简单
    Easy,
    /// 中等
    Medium,
    /// 困难
    Hard,
    /// 专家级
    Expert,
}

/// 风险级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    /// 低风险
    Low,
    /// 中等风险
    Medium,
    /// 高风险
    High,
    /// 极高风险
    Critical,
}

/// 企业级性能监控器
pub struct EnterprisePerformanceMonitor {
    /// 配置
    config: PerformanceMonitorConfig,
    /// 指标收集器
    metrics_collector: Arc<dyn MetricsCollector>,
    /// 性能分析器
    performance_analyzer: Arc<dyn PerformanceAnalyzer>,
    /// 历史性能数据
    performance_history: Arc<RwLock<VecDeque<RealTimePerformanceMetrics>>>,
    /// 性能预测缓存
    prediction_cache: Arc<RwLock<Option<PerformancePrediction>>>,
    /// 优化建议缓存
    optimization_suggestions: Arc<RwLock<Vec<PerformanceOptimizationSuggestion>>>,
    /// 监控统计
    monitoring_stats: Arc<RwLock<MonitoringStatistics>>,
}

/// 监控统计信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonitoringStatistics {
    /// 监控运行时间（秒）
    pub uptime_seconds: u64,
    /// 收集的数据点总数
    pub total_data_points: u64,
    /// 生成的预测数量
    pub predictions_generated: u64,
    /// 提供的优化建议数量
    pub optimization_suggestions_provided: u64,
    /// 检测到的性能问题数量
    pub performance_issues_detected: u64,
    /// 平均监控延迟（毫秒）
    pub avg_monitoring_latency_ms: f64,
}

impl EnterprisePerformanceMonitor {
    /// 创建新的企业级性能监控器
    pub fn new(
        config: PerformanceMonitorConfig,
        metrics_collector: Arc<dyn MetricsCollector>,
        performance_analyzer: Arc<dyn PerformanceAnalyzer>,
    ) -> Self {
        Self {
            config,
            metrics_collector,
            performance_analyzer,
            performance_history: Arc::new(RwLock::new(VecDeque::new())),
            prediction_cache: Arc::new(RwLock::new(None)),
            optimization_suggestions: Arc::new(RwLock::new(Vec::new())),
            monitoring_stats: Arc::new(RwLock::new(MonitoringStatistics::default())),
        }
    }

    /// 启动性能监控
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🚀 启动企业级性能监控系统");

        // 启动实时监控循环
        let monitor = self.clone();
        tokio::spawn(async move {
            monitor.monitoring_loop().await;
        });

        // 启动预测分析循环
        if self.config.prediction_config.enabled {
            let monitor = self.clone();
            tokio::spawn(async move {
                monitor.prediction_loop().await;
            });
        }

        // 启动优化建议循环
        if self.config.auto_optimization_config.enabled {
            let monitor = self.clone();
            tokio::spawn(async move {
                monitor.optimization_loop().await;
            });
        }

        // 启动数据清理循环
        let monitor = self.clone();
        tokio::spawn(async move {
            monitor.cleanup_loop().await;
        });

        Ok(())
    }

    /// 实时监控循环
    async fn monitoring_loop(&self) {
        let mut interval =
            tokio::time::interval(Duration::from_secs(self.config.monitoring_interval_seconds));

        loop {
            interval.tick().await;

            let start_time = SystemTime::now();

            if let Err(e) = self.collect_performance_metrics().await {
                eprintln!("性能指标收集失败: {}", e);
            }

            // 更新监控延迟统计
            let monitoring_latency = start_time.elapsed().unwrap_or_default().as_millis() as f64;
            self.update_monitoring_latency(monitoring_latency).await;
        }
    }

    /// 收集性能指标
    async fn collect_performance_metrics(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // 获取基础指标
        let metrics_summary = self
            .metrics_collector
            .get_metrics_summary(None, None, None)
            .await?;

        // 构建实时性能指标
        let performance_metrics = RealTimePerformanceMetrics {
            timestamp: now,
            response_time: self.calculate_response_time_metrics(&metrics_summary).await,
            throughput: self.calculate_throughput_metrics(&metrics_summary).await,
            resource_usage: self.calculate_resource_usage_metrics().await,
            error_metrics: self.calculate_error_metrics(&metrics_summary).await,
            performance_trend: self.analyze_performance_trend().await,
        };

        // 存储到历史数据
        {
            let mut history = self.performance_history.write().await;
            history.push_back(performance_metrics.clone());

            // 限制历史数据大小
            let max_data_points =
                (self.config.data_retention_hours * 3600) / self.config.monitoring_interval_seconds;
            while history.len() > max_data_points as usize {
                history.pop_front();
            }
        }

        // 更新统计信息
        self.update_monitoring_statistics().await;

        // 检查性能阈值
        self.check_performance_thresholds(&performance_metrics)
            .await;

        Ok(())
    }

    /// 计算响应时间指标
    async fn calculate_response_time_metrics(
        &self,
        summary: &MetricsSummary,
    ) -> ResponseTimeMetrics {
        // 简化实现 - 在实际应用中应该从详细的执行时间数据计算
        let avg_ms = summary.avg_execution_time_ms;

        ResponseTimeMetrics {
            avg_ms,
            median_ms: avg_ms * 0.9,
            p95_ms: avg_ms * 1.5,
            p99_ms: avg_ms * 2.0,
            max_ms: avg_ms * 3.0,
            min_ms: avg_ms * 0.3,
        }
    }

    /// 计算吞吐量指标
    async fn calculate_throughput_metrics(&self, summary: &MetricsSummary) -> ThroughputMetrics {
        let total_requests = summary.total_executions;
        let successful_requests = summary.successful_executions;
        let failed_requests = summary.failed_executions;

        // 简化的吞吐量计算
        let requests_per_second = total_requests as f64 / 60.0; // 假设基于1分钟窗口

        ThroughputMetrics {
            requests_per_second,
            requests_per_minute: requests_per_second * 60.0,
            total_requests,
            successful_requests,
            failed_requests,
        }
    }

    /// 计算资源使用指标
    async fn calculate_resource_usage_metrics(&self) -> ResourceUsageMetrics {
        // 模拟资源使用数据 - 在实际应用中应该从系统API获取
        ResourceUsageMetrics {
            cpu_usage_percent: 45.0 + (rand::random::<f64>() * 20.0),
            memory_usage_percent: 60.0 + (rand::random::<f64>() * 15.0),
            disk_usage_percent: 30.0 + (rand::random::<f64>() * 10.0),
            network_bandwidth_mbps: 10.0 + (rand::random::<f64>() * 5.0),
            active_connections: 50 + (rand::random::<u64>() % 20),
        }
    }

    /// 计算错误指标
    async fn calculate_error_metrics(&self, summary: &MetricsSummary) -> ErrorMetrics {
        let error_rate_percent = if summary.total_executions > 0 {
            (summary.failed_executions as f64 / summary.total_executions as f64) * 100.0
        } else {
            0.0
        };

        let mut errors_by_type = HashMap::new();
        errors_by_type.insert("timeout".to_string(), summary.failed_executions / 3);
        errors_by_type.insert("connection".to_string(), summary.failed_executions / 4);
        errors_by_type.insert("validation".to_string(), summary.failed_executions / 5);

        let mut errors_by_severity = HashMap::new();
        errors_by_severity.insert("warning".to_string(), summary.failed_executions / 2);
        errors_by_severity.insert("error".to_string(), summary.failed_executions / 3);
        errors_by_severity.insert("critical".to_string(), summary.failed_executions / 6);

        ErrorMetrics {
            error_rate_percent,
            errors_by_type,
            errors_by_severity,
            recent_errors: Vec::new(), // 简化实现
        }
    }

    /// 分析性能趋势
    async fn analyze_performance_trend(&self) -> PerformanceTrend {
        let history = self.performance_history.read().await;

        if history.len() < 3 {
            return PerformanceTrend::Stable;
        }

        // 获取最近的性能数据点
        let recent_points: Vec<_> = history.iter().rev().take(5).collect();

        // 计算响应时间趋势
        let response_times: Vec<f64> = recent_points
            .iter()
            .map(|p| p.response_time.avg_ms)
            .collect();

        // 简单的趋势分析
        let trend_score = self.calculate_trend_score(&response_times);

        match trend_score {
            score if score > 0.1 => PerformanceTrend::Degrading,
            score if score < -0.1 => PerformanceTrend::Improving,
            score if score.abs() > 0.05 => PerformanceTrend::Fluctuating,
            _ => PerformanceTrend::Stable,
        }
    }

    /// 计算趋势分数
    fn calculate_trend_score(&self, values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        let mut trend_sum = 0.0;
        for i in 1..values.len() {
            let change = (values[i] - values[i - 1]) / values[i - 1];
            trend_sum += change;
        }

        trend_sum / (values.len() - 1) as f64
    }

    /// 检查性能阈值
    async fn check_performance_thresholds(&self, metrics: &RealTimePerformanceMetrics) {
        let thresholds = &self.config.thresholds;

        if metrics.response_time.avg_ms > thresholds.response_time_ms {
            println!(
                "⚠️  响应时间超过阈值: {:.2}ms > {:.2}ms",
                metrics.response_time.avg_ms, thresholds.response_time_ms
            );
            self.increment_performance_issues().await;
        }

        if metrics.resource_usage.cpu_usage_percent > thresholds.cpu_usage_percent {
            println!(
                "⚠️  CPU使用率超过阈值: {:.1}% > {:.1}%",
                metrics.resource_usage.cpu_usage_percent, thresholds.cpu_usage_percent
            );
            self.increment_performance_issues().await;
        }

        if metrics.error_metrics.error_rate_percent > thresholds.error_rate_percent {
            println!(
                "⚠️  错误率超过阈值: {:.1}% > {:.1}%",
                metrics.error_metrics.error_rate_percent, thresholds.error_rate_percent
            );
            self.increment_performance_issues().await;
        }
    }

    /// 预测分析循环
    async fn prediction_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(
            self.config.prediction_config.prediction_window_hours * 3600 / 4,
        ));

        loop {
            interval.tick().await;

            if let Err(e) = self.generate_performance_prediction().await {
                eprintln!("性能预测生成失败: {}", e);
            }
        }
    }

    /// 生成性能预测
    async fn generate_performance_prediction(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let history = self.performance_history.read().await;

        if history.len() < 10 {
            return Ok(()); // 数据不足，无法预测
        }

        // 获取历史数据用于预测
        let history_window = self.config.prediction_config.history_window_hours * 3600
            / self.config.monitoring_interval_seconds;
        let recent_data: Vec<_> = history.iter().rev().take(history_window as usize).collect();

        // 简化的线性预测模型
        let prediction = self.create_linear_prediction(&recent_data).await;

        // 缓存预测结果
        {
            let mut cache = self.prediction_cache.write().await;
            *cache = Some(prediction);
        }

        // 更新统计信息
        self.increment_predictions_generated().await;

        println!("🔮 生成性能预测 (基于{}个数据点)", recent_data.len());

        Ok(())
    }

    /// 创建线性预测
    async fn create_linear_prediction(
        &self,
        data: &[&RealTimePerformanceMetrics],
    ) -> PerformancePrediction {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // 简化的预测算法
        let response_times: Vec<f64> = data.iter().map(|d| d.response_time.avg_ms).collect();
        let throughputs: Vec<f64> = data
            .iter()
            .map(|d| d.throughput.requests_per_second)
            .collect();

        let predicted_response_time = self.predict_value(&response_times);
        let predicted_throughput = self.predict_value(&throughputs);

        // 预测资源使用
        let cpu_values: Vec<f64> = data
            .iter()
            .map(|d| d.resource_usage.cpu_usage_percent)
            .collect();
        let memory_values: Vec<f64> = data
            .iter()
            .map(|d| d.resource_usage.memory_usage_percent)
            .collect();

        let predicted_resource_usage = ResourceUsageMetrics {
            cpu_usage_percent: self.predict_value(&cpu_values),
            memory_usage_percent: self.predict_value(&memory_values),
            disk_usage_percent: 35.0,     // 简化
            network_bandwidth_mbps: 12.0, // 简化
            active_connections: 55,       // 简化
        };

        PerformancePrediction {
            prediction_timestamp: now
                + (self.config.prediction_config.prediction_window_hours * 3600 * 1000),
            predicted_response_time_ms: predicted_response_time,
            predicted_throughput_rps: predicted_throughput,
            predicted_resource_usage,
            confidence: 0.75, // 简化的置信度
            data_points_used: data.len(),
        }
    }

    /// 预测数值
    fn predict_value(&self, values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        // 简单的移动平均预测
        let recent_avg =
            values.iter().rev().take(5).sum::<f64>() / 5.0_f64.min(values.len() as f64);
        let overall_avg = values.iter().sum::<f64>() / values.len() as f64;

        // 加权平均：70%最近趋势 + 30%整体趋势
        recent_avg * 0.7 + overall_avg * 0.3
    }

    /// 优化建议循环
    async fn optimization_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(
            self.config
                .auto_optimization_config
                .execution_interval_minutes
                * 60,
        ));

        loop {
            interval.tick().await;

            if let Err(e) = self.generate_optimization_suggestions().await {
                eprintln!("优化建议生成失败: {}", e);
            }
        }
    }

    /// 生成优化建议
    async fn generate_optimization_suggestions(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let history = self.performance_history.read().await;

        if history.is_empty() {
            return Ok(());
        }

        let latest_metrics = history.back().unwrap();
        let mut suggestions = Vec::new();

        // 基于当前性能指标生成建议
        if latest_metrics.response_time.avg_ms > self.config.thresholds.response_time_ms {
            suggestions.push(
                self.create_response_time_optimization_suggestion(latest_metrics)
                    .await,
            );
        }

        if latest_metrics.resource_usage.cpu_usage_percent
            > self.config.thresholds.cpu_usage_percent
        {
            suggestions.push(
                self.create_cpu_optimization_suggestion(latest_metrics)
                    .await,
            );
        }

        if latest_metrics.resource_usage.memory_usage_percent
            > self.config.thresholds.memory_usage_percent
        {
            suggestions.push(
                self.create_memory_optimization_suggestion(latest_metrics)
                    .await,
            );
        }

        if latest_metrics.error_metrics.error_rate_percent
            > self.config.thresholds.error_rate_percent
        {
            suggestions.push(
                self.create_error_rate_optimization_suggestion(latest_metrics)
                    .await,
            );
        }

        // 更新优化建议缓存
        {
            let mut cache = self.optimization_suggestions.write().await;
            *cache = suggestions;
        }

        // 更新统计信息
        self.increment_optimization_suggestions().await;

        println!(
            "💡 生成了{}条优化建议",
            self.optimization_suggestions.read().await.len()
        );

        Ok(())
    }

    /// 创建响应时间优化建议
    async fn create_response_time_optimization_suggestion(
        &self,
        metrics: &RealTimePerformanceMetrics,
    ) -> PerformanceOptimizationSuggestion {
        PerformanceOptimizationSuggestion {
            id: uuid::Uuid::new_v4().to_string(),
            title: "优化响应时间".to_string(),
            description: format!(
                "当前平均响应时间为{:.2}ms，超过阈值{:.2}ms",
                metrics.response_time.avg_ms, self.config.thresholds.response_time_ms
            ),
            optimization_type: OptimizationStrategy::CacheOptimization,
            expected_improvement: 25.0,
            implementation_difficulty: DifficultyLevel::Medium,
            implementation_steps: vec![
                "分析慢查询和热点数据".to_string(),
                "实施Redis缓存层".to_string(),
                "优化数据库索引".to_string(),
                "启用HTTP缓存头".to_string(),
            ],
            risk_assessment: RiskLevel::Low,
        }
    }

    /// 创建CPU优化建议
    async fn create_cpu_optimization_suggestion(
        &self,
        metrics: &RealTimePerformanceMetrics,
    ) -> PerformanceOptimizationSuggestion {
        PerformanceOptimizationSuggestion {
            id: uuid::Uuid::new_v4().to_string(),
            title: "优化CPU使用率".to_string(),
            description: format!(
                "当前CPU使用率为{:.1}%，超过阈值{:.1}%",
                metrics.resource_usage.cpu_usage_percent, self.config.thresholds.cpu_usage_percent
            ),
            optimization_type: OptimizationStrategy::AutoScaling,
            expected_improvement: 30.0,
            implementation_difficulty: DifficultyLevel::Easy,
            implementation_steps: vec![
                "启用水平自动扩容".to_string(),
                "优化算法复杂度".to_string(),
                "实施异步处理".to_string(),
                "使用连接池".to_string(),
            ],
            risk_assessment: RiskLevel::Medium,
        }
    }

    /// 创建内存优化建议
    async fn create_memory_optimization_suggestion(
        &self,
        metrics: &RealTimePerformanceMetrics,
    ) -> PerformanceOptimizationSuggestion {
        PerformanceOptimizationSuggestion {
            id: uuid::Uuid::new_v4().to_string(),
            title: "优化内存使用".to_string(),
            description: format!(
                "当前内存使用率为{:.1}%，超过阈值{:.1}%",
                metrics.resource_usage.memory_usage_percent,
                self.config.thresholds.memory_usage_percent
            ),
            optimization_type: OptimizationStrategy::GarbageCollectionTuning,
            expected_improvement: 20.0,
            implementation_difficulty: DifficultyLevel::Hard,
            implementation_steps: vec![
                "分析内存泄漏".to_string(),
                "优化数据结构".to_string(),
                "调整垃圾回收参数".to_string(),
                "实施对象池".to_string(),
            ],
            risk_assessment: RiskLevel::High,
        }
    }

    /// 创建错误率优化建议
    async fn create_error_rate_optimization_suggestion(
        &self,
        metrics: &RealTimePerformanceMetrics,
    ) -> PerformanceOptimizationSuggestion {
        PerformanceOptimizationSuggestion {
            id: uuid::Uuid::new_v4().to_string(),
            title: "降低错误率".to_string(),
            description: format!(
                "当前错误率为{:.1}%，超过阈值{:.1}%",
                metrics.error_metrics.error_rate_percent, self.config.thresholds.error_rate_percent
            ),
            optimization_type: OptimizationStrategy::LoadBalancingAdjustment,
            expected_improvement: 40.0,
            implementation_difficulty: DifficultyLevel::Medium,
            implementation_steps: vec![
                "分析错误日志模式".to_string(),
                "实施重试机制".to_string(),
                "优化负载均衡策略".to_string(),
                "增强错误处理".to_string(),
            ],
            risk_assessment: RiskLevel::Low,
        }
    }

    /// 数据清理循环
    async fn cleanup_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(3600)); // 每小时清理一次

        loop {
            interval.tick().await;

            if let Err(e) = self.cleanup_old_data().await {
                eprintln!("数据清理失败: {}", e);
            }
        }
    }

    /// 清理过期数据
    async fn cleanup_old_data(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut history = self.performance_history.write().await;
        let max_data_points =
            (self.config.data_retention_hours * 3600) / self.config.monitoring_interval_seconds;

        while history.len() > max_data_points as usize {
            history.pop_front();
        }

        println!("🧹 数据清理完成，保留{}个数据点", history.len());
        Ok(())
    }

    /// 更新监控统计信息
    async fn update_monitoring_statistics(&self) {
        let mut stats = self.monitoring_stats.write().await;
        stats.total_data_points += 1;
        stats.uptime_seconds += self.config.monitoring_interval_seconds;
    }

    /// 更新监控延迟
    async fn update_monitoring_latency(&self, latency_ms: f64) {
        let mut stats = self.monitoring_stats.write().await;
        let total_points = stats.total_data_points as f64;
        if total_points > 0.0 {
            stats.avg_monitoring_latency_ms =
                (stats.avg_monitoring_latency_ms * (total_points - 1.0) + latency_ms)
                    / total_points;
        } else {
            stats.avg_monitoring_latency_ms = latency_ms;
        }
    }

    /// 增加性能问题计数
    async fn increment_performance_issues(&self) {
        let mut stats = self.monitoring_stats.write().await;
        stats.performance_issues_detected += 1;
    }

    /// 增加预测生成计数
    async fn increment_predictions_generated(&self) {
        let mut stats = self.monitoring_stats.write().await;
        stats.predictions_generated += 1;
    }

    /// 增加优化建议计数
    async fn increment_optimization_suggestions(&self) {
        let mut stats = self.monitoring_stats.write().await;
        stats.optimization_suggestions_provided += 1;
    }

    /// 获取当前性能指标
    pub async fn get_current_performance_metrics(&self) -> Option<RealTimePerformanceMetrics> {
        let history = self.performance_history.read().await;
        history.back().cloned()
    }

    /// 获取性能历史数据
    pub async fn get_performance_history(
        &self,
        limit: Option<usize>,
    ) -> Vec<RealTimePerformanceMetrics> {
        let history = self.performance_history.read().await;
        match limit {
            Some(n) => history.iter().rev().take(n).cloned().collect(),
            None => history.iter().cloned().collect(),
        }
    }

    /// 获取性能预测
    pub async fn get_performance_prediction(&self) -> Option<PerformancePrediction> {
        self.prediction_cache.read().await.clone()
    }

    /// 获取优化建议
    pub async fn get_optimization_suggestions(&self) -> Vec<PerformanceOptimizationSuggestion> {
        self.optimization_suggestions.read().await.clone()
    }

    /// 获取监控统计信息
    pub async fn get_monitoring_statistics(&self) -> MonitoringStatistics {
        self.monitoring_stats.read().await.clone()
    }

    /// 获取性能摘要报告
    pub async fn get_performance_summary(&self) -> PerformanceSummaryReport {
        let current_metrics = self.get_current_performance_metrics().await;
        let prediction = self.get_performance_prediction().await;
        let suggestions = self.get_optimization_suggestions().await;
        let stats = self.get_monitoring_statistics().await;

        PerformanceSummaryReport {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            current_metrics,
            prediction,
            optimization_suggestions: suggestions,
            monitoring_statistics: stats,
            health_score: self.calculate_health_score().await,
        }
    }

    /// 计算健康分数
    async fn calculate_health_score(&self) -> f64 {
        let current_metrics = match self.get_current_performance_metrics().await {
            Some(metrics) => metrics,
            None => return 50.0, // 默认中等健康分数
        };

        let mut score = 100.0;
        let thresholds = &self.config.thresholds;

        // 响应时间评分
        if current_metrics.response_time.avg_ms > thresholds.response_time_ms {
            let penalty =
                (current_metrics.response_time.avg_ms / thresholds.response_time_ms - 1.0) * 20.0;
            score -= penalty.min(25.0);
        }

        // CPU使用率评分
        if current_metrics.resource_usage.cpu_usage_percent > thresholds.cpu_usage_percent {
            let penalty = (current_metrics.resource_usage.cpu_usage_percent
                / thresholds.cpu_usage_percent
                - 1.0)
                * 15.0;
            score -= penalty.min(20.0);
        }

        // 内存使用率评分
        if current_metrics.resource_usage.memory_usage_percent > thresholds.memory_usage_percent {
            let penalty = (current_metrics.resource_usage.memory_usage_percent
                / thresholds.memory_usage_percent
                - 1.0)
                * 15.0;
            score -= penalty.min(20.0);
        }

        // 错误率评分
        if current_metrics.error_metrics.error_rate_percent > thresholds.error_rate_percent {
            let penalty = (current_metrics.error_metrics.error_rate_percent
                / thresholds.error_rate_percent
                - 1.0)
                * 25.0;
            score -= penalty.min(30.0);
        }

        score.max(0.0).min(100.0)
    }
}

/// 性能摘要报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummaryReport {
    /// 报告时间戳
    pub timestamp: u64,
    /// 当前性能指标
    pub current_metrics: Option<RealTimePerformanceMetrics>,
    /// 性能预测
    pub prediction: Option<PerformancePrediction>,
    /// 优化建议
    pub optimization_suggestions: Vec<PerformanceOptimizationSuggestion>,
    /// 监控统计信息
    pub monitoring_statistics: MonitoringStatistics,
    /// 健康分数 (0-100)
    pub health_score: f64,
}

// 为EnterprisePerformanceMonitor实现Clone trait
impl Clone for EnterprisePerformanceMonitor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            metrics_collector: self.metrics_collector.clone(),
            performance_analyzer: self.performance_analyzer.clone(),
            performance_history: self.performance_history.clone(),
            prediction_cache: self.prediction_cache.clone(),
            optimization_suggestions: self.optimization_suggestions.clone(),
            monitoring_stats: self.monitoring_stats.clone(),
        }
    }
}

impl Default for PerformanceMonitorConfig {
    fn default() -> Self {
        Self {
            monitoring_interval_seconds: 10,
            data_retention_hours: 24,
            thresholds: PerformanceThresholds {
                response_time_ms: 1000.0,
                cpu_usage_percent: 80.0,
                memory_usage_percent: 85.0,
                error_rate_percent: 5.0,
                throughput_rps: 100.0,
            },
            prediction_config: PredictionConfig {
                enabled: true,
                prediction_window_hours: 1,
                history_window_hours: 6,
                accuracy_threshold: 0.8,
            },
            auto_optimization_config: AutoOptimizationConfig {
                enabled: false, // 默认关闭自动优化
                strategies: vec![
                    OptimizationStrategy::AutoScaling,
                    OptimizationStrategy::CacheOptimization,
                ],
                execution_interval_minutes: 30,
            },
        }
    }
}
