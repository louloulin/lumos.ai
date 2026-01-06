//! 监控告警工具演示程序

use lumosai_core::tool::builtin::monitoring::*;
use lumosai_core::tool::{ToolExecutionContext, ToolExecutionOptions};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("📊 监控告警工具演示\n");
    println!("{}", "=".repeat(80));
    println!();

    let context = ToolExecutionContext::default();
    let options = ToolExecutionOptions::default();

    // 测试 1: 系统监控
    println!("🖥️  测试 1: 系统监控");
    println!("{}", "-".repeat(80));

    let tools = get_all_monitoring_tools();
    let monitor_tool = &tools[0];

    println!("工具名称: {}", monitor_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", monitor_tool.description());
    println!();

    let monitoring_scenarios = vec![
        ("场景 1: 监控所有指标", "server-01", "all"),
        ("场景 2: 仅监控 CPU", "server-02", "cpu"),
        ("场景 3: 仅监控内存", "server-03", "memory"),
        ("场景 4: 仅监控磁盘", "server-04", "disk"),
    ];

    for (desc, target, metrics) in monitoring_scenarios {
        println!("{}", desc);

        let params = json!({
            "target": target,
            "metrics": metrics,
            "interval_seconds": 60
        });

        match monitor_tool
            .execute(params, context.clone(), &options)
            .await
        {
            Ok(result) => {
                if result["success"] == true {
                    println!("  ✅ 监控目标: {}", result["target"]);
                    println!("  监控类型: {}", result["metrics_type"]);
                    println!("  监控间隔: {}秒", result["interval_seconds"]);

                    if let Some(cpu) = result.get("cpu") {
                        println!("  📈 CPU:");
                        println!("    使用率: {}%", cpu["usage_percent"]);
                        println!("    核心数: {}", cpu["cores"]);
                        println!("    温度: {}°C", cpu["temperature_celsius"]);
                        println!("    状态: {}", cpu["status"]);
                    }

                    if let Some(memory) = result.get("memory") {
                        println!("  💾 内存:");
                        println!("    总容量: {} GB", memory["total_gb"]);
                        println!("    已使用: {} GB", memory["used_gb"]);
                        println!("    使用率: {}%", memory["usage_percent"]);
                        println!("    状态: {}", memory["status"]);
                    }

                    if let Some(disk) = result.get("disk") {
                        println!("  💿 磁盘:");
                        println!("    总容量: {} GB", disk["total_gb"]);
                        println!("    已使用: {} GB", disk["used_gb"]);
                        println!("    使用率: {}%", disk["usage_percent"]);
                        println!("    状态: {}", disk["status"]);
                    }

                    if let Some(network) = result.get("network") {
                        println!("  🌐 网络:");
                        println!("    接收: {} Mbps", network["rx_mbps"]);
                        println!("    发送: {} Mbps", network["tx_mbps"]);
                        println!("    连接数: {}", network["connections"]);
                        println!("    状态: {}", network["status"]);
                    }
                } else {
                    println!("  ❌ 监控失败: {}", result["error"]);
                }
            }
            Err(e) => println!("  ❌ 监控失败: {}", e),
        }
        println!();
    }

    // 测试 2: 性能分析
    println!("⚡ 测试 2: 性能分析");
    println!("{}", "-".repeat(80));

    let analyzer_tool = &tools[1];

    println!("工具名称: {}", analyzer_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", analyzer_tool.description());
    println!();

    let analysis_scenarios = vec![
        ("场景 1: 分析所有性能指标", "api-service", "all", 60),
        ("场景 2: 分析响应时间", "web-service", "response_time", 30),
        ("场景 3: 分析吞吐量", "data-service", "throughput", 120),
        ("场景 4: 分析错误率", "auth-service", "error_rate", 60),
    ];

    for (desc, service, metrics, time_range) in analysis_scenarios {
        println!("{}", desc);

        let params = json!({
            "service_name": service,
            "time_range_minutes": time_range,
            "metrics": metrics
        });

        match analyzer_tool
            .execute(params, context.clone(), &options)
            .await
        {
            Ok(result) => {
                if result["success"] == true {
                    println!("  ✅ 服务名称: {}", result["service_name"]);
                    println!("  时间范围: {}分钟", result["time_range_minutes"]);

                    if let Some(response_time) = result.get("response_time") {
                        println!("  ⏱️  响应时间:");
                        println!("    平均: {} ms", response_time["avg_ms"]);
                        println!("    P50: {} ms", response_time["p50_ms"]);
                        println!("    P95: {} ms", response_time["p95_ms"]);
                        println!("    P99: {} ms", response_time["p99_ms"]);
                        println!("    状态: {}", response_time["status"]);
                    }

                    if let Some(throughput) = result.get("throughput") {
                        println!("  📊 吞吐量:");
                        println!("    RPS: {}", throughput["requests_per_second"]);
                        println!("    总请求: {}", throughput["requests_total"]);
                        println!("    峰值 RPS: {}", throughput["peak_rps"]);
                        println!("    状态: {}", throughput["status"]);
                    }

                    if let Some(error_rate) = result.get("error_rate") {
                        println!("  ❌ 错误率:");
                        println!("    错误率: {}%", error_rate["error_percent"]);
                        println!("    总错误: {}", error_rate["total_errors"]);
                        println!("    状态: {}", error_rate["status"]);
                    }
                } else {
                    println!("  ❌ 分析失败: {}", result["error"]);
                }
            }
            Err(e) => println!("  ❌ 分析失败: {}", e),
        }
        println!();
    }

    // 测试 3: 告警配置
    println!("🚨 测试 3: 告警配置");
    println!("{}", "-".repeat(80));

    let alert_tool = &tools[2];

    println!("工具名称: {}", alert_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", alert_tool.description());
    println!();

    let alert_scenarios = vec![
        (
            "场景 1: CPU 使用率告警",
            json!({
                "alert_name": "High CPU Alert",
                "metric": "cpu_usage",
                "threshold": 80,
                "operator": ">",
                "notification_channels": "email,slack"
            }),
        ),
        (
            "场景 2: 内存使用率告警",
            json!({
                "alert_name": "High Memory Alert",
                "metric": "memory_usage",
                "threshold": 90,
                "operator": ">=",
                "notification_channels": "email,sms,pagerduty"
            }),
        ),
        (
            "场景 3: 响应时间告警",
            json!({
                "alert_name": "Slow Response Alert",
                "metric": "response_time",
                "threshold": 500,
                "operator": ">",
                "notification_channels": "slack,webhook"
            }),
        ),
        (
            "场景 4: 错误率告警",
            json!({
                "alert_name": "High Error Rate Alert",
                "metric": "error_rate",
                "threshold": 5,
                "operator": ">",
                "notification_channels": "email,pagerduty",
                "enabled": true
            }),
        ),
    ];

    for (desc, params) in alert_scenarios {
        println!("{}", desc);

        match alert_tool.execute(params, context.clone(), &options).await {
            Ok(result) => {
                if result["success"] == true {
                    println!("  ✅ 告警 ID: {}", result["alert_id"]);
                    println!("  告警名称: {}", result["alert_name"]);
                    println!("  监控指标: {}", result["config"]["metric"]);
                    println!(
                        "  阈值: {} {}",
                        result["config"]["operator"], result["config"]["threshold"]
                    );
                    println!(
                        "  通知渠道: {:?}",
                        result["config"]["notification_channels"]
                    );
                    println!("  状态: {}", result["status"]);
                    println!("  消息: {}", result["message"]);
                } else {
                    println!("  ❌ 配置失败: {}", result["error"]);
                }
            }
            Err(e) => println!("  ❌ 配置失败: {}", e),
        }
        println!();
    }

    // 测试 4: 参数验证
    println!("🔍 测试 4: 参数验证");
    println!("{}", "-".repeat(80));

    println!("测试 4.1: 无效的监控指标");
    let params = json!({
        "target": "server-01",
        "metrics": "invalid_metric"
    });

    match monitor_tool
        .execute(params, context.clone(), &options)
        .await
    {
        Ok(result) => {
            if result["success"] == false {
                println!("  ✅ 正确拒绝: {}", result["error"]);
            }
        }
        Err(e) => println!("  ❌ 执行失败: {}", e),
    }
    println!();

    println!("测试 4.2: 无效的比较操作符");
    let params = json!({
        "alert_name": "Test Alert",
        "metric": "cpu_usage",
        "threshold": 80,
        "operator": "invalid"
    });

    match alert_tool.execute(params, context.clone(), &options).await {
        Ok(result) => {
            if result["success"] == false {
                println!("  ✅ 正确拒绝: {}", result["error"]);
            }
        }
        Err(e) => println!("  ❌ 执行失败: {}", e),
    }
    println!();

    // 测试 5: 批量获取工具
    println!("📦 测试 5: 获取所有监控告警工具");
    println!("{}", "-".repeat(80));

    let all_tools = get_all_monitoring_tools();
    println!("总共 {} 个监控告警工具:\n", all_tools.len());

    for (i, tool) in all_tools.iter().enumerate() {
        println!(
            "{}. {} - {}",
            i + 1,
            tool.name().unwrap_or("unknown"),
            tool.description()
        );
    }
    println!();

    // 总结
    println!("{}", "=".repeat(80));
    println!("✅ 监控告警工具演示完成！");
    println!("{}", "=".repeat(80));
    println!();
    println!("📊 工具统计:");
    println!("  - 系统监控工具: system_monitor");
    println!("  - 性能分析工具: performance_analyzer");
    println!("  - 告警配置工具: alert_config");
    println!();
    println!("💡 使用建议:");
    println!("  1. 系统监控: 实时监控服务器资源使用情况");
    println!("  2. 性能分析: 分析服务性能指标和瓶颈");
    println!("  3. 告警配置: 设置告警规则和通知渠道");
    println!();
    println!("🚀 下一步:");
    println!("  - 集成真实的监控系统（如 Prometheus, Grafana）");
    println!("  - 添加更多监控指标和告警类型");
    println!("  - 支持自定义告警规则和聚合");
    println!("{}", "=".repeat(80));
}
