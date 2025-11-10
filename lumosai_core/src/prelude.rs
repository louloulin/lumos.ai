//! LumosAI Prelude - 统一简化的 API 接口
//!
//! 这个模块提供了 LumosAI 的统一 API，遵循"简洁优于复杂"的设计原则。
//! 参考了 Rig 和 Mastra 的设计理念，提供直观易用的开发体验。
//!
//! # 基本使用
//!
//! ```rust
//! use lumosai_core::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // 快速创建 Agent
//!     let agent = Agent::new("assistant", "你是一个AI助手")
//!         .model(openai("gpt-4")?)
//!         .build()?;
//!
//!     // 生成响应
//!     let response = agent.generate("Hello!").await?;
//!     println!("{}", response.content);
//!
//!     Ok(())
//! }
//! ```
//!
//! # 高级使用
//!
//! ```rust
//! use lumosai_core::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // 创建专业化 Agent
//!     let agent = Agent::new("research_assistant", "你是专业的研究助手")
//!         .model(anthropic("claude-3-sonnet")?)
//!         .tools(vec![web_search(), file_reader(), calculator()])
//!         .max_tool_calls(10)
//!         .build()?;
//!
//!     let response = agent.generate("帮我研究AI的最新发展").await?;
//!     println!("{}", response.content);
//!
//!     Ok(())
//! }
//! ```

// ============================================================================
// 核心类型导出
// ============================================================================

/// 核心错误和结果类型
pub use crate::{Error, Result};

// ============================================================================
// Agent API - 统一的渐进式 Agent 创建接口
// ============================================================================

/// 统一的 Agent 接口 - 渐进式 API 设计
///
/// **Level 1 - 5分钟上手**:
/// ```rust
/// let agent = Agent::new("assistant", "你是一个AI助手").await?;
/// let response = agent.generate("你好").await?;
/// ```
///
/// **Level 2 - 链式配置**:
/// ```rust
/// let agent = Agent::new("assistant", "你是一个AI助手").await?
///     .with_model("gpt-4")?
///     .with_tools(vec![calculator(), web_search()])?;
/// ```
///
/// **Level 3 - 完整构建器**:
/// ```rust
/// let agent = Agent::builder()
///     .name("research_agent")
///     .instructions("专业研究助手")
///     .model(openai("gpt-4")?)
///     .max_tool_calls(10)
///     .build()?;
/// ```
pub use crate::agent::simplified_api::{Agent, AgentInstance};
pub use crate::agent::AgentBuilder;

// ============================================================================
// 工具系统 - 简化的工具创建接口
// ============================================================================

/// 核心工具创建函数（按类别组织）
pub use crate::tool::builtin::{
    // 数学和计算工具
    create_calculator_tool,
    create_csv_parser_tool,
    create_data_transformer_tool,

    // 系统工具
    create_datetime_tool,
    create_directory_lister_tool,
    create_file_info_tool,
    // 文件和数据处理工具
    create_file_reader_tool,
    create_file_writer_tool,
    create_hash_generator_tool,
    // 网络和 Web 工具
    create_http_request_tool,
    create_json_api_tool,
    create_json_parser_tool,
    create_statistics_tool,

    create_url_validator_tool,

    create_uuid_generator_tool,
    create_web_scraper_tool,
};

// ============================================================================
// 工具便利函数 - 简化的工具实例化
// ============================================================================

/// 网络和 Web 工具
pub fn web_search() -> Box<dyn crate::tool::Tool> {
    Box::new(create_http_request_tool())
}

pub fn web_scraper() -> Box<dyn crate::tool::Tool> {
    Box::new(create_web_scraper_tool())
}

pub fn json_api() -> Box<dyn crate::tool::Tool> {
    Box::new(create_json_api_tool())
}

/// 文件和数据工具
pub fn file_reader() -> Box<dyn crate::tool::Tool> {
    Box::new(create_file_reader_tool())
}

pub fn file_writer() -> Box<dyn crate::tool::Tool> {
    Box::new(create_file_writer_tool())
}

/// 数据处理工具
pub fn json_parser() -> Box<dyn crate::tool::Tool> {
    Box::new(create_json_parser_tool())
}

pub fn csv_parser() -> Box<dyn crate::tool::Tool> {
    Box::new(create_csv_parser_tool())
}

/// 数学和计算工具
pub fn calculator() -> Box<dyn crate::tool::Tool> {
    Box::new(create_calculator_tool())
}

pub fn statistics() -> Box<dyn crate::tool::Tool> {
    Box::new(create_statistics_tool())
}

/// 系统工具
pub fn datetime() -> Box<dyn crate::tool::Tool> {
    Box::new(create_datetime_tool())
}

pub fn uuid_generator() -> Box<dyn crate::tool::Tool> {
    Box::new(create_uuid_generator_tool())
}

// ============================================================================
// LLM 提供商 - 简化的模型访问接口
// ============================================================================

/// LLM 提供商便利函数
pub use crate::agent::convenience::{
    anthropic, deepseek, openai, qwen, LlmProviderExt, ModelBuilder,
};

// ============================================================================
// 核心组件 - 内存、向量存储、消息类型
// ============================================================================

/// 内存系统
pub use crate::memory::{WorkingMemory, WorkingMemoryContent};

/// 向量存储
pub use crate::vector::{
    create_vector_storage, QueryResult, SimilarityMetric, VectorStorage, VectorStorageConfig,
};

