use async_trait::async_trait;
use futures::stream::BoxStream;

use super::function_calling::{FunctionCall, FunctionDefinition, ToolChoice};
use super::types::{LlmOptions, Message};
use crate::Result;

/// Core trait for Large Language Model (LLM) providers
///
/// The `LlmProvider` trait defines the unified interface for all LLM
/// implementations in LumosAI. It provides methods for text generation,
/// streaming, embeddings, and function calling.
///
/// # Overview
///
/// LumosAI supports multiple LLM providers through this unified interface:
///
/// - **OpenAI** - GPT-3.5, GPT-4, GPT-4 Turbo
/// - **Anthropic** - Claude 3 (Opus, Sonnet, Haiku)
/// - **Chinese Providers** - Qwen, Zhipu (GLM), Baidu (ERNIE), DeepSeek
/// - **Open Source** - Ollama (local models)
/// - **Other Providers** - Cohere, Gemini, Together AI
///
/// # Creating Providers
///
/// ## Using Factory Functions (Recommended)
///
/// ```rust
/// use lumosai_core::llm::providers;
///
/// # async fn example() -> lumosai_core::Result<()> {
/// // Create from environment variables
/// let openai = providers::openai_from_env()?;
/// let anthropic = providers::anthropic_from_env()?;
/// let qwen = providers::qwen_from_env()?;
///
/// // Auto-select available provider
/// let provider = providers::auto_provider()?;
/// # Ok(())
/// # }
/// ```
///
/// ## Manual Creation
///
/// ```rust
/// use lumosai_core::llm::{OpenAiProvider, AnthropicProvider, QwenProvider};
///
/// // OpenAI
/// let openai = OpenAiProvider::new(
///     "your-api-key".to_string(),
///     "gpt-4".to_string(),
/// );
///
/// // Anthropic Claude
/// let claude = AnthropicProvider::new(
///     "your-api-key".to_string(),
///     "claude-3-opus-20240229".to_string(),
/// );
///
/// // Qwen (Alibaba)
/// let qwen = QwenProvider::new(
///     "your-api-key".to_string(),
///     Some("qwen-turbo".to_string()),
/// );
/// ```
///
/// # Basic Usage
///
/// ## Simple Text Generation
///
/// ```rust
/// use lumosai_core::llm::{LlmProvider, LlmOptions};
///
/// # async fn example(provider: &dyn LlmProvider) -> lumosai_core::Result<()> {
/// let options = LlmOptions::default();
/// let response = provider.generate("What is Rust?", &options).await?;
/// println!("Response: {}", response);
/// # Ok(())
/// # }
/// ```
///
/// ## Conversation with Messages
///
/// ```rust
/// use lumosai_core::llm::{LlmProvider, LlmOptions, Message, Role};
///
/// # async fn example(provider: &dyn LlmProvider) -> lumosai_core::Result<()> {
/// let messages = vec![
///     Message {
///         role: Role::System,
///         content: "You are a helpful AI assistant.".to_string(),
///         metadata: None,
///         name: None,
///     },
///     Message {
///         role: Role::User,
///         content: "Explain quantum computing.".to_string(),
///         metadata: None,
///         name: None,
///     },
/// ];
///
/// let options = LlmOptions::default().with_temperature(0.7);
/// let response = provider.generate_with_messages(&messages, &options).await?;
/// # Ok(())
/// # }
/// ```
///
/// ## Streaming Responses
///
/// ```rust
/// use lumosai_core::llm::{LlmProvider, LlmOptions};
/// use futures::StreamExt;
///
/// # async fn example(provider: &dyn LlmProvider) -> lumosai_core::Result<()> {
/// let options = LlmOptions::default();
/// let mut stream = provider.generate_stream("Tell me a story", &options).await?;
///
/// while let Some(chunk) = stream.next().await {
///     match chunk {
///         Ok(text) => print!("{}", text),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Thread Safety
///
/// All LLM providers must be `Send + Sync`, allowing them to be safely
/// shared across threads and used in async contexts.
///
/// # See Also
///
/// - [`LlmOptions`] - Configuration options for generation
/// - [`Message`] - Message structure for conversations
/// - [`FunctionDefinition`] - Function calling definitions
/// - [`providers`] - Factory functions for creating providers
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Returns the name of the LLM provider
    ///
    /// # Returns
    ///
    /// A string identifier for the provider (e.g., "openai", "anthropic", "qwen").
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::llm::LlmProvider;
    ///
    /// # fn example(provider: &dyn LlmProvider) {
    /// println!("Using provider: {}", provider.name());
    /// # }
    /// ```
    fn name(&self) -> &str;

    /// Checks if the LLM provider is healthy and available
    ///
    /// This method performs a lightweight health check to determine if the provider
    /// is currently available and responsive. The default implementation performs
    /// a simple generation test with a minimal prompt.
    ///
    /// # Returns
    ///
    /// `true` if the provider is healthy and available, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::llm::LlmProvider;
    ///
    /// # async fn example(provider: &dyn LlmProvider) -> lumosai_core::Result<()> {
    /// if provider.is_healthy().await {
    ///     println!("Provider {} is available", provider.name());
    /// } else {
    ///     println!("Provider {} is unavailable", provider.name());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn is_healthy(&self) -> bool {
        // Default implementation: try a minimal generation request
        let options = LlmOptions::default()
            .with_max_tokens(1)
            .with_temperature(0.0);

        match self.generate("test", &options).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Generates text from a simple prompt
    ///
    /// This is the simplest way to generate text, suitable for single-turn
    /// interactions without conversation history.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The input prompt text
    /// * `options` - Generation options (temperature, max_tokens, etc.)
    ///
    /// # Returns
    ///
    /// The generated text response.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The API request fails
    /// - The API key is invalid
    /// - Rate limits are exceeded
    /// - Network connectivity issues occur
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::llm::{LlmProvider, LlmOptions};
    ///
    /// # async fn example(provider: &dyn LlmProvider) -> lumosai_core::Result<()> {
    /// let options = LlmOptions::default()
    ///     .with_temperature(0.7)
    ///     .with_max_tokens(100);
    ///
    /// let response = provider.generate("What is Rust?", &options).await?;
    /// println!("Response: {}", response);
    /// # Ok(())
    /// # }
    /// ```
    async fn generate(&self, prompt: &str, options: &LlmOptions) -> Result<String>;

    /// Generates text from a sequence of messages
    ///
    /// This method supports multi-turn conversations with system prompts,
    /// user messages, and assistant responses.
    ///
    /// # Arguments
    ///
    /// * `messages` - Conversation history as a sequence of messages
    /// * `options` - Generation options (temperature, max_tokens, etc.)
    ///
    /// # Returns
    ///
    /// The generated text response.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The API request fails
    /// - The message format is invalid
    /// - Token limits are exceeded
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::llm::{LlmProvider, LlmOptions, Message, Role};
    ///
    /// # async fn example(provider: &dyn LlmProvider) -> lumosai_core::Result<()> {
    /// let messages = vec![
    ///     Message {
    ///         role: Role::System,
    ///         content: "You are a helpful assistant.".to_string(),
    ///         metadata: None,
    ///         name: None,
    ///     },
    ///     Message {
    ///         role: Role::User,
    ///         content: "Hello!".to_string(),
    ///         metadata: None,
    ///         name: None,
    ///     },
    /// ];
    ///
    /// let options = LlmOptions::default();
    /// let response = provider.generate_with_messages(&messages, &options).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn generate_with_messages(
        &self,
        messages: &[Message],
        options: &LlmOptions,
    ) -> Result<String>;

    /// Generates a stream of text chunks from a prompt
    ///
    /// This method enables real-time streaming of the generated text,
    /// useful for displaying progressive responses to users.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The input prompt text
    /// * `options` - Generation options (temperature, max_tokens, etc.)
    ///
    /// # Returns
    ///
    /// A stream of text chunks. Each chunk is a `Result<String>`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The API request fails
    /// - Streaming is not supported by the provider
    /// - Network connectivity issues occur
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::llm::{LlmProvider, LlmOptions};
    /// use futures::StreamExt;
    ///
    /// # async fn example(provider: &dyn LlmProvider) -> lumosai_core::Result<()> {
    /// let options = LlmOptions::default();
    /// let mut stream = provider.generate_stream("Tell me a story", &options).await?;
    ///
    /// while let Some(chunk) = stream.next().await {
    ///     match chunk {
    ///         Ok(text) => print!("{}", text),
    ///         Err(e) => eprintln!("Error: {}", e),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn generate_stream<'a>(
        &'a self,
        prompt: &'a str,
        options: &'a LlmOptions,
    ) -> Result<BoxStream<'a, Result<String>>>;

    /// Generates embeddings for a text
    ///
    /// Embeddings are vector representations of text, useful for semantic
    /// search, similarity comparison, and RAG systems.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to generate embeddings for
    ///
    /// # Returns
    ///
    /// A vector of floating-point numbers representing the text embedding.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The provider doesn't support embeddings
    /// - The API request fails
    /// - The text is too long
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::llm::LlmProvider;
    ///
    /// # async fn example(provider: &dyn LlmProvider) -> lumosai_core::Result<()> {
    /// let embedding = provider.get_embedding("Hello, world!").await?;
    /// println!("Embedding dimension: {}", embedding.len());
    /// # Ok(())
    /// # }
    /// ```
    async fn get_embedding(&self, text: &str) -> Result<Vec<f32>>;

    /// Checks if the provider supports OpenAI-style function calling
    ///
    /// Function calling allows the LLM to invoke tools and functions
    /// natively, without regex-based parsing.
    ///
    /// # Returns
    ///
    /// `true` if function calling is supported, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::llm::LlmProvider;
    ///
    /// # fn example(provider: &dyn LlmProvider) {
    /// if provider.supports_function_calling() {
    ///     println!("Provider supports function calling");
    /// } else {
    ///     println!("Provider does not support function calling");
    /// }
    /// # }
    /// ```
    fn supports_function_calling(&self) -> bool {
        false
    }

    /// Generates text with function calling support
    ///
    /// This method allows the LLM to call functions/tools during generation.
    /// The model can choose to call one or more functions, or respond with text.
    ///
    /// # Arguments
    ///
    /// * `messages` - Conversation history
    /// * `functions` - Available function definitions
    /// * `tool_choice` - How the model should choose functions (auto, required, specific)
    /// * `options` - Generation options
    ///
    /// # Returns
    ///
    /// A [`FunctionCallingResponse`] containing either text content,
    /// function calls, or both.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The provider doesn't support function calling
    /// - The function definitions are invalid
    /// - The API request fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::llm::{LlmProvider, LlmOptions, Message, Role};
    /// use lumosai_core::llm::function_calling::{FunctionDefinition, ToolChoice};
    /// use serde_json::json;
    ///
    /// # async fn example(provider: &dyn LlmProvider) -> lumosai_core::Result<()> {
    /// let messages = vec![
    ///     Message {
    ///         role: Role::User,
    ///         content: "What's the weather in Tokyo?".to_string(),
    ///         metadata: None,
    ///         name: None,
    ///     },
    /// ];
    ///
    /// let functions = vec![
    ///     FunctionDefinition {
    ///         name: "get_weather".to_string(),
    ///         description: Some("Get weather for a city".to_string()),
    ///         parameters: json!({
    ///             "type": "object",
    ///             "properties": {
    ///                 "city": {"type": "string"}
    ///             },
    ///             "required": ["city"]
    ///         }),
    ///     },
    /// ];
    ///
    /// let options = LlmOptions::default();
    /// let response = provider.generate_with_functions(
    ///     &messages,
    ///     &functions,
    ///     &ToolChoice::Auto,
    ///     &options,
    /// ).await?;
    ///
    /// if !response.function_calls.is_empty() {
    ///     println!("Model wants to call: {}", response.function_calls[0].name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn generate_with_functions(
        &self,
        messages: &[Message],
        functions: &[FunctionDefinition],
        tool_choice: &ToolChoice,
        options: &LlmOptions,
    ) -> Result<FunctionCallingResponse> {
        // Default implementation for providers that don't support function calling
        let _ = (functions, tool_choice);
        let response = self.generate_with_messages(messages, options).await?;
        Ok(FunctionCallingResponse {
            content: Some(response),
            function_calls: Vec::new(),
            finish_reason: "stop".to_string(),
        })
    }

    /// Checks if the provider supports native structured output
    ///
    /// Structured output allows the LLM to return responses that conform
    /// to a specific JSON schema, ensuring type safety and validation.
    ///
    /// # Returns
    ///
    /// `true` if structured output is supported, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::llm::LlmProvider;
    ///
    /// # fn example(provider: &dyn LlmProvider) {
    /// if provider.supports_structured_output() {
    ///     println!("Provider supports structured output");
    /// }
    /// # }
    /// ```
    fn supports_structured_output(&self) -> bool {
        false
    }

    /// Generates text with structured output support
    ///
    /// This method uses the LLM's native structured output API (e.g., OpenAI's
    /// `response_format` or Anthropic's `structured_outputs`) to ensure the
    /// response conforms to the provided JSON schema.
    ///
    /// # Arguments
    ///
    /// * `messages` - Conversation history
    /// * `schema` - JSON Schema that the response must conform to
    /// * `options` - Generation options
    ///
    /// # Returns
    ///
    /// A JSON `Value` that conforms to the provided schema.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The provider doesn't support structured output
    /// - The schema is invalid
    /// - The API request fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::llm::{LlmProvider, LlmOptions, Message, Role};
    /// use serde_json::json;
    ///
    /// # async fn example(provider: &dyn LlmProvider) -> lumosai_core::Result<()> {
    /// let messages = vec![
    ///     Message {
    ///         role: Role::User,
    ///         content: "Extract tasks from: Buy milk, Call mom".to_string(),
    ///         metadata: None,
    ///         name: None,
    ///     },
    /// ];
    ///
    /// let schema = json!({
    ///     "type": "object",
    ///     "properties": {
    ///         "tasks": {
    ///             "type": "array",
    ///             "items": {"type": "string"}
    ///         }
    ///     },
    ///     "required": ["tasks"]
    /// });
    ///
    /// let options = LlmOptions::default();
    /// let response = provider.generate_structured(&messages, &schema, &options).await?;
    /// println!("Structured response: {}", response);
    /// # Ok(())
    /// # }
    /// ```
    async fn generate_structured(
        &self,
        messages: &[Message],
        schema: &serde_json::Value,
        options: &LlmOptions,
    ) -> Result<serde_json::Value> {
        // Default implementation: fallback to regular generation
        // This should be overridden by providers that support structured output
        let _ = schema;
        let response = self.generate_with_messages(messages, options).await?;
        // Try to parse as JSON, but this is not guaranteed to match the schema
        Ok(serde_json::from_str(&response)
            .unwrap_or_else(|_| serde_json::json!({ "content": response })))
    }
}

