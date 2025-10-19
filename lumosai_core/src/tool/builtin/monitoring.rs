//! 监控告警工具模块
//!
//! 提供系统监控、性能分析和告警配置等功能。
//!
//! ## 工具列表
//!
//! - `system_monitor_tool`: 系统监控（CPU、内存、磁盘）
//! - `performance_analyzer_tool`: 性能分析（响应时间、吞吐量）
//! - `alert_config_tool`: 告警配置（阈值设置、通知规则）

use crate::error::Result;
use lumos_macro::tool;
use serde_json::{json, Value};

/// 系统监控（CPU、内存、磁盘）
#[tool(
    name = "system_monitor",
    description = "系统监控（CPU、内存、磁盘）"
)]
async fn system_monitor(
    target: String,
    metrics: Option<String>,
    interval_seconds: Option<i64>,
) -> Result<Value> {
    let metrics = metrics.unwrap_or_else(|| "all".to_string());
    let interval_seconds = interval_seconds.unwrap_or(60);

    let valid_metrics = vec!["all", "cpu", "memory", "disk", "network"];
    let metrics_lower = metrics.to_lowercase();
    if !valid_metrics.contains(&metrics_lower.as_str()) {
        return Ok(json!({
            "success": false,
            "error": format!("不支持的监控指标: {}. 支持的指标: all, cpu, memory, disk, network", metrics)
        }));
    }

    if interval_seconds < 1 || interval_seconds > 3600 {
        return Ok(json!({
            "success": false,
            "error": format!("监控间隔必须在 1-3600 秒之间，当前值: {}", interval_seconds)
        }));
    }

    // Mock 实现：生成模拟的监控数据
    let mut result = json!({
        "success": true,
        "target": target,
        "metrics_type": metrics_lower,
        "interval_seconds": interval_seconds,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    if metrics_lower == "all" || metrics_lower == "cpu" {
        result["cpu"] = json!({
            "usage_percent": 45.2,
            "cores": 8,
            "load_average": [2.1, 1.8, 1.5],
            "temperature_celsius": 65.0,
            "status": "normal"
        });
    }

    if metrics_lower == "all" || metrics_lower == "memory" {
        result["memory"] = json!({
            "total_gb": 16.0,
            "used_gb": 8.5,
            "free_gb": 7.5,
            "usage_percent": 53.1,
            "swap_total_gb": 4.0,
            "swap_used_gb": 0.5,
            "status": "normal"
        });
    }

    if metrics_lower == "all" || metrics_lower == "disk" {
        result["disk"] = json!({
            "total_gb": 512.0,
            "used_gb": 256.0,
            "free_gb": 256.0,
            "usage_percent": 50.0,
            "read_iops": 150,
            "write_iops": 80,
            "status": "normal"
        });
    }

    if metrics_lower == "all" || metrics_lower == "network" {
        result["network"] = json!({
            "rx_mbps": 12.5,
            "tx_mbps": 8.3,
            "connections": 245,
            "errors": 0,
            "status": "normal"
        });
    }

    Ok(result)
}

/// 性能分析（响应时间、吞吐量）
#[tool(
    name = "performance_analyzer",
    description = "性能分析（响应时间、吞吐量）"
)]
async fn performance_analyzer(
    service_name: String,
    time_range_minutes: Option<i64>,
    metrics: Option<String>,
) -> Result<Value> {
    let time_range_minutes = time_range_minutes.unwrap_or(60);
    let metrics = metrics.unwrap_or_else(|| "all".to_string());

    let valid_metrics = vec!["all", "response_time", "throughput", "error_rate", "latency"];
    let metrics_lower = metrics.to_lowercase();
    if !valid_metrics.contains(&metrics_lower.as_str()) {
        return Ok(json!({
            "success": false,
            "error": format!("不支持的性能指标: {}. 支持的指标: all, response_time, throughput, error_rate, latency", metrics)
        }));
    }

    if time_range_minutes < 1 || time_range_minutes > 1440 {
        return Ok(json!({
            "success": false,
            "error": format!("时间范围必须在 1-1440 分钟之间，当前值: {}", time_range_minutes)
        }));
    }

    // Mock 实现：生成模拟的性能数据
    let mut result = json!({
        "success": true,
        "service_name": service_name,
        "time_range_minutes": time_range_minutes,
        "metrics_type": metrics_lower,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    if metrics_lower == "all" || metrics_lower == "response_time" {
        result["response_time"] = json!({
            "avg_ms": 125.5,
            "min_ms": 45.0,
            "max_ms": 850.0,
            "p50_ms": 110.0,
            "p95_ms": 320.0,
            "p99_ms": 650.0,
            "status": "good"
        });
    }

    if metrics_lower == "all" || metrics_lower == "throughput" {
        result["throughput"] = json!({
            "requests_per_second": 1250.0,
            "requests_total": 4500000,
            "peak_rps": 2100.0,
            "status": "good"
        });
    }

    if metrics_lower == "all" || metrics_lower == "error_rate" {
        result["error_rate"] = json!({
            "error_percent": 0.15,
            "total_errors": 6750,
            "error_types": {
                "4xx": 5400,
                "5xx": 1350
            },
            "status": "good"
        });
    }

    if metrics_lower == "all" || metrics_lower == "latency" {
        result["latency"] = json!({
            "network_ms": 15.2,
            "processing_ms": 85.3,
            "database_ms": 25.0,
            "total_ms": 125.5,
            "status": "good"
        });
    }

    Ok(result)
}

/// 告警配置（阈值设置、通知规则）
#[tool(
    name = "alert_config",
    description = "告警配置（阈值设置、通知规则）"
)]
async fn alert_config(
    alert_name: String,
    metric: String,
    threshold: i64,
    operator: String,
    notification_channels: Option<String>,
    enabled: Option<bool>,
) -> Result<Value> {
    let enabled = enabled.unwrap_or(true);
    let notification_channels = notification_channels.unwrap_or_else(|| "email".to_string());

    let valid_metrics = vec!["cpu_usage", "memory_usage", "disk_usage", "response_time", "error_rate"];
    let metric_lower = metric.to_lowercase();
    if !valid_metrics.contains(&metric_lower.as_str()) {
        return Ok(json!({
            "success": false,
            "error": format!("不支持的告警指标: {}. 支持的指标: cpu_usage, memory_usage, disk_usage, response_time, error_rate", metric)
        }));
    }

    let valid_operators = vec![">", ">=", "<", "<=", "==", "!="];
    if !valid_operators.contains(&operator.as_str()) {
        return Ok(json!({
            "success": false,
            "error": format!("不支持的比较操作符: {}. 支持的操作符: >, >=, <, <=, ==, !=", operator)
        }));
    }

    let valid_channels = vec!["email", "sms", "slack", "webhook", "pagerduty"];
    let channels: Vec<&str> = notification_channels.split(',').map(|s| s.trim()).collect();
    for channel in &channels {
        if !valid_channels.contains(channel) {
            return Ok(json!({
                "success": false,
                "error": format!("不支持的通知渠道: {}. 支持的渠道: email, sms, slack, webhook, pagerduty", channel)
            }));
        }
    }

    // Mock 实现：生成告警配置
    let alert_id = format!("alert_{}", chrono::Utc::now().timestamp());
    
    Ok(json!({
        "success": true,
        "alert_id": alert_id,
        "alert_name": alert_name,
        "config": {
            "metric": metric_lower,
            "threshold": threshold,
            "operator": operator,
            "notification_channels": channels,
            "enabled": enabled
        },
        "status": if enabled { "active" } else { "inactive" },
        "created_at": chrono::Utc::now().to_rfc3339(),
        "message": format!("告警规则已创建: 当 {} {} {} 时触发", metric_lower, operator, threshold)
    }))
}

