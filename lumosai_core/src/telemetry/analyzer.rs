//! 智能性能分析器
//! 
//! 提供自动化性能分析、异常检测、趋势预测等功能

use crate::telemetry::metrics::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// 性能分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    /// 分析时间戳
    pub timestamp: u64,
    /// 分析时间范围
    pub time_range: TimeRange,
    /// 整体性能评分 (0-100)
    pub overall_score: f64,
    /// 性能趋势
    pub trend: PerformanceTrend,
    /// 瓶颈分析
    pub bottlenecks: Vec<PerformanceBottleneck>,
    /// 异常检测结果
    pub anomalies: Vec<PerformanceAnomaly>,
    /// 优化建议
    pub recommendations: Vec<OptimizationRecommendation>,
    /// 预测分析
    pub predictions: Vec<PerformancePrediction>,
}

/// 性能趋势
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceTrend {
    /// 性能改善
    Improving { rate: f64 },
    /// 性能稳定
    Stable { variance: f64 },
    /// 性能下降
    Degrading { rate: f64 },
    /// 性能波动
    Volatile { amplitude: f64 },
}

/// 性能瓶颈
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBottleneck {
    /// 瓶颈类型
    pub bottleneck_type: BottleneckType,
    /// 严重程度 (0-100)
    pub severity: f64,
    /// 影响描述
    pub impact: String,
    /// 相关指标
    pub metrics: HashMap<String, f64>,
    /// 建议解决方案
    pub solutions: Vec<String>,
}

/// 瓶颈类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BottleneckType {
    /// CPU瓶颈
    CPU,
    /// 内存瓶颈
    Memory,
    /// 网络瓶颈
    Network,
    /// 数据库瓶颈
    Database,
    /// 工具执行瓶颈
    ToolExecution,
    /// 代理响应瓶颈
    AgentResponse,
}

/// 性能异常
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnomaly {
    /// 异常类型
    pub anomaly_type: AnomalyType,
    /// 异常时间
    pub detected_at: u64,
    /// 异常值
    pub value: f64,
    /// 期望值
    pub expected_value: f64,
    /// 偏差程度
    pub deviation: f64,
    /// 置信度
    pub confidence: f64,
    /// 描述
    pub description: String,
}

/// 异常类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    /// 响应时间异常
    ResponseTimeSpike,
    /// 错误率异常
    ErrorRateSpike,
    /// 内存使用异常
    MemoryUsageSpike,
    /// CPU使用异常
    CpuUsageSpike,
    /// 工具调用异常
    ToolCallAnomaly,
    /// 自定义指标异常
    CustomMetricAnomaly { metric_name: String },
}

/// 优化建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    /// 建议类型
    pub recommendation_type: RecommendationType,
    /// 优先级 (1-10)
    pub priority: u8,
    /// 标题
    pub title: String,
    /// 描述
    pub description: String,
    /// 预期收益
    pub expected_benefit: String,
    /// 实施难度
    pub implementation_difficulty: DifficultyLevel,
    /// 具体步骤
    pub steps: Vec<String>,
    /// 风险评估
    pub risks: Vec<String>,
}

/// 建议类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    /// 配置优化
    Configuration,
    /// 代码优化
    Code,
    /// 架构优化
    Architecture,
    /// 资源扩容
    Scaling,
    /// 缓存优化
    Caching,
    /// 数据库优化
    Database,
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

/// 性能预测
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePrediction {
    /// 预测指标
    pub metric_name: String,
    /// 预测时间点
    pub predicted_at: u64,
    /// 预测值
    pub predicted_value: f64,
    /// 置信区间
    pub confidence_interval: (f64, f64),
    /// 预测模型
    pub model_type: PredictionModel,
    /// 预测准确度
    pub accuracy: f64,
}

/// 预测模型类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PredictionModel {
    /// 线性回归
    LinearRegression,
    /// 移动平均
    MovingAverage,
    /// 指数平滑
    ExponentialSmoothing,
    /// 季节性分解
    SeasonalDecomposition,
}

