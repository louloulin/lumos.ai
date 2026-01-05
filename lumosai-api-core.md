# LumosAI 统一 API 核心架构设计

**核心理念**: 统一的 API 抽象层 + 多提供商支持 + 智能路由
**设计目标**: 零成本抽象、类型安全、易于扩展

---

## 📊 当前 API 实现分析

### 已有优势 ✅

LumosAI 已经实现了**优秀的 LLM API 抽象层**：

1. **统一的 `LlmProvider` trait**
   - 标准化的接口定义
   - 支持所有主流操作（生成、流式、嵌入、函数调用）
   - 类型安全的消息和选项系统

2. **丰富的提供商支持** (12+ 提供商)
   - 国际: OpenAI, Anthropic, Claude, Cohere, Gemini, Together, Ollama
   - 中国: Qwen, Zhipu, Baidu, DeepSeek, Huawei MaaS

3. **智能路由系统**
   - 基于负载、成本、延迟的路由策略
   - Provider 统计和监控
   - 自动故障转移

4. **便捷的工厂函数**
   - `providers::openai_from_env()`
   - `providers::auto_provider()` - 自动选择可用提供商
   - 简化 API 创建流程

### 存在的问题 ⚠️

1. **缺少统一的配置管理**
   - 每个 Provider 配置方式不一致
   - 缺少集中式配置文件

2. **API 密钥管理不完善**
   - 环境变量方式（单一）
   - 缺少密钥轮换和加密

3. **错误处理不够友好**
   - Provider 错误信息不统一
   - 缺少重试和降级机制

4. **监控和可观测性不足**
   - 缺少详细的调用日志
   - 性能指标收集不完整

---

## 🎯 统一 API 核心架构

### 架构分层

```
┌─────────────────────────────────────────────────────┐
│                   Application Layer                  │
│              (Agent, Workflow, RAG, etc.)            │
└─────────────────────────────────────────────────────┘
                         ▲
                         │
┌─────────────────────────────────────────────────────┐
│              Unified API Facade (v2)                 │
│  - Simple API: agent!("gpt-4", "Hello")             │
│  - Builder API: Agent::builder().model("gpt-4")     │
│  - Auto-discovery: auto_provider()                  │
└─────────────────────────────────────────────────────┘
                         ▲
                         │
┌─────────────────────────────────────────────────────┐
│              LlmProvider Trait (Core)                │
│  - generate()                                       │
│  - generate_stream()                                │
│  - generate_with_functions()                        │
│  - embed()                                          │
└─────────────────────────────────────────────────────┘
                         ▲
                         │
┌─────────────────────────────────────────────────────┐
│           Provider Implementations (12+)             │
│  OpenAI | Anthropic | Qwen | Zhipu | ...           │
└─────────────────────────────────────────────────────┘
                         ▲
                         │
┌─────────────────────────────────────────────────────┐
│              Smart Router & Load Balancer            │
│  - Routing Strategy                                 │
│  - Failover                                         │
│  - Rate Limiting                                    │
└─────────────────────────────────────────────────────┘
```

---

## 🔧 核心组件设计

### 1. 统一 API Facade (v2)

**目标**: 提供最简单的使用体验

