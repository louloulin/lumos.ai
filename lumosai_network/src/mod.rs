//! Agent网络模块
//! 
//! 该模块提供了Agent之间进行通信的网络基础设施，包括：
//! 
//! - 消息传递系统
//! - 网络拓扑管理
//! - 消息路由策略
//! - 服务发现机制
//! - Agent网络管理

mod error;
mod types;
mod message;
mod router;
mod topology;
mod discovery;
mod network;

// 重新导出
pub use error::{Error, Result};
pub use types::{AgentId, AgentType, AgentStatus, AgentCapability};
pub use message::{Message, MessageType, MessageStatus};
pub use router::{MessageRouter, DefaultMessageRouter, RoutingStrategy, RoutingRule};
pub use topology::{NetworkTopology, GraphTopology, TopologyType, EdgeAttributes, NodeAttributes};
pub use discovery::{ServiceDiscovery, InMemoryServiceDiscovery, ServiceRegistration, ServiceQuery};
pub use network::{AgentNetwork, AgentNode, AgentConfig}; 