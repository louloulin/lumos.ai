#!/usr/bin/env cargo
//! # 实时数据处理示例
//!
//! 本示例展示如何使用 LumosAI 构建实时数据处理系统，
//! 处理流式数据并提供智能分析和预警。
//!
//! ## 功能特性
//! - 实时数据流处理
//! - 智能异常检测
//! - 动态阈值调整
//! - 实时报警和通知

use lumosai_core::agent::types::AgentGenerateOptions;
use lumosai_core::agent::BasicAgent;
use lumosai_core::llm::types::user_message;
use lumosai_core::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

/// 数据点
#[derive(Debug, Clone)]
struct DataPoint {
    timestamp: u64,
    value: f64,
    source: String,
    metadata: HashMap<String, String>,
}

/// 异常检测结果
#[derive(Debug, Clone)]
struct AnomalyResult {
    is_anomaly: bool,
    confidence: f32,
    severity: AnomalySeverity,
    description: String,
    recommended_action: String,
}

/// 异常严重程度
#[derive(Debug, Clone, PartialEq)]
enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// 处理统计信息
#[derive(Debug)]
struct ProcessingStats {
    total_processed: usize,
    anomalies_detected: usize,
    processing_rate: f64,
    average_latency: Duration,
    uptime: Duration,
}

/// 实时数据处理引擎
struct RealtimeDataProcessor {
    analyzer: BasicAgent,
    detector: BasicAgent,
    alerter: BasicAgent,
    data_buffer: VecDeque<DataPoint>,
    anomaly_history: Vec<AnomalyResult>,
    processing_stats: ProcessingStats,
    start_time: Instant,
    thresholds: HashMap<String, f64>,
}

impl RealtimeDataProcessor {
    /// 创建实时数据处理引擎
    async fn new() -> Result<Self> {
        println!("🚀 正在初始化实时数据处理引擎...");

        // 创建数据分析器 Agent
        let analyzer = data_agent(
            "数据分析器",
            "你是一个专业的实时数据分析器，负责：
            1. 分析实时数据流的趋势和模式
            2. 计算统计指标和关键性能指标
            3. 识别数据中的异常模式
            4. 提供数据质量评估",
        )
        .build()?;

        // 创建异常检测器 Agent
        let detector = quick_agent(
            "异常检测器",
            "你是一个智能异常检测专家，负责：
            1. 实时监控数据异常
            2. 评估异常的严重程度
            3. 分析异常的可能原因
            4. 提供处理建议和预防措施",
        )
        .build()?;

        // 创建报警器 Agent
        let alerter = quick_agent(
            "智能报警器",
            "你是一个智能报警系统，负责：
            1. 根据异常严重程度发送报警
            2. 生成详细的报警信息
            3. 建议紧急响应措施
            4. 跟踪报警处理状态",
        )
        .build()?;

        // 初始化阈值配置
        let mut thresholds = HashMap::new();
        thresholds.insert("cpu_usage".to_string(), 80.0);
        thresholds.insert("memory_usage".to_string(), 85.0);
        thresholds.insert("disk_usage".to_string(), 90.0);
        thresholds.insert("network_latency".to_string(), 100.0);
        thresholds.insert("error_rate".to_string(), 5.0);

        println!("✅ 实时数据处理引擎初始化完成！");
        println!("   📊 数据分析器 - 实时趋势分析");
        println!("   🔍 异常检测器 - 智能异常识别");
        println!("   🚨 智能报警器 - 自动报警通知");

        Ok(Self {
            analyzer,
            detector,
            alerter,
            data_buffer: VecDeque::with_capacity(1000),
            anomaly_history: Vec::new(),
            processing_stats: ProcessingStats {
                total_processed: 0,
                anomalies_detected: 0,
                processing_rate: 0.0,
                average_latency: Duration::from_millis(0),
                uptime: Duration::from_secs(0),
            },
            start_time: Instant::now(),
            thresholds,
        })
    }