```rust
// lumosai_core/src/llm/facade.rs

use crate::Result;

/// 最简化的 API - 宏方式
#[macro_export]
macro_rules! llm {
    // 单次调用
    ($prompt:expr) => {
        llm!("gpt-4", $prompt)
    };

    // 指定模型
    ($model:expr, $prompt:expr) => {{
        use lumosai_core::llm::LlmProvider;
        let provider = lumosai_core::llm::create_provider($model)?;
        provider.generate($prompt, &Default::default()).await?
    }};

    // 带选项
    ($model:expr, $prompt:expr, $($key:ident = $value:expr),*) => {{
        use lumosai_core::llm::{LlmProvider, LlmOptions};
        let provider = lumosai_core::llm::create_provider($model)?;
        let options = LlmOptions::default()
            $(.$key($value))*;
        provider.generate($prompt, &options).await?
    }};
}

/// 统一 API 函数
pub struct LlmApi;

impl LlmApi {
    /// 自动选择最佳 Provider
    pub async fn auto(prompt: &str) -> Result<String> {
        let provider = providers::auto_provider()?;
        provider.generate(prompt, &Default::default()).await
    }

    /// 指定模型
    pub async fn generate(model: &str, prompt: &str) -> Result<String> {
        let provider = create_provider(model)?;
        provider.generate(prompt, &Default::default()).await
    }

    /// 带选项生成
    pub async fn generate_with_options(
        model: &str,
        prompt: &str,
        options: &LlmOptions,
    ) -> Result<String> {
        let provider = create_provider(model)?;
        provider.generate(prompt, options).await
    }

    /// 流式生成
    pub async fn generate_stream(
        model: &str,
        prompt: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let provider = create_provider(model)?;
        provider.generate_stream(prompt, &Default::default()).await
    }

    /// 批量生成
    pub async fn generate_batch(
        model: &str,
        prompts: Vec<&str>,
    ) -> Result<Vec<String>> {
        let provider = create_provider(model)?;
        let mut results = Vec::new();

        for prompt in prompts {
            let result = provider.generate(prompt, &Default::default()).await?;
            results.push(result);
        }

        Ok(results)
    }
}

/// 智能模型解析器
fn create_provider(model: &str) -> Result<Box<dyn LlmProvider>> {
    // 支持的模型别名映射
    let model_aliases = HashMap::from([
        // OpenAI 别名
        ("gpt-3.5", "gpt-3.5-turbo"),
        ("gpt-4", "gpt-4"),
        ("gpt-4-turbo", "gpt-4-turbo-preview"),

        // Claude 别名
        ("claude", "claude-3-sonnet-20240229"),
        ("claude-3", "claude-3-sonnet-20240229"),
        ("claude-opus", "claude-3-opus-20240229"),

        // Qwen 别名
        ("qwen", "qwen-turbo"),
        ("tongyi", "qwen-turbo"),

        // Zhipu 别名
        ("zhipu", "glm-4"),
        ("chatglm", "glm-4"),

        // 本地模型
        ("llama", "ollama/llama2"),
        ("mistral", "ollama/mistral"),
    ]);

    // 解析模型名称
    let (provider_name, model_name) = parse_model_string(model)?;

    match provider_name.as_str() {
        "openai" | "gpt" => Ok(Box::new(providers::openai_from_env()?)),
        "anthropic" | "claude" => Ok(Box::new(providers::claude_from_env()?)),
        "qwen" | "tongyi" => Ok(Box::new(providers::qwen_from_env()?)),
        "zhipu" | "chatglm" => Ok(Box::new(providers::zhipu_from_env()?)),
        "deepseek" => Ok(Box::new(providers::deepseek_from_env()?)),
        "ollama" => Ok(Box::new(providers::ollama_local(
            model_name.unwrap_or("llama2".to_string())
        ))),
        _ => {
            // 尝试自动检测
            if let Ok(provider) = providers::auto_provider() {
                Ok(provider)
            } else {
                Err(Error::Llm(format!(
                    "Unknown provider: {}. Available: openai, claude, qwen, zhipu, deepseek, ollama",
                    provider_name
                )))
            }
        }
    }
}

/// 解析模型字符串
/// 支持格式:
/// - "gpt-4" -> (provider="openai", model="gpt-4")
/// - "openai/gpt-4" -> (provider="openai", model="gpt-4")
/// - "ollama/llama2" -> (provider="ollama", model="llama2")
fn parse_model_string(model: &str) -> Result<(String, Option<String>)> {
    if model.contains('/') {
        let parts: Vec<&str> = model.splitn(2, '/').collect();
        Ok((parts[0].to_string(), Some(parts[1].to_string())))
    } else {
        // 自动推断 provider
        let provider = infer_provider_from_model(model)?;
        Ok((provider, Some(model.to_string())))
    }
}

fn infer_provider_from_model(model: &str) -> Result<String> {
    if model.starts_with("gpt") {
        Ok("openai".to_string())
    } else if model.starts_with("claude") {
        Ok("claude".to_string())
    } else if model.starts_with("qwen") || model.starts_with("qwen-") {
        Ok("qwen".to_string())
    } else if model.starts_with("glm") {
        Ok("zhipu".to_string())
    } else if model.starts_with("deepseek") {
        Ok("deepseek".to_string())
    } else {
        Ok("openai".to_string()) // 默认
    }
}
```

