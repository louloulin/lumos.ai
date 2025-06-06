//! SLA监控模块
//! 
//! 提供服务级别协议监控和管理功能

use async_trait::async_trait;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{EnterpriseError, Result};

/// SLA监控器
pub struct SLAMonitor {
    /// SLA定义
    sla_definitions: HashMap<String, ServiceLevelAgreement>,
    
    /// SLA指标
    sla_metrics: HashMap<String, SLAMetrics>,
    
    /// 违约记录
    violations: Vec<SLAViolation>,
}

/// 服务级别协议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceLevelAgreement {
    /// SLA ID
    pub id: String,
    
    /// 服务名称
    pub service_name: String,
    
    /// 可用性目标（百分比）
    pub availability_target: f64,
    
    /// 响应时间目标（毫秒）
    pub response_time_target: f64,
    
    /// 错误率目标（百分比）
    pub error_rate_target: f64,
    
    /// 吞吐量目标（请求/秒）
    pub throughput_target: f64,
    
    /// 监控周期
    pub monitoring_period: MonitoringPeriod,
    
    /// 是否启用
    pub enabled: bool,
}

/// 监控周期
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitoringPeriod {
    /// 实时
    RealTime,
    /// 每分钟
    Minutely,
    /// 每小时
    Hourly,
    /// 每日
    Daily,
    /// 每月
    Monthly,
}

/// SLA指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLAMetrics {
    /// 服务名称
    pub service_name: String,
    
    /// 可用性（百分比）
    pub availability: f64,
    
    /// 平均响应时间（毫秒）
    pub avg_response_time: f64,
    
    /// 错误率（百分比）
    pub error_rate: f64,
    
    /// 吞吐量（请求/秒）
    pub throughput: f64,
    
    /// 测量时间
    pub measured_at: DateTime<Utc>,
    
    /// 测量周期
    pub measurement_period: chrono::Duration,
}

/// SLA违约记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLAViolation {
    /// 违约ID
    pub id: Uuid,
    
    /// SLA ID
    pub sla_id: String,
    
    /// 服务名称
    pub service_name: String,
    
    /// 违约类型
    pub violation_type: ViolationType,
    
    /// 目标值
    pub target_value: f64,
    
    /// 实际值
    pub actual_value: f64,
    
    /// 违约时间
    pub violation_time: DateTime<Utc>,
    
    /// 持续时间
    pub duration: chrono::Duration,
    
    /// 严重程度
    pub severity: ViolationSeverity,
}

/// 违约类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    /// 可用性违约
    Availability,
    /// 响应时间违约
    ResponseTime,
    /// 错误率违约
    ErrorRate,
    /// 吞吐量违约
    Throughput,
}

/// 违约严重程度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
    /// 严重
    Critical,
}

impl SLAMonitor {
    /// 创建新的SLA监控器
    pub fn new() -> Self {
        Self {
            sla_definitions: HashMap::new(),
            sla_metrics: HashMap::new(),
            violations: Vec::new(),
        }
    }
    
    /// 添加SLA定义
    pub async fn add_sla(&mut self, sla: ServiceLevelAgreement) -> Result<()> {
        self.sla_definitions.insert(sla.id.clone(), sla);
        Ok(())
    }
    
    /// 更新SLA指标
    pub async fn update_metrics(&mut self, metrics: SLAMetrics) -> Result<()> {
        let service_name = metrics.service_name.clone();
        
        // 检查是否有SLA违约
        if let Some(sla) = self.sla_definitions.values().find(|s| s.service_name == service_name) {
            self.check_violations(sla, &metrics).await?;
        }
        
        self.sla_metrics.insert(service_name, metrics);
        Ok(())
    }
    
