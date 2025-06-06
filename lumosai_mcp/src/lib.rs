//! Lumosai MCP module provides a Rust implementation of the Model Context Protocol.
//!
//! This module allows for interaction with MCP servers to access tools and resources
//! that can be used by agents and other components of the Lumosai framework.

mod error;
mod types;
mod client;
mod configuration;
mod transport;
mod enhanced;
mod tool_adapter;
mod discovery;
#[cfg(test)]
mod tests;

pub use error::{MCPError, Result};
pub use types::*;
pub use client::MCPClient;
pub use configuration::{MCPConfiguration, ServerDefinition};
pub use transport::{StdioTransport, SSETransport, Transport};
pub use enhanced::{EnhancedMCPManager, HealthStatus, PerformanceMetrics, ManagerConfig, ServerStatus};
pub use tool_adapter::{MCPToolAdapter, MCPToolFactory, MCPIntegration};
pub use discovery::{MCPServerRegistry, ServerConfig, ServerType, ConnectionConfig};