**使用示例**:

```rust
// 最简单的方式
let response = llm!("Hello, world!").await?;
println!("{}", response);

// 指定模型
let response = llm!("gpt-4", "Explain Rust").await?;

// 使用本地模型
let response = llm!("ollama/llama2", "Hello").await?;

// 带选项
let response = llm!(
    "gpt-4",
    "Write code",
    temperature = 0.7,
    max_tokens = 1000
).await?;

// 使用函数 API
let response = LlmApi::auto("Hello").await?;
let response = LlmApi::generate("gpt-4", "Hello").await?;

// 流式生成
let mut stream = LlmApi::generate_stream("gpt-4", "Tell me a story").await?;
while let Some(chunk) = stream.next().await {
    print!("{}", chunk?);
}
```

### 2. 增强的配置系统

**目标**: 统一配置管理，支持多种配置源

```rust
// lumosai_core/src/llm/config.rs

use serde::{Deserialize, Serialize};
use std::path::Path;

/// 统一的 LLM 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// 默认模型
    pub default_model: String,

    /// Provider 配置
    pub providers: HashMap<String, ProviderConfig>,

    /// 全局选项
    pub defaults: LlmDefaults,

    /// 路由配置
    pub routing: RoutingConfig,

    /// 监控配置
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// API 密钥
    #[serde(skip_serializing)]
    pub api_key: Option<String>,

    /// Base URL
    pub base_url: Option<String>,

    /// 模型映射
    pub models: HashMap<String, String>,

    /// 并发限制
    pub max_concurrency: Option<usize>,

    /// 超时设置
    pub timeout_secs: Option<u64>,

    /// 重试配置
    pub retry: RetryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmDefaults {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    pub strategy: String, // "round-robin", "least-load", "least-cost"
    pub enable_failover: bool,
    pub health_check_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enable_logging: bool,
    pub enable_metrics: bool,
    pub log_level: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            default_model: "gpt-4".to_string(),
            providers: HashMap::new(),
            defaults: LlmDefaults {
                temperature: Some(0.7),
                max_tokens: Some(2000),
                top_p: None,
                stream: false,
            },
            routing: RoutingConfig {
                strategy: "balanced".to_string(),
                enable_failover: true,
                health_check_interval_secs: 60,
            },
            monitoring: MonitoringConfig {
                enable_logging: true,
                enable_metrics: true,
                log_level: "info".to_string(),
            },
        }
    }
}

impl LlmConfig {
    /// 从文件加载配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: LlmConfig = serde_yaml::from_str(&content)
            .map_err(|e| Error::Configuration {
                path: path.as_ref().to_string_lossy().to_string(),
                message: e.to_string(),
                line: None,
            })?;
        Ok(config)
    }

    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();

        // 从环境变量读取 API 密钥
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            config.providers.insert("openai".to_string(), ProviderConfig {
                api_key: Some(key),
                base_url: None,
                models: HashMap::new(),
                max_concurrency: Some(10),
                timeout_secs: Some(30),
                retry: RetryConfig::default(),
            });
        }

        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            config.providers.insert("anthropic".to_string(), ProviderConfig {
                api_key: Some(key),
                base_url: None,
                models: HashMap::new(),
                max_concurrency: Some(5),
                timeout_secs: Some(30),
                retry: RetryConfig::default(),
            });
        }

        // 支持更多提供商...

        Ok(config)
    }

    /// 合并配置（文件 + 环境变量）
    pub fn merge(mut self, other: LlmConfig) -> Self {
        // 合并 providers
        for (key, value) in other.providers {
            self.providers.insert(key, value);
        }

        // 合并 defaults
        if other.defaults.temperature.is_some() {
            self.defaults.temperature = other.defaults.temperature;
        }

        self
    }

    /// 保存配置到文件
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_yaml::to_string(self)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
```

**配置文件示例** (`lumosai.yaml`):

