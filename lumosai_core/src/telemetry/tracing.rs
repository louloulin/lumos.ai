//! OpenTelemetry Tracing 集成
//!
//! 提供与 OpenTelemetry 标准兼容的分布式追踪功能
//!
//! # 示例
//!
//! ```rust,no_run
//! use lumosai_core::telemetry::tracing::{OpenTelemetryTracer, TracerConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = TracerConfig::builder()
//!         .with_service_name("my-agent")
//!         .build()?;
//!
//!     let tracer = OpenTelemetryTracer::new(config)?;
//!     tracer.init()?;
//!
//!     // 使用 tracing instrument 宏自动追踪
//!     // ...
//!
//!     Ok(())
//! }
//! ```

use opentelemetry::trace::TracerProvider;
use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::{
    runtime::Tokio,
    trace::{TracerProvider as SdkTracerProvider},
    Resource,
};
use std::sync::Arc;

/// OpenTelemetry Tracer 配置
#[derive(Debug, Clone)]
pub struct TracerConfig {
    /// 服务名称
    pub service_name: String,

    /// 采样率 (0.0 - 1.0)
    pub sample_rate: f64,

    /// 其他属性
    pub attributes: Vec<(String, String)>,
}

impl Default for TracerConfig {
    fn default() -> Self {
        Self {
            service_name: "lumosai".to_string(),
            sample_rate: 1.0,
            attributes: Vec::new(),
        }
    }
}

impl TracerConfig {
    /// 创建配置构建器
    pub fn builder() -> TracerConfigBuilder {
        TracerConfigBuilder::default()
    }

    /// 构建 Resource
    pub fn build_resource(&self) -> Resource {
        let mut attrs = vec![
            KeyValue::new("service.name", self.service_name.clone()),
            KeyValue::new("telemetry.sdk.language", "rust"),
            KeyValue::new("telemetry.sdk.name", "lumosai"),
        ];

        // 添加自定义属性
        for (key, value) in &self.attributes {
            attrs.push(KeyValue::new(key.clone(), value.clone()));
        }

        Resource::new(attrs)
    }
}

/// Tracer 配置构建器
#[derive(Debug, Default)]
pub struct TracerConfigBuilder {
    config: TracerConfig,
}

impl TracerConfigBuilder {
    /// 设置服务名称
    pub fn with_service_name(mut self, name: impl Into<String>) -> Self {
        self.config.service_name = name.into();
        self
    }

    /// 设置采样率
    pub fn with_sample_rate(mut self, rate: f64) -> Self {
        self.config.sample_rate = rate;
        self
    }

    /// 添加属性
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.attributes.push((key.into(), value.into()));
        self
    }

    /// 构建配置
    pub fn build(self) -> Result<TracerConfig, String> {
        if self.config.sample_rate < 0.0 || self.config.sample_rate > 1.0 {
            return Err("sample_rate must be between 0.0 and 1.0".to_string());
        }

        Ok(self.config)
    }
}

/// OpenTelemetry Tracer
pub struct OpenTelemetryTracer {
    config: TracerConfig,
}

impl OpenTelemetryTracer {
    /// 创建新的 Tracer
    pub fn new(config: TracerConfig) -> Result<Self, String> {
        Ok(Self { config })
    }

    /// 初始化全局 Tracer
    pub fn init(self) -> Result<(), String> {
        // 创建 Resource
        let resource = self.config.build_resource();

        // 创建简单的 TracerProvider
        let provider = SdkTracerProvider::builder().build();

        // 设置为全局 tracer
        global::set_tracer_provider(provider);

        // 设置 tracing 订阅者
        self.setup_tracing_subscriber()?;

        Ok(())
    }

    /// 设置 tracing 订阅者
    fn setup_tracing_subscriber(&self) -> Result<(), String> {
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

        let telemetry_layer = tracing_opentelemetry::layer();

        tracing_subscriber::registry()
            .with(telemetry_layer)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(std::io::stdout)
                    .with_ansi(true)
                    .with_target(false)
                    .with_level(true)
            )
            .try_init()
            .map_err(|e| format!("Failed to initialize tracing subscriber: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracer_config_builder() {
        let config = TracerConfig::builder()
            .with_service_name("test-service")
            .with_sample_rate(0.5)
            .with_attribute("env", "test")
            .build()
            .unwrap();

        assert_eq!(config.service_name, "test-service");
        assert_eq!(config.sample_rate, 0.5);
        assert_eq!(config.attributes.len(), 1);
    }

    #[test]
    fn test_invalid_sample_rate() {
        let result = TracerConfig::builder()
            .with_sample_rate(1.5)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_build_resource() {
        let config = TracerConfig {
            service_name: "test-service".to_string(),
            ..Default::default()
        };

        let resource = config.build_resource();
        let attrs = resource.to_vec();

        assert!(!attrs.is_empty());
        assert!(attrs.iter().any(|kv| kv.key.as_ref() == "service.name"));
    }
}
