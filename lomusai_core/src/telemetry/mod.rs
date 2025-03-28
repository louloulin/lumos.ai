//! Telemetry module for logging and tracing

/// Basic event type
#[derive(Debug, Clone)]
pub struct Event {
    pub name: String,
    pub data: serde_json::Value,
}

/// Telemetry sink trait
pub trait TelemetrySink: Send + Sync {
    fn record_event(&self, event: Event);
} 