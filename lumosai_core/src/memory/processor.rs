//! Memory processors for filtering and transforming messages

use std::sync::Arc;
use async_trait::async_trait;

use crate::base::Base;
use crate::llm::Message;
use crate::logger::{Component, Logger};
use crate::telemetry::TelemetrySink;
use crate::Result;

/// Options for memory processors
#[derive(Clone, Default)]
pub struct MemoryProcessorOptions {
    /// System message content
    pub system_message: Option<String>,
    /// Memory system message content
    pub memory_system_message: Option<String>,
    /// New messages being processed
    pub new_messages: Vec<Message>,
}

/// Trait for message processors that can filter or transform messages
/// before they're sent to the LLM.
#[async_trait]
pub trait MemoryProcessor: Base + Send + Sync {
    /// Process a list of messages and return a filtered or transformed list.
    /// @param messages The messages to process
    /// @param options Processing options
    /// @returns The processed messages
    async fn process(&self, messages: Vec<Message>, options: &MemoryProcessorOptions) -> Result<Vec<Message>>;

    /// Get the name of this processor for debugging
    fn processor_name(&self) -> &str {
        "MemoryProcessor"
    }
}

/// A processor that limits the number of messages
#[derive(Clone)]
pub struct MessageLimitProcessor {
    /// Maximum number of messages to keep
    pub max_messages: usize,
    /// Logger
    logger: Arc<dyn Logger>,
}

impl MessageLimitProcessor {
    /// Create a new message limit processor
    pub fn new(max_messages: usize, logger: Arc<dyn Logger>) -> Self {
        Self {
            max_messages,
            logger,
        }
    }
}

impl Base for MessageLimitProcessor {
    fn name(&self) -> Option<&str> {
        Some("MessageLimitProcessor")
    }

    fn component(&self) -> Component {
        Component::Memory
    }

    fn logger(&self) -> Arc<dyn Logger> {
        self.logger.clone()
    }

    fn set_logger(&mut self, logger: Arc<dyn Logger>) {
        self.logger = logger;
    }

    fn telemetry(&self) -> Option<Arc<dyn TelemetrySink>> {
        None
    }

    fn set_telemetry(&mut self, _telemetry: Arc<dyn TelemetrySink>) {
        // No-op for now
    }
}

#[async_trait]
impl MemoryProcessor for MessageLimitProcessor {
    async fn process(&self, messages: Vec<Message>, _options: &MemoryProcessorOptions) -> Result<Vec<Message>> {
        if messages.len() <= self.max_messages {
            return Ok(messages);
        }
        
        // Keep the most recent messages
        let start_index = messages.len() - self.max_messages;
        let limited_messages = messages[start_index..].to_vec();
        
        self.logger.debug(&format!(
            "Limited messages from {} to {} (max: {})",
            messages.len(),
            limited_messages.len(),
            self.max_messages
        ), None);
        
        Ok(limited_messages)
    }

    fn processor_name(&self) -> &str {
        "MessageLimitProcessor"
    }
}

/// A processor that filters messages by role
#[derive(Clone)]
pub struct RoleFilterProcessor {
    /// Roles to keep
    pub allowed_roles: Vec<crate::llm::Role>,
    /// Logger
    logger: Arc<dyn Logger>,
}

impl RoleFilterProcessor {
    /// Create a new role filter processor
    pub fn new(allowed_roles: Vec<crate::llm::Role>, logger: Arc<dyn Logger>) -> Self {
        Self {
            allowed_roles,
            logger,
        }
    }
}

impl Base for RoleFilterProcessor {
    fn name(&self) -> Option<&str> {
        Some("RoleFilterProcessor")
    }

    fn component(&self) -> Component {
        Component::Memory
    }

    fn logger(&self) -> Arc<dyn Logger> {
        self.logger.clone()
    }

    fn set_logger(&mut self, logger: Arc<dyn Logger>) {
        self.logger = logger;
    }

    fn telemetry(&self) -> Option<Arc<dyn TelemetrySink>> {
        None
    }

    fn set_telemetry(&mut self, _telemetry: Arc<dyn TelemetrySink>) {
        // No-op for now
    }
}

#[async_trait]
impl MemoryProcessor for RoleFilterProcessor {
    async fn process(&self, messages: Vec<Message>, _options: &MemoryProcessorOptions) -> Result<Vec<Message>> {
        let original_count = messages.len();
        let filtered_messages: Vec<Message> = messages
            .into_iter()
            .filter(|msg| self.allowed_roles.contains(&msg.role))
            .collect();
        
        self.logger.debug(&format!(
            "Filtered messages from {} to {} (allowed roles: {:?})",
            original_count,
            filtered_messages.len(),
            self.allowed_roles
        ), None);
        
        Ok(filtered_messages)
    }

