//! Lumos.ai企业级功能扩展模块
//!
//! 提供企业级监控、安全、合规、多租户等高级功能。

pub mod alerting;
pub mod anomaly_detection;
pub mod capacity_planning;
pub mod compliance;
pub mod config;
pub mod cost_tracking;
pub mod error;
pub mod incident_management;
pub mod monitoring;
pub mod multi_tenant;
pub mod reporting;
pub mod security;
pub mod sla_monitoring;

// Re-export main types
pub use alerting::{AlertRule, AlertingSystem, NotificationChannel};
pub use anomaly_detection::{AnomalyAlert, AnomalyDetector, MLAnomalyEngine};
pub use capacity_planning::{CapacityMetrics, CapacityPlanner, ScalingRecommendation};
pub use compliance::{AuditManager, ComplianceManager};
pub use config::ComplianceStandard;
pub use config::EnterpriseConfig;
pub use cost_tracking::{BillingManager, CostMetrics, CostTracker};
pub use error::{EnterpriseError, Result};
pub use incident_management::{Incident, IncidentManager, IncidentResponse};
pub use monitoring::{ComplianceMonitor, EnterpriseMetric, EnterpriseMonitoring};
pub use multi_tenant::{MultiTenantManager, ResourceLimits, Tenant, TenantConfig, TenantStatus};
pub use reporting::{ComplianceReport, PerformanceReport, ReportGenerator};
pub use security::{SecurityFramework, SecurityPolicy, ThreatDetectionEngine};
pub use sla_monitoring::{SLAMetrics, SLAMonitor, ServiceLevelAgreement};

/// 企业级监控快速设置
///
/// # Example
///
/// ```rust
/// use lumosai_enterprise::quick_setup_enterprise;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let enterprise = quick_setup_enterprise().await?;
///
///     // 启动监控
///     enterprise.start_monitoring().await?;
///
///     Ok(())
/// }
/// ```
pub async fn quick_setup_enterprise() -> Result<EnterpriseMonitoring> {
    let config = EnterpriseConfig::default();
    EnterpriseMonitoring::new(config).await
}

/// 企业级功能构建器
///
/// # Example
///
/// ```rust
/// use lumosai_enterprise::EnterpriseBuilder;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let enterprise = EnterpriseBuilder::new()
///         .enable_compliance_monitoring(true)
///         .enable_security_auditing(true)
///         .enable_cost_tracking(true)
///         .enable_sla_monitoring(true)
///         .enable_anomaly_detection(true)
///         .database_url("postgresql://user:pass@localhost/enterprise")
///         .redis_url("redis://localhost:6379")
///         .build()
///         .await?;
///
///     Ok(())
/// }
/// ```
pub struct EnterpriseBuilder {
    config: EnterpriseConfig,
}

impl EnterpriseBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            config: EnterpriseConfig::default(),
        }
    }

    /// 启用合规监控
    pub fn enable_compliance_monitoring(mut self, enabled: bool) -> Self {
        self.config.compliance_monitoring_enabled = enabled;
        self
    }

    /// 启用安全审计
    pub fn enable_security_auditing(mut self, enabled: bool) -> Self {
        self.config.security_auditing_enabled = enabled;
        self
    }

    /// 启用成本跟踪
    pub fn enable_cost_tracking(mut self, enabled: bool) -> Self {
        self.config.cost_tracking_enabled = enabled;
        self
    }

    /// 启用SLA监控
    pub fn enable_sla_monitoring(mut self, enabled: bool) -> Self {
        self.config.sla_monitoring_enabled = enabled;
        self
    }

    /// 启用异常检测
    pub fn enable_anomaly_detection(mut self, enabled: bool) -> Self {
        self.config.anomaly_detection_enabled = enabled;
        self
    }

    /// 设置数据库URL
    pub fn database_url(mut self, url: impl Into<String>) -> Self {
        self.config.database_url = url.into();
        self
    }

    /// 设置Redis URL
    pub fn redis_url(mut self, url: impl Into<String>) -> Self {
        self.config.redis_url = Some(url.into());
        self
    }

    /// 设置Prometheus端点
    pub fn prometheus_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.config.prometheus_endpoint = Some(endpoint.into());
        self
    }

    /// 设置Jaeger端点
    pub fn jaeger_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.config.jaeger_endpoint = Some(endpoint.into());
        self
    }

    /// 构建企业级监控
    pub async fn build(self) -> Result<EnterpriseMonitoring> {
        EnterpriseMonitoring::new(self.config).await
    }
}

impl Default for EnterpriseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_quick_setup() {
        let result = quick_setup_enterprise().await;
        assert!(result.is_ok(), "快速设置应该成功");
    }

    #[tokio::test]
    async fn test_enterprise_builder() {
        let result = EnterpriseBuilder::new()
            .enable_compliance_monitoring(true)
            .enable_security_auditing(true)
            .database_url("sqlite://:memory:")
            .build()
            .await;

        assert!(result.is_ok(), "构建器应该成功创建企业级监控");
    }
}