    /// 开始实时数据处理
    async fn start_processing(&mut self) -> Result<()> {
        println!("\n🔄 开始实时数据处理...");

        // 模拟实时数据流处理
        for i in 0..20 {
            // 生成模拟数据
            let data_point = self.generate_sample_data(i).await;

            // 处理数据点
            self.process_data_point(data_point).await?;

            // 短暂延迟模拟实时处理
            sleep(Duration::from_millis(200)).await;
        }

        // 生成最终报告
        self.generate_processing_report().await?;

        println!("✅ 实时数据处理完成！");
        Ok(())
    }

    /// 生成模拟数据
    async fn generate_sample_data(&self, index: usize) -> DataPoint {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let sources = [
            "server-01",
            "server-02",
            "server-03",
            "database",
            "api-gateway",
        ];
        let source = sources[index % sources.len()].to_string();

        // 生成带有一些异常的模拟数据
        let base_value = match source.as_str() {
            "server-01" => 45.0 + (index as f64 * 2.0) % 30.0,
            "server-02" => 60.0 + (index as f64 * 1.5) % 25.0,
            "server-03" => 35.0 + (index as f64 * 3.0) % 40.0,
            "database" => 70.0 + (index as f64 * 0.8) % 20.0,
            "api-gateway" => 25.0 + (index as f64 * 4.0) % 50.0,
            _ => 50.0,
        };

        // 偶尔注入异常值
        let value = if index % 7 == 0 {
            base_value * 1.8 // 异常高值
        } else if index % 11 == 0 {
            base_value * 0.3 // 异常低值
        } else {
            base_value
        };

        let mut metadata = HashMap::new();
        metadata.insert("metric_type".to_string(), "cpu_usage".to_string());
        metadata.insert("unit".to_string(), "percentage".to_string());
        metadata.insert("region".to_string(), "us-west-2".to_string());

        DataPoint {
            timestamp,
            value,
            source,
            metadata,
        }
    }

    /// 处理单个数据点
    async fn process_data_point(&mut self, data_point: DataPoint) -> Result<()> {
        let process_start = Instant::now();

        println!(
            "📊 处理数据: {} = {:.2} (来源: {})",
            data_point.timestamp, data_point.value, data_point.source
        );

        // 添加到缓冲区
        self.data_buffer.push_back(data_point.clone());
        if self.data_buffer.len() > 100 {
            self.data_buffer.pop_front();
        }

        // 分析数据趋势
        self.analyze_data_trends(&data_point).await?;

        // 检测异常
        let anomaly_result = self.detect_anomalies(&data_point).await?;

        // 处理异常
        if anomaly_result.is_anomaly {
            self.handle_anomaly(&data_point, &anomaly_result).await?;
        }

        // 更新统计信息
        self.update_processing_stats(process_start.elapsed());

        Ok(())
    }

    /// 分析数据趋势
    async fn analyze_data_trends(&self, data_point: &DataPoint) -> Result<()> {
        if self.data_buffer.len() < 5 {
            return Ok(()); // 需要足够的历史数据
        }

        let recent_values: Vec<f64> = self
            .data_buffer
            .iter()
            .rev()
            .take(10)
            .map(|dp| dp.value)
            .collect();

        let analysis_prompt = format!(
            "请分析以下实时数据趋势：

            当前值: {:.2}
            数据源: {}
            最近10个值: {:?}
            
            请提供：
            1. 趋势分析（上升/下降/稳定）
            2. 变化幅度评估
            3. 是否存在周期性模式
            4. 数据质量评估",
            data_point.value, data_point.source, recent_values
        );

        let messages = vec![user_message(&analysis_prompt)];
        let options = AgentGenerateOptions::default();
        let _response = self.analyzer.generate(&messages, &options).await?;

        Ok(())
    }