```yaml
# LumosAI 配置文件

default_model: "gpt-4"

defaults:
  temperature: 0.7
  max_tokens: 2000
  stream: false

providers:
  openai:
    api_key: "${OPENAI_API_KEY}"
    base_url: "https://api.openai.com/v1"
    max_concurrency: 10
    timeout_secs: 30
    retry:
      max_attempts: 3
      initial_delay_ms: 1000
      max_delay_ms: 10000
      backoff_multiplier: 2.0

  claude:
    api_key: "${ANTHROPIC_API_KEY}"
    max_concurrency: 5
    timeout_secs: 30

  qwen:
    api_key: "${QWEN_API_KEY}"
    base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1"
    max_concurrency: 10

  ollama:
    base_url: "http://localhost:11434"
    models:
      llama2: "llama2"
      mistral: "mistral"

routing:
  strategy: "balanced"
  enable_failover: true
  health_check_interval_secs: 60

monitoring:
  enable_logging: true
  enable_metrics: true
  log_level: "info"
```

**使用示例**:

```rust
// 从配置文件创建 API
let config = LlmConfig::from_file("lumosai.yaml")?;
let config = config.merge(LlmConfig::from_env()?);

let api = LlmApi::with_config(config);

let response = api.generate("Hello").await?;
```

### 3. 增强的错误处理和重试

```rust
// lumosai_core/src/llm/retry.rs

use crate::{Error, Result};

/// 带重试的 LLM 调用
pub struct RetryWrapper {
    provider: Box<dyn LlmProvider>,
    config: RetryConfig,
}

impl RetryWrapper {
    pub fn new(provider: Box<dyn LlmProvider>, config: RetryConfig) -> Self {
        Self { provider, config }
    }

    /// 执行带重试的操作
    pub async fn execute_with_retry<F, T>(
        &self,
        operation: F,
    ) -> Result<T>
    where
        F: Fn() -> Pin<Box<dyn Future<Output = Result<T>> + Send>>,
    {
        let mut delay = self.config.initial_delay_ms;
        let mut last_error = None;

        for attempt in 0..self.config.max_attempts {
            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        tracing::info!(
                            "Operation succeeded after {} retries",
                            attempt
                        );
                    }
                    return Ok(result);
                }
                Err(e) => {
                    last_error = Some(e.clone());

                    // 判断是否可重试
                    if !e.is_retryable() {
                        return Err(e);
                    }

                    if attempt < self.config.max_attempts - 1 {
                        tracing::warn!(
                            "Attempt {} failed, retrying in {}ms: {}",
                            attempt + 1,
                            delay,
                            e
                        );

                        tokio::time::sleep(Duration::from_millis(delay)).await;

                        // 指数退避
                        delay = std::cmp::min(
                            (delay as f32 * self.config.backoff_multiplier) as u64,
                            self.config.max_delay_ms,
                        );
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            Error::Internal {
                message: "Max retries exceeded".to_string(),
                source: None,
            }
        }))
    }
}

impl Error {
    /// 判断错误是否可重试
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::Network { .. } => true,
            Error::LlmProvider {
                status_code: Some(code),
                ..
            } if *code >= 500 || *code == 429 => true,
            _ => false,
        }
    }
}
```

### 4. 监控和可观测性

```rust
// lumosai_core/src/llm/telemetry.rs

use opentelemetry::trace::{Span, Tracer};
use opentelemetry::metrics::Meter;

/// LLM 调用追踪
pub struct LlmTelemetry {
    tracer: Box<dyn Tracer + Send + Sync>,
    meter: Box<dyn Meter + Send + Sync>,
}

impl LlmTelemetry {
    /// 追踪 LLM 调用
    pub fn trace_llm_call<F, T>(
        &self,
        provider: &str,
        model: &str,
        operation: F,
    ) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        let span = self.tracer
            .span_builder("llm.call")
            .with_attribute("provider", provider)
            .with_attribute("model", model)
            .start(&self.tracer);

        let start = std::time::Instant::now();

        let result = operation();

        let duration = start.elapsed();

        match &result {
            Ok(_) => {
                span.add_event("success", vec![]);
                self.record_success(provider, model, duration);
            }
            Err(e) => {
                span.add_event("error", vec![]);
                self.record_error(provider, model, duration, e);
            }
        }

        span.end();

        result
    }

    fn record_success(&self, provider: &str, model: &str, duration: Duration) {
        // 记录成功指标
        self.meter
            .u64_counter("llm.calls.total")
            .with_description("Total LLM calls")
            .build()
            .record(
                1,
                &[("provider", provider), ("model", model), ("status", "success")],
            );

        self.meter
            .f64_histogram("llm.duration")
            .with_description("LLM call duration")
            .build()
            .record(
                duration.as_secs_f64(),
                &[("provider", provider), ("model", model)],
            );
    }

    fn record_error(&self, provider: &str, model: &str, duration: Duration, error: &Error) {
        self.meter
            .u64_counter("llm.calls.total")
            .with_description("Total LLM calls")
            .build()
            .record(
                1,
                &[("provider", provider), ("model", model), ("status", "error")],
            );

        self.meter
            .u64_counter("llm.errors.total")
            .with_description("Total LLM errors")
            .build()
            .record(
                1,
                &[
                    ("provider", provider),
                    ("model", model),
                    ("error_type", error.error_type()),
                ],
            );
    }
}
```

