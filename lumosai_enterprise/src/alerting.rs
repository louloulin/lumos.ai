//! 告警系统模块
//! 
//! 提供企业级告警和通知功能

use async_trait::async_trait;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{EnterpriseError, Result};

/// 告警系统
pub struct AlertingSystem {
    /// 告警规则
    alert_rules: HashMap<String, AlertRule>,
    
    /// 通知渠道
    notification_channels: HashMap<String, NotificationChannel>,
}

/// 告警规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// 规则ID
    pub id: String,
    
    /// 规则名称
    pub name: String,
    
    /// 指标名称
    pub metric_name: String,
    
    /// 阈值
    pub threshold: f64,
    
    /// 比较操作
    pub comparison: ComparisonOperator,
    
    /// 是否启用
    pub enabled: bool,
    
    /// 通知渠道
    pub notification_channels: Vec<String>,
}

/// 比较操作符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    Equal,
    NotEqual,
}

/// 通知渠道
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    /// 渠道ID
    pub id: String,
    
    /// 渠道名称
    pub name: String,
    
    /// 渠道类型
    pub channel_type: ChannelType,
    
    /// 配置
    pub config: HashMap<String, String>,
    
    /// 是否启用
    pub enabled: bool,
}

/// 渠道类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelType {
    Email,
    Slack,
    Webhook,
    SMS,
}

impl AlertingSystem {
    /// 创建新的告警系统
    pub fn new() -> Self {
        Self {
            alert_rules: HashMap::new(),
            notification_channels: HashMap::new(),
        }
    }
    
    /// 添加告警规则
    pub async fn add_alert_rule(&mut self, rule: AlertRule) -> Result<()> {
        self.alert_rules.insert(rule.id.clone(), rule);
        Ok(())
    }
    
    /// 添加通知渠道
    pub async fn add_notification_channel(&mut self, channel: NotificationChannel) -> Result<()> {
        self.notification_channels.insert(channel.id.clone(), channel);
        Ok(())
    }
    
    /// 检查告警
    pub async fn check_alerts(&self, metric_name: &str, value: f64) -> Result<Vec<String>> {
        let mut triggered_alerts = Vec::new();
        
        for rule in self.alert_rules.values() {
            if rule.enabled && rule.metric_name == metric_name {
                let triggered = match rule.comparison {
                    ComparisonOperator::GreaterThan => value > rule.threshold,
                    ComparisonOperator::LessThan => value < rule.threshold,
                    ComparisonOperator::Equal => (value - rule.threshold).abs() < f64::EPSILON,
                    ComparisonOperator::NotEqual => (value - rule.threshold).abs() >= f64::EPSILON,
                };
                
                if triggered {
                    triggered_alerts.push(rule.id.clone());
                }
            }
        }
        
        Ok(triggered_alerts)
    }
}

impl Default for AlertingSystem {
    fn default() -> Self {
        Self::new()
    }
}
