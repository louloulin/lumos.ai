//! # Lumos - 企业级AI应用开发框架
//!
//! Lumos是一个高性能、类型安全的Rust AI框架，专为企业级应用设计。
//! 提供完整的RAG系统、Agent框架、多Agent编排和事件驱动架构。
//!
//! ## 🚀 快速开始
//!
//! ### 创建一个简单的Agent
//! ```rust,no_run
//! use lumos::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // 一行代码创建Agent
//!     let agent = lumos::agent::simple("gpt-4", "You are a helpful assistant").await?;
//!
//!     // 开始对话
//!     let response = agent.chat("Hello, how are you?").await?;
//!     println!("Agent: {}", response);
//!
//!     Ok(())
//! }
//! ```
//!
//! ### 创建RAG系统
//! ```rust,no_run
//! use lumos::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // 一行代码创建向量存储
//!     let storage = lumos::vector::memory().await?;
//!
//!     // 一行代码创建RAG系统
//!     let rag = lumos::rag::simple(storage, "openai").await?;
//!
//!     // 添加文档并查询
//!     rag.add_document("AI is transforming the world").await?;
//!     let results = rag.search("What is AI doing?", 5).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### 多Agent协作
//! ```rust,no_run
//! use lumos::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // 创建多个Agent
//!     let researcher = lumos::agent::simple("gpt-4", "You are a researcher").await?;
//!     let writer = lumos::agent::simple("gpt-4", "You are a writer").await?;
//!
//!     // 创建协作任务
//!     let task = lumos::orchestration::task()
//!         .name("Research and Write")
//!         .participants(vec![researcher, writer])
//!         .pattern(lumos::orchestration::Pattern::Sequential)
//!         .build();
//!
//!     // 执行协作
//!     let results = lumos::orchestration::execute(task).await?;
//!
//!     Ok(())
//! }
//! ```

// 核心模块重导出
pub use lumosai_core as core;
pub use lumosai_rag as rag_core;
pub use lumosai_vector as vector_core;

// 简化API模块
pub mod prelude;
pub mod vector;
pub mod rag;
pub mod agent;
pub mod orchestration;
pub mod session;
pub mod events;

// 便利类型重导出
pub use lumosai_core::{
    error::{Error, Result},
    llm::{Message, Role},
};

/// Lumos框架版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 框架信息
pub const FRAMEWORK_INFO: &str = "Lumos - 企业级AI应用开发框架";

// 测试模块
#[cfg(test)]
mod simplified_api_test;