---

## 🚀 完整的使用示例

### 示例 1: 最简单的使用

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 使用宏（最简单）
    let response = llm!("Hello, world!").await?;
    println!("{}", response);

    Ok(())
}
```

### 示例 2: 指定模型

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // OpenAI
    let response = llm!("gpt-4", "Explain Rust in simple terms").await?;

    // Claude
    let response = llm!("claude-3", "Write a poem").await?;

    // Qwen (中国)
    let response = llm!("qwen", "用中文介绍你自己").await?;

    // 本地模型
    let response = llm!("ollama/llama2", "Hello").await?;

    Ok(())
}
```

### 示例 3: 使用配置文件

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 加载配置
    let config = LlmConfig::from_file("lumosai.yaml")?
        .merge(LlmConfig::from_env()?);

    // 创建配置化的 API
    let api = LlmApi::with_config(config);

    // 使用默认模型
    let response = api.generate("Hello").await?;

    Ok(())
}
```

### 示例 4: 智能路由

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建智能路由器
    let router = LlmRouter::builder()
        .add_provider(openai_from_env()?)
        .add_provider(claude_from_env()?)
        .add_provider(qwen_from_env()?)
        .strategy(RoutingStrategy::Balanced)
        .enable_failover(true)
        .build();

    // 自动选择最佳提供商
    let response = router.generate("Hello", &Default::default()).await?;

    println!("Used provider: {}", router.last_used_provider());

    Ok(())
}
```

### 示例 5: 流式生成

```rust
use lumosai::prelude::*;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    let mut stream = LlmApi::generate_stream("gpt-4", "Tell me a story").await?;

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(text) => print!("{}", text),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    println!();

    Ok(())
}
```

### 示例 6: 批量处理

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let prompts = vec![
        "What is Rust?",
        "Explain async/await",
        "What are traits?",
    ];

    let results = LlmApi::generate_batch("gpt-3.5-turbo", prompts).await?;

    for (i, response) in results.iter().enumerate() {
        println!("Q{}: {}\nA: {}\n", i + 1, prompts[i], response);
    }

    Ok(())
}
```

### 示例 7: 带重试和监控

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let config = RetryConfig {
        max_attempts: 3,
        initial_delay_ms: 1000,
        max_delay_ms: 10000,
        backoff_multiplier: 2.0,
    };

    let provider = openai_from_env()?;
    let wrapped = RetryWrapper::new(Box::new(provider), config);

    let response = wrapped
        .execute_with_retry(|| async {
            wrapped.provider.generate("Hello", &Default::default()).await
        })
        .await?;

    println!("{}", response);

    Ok(())
}
```

---

## 📚 API 提供商扩展指南

### 添加新的 Provider

**步骤**: 3 步完成

#### 1. 实现 `LlmProvider` trait

