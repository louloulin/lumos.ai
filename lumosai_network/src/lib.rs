//! # Lumosai Network
//! 
//! `lumosai_network` 实现了Agent网络通信和管理功能，支持各种类型的Agent之间的通信和协作。
//! 
//! 主要功能包括：
//! - Agent网络管理和拓扑
//! - 消息路由和传递
//! - 网络事件处理
//! - 分布式Agent协调

pub mod error;
pub mod types;
pub mod network;
pub mod message;
pub mod router;
pub mod topology;
pub mod discovery;

// 重新导出主要类型
pub use error::Error;
pub use types::{AgentId, AgentType, AgentStatus, AgentCapability};
pub use network::{AgentNetwork, AgentNode};
pub use message::{Message, MessageType, MessageStatus};
pub use router::MessageRouter;
pub use topology::{NetworkTopology, TopologyType};
pub use discovery::ServiceDiscovery;

/// 创建Agent网络
pub async fn create_agent_network() -> AgentNetwork {
    network::AgentNetwork::new().await
}

/// 库版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION"); 