/// 获取所有监控告警工具
pub fn get_all_monitoring_tools() -> Vec<Box<dyn crate::tool::Tool>> {
    vec![
        system_monitor_tool(),
        performance_analyzer_tool(),
        alert_config_tool(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::{ToolExecutionContext, ToolExecutionOptions};

    #[tokio::test]
    async fn test_system_monitor() {
        let tool = system_monitor_tool();
        let context = ToolExecutionContext::default();
        let options = ToolExecutionOptions::default();

        let params = json!({
            "target": "server-01",
            "metrics": "cpu"
        });

        let result = tool.execute(params, context, &options).await.unwrap();
        assert_eq!(result["success"], true);
        assert!(result["cpu"].is_object());
    }

    #[tokio::test]
    async fn test_performance_analyzer() {
        let tool = performance_analyzer_tool();
        let context = ToolExecutionContext::default();
        let options = ToolExecutionOptions::default();

        let params = json!({
            "service_name": "api-service",
            "time_range_minutes": 60
        });

        let result = tool.execute(params, context, &options).await.unwrap();
        assert_eq!(result["success"], true);
    }

    #[tokio::test]
    async fn test_alert_config() {
        let tool = alert_config_tool();
        let context = ToolExecutionContext::default();
        let options = ToolExecutionOptions::default();

        let params = json!({
            "alert_name": "High CPU Alert",
            "metric": "cpu_usage",
            "threshold": 80,
            "operator": ">"
        });

        let result = tool.execute(params, context, &options).await.unwrap();
        assert_eq!(result["success"], true);
        assert!(result["alert_id"].is_string());
    }
}