```rust
// lumosai_core/src/llm/custom_provider.rs

use crate::llm::{LlmProvider, LlmOptions, Message, EmbeddingOptions};

pub struct CustomProvider {
    api_key: String,
    base_url: String,
    model: String,
}

impl CustomProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.custom.com/v1".to_string(),
            model,
        }
    }
}

#[async_trait]
impl LlmProvider for CustomProvider {
    async fn generate(
        &self,
        prompt: &str,
        options: &LlmOptions,
    ) -> Result<String> {
        // 实现生成逻辑
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": self.model,
                "prompt": prompt,
                "temperature": options.temperature,
            }))
            .send()
            .await?
            .error_for_status()?;

        let result: serde_json::Value = response.json().await?;
        Ok(result["text"].as_str().unwrap().to_string())
    }

    async fn generate_with_messages(
        &self,
        messages: &[Message],
        options: &LlmOptions,
    ) -> Result<String> {
        // 实现多轮对话
    }

    async fn generate_stream(
        &self,
        prompt: &str,
        options: &LlmOptions,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        // 实现流式生成
    }

    async fn embed(
        &self,
        texts: &[&str],
        options: &EmbeddingOptions,
    ) -> Result<Vec<Vec<f32>>> {
        // 实现嵌入
    }

    fn name(&self) -> &str {
        "custom"
    }

    fn model(&self) -> &str {
        &self.model
    }
}
```

#### 2. 添加工厂函数

```rust
// lumosai_core/src/llm/providers.rs

/// 创建 Custom Provider
pub fn custom(api_key: String, model: String) -> CustomProvider {
    CustomProvider::new(api_key, model)
}

/// 从环境变量创建 Custom Provider
pub fn custom_from_env() -> Result<CustomProvider> {
    let api_key = std::env::var("CUSTOM_API_KEY").map_err(|_| {
        Error::Llm("CUSTOM_API_KEY environment variable not set".to_string())
    })?;
    Ok(custom(api_key, "custom-model".to_string()))
}
```

#### 3. 注册到自动发现

```rust
// lumosai_core/src/llm/providers.rs

pub fn auto_provider() -> Result<Box<dyn LlmProvider>> {
    // 现有 providers...

    // 添加新的 provider
    if let Ok(provider) = custom_from_env() {
        return Ok(Box::new(provider));
    }

    Err(Error::Llm("No provider found in environment".to_string()))
}
```

---

## 🎯 核心优势总结

### 1. 统一的 API 抽象

✅ **一个接口，多种提供商**
- 统一的 `LlmProvider` trait
- 标准化的消息和选项
- 类型安全的保证

✅ **零成本抽象**
- 编译时优化
- 无运行时开销
- Rust 的零成本抽象

### 2. 灵活的配置方式

✅ **多种配置源**
- 环境变量
- 配置文件 (YAML)
- 代码配置
- 运行时动态配置

✅ **智能模型解析**
- "gpt-4" → OpenAI
- "claude-3" → Anthropic
- "qwen" → 阿里云
- "ollama/llama2" → 本地

### 3. 企业级特性

✅ **智能路由**
- 负载均衡
- 故障转移
- 成本优化

✅ **可靠性**
- 自动重试
- 错误处理
- 超时控制

✅ **可观测性**
- 调用追踪
- 性能监控
- 错误统计

### 4. 开发者友好

✅ **极简 API**
```rust
llm!("Hello")  // 最简单
llm!("gpt-4", "Hello")  // 指定模型
```

✅ **渐进式复杂度**
```rust
// 简单场景
LlmApi::auto("Hello")

// 复杂场景
LlmApi::with_config(config)
    .generate("Hello")
    .await?
```

✅ **完整的工具链**
- CLI 工具
- 配置管理
- 监控面板

---

## 📈 迁移路径

### 从现有代码迁移

**旧代码**:
```rust
let provider = OpenAiProvider::new(api_key, model);
let response = provider.generate("Hello", &options).await?;
```

**新代码** (更简单):
```rust
let response = llm!("Hello").await?;

// 或
let response = LlmApi::generate("gpt-4", "Hello").await?;
```

### 向后兼容

✅ **完全兼容现有 API**
- 所有现有的 Provider 实现保持不变
- 新的 Facade 层是可选的
- 渐进式迁移

---

## 🎉 结论

LumosAI 的统一 API 核心架构提供：

1. **极简的使用体验** - `llm!("Hello")`
2. **强大的扩展能力** - 轻松添加新 Provider
3. **企业级的可靠性** - 重试、监控、路由
4. **零成本抽象** - Rust 的性能保证
5. **完全的类型安全** - 编译时检查

这个设计将成为 LumosAI 1.2 的核心基础！🚀