    fn processor_name(&self) -> &str {
        "RoleFilterProcessor"
    }
}

/// A processor that removes duplicate messages
#[derive(Clone)]
pub struct DeduplicationProcessor {
    /// Logger
    logger: Arc<dyn Logger>,
}

impl DeduplicationProcessor {
    /// Create a new deduplication processor
    pub fn new(logger: Arc<dyn Logger>) -> Self {
        Self { logger }
    }
}

impl Base for DeduplicationProcessor {
    fn name(&self) -> Option<&str> {
        Some("DeduplicationProcessor")
    }

    fn component(&self) -> Component {
        Component::Memory
    }

    fn logger(&self) -> Arc<dyn Logger> {
        self.logger.clone()
    }

    fn set_logger(&mut self, logger: Arc<dyn Logger>) {
        self.logger = logger;
    }

    fn telemetry(&self) -> Option<Arc<dyn TelemetrySink>> {
        None
    }

    fn set_telemetry(&mut self, _telemetry: Arc<dyn TelemetrySink>) {
        // No-op for now
    }
}

#[async_trait]
impl MemoryProcessor for DeduplicationProcessor {
    async fn process(&self, messages: Vec<Message>, _options: &MemoryProcessorOptions) -> Result<Vec<Message>> {
        let original_count = messages.len();
        let mut seen_contents = std::collections::HashSet::new();
        let mut deduplicated_messages = Vec::new();
        
        for message in messages {
            let role_str = match message.role {
                crate::llm::Role::System => "system",
                crate::llm::Role::User => "user",
                crate::llm::Role::Assistant => "assistant",
                crate::llm::Role::Tool => "tool",
                crate::llm::Role::Function => "function",
                crate::llm::Role::Custom(ref custom) => custom,
            };
            let content_key = format!("{}:{}", role_str, message.content);
            if seen_contents.insert(content_key) {
                deduplicated_messages.push(message);
            }
        }
        
        self.logger.debug(&format!(
            "Deduplicated messages from {} to {}",
            original_count,
            deduplicated_messages.len()
        ), None);
        
        Ok(deduplicated_messages)
    }

    fn processor_name(&self) -> &str {
        "DeduplicationProcessor"
    }
}

/// A processor that applies multiple processors in sequence
pub struct CompositeProcessor {
    /// List of processors to apply
    processors: Vec<Box<dyn MemoryProcessor>>,
    /// Logger
    logger: Arc<dyn Logger>,
}

impl CompositeProcessor {
    /// Create a new composite processor
    pub fn new(processors: Vec<Box<dyn MemoryProcessor>>, logger: Arc<dyn Logger>) -> Self {
        Self { processors, logger }
    }
    
    /// Add a processor to the chain
    pub fn add_processor(&mut self, processor: Box<dyn MemoryProcessor>) {
        self.processors.push(processor);
    }
}

impl Base for CompositeProcessor {
    fn name(&self) -> Option<&str> {
        Some("CompositeProcessor")
    }

    fn component(&self) -> Component {
        Component::Memory
    }

    fn logger(&self) -> Arc<dyn Logger> {
        self.logger.clone()
    }

    fn set_logger(&mut self, logger: Arc<dyn Logger>) {
        self.logger = logger;
    }

    fn telemetry(&self) -> Option<Arc<dyn TelemetrySink>> {
        None
    }

    fn set_telemetry(&mut self, _telemetry: Arc<dyn TelemetrySink>) {
        // No-op for now
    }
}

#[async_trait]
impl MemoryProcessor for CompositeProcessor {
    async fn process(&self, mut messages: Vec<Message>, options: &MemoryProcessorOptions) -> Result<Vec<Message>> {
        for processor in &self.processors {
            messages = processor.process(messages, options).await?;
        }
        Ok(messages)
    }

    fn processor_name(&self) -> &str {
        "CompositeProcessor"
    }
}

/// Create a default message processor chain
pub fn create_default_processor_chain(logger: Arc<dyn Logger>) -> CompositeProcessor {
    let processors: Vec<Box<dyn MemoryProcessor>> = vec![
        Box::new(DeduplicationProcessor::new(logger.clone())),
        Box::new(MessageLimitProcessor::new(50, logger.clone())),
    ];
    
    CompositeProcessor::new(processors, logger)
}
