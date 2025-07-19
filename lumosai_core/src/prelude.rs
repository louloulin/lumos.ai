//! Lumos.ai Prelude - Simplified API for easy imports
//! 
//! This module provides the simplified API as specified in plan6.md Phase 1,
//! offering Rig-level simplicity while maintaining Rust performance.
//!
//! # Example
//!
//! ```rust
//! use lumosai_core::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let agent = Agent::quick("assistant", "你是一个AI助手")
//!         .model("gpt-4")
//!         .build()?;
//!     
//!     let response = agent.generate("Hello!").await?;
//!     println!("{}", response.content);
//!     
//!     Ok(())
//! }
//! ```

// Re-export core types
pub use crate::{Result, Error};

// Re-export simplified Agent API
pub use crate::agent::{
    Agent, AgentBuilder,
    quick, web_agent, file_agent, data_agent,
};

// Re-export tool creation functions
pub use crate::tool::builtin::{
    // Web tools
    create_http_request_tool, create_web_scraper_tool, create_json_api_tool, create_url_validator_tool,
    // File tools
    create_file_reader_tool, create_file_writer_tool, create_directory_lister_tool, create_file_info_tool,
    // Data tools
    create_json_parser_tool, create_csv_parser_tool, create_data_transformer_tool, create_excel_reader_tool,
    // Math tools
    create_calculator_tool, create_statistics_tool,
    // System tools
    create_datetime_tool, create_uuid_generator_tool, create_hash_generator_tool,
    // AI tools (when implemented)
    // create_image_analyzer_tool, create_text_summarizer_tool, create_sentiment_analyzer_tool,
    // Database tools (when implemented)
    // create_sql_executor_tool, create_mongodb_client_tool,
    // Communication tools (when implemented)
    // create_email_sender_tool, create_slack_notifier_tool, create_webhook_caller_tool,
};

// Convenience functions for tool creation
pub fn web_search() -> Box<dyn crate::tool::Tool> {
    Box::new(create_http_request_tool())
}

pub fn http_request() -> Box<dyn crate::tool::Tool> {
    Box::new(create_http_request_tool())
}

pub fn web_scraper() -> Box<dyn crate::tool::Tool> {
    Box::new(create_web_scraper_tool())
}

pub fn json_api() -> Box<dyn crate::tool::Tool> {
    Box::new(create_json_api_tool())
}

pub fn url_validator() -> Box<dyn crate::tool::Tool> {
    Box::new(create_url_validator_tool())
}

pub fn file_reader() -> Box<dyn crate::tool::Tool> {
    Box::new(create_file_reader_tool())
}

pub fn file_writer() -> Box<dyn crate::tool::Tool> {
    Box::new(create_file_writer_tool())
}

pub fn directory_lister() -> Box<dyn crate::tool::Tool> {
    Box::new(create_directory_lister_tool())
}

pub fn file_info() -> Box<dyn crate::tool::Tool> {
    Box::new(create_file_info_tool())
}

pub fn json_parser() -> Box<dyn crate::tool::Tool> {
    Box::new(create_json_parser_tool())
}

pub fn csv_parser() -> Box<dyn crate::tool::Tool> {
    Box::new(create_csv_parser_tool())
}

pub fn data_transformer() -> Box<dyn crate::tool::Tool> {
    Box::new(create_data_transformer_tool())
}

pub fn excel_reader() -> Box<dyn crate::tool::Tool> {
    Box::new(create_excel_reader_tool())
}

pub fn calculator() -> Box<dyn crate::tool::Tool> {
    Box::new(create_calculator_tool())
}

pub fn statistics() -> Box<dyn crate::tool::Tool> {
    Box::new(create_statistics_tool())
}

pub fn time_tool() -> Box<dyn crate::tool::Tool> {
    Box::new(create_datetime_tool())
}

pub fn uuid_generator() -> Box<dyn crate::tool::Tool> {
    Box::new(create_uuid_generator_tool())
}

pub fn hash_tool() -> Box<dyn crate::tool::Tool> {
    Box::new(create_hash_generator_tool())
}

// Additional convenience functions for examples
pub fn weather_tool() -> Box<dyn crate::tool::Tool> {
    // For now, return a web request tool that can be used for weather APIs
    Box::new(create_http_request_tool())
}

