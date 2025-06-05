//! 高级监控系统演示
//! 
//! 展示Lumos.ai的企业级监控功能，包括：
//! - 智能告警系统
//! - 性能分析和异常检测
//! - 自动化诊断和优化建议
//! - 趋势预测和瓶颈识别

use lumosai_core::telemetry::*;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 Lumos.ai 高级监控系统演示");
    println!("===============================\n");

    // 1. 初始化监控组件
    println!("1️⃣ 初始化高级监控组件");
    println!("------------------------");
    
    let metrics_collector = InMemoryMetricsCollector::new();
    let alert_manager = InMemoryAlertManager::new();
    let performance_analyzer = IntelligentPerformanceAnalyzer::new();
    
    println!("✅ 指标收集器初始化完成");
    println!("✅ 告警管理器初始化完成");
    println!("✅ 性能分析器初始化完成");
    println!();

    // 2. 配置告警规则
    println!("2️⃣ 配置智能告警规则");
    println!("--------------------");
    
    // 响应时间告警规则
    let response_time_rule = AlertRule {
        id: "response-time-alert".to_string(),
        name: "响应时间告警".to_string(),
        description: "当代理响应时间超过阈值时触发告警".to_string(),
        condition: AlertCondition::ResponseTime {
            threshold_ms: 1000,
            window_minutes: 5,
            percentile: 95.0,
        },
        severity: AlertSeverity::Warning,
        enabled: true,
        cooldown_duration: Duration::from_secs(300),
        channels: vec!["email".to_string(), "slack".to_string()],
        labels: {
            let mut labels = HashMap::new();
            labels.insert("service".to_string(), "lumos-agent".to_string());
            labels.insert("environment".to_string(), "production".to_string());
            labels
        },
        created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
        updated_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
    };
    
    // 错误率告警规则
    let error_rate_rule = AlertRule {
        id: "error-rate-alert".to_string(),
        name: "错误率告警".to_string(),
        description: "当错误率超过阈值时触发告警".to_string(),
        condition: AlertCondition::ErrorRate {
            threshold_percent: 5.0,
            window_minutes: 10,
            min_requests: 10,
        },
        severity: AlertSeverity::Error,
        enabled: true,
        cooldown_duration: Duration::from_secs(600),
        channels: vec!["email".to_string(), "webhook".to_string()],
        labels: {
            let mut labels = HashMap::new();
            labels.insert("service".to_string(), "lumos-agent".to_string());
            labels.insert("critical".to_string(), "true".to_string());
            labels
        },
        created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
        updated_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
    };
    
    alert_manager.add_rule(response_time_rule).await?;
    alert_manager.add_rule(error_rate_rule).await?;
    
    println!("✅ 响应时间告警规则已配置 (阈值: 1000ms)");
    println!("✅ 错误率告警规则已配置 (阈值: 5%)");
    println!();

    // 3. 配置告警通道
    println!("3️⃣ 配置告警通知渠道");
    println!("--------------------");
    
    let email_channel = AlertChannel {
        id: "email".to_string(),
        name: "邮件通知".to_string(),
        channel_type: AlertChannelType::Email,
        config: serde_json::json!({
            "smtp_server": "smtp.example.com",
            "port": 587,
            "username": "alerts@lumos.ai",
            "recipients": ["admin@lumos.ai", "ops@lumos.ai"]
        }),
        enabled: true,
    };
    
    let slack_channel = AlertChannel {
        id: "slack".to_string(),
        name: "Slack通知".to_string(),
        channel_type: AlertChannelType::Slack,
        config: serde_json::json!({
            "webhook_url": "https://hooks.slack.com/services/...",
            "channel": "#alerts",
            "username": "Lumos Alert Bot"
        }),
        enabled: true,
    };
    
    let webhook_channel = AlertChannel {
        id: "webhook".to_string(),
        name: "Webhook通知".to_string(),
        channel_type: AlertChannelType::Webhook,
        config: serde_json::json!({
            "url": "https://api.example.com/alerts",
            "method": "POST",
            "headers": {
                "Authorization": "Bearer token123",
                "Content-Type": "application/json"
            }
        }),
        enabled: true,
    };
    
    alert_manager.add_channel(email_channel).await?;
    alert_manager.add_channel(slack_channel).await?;
    alert_manager.add_channel(webhook_channel).await?;
    
    println!("✅ 邮件通知渠道已配置");
    println!("✅ Slack通知渠道已配置");
    println!("✅ Webhook通知渠道已配置");
    println!();

    // 4. 生成模拟监控数据
    println!("4️⃣ 生成模拟监控数据");
    println!("--------------------");
    
    let mut agent_metrics = Vec::new();
    
    for i in 0..50 {
        let execution_context = ExecutionContext {
            user_id: Some(format!("user_{}", i % 10)),
            session_id: Some(Uuid::new_v4().to_string()),
            request_id: Some(Uuid::new_v4().to_string()),
            environment: "production".to_string(),
            version: Some("1.0.0".to_string()),
        };
        
        let mut metrics = AgentMetrics::new(
            format!("ai-assistant-{}", i % 3),
            execution_context
        );
        
        // 模拟不同的性能场景
        if i < 20 {
            // 正常性能
            metrics.execution_time_ms = 200 + (i % 10) * 50;
            metrics.success = true;
        } else if i < 35 {
            // 性能下降
            metrics.execution_time_ms = 800 + (i % 10) * 100;
            metrics.success = i % 8 != 0; // 12.5% 错误率
        } else {
            // 严重性能问题
            metrics.execution_time_ms = 1500 + (i % 10) * 200;
            metrics.success = i % 4 != 0; // 25% 错误率
        }
        
        metrics.end_timing();
        metrics.tool_calls_count = (2 + (i % 4)) as usize;
        metrics.memory_operations = (3 + (i % 3)) as usize;
        metrics.token_usage = TokenUsage {
            prompt_tokens: (150 + i * 10) as u32,
            completion_tokens: (80 + i * 5) as u32,
            total_tokens: (230 + i * 15) as u32,
        };

        if !metrics.success {
            metrics.error_count = 1;
            // 注意：AgentMetrics 没有 error_message 字段，我们可以在其他地方记录错误信息
        }
        
        metrics_collector.record_agent_execution(metrics.clone()).await?;
        agent_metrics.push(metrics);
        
        if (i + 1) % 10 == 0 {
            println!("✅ 已生成 {} 条监控数据", i + 1);
        }
    }
    
    println!("✅ 总共生成了 50 条代理执行数据");
    println!();

    // 5. 执行性能分析
    println!("5️⃣ 智能性能分析");
    println!("----------------");
    
    let time_range = TimeRange {
        start: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64 - 3600000, // 1小时前
        end: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
    };
    
    let analysis = performance_analyzer.analyze(&agent_metrics, time_range).await?;
    
    println!("📊 性能分析结果:");
    println!("   • 整体性能评分: {:.1}/100", analysis.overall_score);
    
    match &analysis.trend {
        PerformanceTrend::Improving { rate } => {
            println!("   • 性能趋势: 📈 改善中 (改善率: {:.2})", rate);
        },
        PerformanceTrend::Stable { variance } => {
            println!("   • 性能趋势: ➡️ 稳定 (方差: {:.2})", variance);
        },
        PerformanceTrend::Degrading { rate } => {
            println!("   • 性能趋势: 📉 下降中 (下降率: {:.2})", rate);
        },
        PerformanceTrend::Volatile { amplitude } => {
            println!("   • 性能趋势: 📊 波动 (波动幅度: {:.2})", amplitude);
        },
    }
    
    println!("   • 检测到 {} 个性能瓶颈", analysis.bottlenecks.len());
    println!("   • 检测到 {} 个异常", analysis.anomalies.len());
    println!("   • 生成了 {} 条优化建议", analysis.recommendations.len());
    println!("   • 生成了 {} 个性能预测", analysis.predictions.len());
    println!();

    // 6. 显示瓶颈分析
    if !analysis.bottlenecks.is_empty() {
        println!("6️⃣ 性能瓶颈分析");
        println!("----------------");
        
        for (i, bottleneck) in analysis.bottlenecks.iter().enumerate() {
            println!("🔍 瓶颈 #{}: {:?}", i + 1, bottleneck.bottleneck_type);
            println!("   • 严重程度: {:.1}/100", bottleneck.severity);
            println!("   • 影响描述: {}", bottleneck.impact);
            println!("   • 解决方案:");
            for (j, solution) in bottleneck.solutions.iter().enumerate() {
                println!("     {}. {}", j + 1, solution);
            }
            println!();
        }
    }

    // 7. 显示异常检测结果
    if !analysis.anomalies.is_empty() {
        println!("7️⃣ 异常检测结果");
        println!("----------------");
        
        for (i, anomaly) in analysis.anomalies.iter().enumerate() {
            println!("⚠️ 异常 #{}: {:?}", i + 1, anomaly.anomaly_type);
            println!("   • 异常值: {:.1}", anomaly.value);
            println!("   • 期望值: {:.1}", anomaly.expected_value);
            println!("   • 偏差程度: {:.2}σ", anomaly.deviation);
            println!("   • 置信度: {:.1}%", anomaly.confidence * 100.0);
            println!("   • 描述: {}", anomaly.description);
            println!();
        }
    }

    // 8. 显示优化建议
    if !analysis.recommendations.is_empty() {
        println!("8️⃣ 智能优化建议");
        println!("----------------");
        
        for (i, rec) in analysis.recommendations.iter().take(3).enumerate() {
            println!("💡 建议 #{}: {}", i + 1, rec.title);
            println!("   • 优先级: {}/10", rec.priority);
            println!("   • 类型: {:?}", rec.recommendation_type);
            println!("   • 描述: {}", rec.description);
            println!("   • 预期收益: {}", rec.expected_benefit);
            println!("   • 实施难度: {:?}", rec.implementation_difficulty);
            println!("   • 实施步骤:");
            for (j, step) in rec.steps.iter().enumerate() {
                println!("     {}. {}", j + 1, step);
            }
            if !rec.risks.is_empty() {
                println!("   • 风险评估:");
                for risk in &rec.risks {
                    println!("     ⚠️ {}", risk);
                }
            }
            println!();
        }
    }

    println!("🎉 高级监控系统演示完成！");
    println!("==========================");
    
    println!("\n🚀 企业级监控功能亮点:");
    println!("   • ✅ 智能告警系统 - 多维度条件检测");
    println!("   • ✅ 性能分析引擎 - 自动瓶颈识别");
    println!("   • ✅ 异常检测算法 - 统计学异常识别");
    println!("   • ✅ 优化建议生成 - AI驱动的改进建议");
    println!("   • ✅ 趋势预测分析 - 前瞻性性能预测");
    println!("   • ✅ 多渠道告警通知 - 邮件/Slack/Webhook");
    println!("   • ✅ 自动化诊断系统 - 根因分析和修复建议");
    
    println!("\n🔒 生产级监控系统已就绪！");

    Ok(())
}