    /// 检查SLA违约
    async fn check_violations(&mut self, sla: &ServiceLevelAgreement, metrics: &SLAMetrics) -> Result<()> {
        // 检查可用性
        if metrics.availability < sla.availability_target {
            let violation = SLAViolation {
                id: Uuid::new_v4(),
                sla_id: sla.id.clone(),
                service_name: sla.service_name.clone(),
                violation_type: ViolationType::Availability,
                target_value: sla.availability_target,
                actual_value: metrics.availability,
                violation_time: metrics.measured_at,
                duration: metrics.measurement_period,
                severity: self.calculate_severity(sla.availability_target, metrics.availability),
            };
            self.violations.push(violation);
        }
        
        // 检查响应时间
        if metrics.avg_response_time > sla.response_time_target {
            let violation = SLAViolation {
                id: Uuid::new_v4(),
                sla_id: sla.id.clone(),
                service_name: sla.service_name.clone(),
                violation_type: ViolationType::ResponseTime,
                target_value: sla.response_time_target,
                actual_value: metrics.avg_response_time,
                violation_time: metrics.measured_at,
                duration: metrics.measurement_period,
                severity: self.calculate_severity(sla.response_time_target, metrics.avg_response_time),
            };
            self.violations.push(violation);
        }
        
        // 检查错误率
        if metrics.error_rate > sla.error_rate_target {
            let violation = SLAViolation {
                id: Uuid::new_v4(),
                sla_id: sla.id.clone(),
                service_name: sla.service_name.clone(),
                violation_type: ViolationType::ErrorRate,
                target_value: sla.error_rate_target,
                actual_value: metrics.error_rate,
                violation_time: metrics.measured_at,
                duration: metrics.measurement_period,
                severity: self.calculate_severity(sla.error_rate_target, metrics.error_rate),
            };
            self.violations.push(violation);
        }
        
        // 检查吞吐量
        if metrics.throughput < sla.throughput_target {
            let violation = SLAViolation {
                id: Uuid::new_v4(),
                sla_id: sla.id.clone(),
                service_name: sla.service_name.clone(),
                violation_type: ViolationType::Throughput,
                target_value: sla.throughput_target,
                actual_value: metrics.throughput,
                violation_time: metrics.measured_at,
                duration: metrics.measurement_period,
                severity: self.calculate_severity(sla.throughput_target, metrics.throughput),
            };
            self.violations.push(violation);
        }
        
        Ok(())
    }
    
    /// 计算违约严重程度
    fn calculate_severity(&self, target: f64, actual: f64) -> ViolationSeverity {
        let deviation = (actual - target).abs() / target;
        
        if deviation > 0.5 {
            ViolationSeverity::Critical
        } else if deviation > 0.3 {
            ViolationSeverity::High
        } else if deviation > 0.1 {
            ViolationSeverity::Medium
        } else {
            ViolationSeverity::Low
        }
    }
    
    /// 获取SLA违约记录
    pub async fn get_violations(&self, service_name: Option<&str>) -> Result<Vec<SLAViolation>> {
        if let Some(name) = service_name {
            Ok(self.violations.iter()
                .filter(|v| v.service_name == name)
                .cloned()
                .collect())
        } else {
            Ok(self.violations.clone())
        }
    }
    
    /// 获取SLA合规性报告
    pub async fn get_compliance_report(&self, service_name: &str) -> Result<SLAComplianceReport> {
        let sla = self.sla_definitions.values()
            .find(|s| s.service_name == service_name)
            .ok_or_else(|| EnterpriseError::SlaMonitoring(format!("SLA not found for service: {}", service_name)))?;
        
        let metrics = self.sla_metrics.get(service_name)
            .ok_or_else(|| EnterpriseError::SlaMonitoring(format!("No metrics found for service: {}", service_name)))?;
        
        let violations = self.violations.iter()
            .filter(|v| v.service_name == service_name)
            .cloned()
            .collect();
        
        Ok(SLAComplianceReport {
            service_name: service_name.to_string(),
            sla_target: sla.clone(),
            current_metrics: metrics.clone(),
            violations,
            compliance_score: self.calculate_compliance_score(sla, metrics),
            report_generated_at: Utc::now(),
        })
    }
    