pub use crate::agent::{AgentGenerateResult, AgentStep};
/// 消息和配置类型
pub use crate::llm::{Message, Role};

// ============================================================================
// Agent 便利创建函数 - 统一的 Agent 创建接口
// ============================================================================

/// 快速创建 Agent（推荐使用）
///
/// 这是创建 Agent 最简单的方式：
/// ```rust
/// use lumosai_core::prelude::*;
///
/// let agent = Agent::new("assistant", "你是一个AI助手")
///     .model(openai("gpt-4")?)
///     .build()?;
/// ```
pub fn quick_agent(name: &str, instructions: &str) -> AgentBuilder {
    AgentBuilder::new()
        .name(name)
        .instructions(instructions)
        .enable_smart_defaults()
}

/// 创建专业化 Agent

/// 创建网络功能 Agent
pub fn web_agent(name: &str, instructions: &str) -> AgentBuilder {
    AgentBuilder::new()
        .name(name)
        .instructions(instructions)
        .tools(vec![web_search(), web_scraper(), json_api()])
        .enable_smart_defaults()
}

/// 创建文件处理 Agent
pub fn file_agent(name: &str, instructions: &str) -> AgentBuilder {
    AgentBuilder::new()
        .name(name)
        .instructions(instructions)
        .tools(vec![file_reader(), file_writer()])
        .enable_smart_defaults()
}

/// 创建数据处理 Agent
pub fn data_agent(name: &str, instructions: &str) -> AgentBuilder {
    AgentBuilder::new()
        .name(name)
        .instructions(instructions)
        .tools(vec![
            json_parser(),
            csv_parser(),
            calculator(),
            statistics(),
        ])
        .enable_smart_defaults()
}

// Vector storage convenience functions

/// Create a memory vector storage (for development and testing)
///
/// # Example
///
/// ```rust
/// use lumosai_core::prelude::*;
///
/// let storage = memory_vector_storage(1536, None)?;
/// ```
pub fn memory_vector_storage(
    dimensions: usize,
    capacity: Option<usize>,
) -> crate::Result<Box<dyn VectorStorage>> {
    let config = VectorStorageConfig::Memory {
        dimensions,
        capacity,
    };
    create_vector_storage(Some(config)).map_err(|e| crate::Error::Storage(e.to_string()))
}

// External vector storage convenience functions temporarily disabled due to dependency conflicts

// /// Create a Qdrant vector storage
// #[cfg(feature = "qdrant")]
// pub fn qdrant_vector_storage(url: &str, api_key: Option<String>) -> crate::Result<Box<dyn VectorStorage>> {
//     let config = VectorStorageConfig::Qdrant {
//         url: url.to_string(),
//         api_key
//     };
//     create_vector_storage(Some(config))
// }

// /// Create a MongoDB vector storage
// #[cfg(feature = "mongodb")]
// pub fn mongodb_vector_storage(connection_string: &str, database: &str) -> crate::Result<Box<dyn VectorStorage>> {
//     let config = VectorStorageConfig::MongoDB {
//         connection_string: connection_string.to_string(),
//         database: database.to_string()
//     };
//     create_vector_storage(Some(config))
// }

// /// Create a PostgreSQL vector storage with pgvector
// #[cfg(feature = "postgres")]
// pub fn postgres_vector_storage(connection_string: &str, database: &str) -> crate::Result<Box<dyn VectorStorage>> {
//     let config = VectorStorageConfig::PostgreSQL {
//         connection_string: connection_string.to_string(),
//         database: database.to_string()
//     };
//     create_vector_storage(Some(config))
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::trait_def::Agent as AgentTrait;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_prelude_quick_agent() {
        let llm = create_test_zhipu_provider_arc();

        let agent = quick_agent("assistant", "You are helpful")
            .model(llm)
            .build()
            .expect("Failed to create agent");

        assert_eq!(agent.get_name(), "assistant");
        assert_eq!(agent.get_instructions(), "You are helpful");
    }

    // 专业化 Agent 快速函数测试
    #[tokio::test]
    async fn test_prelude_web_agent() {
        let llm = create_test_zhipu_provider_arc();

        let agent = web_agent("web_helper", "You can browse the web")
            .model(llm)
            .build()
            .expect("Failed to create web agent");

        assert_eq!(agent.get_name(), "web_helper");
        assert_eq!(agent.get_instructions(), "You can browse the web");
        // Should have web tools
        assert!(agent.get_tools().len() > 0);
    }

    #[tokio::test]
    async fn test_prelude_file_agent() {
        let llm = create_test_zhipu_provider_arc();

        let agent = file_agent("file_helper", "You can manage files")
            .model(llm)
            .build()
            .expect("Failed to create file agent");

        assert_eq!(agent.get_name(), "file_helper");
        assert_eq!(agent.get_instructions(), "You can manage files");
        // Should have file tools
        assert!(agent.get_tools().len() > 0);
    }

    #[tokio::test]
    async fn test_prelude_data_agent() {
        let llm = create_test_zhipu_provider_arc();

        let agent = data_agent("data_helper", "You can process data")
            .model(llm)
            .build()
            .expect("Failed to create data agent");

        assert_eq!(agent.get_name(), "data_helper");
        assert_eq!(agent.get_instructions(), "You can process data");
        // Should have data and math tools
        assert!(agent.get_tools().len() > 0);
    }
}