/// Response from a function calling enabled LLM
///
/// This structure contains the result of a function calling request,
/// which may include text content, function calls, or both.
///
/// # Examples
///
/// ## Text Response
///
/// ```rust
/// use lumosai_core::llm::provider::FunctionCallingResponse;
///
/// let response = FunctionCallingResponse {
///     content: Some("I can help you with that!".to_string()),
///     function_calls: vec![],
///     finish_reason: "stop".to_string(),
/// };
/// ```
///
/// ## Function Call Response
///
/// ```rust
/// use lumosai_core::llm::provider::FunctionCallingResponse;
/// use lumosai_core::llm::function_calling::FunctionCall;
///
/// let response = FunctionCallingResponse {
///     content: None,
///     function_calls: vec![
///         FunctionCall {
///             id: Some("call_123".to_string()),
///             name: "get_weather".to_string(),
///             arguments: r#"{"city": "Tokyo"}"#.to_string(),
///         },
///     ],
///     finish_reason: "function_call".to_string(),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct FunctionCallingResponse {
    /// Text content from the model (if any)
    ///
    /// This is `None` when the model only makes function calls.
    pub content: Option<String>,

    /// Function calls made by the model
    ///
    /// Empty when the model responds with text only.
    pub function_calls: Vec<FunctionCall>,

    /// Reason the generation finished
    ///
    /// Common values: "stop", "length", "function_call", "content_filter"
    pub finish_reason: String,
}
