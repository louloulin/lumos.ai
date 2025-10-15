//! Lumosai MCP module provides a Rust implementation of the Model Context Protocol.
//!
//! This module allows for interaction with MCP servers to access tools and resources
//! that can be used by agents and other components of the Lumosai framework.

mod client;
mod configuration;
mod discovery;
mod enhanced;
mod error;
#[cfg(test)]
mod tests;
mod tool_adapter;
mod transport;
mod types;

pub use client::MCPClient;
pub use configuration::{MCPConfiguration, ServerDefinition};
pub use discovery::{ConnectionConfig, MCPServerRegistry, ServerConfig, ServerType};
pub use enhanced::{
    EnhancedMCPManager, HealthStatus, ManagerConfig, PerformanceMetrics, ServerStatus,
};
pub use error::{MCPError, Result};
pub use tool_adapter::{MCPIntegration, MCPToolAdapter, MCPToolFactory};
pub use transport::{SSETransport, StdioTransport, Transport};
pub use types::*;