/// 性能分析器trait
#[async_trait]
pub trait PerformanceAnalyzer: Send + Sync {
    /// 分析性能数据
    async fn analyze(&self, metrics: &[AgentMetrics], time_range: TimeRange) -> Result<PerformanceAnalysis, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 检测异常
    async fn detect_anomalies(&self, metrics: &[AgentMetrics]) -> Result<Vec<PerformanceAnomaly>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 识别瓶颈
    async fn identify_bottlenecks(&self, metrics: &[AgentMetrics]) -> Result<Vec<PerformanceBottleneck>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 生成优化建议
    async fn generate_recommendations(&self, analysis: &PerformanceAnalysis) -> Result<Vec<OptimizationRecommendation>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 预测性能趋势
    async fn predict_trends(&self, metrics: &[AgentMetrics]) -> Result<Vec<PerformancePrediction>, Box<dyn std::error::Error + Send + Sync>>;
}

/// 智能性能分析器实现
#[derive(Debug)]
pub struct IntelligentPerformanceAnalyzer {
    /// 历史数据窗口大小
    history_window_size: usize,
    /// 异常检测敏感度
    anomaly_sensitivity: f64,
    /// 预测模型配置
    prediction_config: PredictionConfig,
}

/// 预测配置
#[derive(Debug, Clone)]
pub struct PredictionConfig {
    /// 预测窗口大小
    pub window_size: usize,
    /// 预测步长
    pub prediction_steps: usize,
    /// 置信水平
    pub confidence_level: f64,
}

impl IntelligentPerformanceAnalyzer {
    /// 创建新的智能性能分析器
    pub fn new() -> Self {
        Self {
            history_window_size: 1000,
            anomaly_sensitivity: 2.0, // 2个标准差
            prediction_config: PredictionConfig {
                window_size: 100,
                prediction_steps: 10,
                confidence_level: 0.95,
            },
        }
    }
    
    /// 计算性能评分
    fn calculate_performance_score(&self, metrics: &[AgentMetrics]) -> f64 {
        if metrics.is_empty() {
            return 0.0;
        }
        
        let success_rate = metrics.iter()
            .map(|m| if m.success { 1.0 } else { 0.0 })
            .sum::<f64>() / metrics.len() as f64;
        
        let avg_response_time = metrics.iter()
            .map(|m| m.execution_time_ms as f64)
            .sum::<f64>() / metrics.len() as f64;
        
        // 响应时间评分 (越低越好)
        let response_score = if avg_response_time <= 100.0 {
            100.0
        } else if avg_response_time <= 500.0 {
            100.0 - (avg_response_time - 100.0) / 4.0
        } else {
            0.0
        };
        
        // 综合评分
        (success_rate * 100.0 * 0.7) + (response_score * 0.3)
    }
    
    /// 分析性能趋势
    fn analyze_trend(&self, metrics: &[AgentMetrics]) -> PerformanceTrend {
        if metrics.len() < 10 {
            return PerformanceTrend::Stable { variance: 0.0 };
        }
        
        let response_times: Vec<f64> = metrics.iter()
            .map(|m| m.execution_time_ms as f64)
            .collect();
        
        // 简单的线性回归分析趋势
        let n = response_times.len() as f64;
        let sum_x: f64 = (0..response_times.len()).map(|i| i as f64).sum();
        let sum_y: f64 = response_times.iter().sum();
        let sum_xy: f64 = response_times.iter().enumerate()
            .map(|(i, &y)| i as f64 * y)
            .sum();
        let sum_x2: f64 = (0..response_times.len()).map(|i| (i as f64).powi(2)).sum();
        
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));
        
        if slope.abs() < 0.1 {
            let variance = response_times.iter()
                .map(|&x| (x - sum_y / n).powi(2))
                .sum::<f64>() / n;
            PerformanceTrend::Stable { variance }
        } else if slope > 0.1 {
            PerformanceTrend::Degrading { rate: slope }
        } else {
            PerformanceTrend::Improving { rate: -slope }
        }
    }
}