// Re-export provider convenience functions
pub use crate::agent::convenience::{
    openai, openai_with_key, openai_builder,
    anthropic, anthropic_with_key,
    deepseek, deepseek_with_key, deepseek_builder,
    qwen, qwen_with_key,
    ModelBuilder, LlmProviderExt,
};

// Re-export memory types
pub use crate::memory::{
    WorkingMemory, WorkingMemoryContent,
};

// Re-export vector storage types
pub use crate::vector::{
    VectorStorage, VectorStorageConfig, create_vector_storage,
    SimilarityMetric, IndexStats, QueryResult, FilterCondition,
};

// Re-export common types
pub use crate::llm::{Message, Role, LlmOptions};
pub use crate::agent::{
    AgentConfig, AgentGenerateOptions, AgentStreamOptions,
    AgentGenerateResult, AgentStep, AgentToolCall,
};

/// Quick agent creation function (plan6.md convenience function)
/// 
/// This is the most convenient way to create an agent:
/// ```rust
/// use lumosai_core::prelude::*;
/// 
/// let agent = quick_agent("assistant", "You are helpful")
///     .model(openai("gpt-4")?)
///     .build()?;
/// ```
pub fn quick_agent(name: &str, instructions: &str) -> AgentBuilder {
    Agent::quick(name, instructions)
}

/// Create a web-enabled agent with common web tools
/// 
/// # Example
/// 
/// ```rust
/// use lumosai_core::prelude::*;
/// 
/// let agent = web_agent_quick("web_helper", "You can browse the web")
///     .model(openai("gpt-4")?)
///     .build()?;
/// ```
pub fn web_agent_quick(name: &str, instructions: &str) -> AgentBuilder {
    Agent::quick(name, instructions)
        .with_web_tools()
}

/// Create a file-enabled agent with common file tools
/// 
/// # Example
/// 
/// ```rust
/// use lumosai_core::prelude::*;
/// 
/// let agent = file_agent_quick("file_helper", "You can manage files")
///     .model(openai("gpt-4")?)
///     .build()?;
/// ```
pub fn file_agent_quick(name: &str, instructions: &str) -> AgentBuilder {
    Agent::quick(name, instructions)
        .with_file_tools()
}

/// Create a data processing agent with common data tools
/// 
/// # Example
/// 
/// ```rust
/// use lumosai_core::prelude::*;
/// 
/// let agent = data_agent_quick("data_helper", "You can process data")
///     .model(openai("gpt-4")?)
///     .build()?;
/// ```
pub fn data_agent_quick(name: &str, instructions: &str) -> AgentBuilder {
    Agent::quick(name, instructions)
        .with_data_tools()
        .with_math_tools()
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
pub fn memory_vector_storage(dimensions: usize, capacity: Option<usize>) -> crate::Result<Box<dyn VectorStorage>> {
    let config = VectorStorageConfig::Memory { dimensions, capacity };
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
    use crate::llm::MockLlmProvider;
    use crate::agent::trait_def::Agent as AgentTrait;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_prelude_quick_agent() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
        
        let agent = quick_agent("assistant", "You are helpful")
            .model(llm)
            .build()
            .expect("Failed to create agent");

        assert_eq!(agent.get_name(), "assistant");
        assert_eq!(agent.get_instructions(), "You are helpful");
    }

    #[tokio::test]
    async fn test_prelude_web_agent_quick() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
        
        let agent = web_agent_quick("web_helper", "You can browse the web")
            .model(llm)
            .build()
            .expect("Failed to create web agent");

        assert_eq!(agent.get_name(), "web_helper");
        assert_eq!(agent.get_instructions(), "You can browse the web");
        // Should have web tools
        assert!(agent.get_tools().len() > 0);
    }

    #[tokio::test]
    async fn test_prelude_file_agent_quick() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
        
        let agent = file_agent_quick("file_helper", "You can manage files")
            .model(llm)
            .build()
            .expect("Failed to create file agent");

        assert_eq!(agent.get_name(), "file_helper");
        assert_eq!(agent.get_instructions(), "You can manage files");
        // Should have file tools
        assert!(agent.get_tools().len() > 0);
    }

    #[tokio::test]
    async fn test_prelude_data_agent_quick() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
        
        let agent = data_agent_quick("data_helper", "You can process data")
            .model(llm)
            .build()
            .expect("Failed to create data agent");

        assert_eq!(agent.get_name(), "data_helper");
        assert_eq!(agent.get_instructions(), "You can process data");
        // Should have data and math tools
        assert!(agent.get_tools().len() > 0);
    }
}
