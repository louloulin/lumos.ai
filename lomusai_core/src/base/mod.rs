//! Base module for common functionality shared by components

use std::sync::Arc;
use serde::{Serialize, Deserialize};
use crate::logger::{Logger, Component, LogLevel, create_logger};
use crate::telemetry::{Event, TelemetrySink};
use crate::types::Metadata;

/// Component configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfig {
    /// Component name
    pub name: Option<String>,
    /// Component type
    pub component: Component,
    /// Log level
    pub log_level: Option<LogLevel>,
}

impl Default for ComponentConfig {
    fn default() -> Self {
        Self {
            name: None,
            component: Component::default(),
            log_level: None,
        }
    }
}

/// Base trait for all components
pub trait Base: Send + Sync {
    /// Get the component name
    fn name(&self) -> Option<&str>;
    
    /// Get the component type
    fn component(&self) -> Component;
    
    /// Get the logger
    fn logger(&self) -> Arc<dyn Logger>;
    
    /// Set the logger
    fn set_logger(&mut self, logger: Arc<dyn Logger>);
    
    /// Get the telemetry sink
    fn telemetry(&self) -> Option<Arc<dyn TelemetrySink>>;
    
    /// Set the telemetry sink
    fn set_telemetry(&mut self, telemetry: Arc<dyn TelemetrySink>);
    
    /// Record a telemetry event
    fn record_event(&self, event_name: &str, data: Metadata) {
        if let Some(telemetry) = self.telemetry() {
            let event = Event {
                name: event_name.to_string(),
                data: serde_json::to_value(data).unwrap_or_default(),
            };
            telemetry.record_event(event);
        }
    }
}

/// Basic implementation of the Base trait
#[derive(Clone)]
pub struct BaseComponent {
    /// Component name
    name: Option<String>,
    /// Component type
    component: Component,
    /// Logger
    logger: Arc<dyn Logger>,
    /// Telemetry sink
    telemetry: Option<Arc<dyn TelemetrySink>>,
}

impl BaseComponent {
    /// Create a new base component
    pub fn new(config: ComponentConfig) -> Self {
        let name = config.name.unwrap_or_else(|| "unnamed".to_string());
        let component = config.component;
        let log_level = config.log_level.unwrap_or_default();
        
        Self {
            name: Some(name.clone()),
            component,
            logger: create_logger(name, component, log_level),
            telemetry: None,
        }
    }
    
    /// 从名称和组件类型创建BaseComponent的便捷方法
    pub fn new_with_name(name: impl Into<String>, component: Component) -> Self {
        let name = name.into();
        Self {
            name: Some(name.clone()),
            component,
            logger: create_logger(name, component, LogLevel::Info),
            telemetry: None,
        }
    }
}

impl Base for BaseComponent {
    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    
    fn component(&self) -> Component {
        self.component
    }
    
    fn logger(&self) -> Arc<dyn Logger> {
        self.logger.clone()
    }
    
    fn set_logger(&mut self, logger: Arc<dyn Logger>) {
        self.logger = logger;
        if let Some(name) = &self.name {
            self.logger.debug(&format!("Logger updated [component={}] [name={}]", self.component, name), None);
        }
    }
    
    fn telemetry(&self) -> Option<Arc<dyn TelemetrySink>> {
        self.telemetry.clone()
    }
    
    fn set_telemetry(&mut self, telemetry: Arc<dyn TelemetrySink>) {
        self.telemetry = Some(telemetry);
        if let Some(name) = &self.name {
            self.logger.debug(&format!("Telemetry updated [component={}] [name={}]", self.component, name), None);
        }
    }
} 