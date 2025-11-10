# 🎯 LumosAI 最佳实践指南

> **生产环境的最佳实践和设计模式**

---

## 目录

1. [Agent 设计模式](#1-agent-设计模式)
2. [工具设计原则](#2-工具设计原则)
3. [记忆管理策略](#3-记忆管理策略)
4. [工作流设计模式](#4-工作流设计模式)
5. [错误处理](#5-错误处理)
6. [性能优化](#6-性能优化)
7. [安全最佳实践](#7-安全最佳实践)
8. [测试策略](#8-测试策略)
9. [监控和可观测性](#9-监控和可观测性)
10. [代码组织](#10-代码组织)

---

## 1. Agent 设计模式

### 1.1 单一职责原则

**✅ 推荐**:
```rust
// 每个 Agent 专注于一个特定任务
let research_agent = AgentBuilder::new()
    .name("researcher")
    .instructions("You are a research specialist. Focus on gathering accurate information.")
    .model(llm.clone())
    .build()?;

let writing_agent = AgentBuilder::new()
    .name("writer")
    .instructions("You are a content writer. Focus on creating engaging content.")
    .model(llm.clone())
    .build()?;
```

**❌ 不推荐**:
```rust
// Agent 承担过多职责
let super_agent = AgentBuilder::new()
    .name("do_everything")
    .instructions("You can do research, write code, analyze data, create content, etc.")
    .model(llm)
    .build()?;
```

**原因**: 专业化的 Agent 更容易控制、测试和维护。

### 1.2 使用 Agent 组合而非复杂单体

**✅ 推荐**:
```rust
// 使用多个简单的 Agent 协作
async fn process_article(topic: &str) -> Result<String> {
    // 研究阶段
    let research = research_agent.generate_simple(
        &format!("Research key points about: {}", topic)
    ).await?;
    
    // 写作阶段
    let draft = writing_agent.generate_simple(
        &format!("Write an article based on: {}", research)
    ).await?;
    
    // 编辑阶段
    let final_content = editing_agent.generate_simple(
        &format!("Polish this article: {}", draft)
    ).await?;
    
    Ok(final_content)
}
```

**❌ 不推荐**:
```rust
// 使用一个复杂的 prompt 完成所有任务
let result = agent.generate_simple(
    "Research, write, and edit a complete article about AI in one go"
).await?;
```

### 1.3 使用配置模板

```rust
// 创建可复用的 Agent 配置
struct AgentTemplate {
    instructions: String,
    temperature: f32,
    max_tokens: usize,
}

impl AgentTemplate {
    fn customer_service() -> Self {
        Self {
            instructions: "You are a helpful customer service representative.".to_string(),
            temperature: 0.3,  // 更精确
            max_tokens: 1000,
        }
    }
    
    fn creative_writer() -> Self {
        Self {
            instructions: "You are a creative content writer.".to_string(),
            temperature: 0.9,  // 更有创造性
            max_tokens: 2000,
        }
    }
    
    fn build(&self, name: &str, llm: Arc<dyn LlmProvider>) -> Result<BasicAgent> {
        AgentBuilder::new()
            .name(name)
            .instructions(&self.instructions)
            .model(llm)
            .temperature(self.temperature)
            .max_tokens(self.max_tokens)
            .build()
    }
}

// 使用模板
let cs_agent = AgentTemplate::customer_service().build("cs1", llm.clone())?;
let writer = AgentTemplate::creative_writer().build("writer1", llm)?;
```

### 1.4 动态配置的使用场景

**适合使用动态配置的情况**:
```rust
// 根据任务复杂度调整参数
let dynamic_temp = DynamicArgument::Dynamic(Box::new(|context| {
    Box::pin(async move {
        let temp = match context.complexity {
            ComplexityLevel::Simple => 0.3,
            ComplexityLevel::Complex => 0.7,
            ComplexityLevel::Expert => 0.9,
        };
        Ok(temp)
    })
}));
```

**不适合使用动态配置的情况**:
- 固定的业务逻辑
- 性能关键路径（动态配置有额外开销）
- 简单的应用场景

---

## 2. 工具设计原则

### 2.1 工具应该是纯函数

**✅ 推荐**:
```rust
#[tool(name = "calculate", description = "Performs calculation")]
async fn calculate(a: f64, b: f64, op: String) -> Result<Value> {
    // 无副作用，结果仅依赖输入
    let result = match op.as_str() {
        "add" => a + b,
        "subtract" => a - b,
        "multiply" => a * b,
        "divide" => a / b,
        _ => return Err(Error::invalid_input("Unsupported operation")),
    };
    Ok(json!({ "result": result }))
}
```

**❌ 不推荐**:
```rust
// 有副作用的工具
static mut COUNTER: i32 = 0;

#[tool(name = "bad_tool", description = "Has side effects")]
async fn bad_tool(input: String) -> Result<Value> {
    unsafe {
        COUNTER += 1;  // 副作用
        println!("Called {} times", COUNTER);  // 副作用
    }
    Ok(json!({}))
}
```

### 2.2 输入验证

**✅ 推荐**:
```rust
#[tool(name = "send_email", description = "Sends an email")]
async fn send_email(to: String, subject: String, body: String) -> Result<Value> {
    // 验证输入
    if !to.contains('@') {
        return Err(Error::invalid_input("Invalid email address"));
    }
    
    if subject.is_empty() {
        return Err(Error::invalid_input("Subject cannot be empty"));
    }
    
    if body.len() > 10000 {
        return Err(Error::invalid_input("Body too long"));
    }
    
    // 执行发送逻辑
    send_email_internal(&to, &subject, &body).await?;
    
    Ok(json!({ "status": "sent", "to": to }))
}
```

### 2.3 错误处理和日志

```rust
use tracing::{info, warn, error};

#[tool(name = "api_call", description = "Calls external API")]
async fn api_call(endpoint: String, data: Value) -> Result<Value> {
    info!(endpoint = %endpoint, "Calling external API");
    
    match call_external_api(&endpoint, &data).await {
        Ok(response) => {
            info!(endpoint = %endpoint, "API call successful");
            Ok(response)
        }
        Err(e) => {
            error!(endpoint = %endpoint, error = %e, "API call failed");
            Err(Error::external_error(format!("API call failed: {}", e)))
        }
    }
}
```

### 2.4 超时和重试

```rust
use tokio::time::{timeout, Duration};

#[tool(name = "resilient_tool", description = "Tool with timeout and retry")]
async fn resilient_tool(input: String) -> Result<Value> {
    let max_retries = 3;
    let timeout_duration = Duration::from_secs(5);
    
    for attempt in 1..=max_retries {
        match timeout(timeout_duration, perform_operation(&input)).await {
            Ok(Ok(result)) => return Ok(result),
            Ok(Err(e)) if e.is_retryable() && attempt < max_retries => {
                warn!("Attempt {} failed, retrying...", attempt);
                tokio::time::sleep(Duration::from_millis(100 * attempt as u64)).await;
                continue;
            }
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                if attempt < max_retries {
                    warn!("Timeout on attempt {}, retrying...", attempt);
                    continue;
                } else {
                    return Err(Error::timeout("Operation timed out"));
                }
            }
        }
    }
    
    Err(Error::max_retries_exceeded())
}
```

---

## 3. 记忆管理策略

### 3.1 选择合适的记忆类型

**WorkingMemory**: 短期对话
```rust
// 适用于：客服对话、单次会话
let memory = create_working_memory(WorkingMemoryConfig {
    max_messages: 20,
    max_tokens: 4000,
});
```

**SemanticMemory**: 长期知识
```rust
// 适用于：知识库、用户偏好、历史记录
let memory = SemanticMemory::new(MemoryConfig {
    vector_store: "qdrant".to_string(),
    embedding_model: "text-embedding-ada-002".to_string(),
    similarity_threshold: 0.75,
    max_results: 5,
})?;
```

**UnifiedMemory**: 综合场景
```rust
// 适用于：复杂应用、需要多种记忆类型
let memory = UnifiedMemory::new(UnifiedMemoryConfig {
    enable_working: true,
    enable_semantic: true,
    enable_persistent: true,
    ..Default::default()
})?;
```

### 3.2 记忆清理策略

```rust
// 定期清理过期记忆
async fn cleanup_memory(memory: &mut WorkingMemory) -> Result<()> {
    let cutoff_time = Utc::now() - Duration::from_secs(3600);  // 1小时前
    
    memory.remove_before(cutoff_time).await?;
    
    // 或者基于消息数量限制
    while memory.message_count() > 100 {
        memory.remove_oldest().await?;
    }
    
    Ok(())
}
```

### 3.3 记忆隔离

```rust
// 为不同用户或会话隔离记忆
async fn get_user_agent(user_id: &str, llm: Arc<dyn LlmProvider>) -> Result<BasicAgent> {
    let memory = create_working_memory(WorkingMemoryConfig {
        session_id: Some(user_id.to_string()),
        ..Default::default()
    });
    
    AgentBuilder::new()
        .name(&format!("agent_{}", user_id))
        .instructions("You are a personalized assistant")
        .model(llm)
        .working_memory(memory)
        .build()
}
```

---

## 4. 工作流设计模式

### 4.1 错误恢复模式

```rust
// 工作流步骤应该是幂等的
async fn idempotent_step(input: Value, attempt: usize) -> Result<Value> {
    let task_id = input["task_id"].as_str()
        .ok_or_else(|| Error::invalid_input("Missing task_id"))?;
    
    // 检查任务是否已完成
    if is_task_completed(task_id).await? {
        return get_task_result(task_id).await;
    }
    
    // 执行任务
    let result = perform_task(&input).await?;
    
    // 保存结果
    save_task_result(task_id, &result).await?;
    
    Ok(result)
}
```

### 4.2 补偿事务模式

```rust
// 使用补偿事务处理失败
struct WorkflowStep {
    execute: Box<dyn Fn(Value) -> Pin<Box<dyn Future<Output = Result<Value>>>>>,
    compensate: Box<dyn Fn(Value) -> Pin<Box<dyn Future<Output = Result<()>>>>>,
}

async fn execute_with_compensation(steps: &[WorkflowStep], input: Value) -> Result<Value> {
    let mut completed_steps = Vec::new();
    let mut current_input = input;
    
    for step in steps {
        match (step.execute)(current_input.clone()).await {
            Ok(output) => {
                completed_steps.push((step, current_input.clone()));
                current_input = output;
            }
            Err(e) => {
                // 执行补偿操作
                for (completed_step, step_input) in completed_steps.iter().rev() {
                    if let Err(comp_err) = (completed_step.compensate)(step_input.clone()).await {
                        error!("Compensation failed: {}", comp_err);
                    }
                }
                return Err(e);
            }
        }
    }
    
    Ok(current_input)
}
```

### 4.3 并行度控制

```rust
use tokio::sync::Semaphore;

async fn execute_with_concurrency_limit(
    tasks: Vec<Task>,
    max_concurrency: usize,
) -> Result<Vec<Value>> {
    let semaphore = Arc::new(Semaphore::new(max_concurrency));
    let mut handles = vec![];
    
    for task in tasks {
        let semaphore = semaphore.clone();
        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            task.execute().await
        });
        handles.push(handle);
    }
    
    let results = futures::future::join_all(handles).await;
    results.into_iter().collect::<Result<Vec<_>, _>>()?
}
```

---

## 5. 错误处理

### 5.1 使用 Result 类型

**✅ 推荐**:
```rust
async fn process_data(input: &str) -> Result<String> {
    let validated = validate_input(input)?;
    let processed = process(validated).await?;
    let formatted = format_output(processed)?;
    Ok(formatted)
}
```

**❌ 不推荐**:
```rust
// 使用 panic 或 unwrap
async fn bad_process(input: &str) -> String {
    let processed = process(input).await.unwrap();  // 不好！
    format_output(processed).expect("Failed")  // 不好！
}
```

### 5.2 自定义错误类型

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Business logic error: {0}")]
    BusinessLogic(String),
    
    #[error("LumosAI error: {0}")]
    LumosAI(#[from] lumosai_core::Error),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

pub type AppResult<T> = std::result::Result<T, AppError>;
```

### 5.3 优雅降级

```rust
async fn get_response_with_fallback(query: &str) -> Result<String> {
    // 尝试主要方案
    match primary_agent.generate_simple(query).await {
        Ok(response) => Ok(response),
        Err(e) => {
            warn!("Primary agent failed: {}, trying fallback", e);
            
            // 降级到简单方案
            fallback_agent.generate_simple(query).await
                .or_else(|_| {
                    // 最后的兜底
                    Ok("Sorry, I'm temporarily unavailable.".to_string())
                })
        }
    }
}
```

---

## 6. 性能优化

### 6.1 使用连接池

```rust
use once_cell::sync::Lazy;

static LLM_POOL: Lazy<ConnectionPool<LlmProvider>> = Lazy::new(|| {
    ConnectionPool::new(PoolConfig {
        max_size: 10,
        min_size: 2,
        idle_timeout: Duration::from_secs(60),
    }).expect("Failed to create LLM pool")
});

async fn get_llm() -> Result<PooledConnection<LlmProvider>> {
    LLM_POOL.get().await
}
```

### 6.2 批处理

```rust
// 批量处理而非逐个处理
async fn process_queries_batch(queries: Vec<String>) -> Result<Vec<String>> {
    // 使用批处理 API
    agent.batch_generate(queries).await
}

// 而不是
async fn process_queries_one_by_one(queries: Vec<String>) -> Result<Vec<String>> {
    let mut results = Vec::new();
    for query in queries {
        results.push(agent.generate_simple(&query).await?);
    }
    Ok(results)
}
```

### 6.3 缓存策略

```rust
use moka::future::Cache;

// 创建缓存
let cache: Cache<String, String> = Cache::builder()
    .max_capacity(10_000)
    .time_to_live(Duration::from_secs(3600))
    .build();

async fn get_response_cached(query: &str, cache: &Cache<String, String>) -> Result<String> {
    // 尝试从缓存获取
    if let Some(cached) = cache.get(query).await {
        return Ok(cached);
    }
    
    // 缓存未命中，调用 Agent
    let response = agent.generate_simple(query).await?;
    
    // 保存到缓存
    cache.insert(query.to_string(), response.clone()).await;
    
    Ok(response)
}
```

### 6.4 异步并发

```rust
use futures::future::join_all;

// 并发执行多个独立任务
async fn parallel_processing(tasks: Vec<Task>) -> Result<Vec<Result>> {
    let futures: Vec<_> = tasks.into_iter()
        .map(|task| tokio::spawn(async move { task.execute().await }))
        .collect();
    
    let results = join_all(futures).await;
    Ok(results.into_iter().collect())
}
```

---

## 7. 安全最佳实践

### 7.1 输入清理

```rust
use regex::Regex;

fn sanitize_input(input: &str) -> Result<String> {
    // 移除危险字符
    let cleaned = input.replace(['<', '>', '"', '\''], "");
    
    // 限制长度
    if cleaned.len() > 10000 {
        return Err(Error::invalid_input("Input too long"));
    }
    
    // 检查恶意模式
    let sql_injection_pattern = Regex::new(r"(?i)(union|select|insert|update|delete|drop)").unwrap();
    if sql_injection_pattern.is_match(&cleaned) {
        return Err(Error::security_violation("Potential SQL injection detected"));
    }
    
    Ok(cleaned)
}
```

### 7.2 API 密钥管理

```rust
use secrecy::{Secret, ExposeSecret};

// 使用 Secret 类型保护敏感数据
struct LlmConfig {
    api_key: Secret<String>,
    endpoint: String,
}

impl LlmConfig {
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| Error::config_error("API key not found"))?;
        
        Ok(Self {
            api_key: Secret::new(api_key),
            endpoint: "https://api.openai.com".to_string(),
        })
    }
    
    pub fn api_key(&self) -> &str {
        self.api_key.expose_secret()
    }
}

// 日志中不会暴露密钥
impl std::fmt::Debug for LlmConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlmConfig")
            .field("api_key", &"***")
            .field("endpoint", &self.endpoint)
            .finish()
    }
}
```

### 7.3 速率限制

```rust
use governor::{Quota, RateLimiter, state::InMemoryState, clock::DefaultClock};

// 创建速率限制器
let rate_limiter = Arc::new(RateLimiter::keyed(
    Quota::per_minute(NonZeroU32::new(60).unwrap())
));

async fn rate_limited_request(
    user_id: &str,
    query: &str,
    limiter: &RateLimiter<String, InMemoryState, DefaultClock>,
) -> Result<String> {
    // 检查速率限制
    limiter.check_key(&user_id.to_string())
        .map_err(|_| Error::rate_limit_exceeded())?;
    
    // 处理请求
    agent.generate_simple(query).await
}
```

---

## 8. 测试策略

### 8.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use lumosai_core::llm::test_helpers::create_mock_llm;
    
    #[tokio::test]
    async fn test_agent_with_tools() {
        // Arrange
        let llm = create_mock_llm("Test response");
        let calculator = create_calculator_tool();
        
        let agent = AgentBuilder::new()
            .name("test_agent")
            .instructions("Test")
            .model(Arc::new(llm))
            .tool(calculator)
            .build()
            .unwrap();
        
        // Act
        let response = agent.generate_simple("Calculate 2+2").await.unwrap();
        
        // Assert
        assert!(response.contains("4") || response.contains("four"));
    }
    
    #[tokio::test]
    async fn test_error_handling() {
        let llm = create_failing_llm();
        let agent = AgentBuilder::new()
            .name("test")
            .instructions("Test")
            .model(Arc::new(llm))
            .build()
            .unwrap();
        
        let result = agent.generate_simple("Test").await;
        assert!(result.is_err());
    }
}
```

### 8.2 集成测试

```rust
#[tokio::test]
#[ignore]  // 需要真实 API 密钥
async fn test_end_to_end_workflow() {
    // 使用真实的 LLM 提供商
    let llm = openai("gpt-3.5-turbo").unwrap().build().unwrap();
    
    // 创建完整的工作流
    let workflow = create_article_workflow(llm).await.unwrap();
    
    // 执行并验证结果
    let result = workflow.execute(
        json!({"topic": "AI in healthcare"}),
        &RuntimeContext::new()
    ).await.unwrap();
    
    assert!(result["content"].as_str().unwrap().len() > 100);
}
```

### 8.3 性能测试

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_agent_generation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let agent = rt.block_on(create_test_agent()).unwrap();
    
    c.bench_function("agent_generation", |b| {
        b.to_async(&rt).iter(|| async {
            agent.generate_simple(black_box("Test query")).await.unwrap()
        });
    });
}

criterion_group!(benches, bench_agent_generation);
criterion_main!(benches);
```

---

## 9. 监控和可观测性

### 9.1 结构化日志

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(agent), fields(agent_name = agent.name()))]
async fn process_request(agent: &BasicAgent, query: &str) -> Result<String> {
    info!(query = %query, "Processing request");
    
    let start = Instant::now();
    
    match agent.generate_simple(query).await {
        Ok(response) => {
            let duration = start.elapsed();
            info!(
                duration_ms = duration.as_millis(),
                response_len = response.len(),
                "Request completed successfully"
            );
            Ok(response)
        }
        Err(e) => {
            error!(error = %e, "Request failed");
            Err(e)
        }
    }
}
```

### 9.2 指标收集

```rust
use prometheus::{Counter, Histogram, Registry};

lazy_static! {
    static ref REQUESTS_TOTAL: Counter = Counter::new(
        "agent_requests_total",
        "Total number of agent requests"
    ).unwrap();
    
    static ref REQUEST_DURATION: Histogram = Histogram::new(
        "agent_request_duration_seconds",
        "Agent request duration in seconds"
    ).unwrap();
}

async fn monitored_request(query: &str) -> Result<String> {
    REQUESTS_TOTAL.inc();
    
    let timer = REQUEST_DURATION.start_timer();
    let result = agent.generate_simple(query).await;
    timer.observe_duration();
    
    result
}
```

### 9.3 健康检查

```rust
async fn health_check() -> Result<HealthStatus> {
    let mut status = HealthStatus {
        healthy: true,
        checks: HashMap::new(),
    };
    
    // 检查 LLM 连接
    match test_llm_connection().await {
        Ok(_) => status.checks.insert("llm", "ok".to_string()),
        Err(e) => {
            status.healthy = false;
            status.checks.insert("llm", format!("failed: {}", e))
        }
    };
    
    // 检查数据库连接
    match test_db_connection().await {
        Ok(_) => status.checks.insert("database", "ok".to_string()),
        Err(e) => {
            status.healthy = false;
            status.checks.insert("database", format!("failed: {}", e))
        }
    };
    
    Ok(status)
}
```

---

## 10. 代码组织

### 10.1 项目结构

```
my-lumosai-app/
├── src/
│   ├── main.rs              # 应用入口
│   ├── config.rs            # 配置管理
│   ├── agents/              # Agent 定义
│   │   ├── mod.rs
│   │   ├── customer_service.rs
│   │   └── research.rs
│   ├── tools/               # 工具定义
│   │   ├── mod.rs
│   │   ├── calculator.rs
│   │   └── weather.rs
│   ├── workflows/           # 工作流定义
│   │   ├── mod.rs
│   │   └── article_generation.rs
│   ├── services/            # 业务逻辑
│   │   ├── mod.rs
│   │   └── content_service.rs
│   └── utils/               # 工具函数
│       ├── mod.rs
│       └── validators.rs
├── tests/                   # 集成测试
│   ├── integration_test.rs
│   └── e2e_test.rs
├── benches/                 # 性能测试
│   └── agent_bench.rs
├── Cargo.toml
└── README.md
```

### 10.2 配置管理

```rust
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub llm: LlmConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let config = Config::builder()
            // 从文件加载
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name("config/local").required(false))
            // 从环境变量覆盖
            .add_source(Environment::with_prefix("APP"))
            .build()?;
        
        config.try_deserialize()
    }
}
```

### 10.3 依赖注入

```rust
use std::sync::Arc;

pub struct AppServices {
    pub llm_provider: Arc<dyn LlmProvider>,
    pub agent_registry: Arc<AgentRegistry>,
    pub tool_registry: Arc<ToolRegistry>,
}

impl AppServices {
    pub fn new(config: &AppConfig) -> Result<Self> {
        let llm_provider = create_llm_provider(config)?;
        let agent_registry = Arc::new(AgentRegistry::new());
        let tool_registry = Arc::new(ToolRegistry::new());
        
        Ok(Self {
            llm_provider,
            agent_registry,
            tool_registry,
        })
    }
}

// 在应用中使用
async fn handle_request(services: Arc<AppServices>, query: &str) -> Result<String> {
    let agent = services.agent_registry.get("default")?;
    agent.generate_simple(query).await
}
```

---

## 总结

遵循这些最佳实践可以帮助你构建：

✅ **可维护的代码**: 清晰的结构和职责划分
✅ **高性能应用**: 合理的资源管理和优化策略  
✅ **安全的系统**: 输入验证和密钥管理
✅ **可靠的服务**: 错误处理和降级策略
✅ **可观测的系统**: 日志、指标和监控

**记住**:
- 先让代码工作，再优化
- 测试驱动开发
- 文档和代码同样重要
- 安全永远是第一位的
- 性能优化要基于数据

**进一步学习**:
- 阅读 [用户指南](USER_GUIDE.md) 了解功能详情
- 查看 [示例代码](../lumosai_examples/examples/) 学习实践
- 参考 [API 文档](https://docs.rs/lumosai_core) 了解API详情

Happy Coding! 🚀