    /// 检测异常
    async fn detect_anomalies(&self, data_point: &DataPoint) -> Result<AnomalyResult> {
        // 简单的阈值检测
        let threshold = self.thresholds.get("cpu_usage").unwrap_or(&75.0);
        let is_threshold_anomaly = data_point.value > *threshold;

        // 统计异常检测（基于历史数据）
        let is_statistical_anomaly = if self.data_buffer.len() >= 10 {
            let recent_values: Vec<f64> = self
                .data_buffer
                .iter()
                .filter(|dp| dp.source == data_point.source)
                .map(|dp| dp.value)
                .collect();

            if recent_values.len() >= 5 {
                let mean: f64 = recent_values.iter().sum::<f64>() / recent_values.len() as f64;
                let variance: f64 = recent_values
                    .iter()
                    .map(|v| (v - mean).powi(2))
                    .sum::<f64>()
                    / recent_values.len() as f64;
                let std_dev = variance.sqrt();

                (data_point.value - mean).abs() > 2.0 * std_dev
            } else {
                false
            }
        } else {
            false
        };

        let is_anomaly = is_threshold_anomaly || is_statistical_anomaly;

        if is_anomaly {
            let detection_prompt = format!(
                "检测到数据异常，请分析：

                数据值: {:.2}
                数据源: {}
                阈值: {:.2}
                超出阈值: {}
                统计异常: {}

                请评估：
                1. 异常的严重程度
                2. 可能的原因
                3. 建议的处理措施
                4. 是否需要立即报警",
                data_point.value,
                data_point.source,
                threshold,
                is_threshold_anomaly,
                is_statistical_anomaly
            );

            let messages = vec![user_message(&detection_prompt)];
            let options = AgentGenerateOptions::default();
            let response = self.detector.generate(&messages, &options).await?;

            // 确定严重程度
            let severity = if data_point.value > threshold * 1.5 {
                AnomalySeverity::Critical
            } else if data_point.value > threshold * 1.2 {
                AnomalySeverity::High
            } else if data_point.value > threshold * 1.1 {
                AnomalySeverity::Medium
            } else {
                AnomalySeverity::Low
            };

            Ok(AnomalyResult {
                is_anomaly: true,
                confidence: 0.85,
                severity,
                description: format!("数据值 {:.2} 超出正常范围", data_point.value),
                recommended_action: "监控系统状态，必要时进行干预".to_string(),
            })
        } else {
            Ok(AnomalyResult {
                is_anomaly: false,
                confidence: 0.95,
                severity: AnomalySeverity::Low,
                description: "数据正常".to_string(),
                recommended_action: "继续监控".to_string(),
            })
        }
    }

    /// 处理异常
    async fn handle_anomaly(
        &mut self,
        data_point: &DataPoint,
        anomaly: &AnomalyResult,
    ) -> Result<()> {
        println!(
            "🚨 检测到异常: {} - {}",
            data_point.source, anomaly.description
        );

        // 生成报警
        let alert_prompt = format!(
            "生成异常报警信息：

            数据源: {}
            异常值: {:.2}
            严重程度: {:?}
            置信度: {:.2}
            描述: {}
            建议措施: {}

            请生成：
            1. 详细的报警消息
            2. 紧急响应建议
            3. 后续监控要点
            4. 预防措施建议",
            data_point.source,
            data_point.value,
            anomaly.severity,
            anomaly.confidence,
            anomaly.description,
            anomaly.recommended_action
        );

        let messages = vec![user_message(&alert_prompt)];
        let options = AgentGenerateOptions::default();
        let response = self.alerter.generate(&messages, &options).await?;

        // 记录异常
        self.anomaly_history.push(anomaly.clone());
        self.processing_stats.anomalies_detected += 1;

        // 根据严重程度采取不同的响应
        match anomaly.severity {
            AnomalySeverity::Critical => {
                println!("🔴 严重异常！立即响应");
                // 在实际应用中，这里会发送紧急通知
            }
            AnomalySeverity::High => {
                println!("🟠 高级异常！需要关注");
                // 发送高优先级报警
            }
            AnomalySeverity::Medium => {
                println!("🟡 中级异常！持续监控");
                // 记录并监控
            }
            AnomalySeverity::Low => {
                println!("🟢 低级异常！正常记录");
                // 仅记录
            }
        }

        Ok(())
    }

    /// 更新处理统计信息
    fn update_processing_stats(&mut self, processing_time: Duration) {
        self.processing_stats.total_processed += 1;

        // 计算平均延迟
        let total_latency = self.processing_stats.average_latency.as_nanos() as f64
            * (self.processing_stats.total_processed - 1) as f64
            + processing_time.as_nanos() as f64;
        self.processing_stats.average_latency = Duration::from_nanos(
            (total_latency / self.processing_stats.total_processed as f64) as u64,
        );

        // 计算处理速率
        self.processing_stats.uptime = self.start_time.elapsed();
        self.processing_stats.processing_rate = self.processing_stats.total_processed as f64
            / self.processing_stats.uptime.as_secs_f64();
    }

