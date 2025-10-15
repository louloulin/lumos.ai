//! # Lumos - 企业级AI应用开发框架
//!
//! Lumos是一个高性能、类型安全的Rust AI框架，专为企业级应用设计。
//! 提供完整的RAG系统、Agent框架、多Agent编排和事件驱动架构。
//!
//! ## 🚀 快速开始
//!
//! ### 创建一个简单的Agent
//! ```rust,no_run
//! use lumosai::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
//!     // 一行代码创建Agent
//!     let agent = lumosai::agent::simple("gpt-4", "You are a helpful assistant").await?;
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
//! use lumosai::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
//!     // 一行代码创建向量存储
//!     let storage = lumosai::vector::memory().await?;
//!
//!     // 一行代码创建RAG系统
//!     let rag = lumosai::rag::simple(storage, "openai").await?;
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
//! use lumosai::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
//!     // 创建多个Agent
//!     let researcher = lumosai::agent::simple("gpt-4", "You are a researcher").await?;
//!     let writer = lumosai::agent::simple("gpt-4", "You are a writer").await?;
//!
//!     // 创建协作任务
//!     let task = lumosai::orchestration::task()
//!         .name("Research and Write")
//!         .agent(researcher)
//!         .agent(writer)
//!         .pattern(Pattern::Sequential)
//!         .build();
//!
//!     // 执行协作
//!     let results = lumosai::orchestration::execute(task).await?;
//!
//!     Ok(())
//! }
//! ```

// 核心模块重导出
pub use lumosai_core as core;
pub use lumosai_rag as rag_core;
pub use lumosai_vector as vector_core;

// UI模块重导出 (可选功能)
#[cfg(feature = "ui")]
pub use lumosai_ui as ui;

// 简化API模块
pub mod agent;
pub mod events;
pub mod orchestration;
pub mod prelude;
pub mod rag;
pub mod session;
pub mod vector;

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
mod vector_integration_test;