    /// 计算合规性分数
    fn calculate_compliance_score(&self, sla: &ServiceLevelAgreement, metrics: &SLAMetrics) -> f64 {
        let mut score = 0.0;
        let mut total_weight = 0.0;
        
        // 可用性权重：40%
        if metrics.availability >= sla.availability_target {
            score += 40.0;
        } else {
            score += 40.0 * (metrics.availability / sla.availability_target);
        }
        total_weight += 40.0;
        
        // 响应时间权重：30%
        if metrics.avg_response_time <= sla.response_time_target {
            score += 30.0;
        } else {
            score += 30.0 * (sla.response_time_target / metrics.avg_response_time);
        }
        total_weight += 30.0;
        
        // 错误率权重：20%
        if metrics.error_rate <= sla.error_rate_target {
            score += 20.0;
        } else {
            score += 20.0 * (sla.error_rate_target / metrics.error_rate);
        }
        total_weight += 20.0;
        
        // 吞吐量权重：10%
        if metrics.throughput >= sla.throughput_target {
            score += 10.0;
        } else {
            score += 10.0 * (metrics.throughput / sla.throughput_target);
        }
        total_weight += 10.0;
        
        score / total_weight * 100.0
    }
}

/// SLA合规性报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLAComplianceReport {
    /// 服务名称
    pub service_name: String,
    
    /// SLA目标
    pub sla_target: ServiceLevelAgreement,
    
    /// 当前指标
    pub current_metrics: SLAMetrics,
    
    /// 违约记录
    pub violations: Vec<SLAViolation>,
    
    /// 合规性分数（0-100）
    pub compliance_score: f64,
    
    /// 报告生成时间
    pub report_generated_at: DateTime<Utc>,
}

impl Default for SLAMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_sla_monitoring() {
        let mut monitor = SLAMonitor::new();
        
        // 添加SLA定义
        let sla = ServiceLevelAgreement {
            id: "web_service_sla".to_string(),
            service_name: "web_service".to_string(),
            availability_target: 99.9,
            response_time_target: 200.0,
            error_rate_target: 1.0,
            throughput_target: 1000.0,
            monitoring_period: MonitoringPeriod::Minutely,
            enabled: true,
        };
        
        assert!(monitor.add_sla(sla).await.is_ok());
        
        // 更新指标（正常情况）
        let metrics = SLAMetrics {
            service_name: "web_service".to_string(),
            availability: 99.95,
            avg_response_time: 150.0,
            error_rate: 0.5,
            throughput: 1200.0,
            measured_at: Utc::now(),
            measurement_period: chrono::Duration::minutes(1),
        };
        
        assert!(monitor.update_metrics(metrics).await.is_ok());
        
        // 检查违约记录（应该没有）
        let violations = monitor.get_violations(Some("web_service")).await.unwrap();
        assert_eq!(violations.len(), 0);
        
        // 更新指标（违约情况）
        let bad_metrics = SLAMetrics {
            service_name: "web_service".to_string(),
            availability: 98.0, // 低于99.9%
            avg_response_time: 300.0, // 高于200ms
            error_rate: 2.0, // 高于1%
            throughput: 800.0, // 低于1000
            measured_at: Utc::now(),
            measurement_period: chrono::Duration::minutes(1),
        };
        
        assert!(monitor.update_metrics(bad_metrics).await.is_ok());
        
        // 检查违约记录（应该有4个）
        let violations = monitor.get_violations(Some("web_service")).await.unwrap();
        assert_eq!(violations.len(), 4);
        
        // 生成合规性报告
        let report = monitor.get_compliance_report("web_service").await.unwrap();
        assert!(report.compliance_score < 100.0);
        assert_eq!(report.violations.len(), 4);
    }
}