#[async_trait]
impl PerformanceAnalyzer for IntelligentPerformanceAnalyzer {
    async fn analyze(&self, metrics: &[AgentMetrics], time_range: TimeRange) -> Result<PerformanceAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;

        // 计算整体性能评分
        let overall_score = self.calculate_performance_score(metrics);

        // 分析性能趋势
        let trend = self.analyze_trend(metrics);

        // 检测瓶颈
        let bottlenecks = self.identify_bottlenecks(metrics).await?;

        // 检测异常
        let anomalies = self.detect_anomalies(metrics).await?;

        // 预测趋势
        let predictions = self.predict_trends(metrics).await?;

        let analysis = PerformanceAnalysis {
            timestamp: now,
            time_range,
            overall_score,
            trend,
            bottlenecks: bottlenecks.clone(),
            anomalies,
            recommendations: vec![], // 将在后续调用中生成
            predictions,
        };

        // 生成优化建议
        let recommendations = self.generate_recommendations(&analysis).await?;

        Ok(PerformanceAnalysis {
            recommendations,
            ..analysis
        })
    }

    async fn detect_anomalies(&self, metrics: &[AgentMetrics]) -> Result<Vec<PerformanceAnomaly>, Box<dyn std::error::Error + Send + Sync>> {
        let mut anomalies = Vec::new();

        if metrics.len() < 10 {
            return Ok(anomalies);
        }

        // 检测响应时间异常
        let response_times: Vec<f64> = metrics.iter()
            .map(|m| m.execution_time_ms as f64)
            .collect();

        let mean = response_times.iter().sum::<f64>() / response_times.len() as f64;
        let variance = response_times.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / response_times.len() as f64;
        let std_dev = variance.sqrt();

        for (_i, &time) in response_times.iter().enumerate() {
            let deviation = (time - mean).abs() / std_dev;
            if deviation > self.anomaly_sensitivity {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
                anomalies.push(PerformanceAnomaly {
                    anomaly_type: AnomalyType::ResponseTimeSpike,
                    detected_at: now,
                    value: time,
                    expected_value: mean,
                    deviation,
                    confidence: 1.0 - (1.0 / (1.0 + deviation)),
                    description: format!("响应时间异常: {:.1}ms (期望: {:.1}ms)", time, mean),
                });
            }
        }

        // 检测错误率异常
        let error_rates: Vec<f64> = metrics.windows(10)
            .map(|window| {
                let errors = window.iter().filter(|m| !m.success).count();
                errors as f64 / window.len() as f64
            })
            .collect();

        if !error_rates.is_empty() {
            let mean_error_rate = error_rates.iter().sum::<f64>() / error_rates.len() as f64;
            for &rate in &error_rates {
                if rate > mean_error_rate + 0.1 { // 错误率增加超过10%
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
                    anomalies.push(PerformanceAnomaly {
                        anomaly_type: AnomalyType::ErrorRateSpike,
                        detected_at: now,
                        value: rate,
                        expected_value: mean_error_rate,
                        deviation: (rate - mean_error_rate) / mean_error_rate,
                        confidence: 0.8,
                        description: format!("错误率异常: {:.1}% (期望: {:.1}%)", rate * 100.0, mean_error_rate * 100.0),
                    });
                }
            }
        }

        Ok(anomalies)
    }

    async fn identify_bottlenecks(&self, metrics: &[AgentMetrics]) -> Result<Vec<PerformanceBottleneck>, Box<dyn std::error::Error + Send + Sync>> {
        let mut bottlenecks = Vec::new();

        if metrics.is_empty() {
            return Ok(bottlenecks);
        }

        // 分析响应时间瓶颈
        let avg_response_time = metrics.iter()
            .map(|m| m.execution_time_ms as f64)
            .sum::<f64>() / metrics.len() as f64;

        if avg_response_time > 1000.0 {
            let mut bottleneck_metrics = HashMap::new();
            bottleneck_metrics.insert("avg_response_time_ms".to_string(), avg_response_time);

            bottlenecks.push(PerformanceBottleneck {
                bottleneck_type: BottleneckType::AgentResponse,
                severity: ((avg_response_time - 1000.0) / 1000.0 * 100.0).min(100.0),
                impact: "代理响应时间过长，影响用户体验".to_string(),
                metrics: bottleneck_metrics,
                solutions: vec![
                    "优化代理逻辑，减少不必要的计算".to_string(),
                    "增加缓存机制，避免重复计算".to_string(),
                    "并行处理工具调用".to_string(),
                    "优化LLM调用参数".to_string(),
                ],
            });
        }

        // 分析工具执行瓶颈
        let tool_calls: Vec<usize> = metrics.iter()
            .map(|m| m.tool_calls_count)
            .collect();

        if !tool_calls.is_empty() {
            let avg_tool_calls = tool_calls.iter().sum::<usize>() as f64 / tool_calls.len() as f64;
            if avg_tool_calls > 5.0 {
                let mut bottleneck_metrics = HashMap::new();
                bottleneck_metrics.insert("avg_tool_calls".to_string(), avg_tool_calls);

                bottlenecks.push(PerformanceBottleneck {
                    bottleneck_type: BottleneckType::ToolExecution,
                    severity: ((avg_tool_calls - 5.0) / 5.0 * 100.0).min(100.0),
                    impact: "工具调用次数过多，可能存在效率问题".to_string(),
                    metrics: bottleneck_metrics,
                    solutions: vec![
                        "优化工具选择逻辑".to_string(),
                        "合并相似的工具调用".to_string(),
                        "实现工具调用缓存".to_string(),
                        "使用更高效的工具".to_string(),
                    ],
                });
            }
        }

        Ok(bottlenecks)
    }

    async fn generate_recommendations(&self, analysis: &PerformanceAnalysis) -> Result<Vec<OptimizationRecommendation>, Box<dyn std::error::Error + Send + Sync>> {
        let mut recommendations = Vec::new();

        // 基于性能评分生成建议
        if analysis.overall_score < 70.0 {
            recommendations.push(OptimizationRecommendation {
                recommendation_type: RecommendationType::Configuration,
                priority: 9,
                title: "系统性能优化".to_string(),
                description: "系统整体性能较低，需要进行全面优化".to_string(),
                expected_benefit: "提升系统性能20-30%".to_string(),
                implementation_difficulty: DifficultyLevel::Medium,
                steps: vec![
                    "分析性能瓶颈".to_string(),
                    "优化配置参数".to_string(),
                    "升级硬件资源".to_string(),
                    "监控优化效果".to_string(),
                ],
                risks: vec![
                    "配置变更可能影响系统稳定性".to_string(),
                    "硬件升级需要停机时间".to_string(),
                ],
            });
        }

        // 基于趋势生成建议
        match &analysis.trend {
            PerformanceTrend::Degrading { rate } => {
                recommendations.push(OptimizationRecommendation {
                    recommendation_type: RecommendationType::Architecture,
                    priority: 8,
                    title: "性能下降预警".to_string(),
                    description: format!("检测到性能持续下降，下降率: {:.2}", rate),
                    expected_benefit: "阻止性能进一步恶化".to_string(),
                    implementation_difficulty: DifficultyLevel::Hard,
                    steps: vec![
                        "深入分析性能下降原因".to_string(),
                        "检查系统资源使用情况".to_string(),
                        "优化关键代码路径".to_string(),
                        "考虑架构重构".to_string(),
                    ],
                    risks: vec![
                        "架构变更风险较高".to_string(),
                        "可能需要较长的开发周期".to_string(),
                    ],
                });
            },
            PerformanceTrend::Volatile { amplitude } => {
                recommendations.push(OptimizationRecommendation {
                    recommendation_type: RecommendationType::Configuration,
                    priority: 6,
                    title: "性能稳定性优化".to_string(),
                    description: format!("性能波动较大，波动幅度: {:.2}", amplitude),
                    expected_benefit: "提升系统稳定性".to_string(),
                    implementation_difficulty: DifficultyLevel::Medium,
                    steps: vec![
                        "分析性能波动原因".to_string(),
                        "优化负载均衡策略".to_string(),
                        "增加系统缓冲".to_string(),
                        "实施平滑处理".to_string(),
                    ],
                    risks: vec![
                        "配置调整可能影响峰值性能".to_string(),
                    ],
                });
            },
            _ => {}
        }

        // 基于瓶颈生成建议
        for bottleneck in &analysis.bottlenecks {
            match bottleneck.bottleneck_type {
                BottleneckType::AgentResponse => {
                    recommendations.push(OptimizationRecommendation {
                        recommendation_type: RecommendationType::Code,
                        priority: 7,
                        title: "代理响应优化".to_string(),
                        description: "代理响应时间过长，需要优化处理逻辑".to_string(),
                        expected_benefit: "减少响应时间30-50%".to_string(),
                        implementation_difficulty: DifficultyLevel::Medium,
                        steps: bottleneck.solutions.clone(),
                        risks: vec![
                            "代码优化可能引入新的bug".to_string(),
                            "需要充分测试".to_string(),
                        ],
                    });
                },
                BottleneckType::ToolExecution => {
                    recommendations.push(OptimizationRecommendation {
                        recommendation_type: RecommendationType::Architecture,
                        priority: 6,
                        title: "工具执行优化".to_string(),
                        description: "工具调用效率低下，需要优化工具使用策略".to_string(),
                        expected_benefit: "减少工具调用次数20-40%".to_string(),
                        implementation_difficulty: DifficultyLevel::Medium,
                        steps: bottleneck.solutions.clone(),
                        risks: vec![
                            "工具优化可能影响功能完整性".to_string(),
                        ],
                    });
                },
                _ => {}
            }
        }

        // 按优先级排序
        recommendations.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(recommendations)
    }

    async fn predict_trends(&self, metrics: &[AgentMetrics]) -> Result<Vec<PerformancePrediction>, Box<dyn std::error::Error + Send + Sync>> {
        let mut predictions = Vec::new();

        if metrics.len() < self.prediction_config.window_size {
            return Ok(predictions);
        }

        // 预测响应时间趋势
        let response_times: Vec<f64> = metrics.iter()
            .map(|m| m.execution_time_ms as f64)
            .collect();

        // 简单的移动平均预测
        let window_size = self.prediction_config.window_size.min(response_times.len());
        let recent_avg = response_times[response_times.len() - window_size..]
            .iter()
            .sum::<f64>() / window_size as f64;

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        let future_time = now + (60 * 1000); // 1分钟后

        predictions.push(PerformancePrediction {
            metric_name: "response_time_ms".to_string(),
            predicted_at: future_time,
            predicted_value: recent_avg,
            confidence_interval: (recent_avg * 0.8, recent_avg * 1.2),
            model_type: PredictionModel::MovingAverage,
            accuracy: 0.75,
        });

        // 预测成功率趋势
        let success_rates: Vec<f64> = metrics.windows(10)
            .map(|window| {
                let successes = window.iter().filter(|m| m.success).count();
                successes as f64 / window.len() as f64
            })
            .collect();

        if !success_rates.is_empty() {
            let recent_success_rate = success_rates[success_rates.len() - 1];

            predictions.push(PerformancePrediction {
                metric_name: "success_rate".to_string(),
                predicted_at: future_time,
                predicted_value: recent_success_rate,
                confidence_interval: ((recent_success_rate - 0.1).max(0.0), (recent_success_rate + 0.1).min(1.0)),
                model_type: PredictionModel::MovingAverage,
                accuracy: 0.8,
            });
        }

        Ok(predictions)
    }
}

impl Default for IntelligentPerformanceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