    /// 生成处理报告
    async fn generate_processing_report(&self) -> Result<()> {
        println!("\n📋 生成实时处理报告...");

        let report_prompt = format!(
            "请生成实时数据处理报告：

            处理统计:
            - 总处理数据点: {}
            - 检测异常数: {}
            - 处理速率: {:.2} 点/秒
            - 平均延迟: {:.2} ms
            - 运行时间: {:.2} 秒

            异常分布:
            - 严重异常: {}
            - 高级异常: {}
            - 中级异常: {}
            - 低级异常: {}

            请提供：
            1. 系统性能评估
            2. 异常趋势分析
            3. 系统优化建议
            4. 监控改进建议",
            self.processing_stats.total_processed,
            self.processing_stats.anomalies_detected,
            self.processing_stats.processing_rate,
            self.processing_stats.average_latency.as_millis(),
            self.processing_stats.uptime.as_secs_f64(),
            self.anomaly_history
                .iter()
                .filter(|a| a.severity == AnomalySeverity::Critical)
                .count(),
            self.anomaly_history
                .iter()
                .filter(|a| a.severity == AnomalySeverity::High)
                .count(),
            self.anomaly_history
                .iter()
                .filter(|a| a.severity == AnomalySeverity::Medium)
                .count(),
            self.anomaly_history
                .iter()
                .filter(|a| a.severity == AnomalySeverity::Low)
                .count()
        );

        let messages = vec![user_message(&report_prompt)];
        let options = AgentGenerateOptions::default();
        let response = self.analyzer.generate(&messages, &options).await?;

        println!("✅ 处理报告生成完成！");
        Ok(())
    }

    /// 显示处理统计
    fn show_processing_stats(&self) {
        println!("\n📊 实时处理统计:");
        println!(
            "   📈 总处理数据点: {}",
            self.processing_stats.total_processed
        );
        println!(
            "   🚨 检测异常数: {}",
            self.processing_stats.anomalies_detected
        );
        println!(
            "   ⚡ 处理速率: {:.2} 点/秒",
            self.processing_stats.processing_rate
        );
        println!(
            "   ⏱️  平均延迟: {:.2} ms",
            self.processing_stats.average_latency.as_millis()
        );
        println!(
            "   🕐 运行时间: {:.2} 秒",
            self.processing_stats.uptime.as_secs_f64()
        );
        println!(
            "   📊 异常率: {:.1}%",
            (self.processing_stats.anomalies_detected as f64
                / self.processing_stats.total_processed as f64)
                * 100.0
        );

        if !self.anomaly_history.is_empty() {
            println!("\n🚨 异常分布:");
            let critical = self
                .anomaly_history
                .iter()
                .filter(|a| a.severity == AnomalySeverity::Critical)
                .count();
            let high = self
                .anomaly_history
                .iter()
                .filter(|a| a.severity == AnomalySeverity::High)
                .count();
            let medium = self
                .anomaly_history
                .iter()
                .filter(|a| a.severity == AnomalySeverity::Medium)
                .count();
            let low = self
                .anomaly_history
                .iter()
                .filter(|a| a.severity == AnomalySeverity::Low)
                .count();

            println!("   🔴 严重: {} 个", critical);
            println!("   🟠 高级: {} 个", high);
            println!("   🟡 中级: {} 个", medium);
            println!("   🟢 低级: {} 个", low);
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🤖 LumosAI 实时数据处理示例");
    println!("================================");

    // 创建实时数据处理引擎
    let mut processor = RealtimeDataProcessor::new().await?;

    // 开始实时数据处理
    processor.start_processing().await?;

    // 显示处理统计
    processor.show_processing_stats();

    println!("\n🎉 实时数据处理示例运行完成！");
    println!("💡 这个示例展示了如何使用 LumosAI 构建智能的实时数据处理系统，");
    println!("   实现数据流分析、异常检测、智能报警和性能监控。");

    Ok(())
}
