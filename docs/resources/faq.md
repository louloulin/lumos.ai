# ❓ 常见问题解答 (FAQ)

> LumosAI 使用过程中的常见问题和解决方案

## 🚀 快速开始

### Q: 如何安装 LumosAI？

**A**: 有多种安装方式：

**方式 1: Cargo 安装**
```bash
cargo add lumosai
```

**方式 2: 克隆仓库**
```bash
git clone https://github.com/louloulin/lumos.ai.git
cd lumos.ai
cargo build --release
```

**方式 3: Docker**
```bash
docker pull lumosai/lumosai:latest
```

### Q: 需要什么前置条件？

**A**: 基本要求：
- Rust 1.70+
- Cargo (Rust 包管理器)
- 一个 LLM 提供商的 API 密钥

**可选组件**：
- Docker (用于容器化部署)
- PostgreSQL (用于向量存储)
- Redis (用于缓存)

### Q: 如何配置 API 密钥？

**A**: 设置环境变量：

```bash
# OpenAI
export OPENAI_API_KEY="sk-your-openai-key"

# Anthropic Claude
export ANTHROPIC_API_KEY="your-anthropic-key"

# DeepSeek
export DEEPSEEK_API_KEY="sk-your-deepseek-key"

# Zhipu AI
export ZHIPUAI_API_KEY="your-zhipuai-key"
```

## 🤖 Agent 相关

### Q: 如何创建一个简单的 Agent？

**A**: 使用简化 API：

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = lumosai::agent::simple("gpt-4", "你是一个友好的助手").await?;
    let response = agent.chat("你好！").await?;
    println!("{}", response);
    Ok(())
}
```

### Q: 如何配置 Agent 的参数？

**A**: 使用构建器模式：

```rust
let agent = Agent::builder()
    .name("assistant")
    .model("gpt-4")
    .system_prompt("你是一个专业的助手")
    .max_tokens(2000)
    .temperature(0.7)
    .top_p(0.9)
    .build()
    .await?;
```

### Q: Agent 如何记住对话历史？

**A**: LumosAI 自动管理对话历史：

```rust
let agent = lumosai::agent::simple("gpt-4", "友好的助手").await?;

// 对话会自动保存历史
agent.chat("我叫张三").await?;
agent.chat("我今年25岁").await?;

// Agent 记得之前的对话内容
let response = agent.chat("请告诉我我的名字和年龄").await?;
```

### Q: 如何清除 Agent 的记忆？

**A**: 使用记忆管理方法：

```rust
// 清除对话历史
agent.clear_conversation_history().await?;

// 或者清除特定类型的记忆
agent.clear_memories_by_type("fact").await?;
```

## 🧠 RAG 相关

### Q: 如何创建一个 RAG 系统？

**A**: 基础 RAG 创建：

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建向量存储
    let storage = lumosai::vector::memory().await?;
    
    // 创建 RAG 系统
    let rag = lumosai::rag::simple(storage, "openai").await?;
    
    // 添加文档
    rag.add_document("Rust 是一门系统编程语言").await?;
    
    // 搜索并生成回答
    let response = rag.search_and_generate("什么是 Rust？", 5).await?;
    println!("{}", response);
    
    Ok(())
}
```

### Q: 支持哪些文档格式？

**A**: LumosAI 支持多种格式：
- **纯文本**: `.txt`, `.md`
- **结构化数据**: `.json`, `.yaml`, `.toml`
- **文档**: `.pdf` (需要额外依赖)
- **网页**: `.html` (通过爬虫)

### Q: 如何优化 RAG 的搜索效果？

**A**: 优化策略：

```rust
let rag = lumosai::rag::builder()
    .storage(storage)
    .embedding_provider("openai")
    .chunk_size(1000)        // 调整分块大小
    .chunk_overlap(200)       // 设置重叠
    .similarity_threshold(0.7) // 相似度阈值
    .max_results(10)          // 最大结果数
    .build()
    .await?;
```

### Q: 如何处理大量文档？

**A**: 批量处理和增量更新：

```rust
let documents = vec![
    "文档1内容...",
    "文档2内容...",
    // 更多文档...
];

// 批量添加
for doc in documents {
    rag.add_document(doc).await?;
}

// 检查状态
let stats = rag.get_statistics().await?;
println!("文档数量: {}", stats.document_count);
println!("总向量数: {}", stats.total_vectors);
```

## 🔧 工具集成

### Q: 如何为 Agent 添加工具？

**A**: 内置工具和自定义工具：

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let agent = Agent::builder()
        .name("assistant")
        .model("gpt-4")
        .build()
        .await?;
    
    // 添加内置工具
    agent.add_tool(
        "get_weather",
        "Get current weather for location",
        vec![("location", "City name", "string")]
    ).await?;
    
    // 使用工具
    let response = agent.chat("北京今天天气怎么样？").await?;
    println!("{}", response);
    
    Ok(())
}
```

### Q: 如何创建自定义工具？

**A**: 实现 Tool trait：

```rust
use lumosai_core::tool::*;
use async_trait::async_trait;
use serde_json::{json, Value};

struct WeatherTool;

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> &str {
        "get_weather"
    }
    
    fn description(&self) -> &str {
        "获取天气信息"
    }
    
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "location".to_string(),
            description: "城市名称".to_string(),
            parameter_type: "string".to_string(),
            required: true,
        }]
    }
    
    async fn execute(&self, args: Value) -> Result<Value> {
        let location = args["location"].as_str().unwrap_or("unknown");
        
        // 调用天气API
        let weather = fetch_weather(location).await?;
        
        Ok(json!({
            "location": location,
            "temperature": weather.temperature,
            "condition": weather.condition
        }))
    }
}
```

### Q: 工具调用失败怎么办？

**A**: 错误处理和重试机制：

```rust
let agent = Agent::builder()
    .name("assistant")
    .model("gpt-4")
    .retry_policy(RetryPolicy {
        max_attempts: 3,
        backoff_ms: 1000,
        exponential_backoff: true,
    })
    .build()
    .await?;
```

## 🔒 安全相关

### Q: LumosAI 如何保护 API 密钥？

**A**: LumosAI 使用以下安全措施：
- 环境变量存储，不在代码中硬编码
- 支持加密密钥存储
- API 调用使用 HTTPS 加密传输
- 支持密钥轮换机制

### Q: 如何限制 Agent 的能力？

**A**: 通过配置限制：

```rust
let agent = Agent::builder()
    .name("limited_assistant")
    .model("gpt-4")
    .max_tokens(1000)           // 限制回复长度
    .allowed_tools(vec!["search"]) // 只允许特定工具
    .allowed_domains(vec!["api.example.com"]) // 限制可访问域名
    .rate_limit(RateLimit {
        requests_per_minute: 10,
        tokens_per_minute: 10000,
    })
    .build()
    .await?;
```

### Q: 如何监控 Agent 的行为？

**A**: 使用审计日志：

```rust
let agent = Agent::builder()
    .name("monitored_agent")
    .audit_enabled(true)
    .audit_level(AuditLevel::Detailed)
    .build()
    .await?;

// 获取审计日志
let audit_logs = agent.get_audit_logs().await?;
for log in audit_logs {
    println!("{}: {}", log.timestamp, log.action);
}
```

## 🚀 性能优化

### Q: 如何提高 Agent 响应速度？

**A**: 多种优化策略：

```rust
let agent = Agent::builder()
    .name("fast_agent")
    .model("gpt-4")
    .cache_enabled(true)           // 启用缓存
    .streaming_enabled(true)       // 启用流式响应
    .temperature(0.1)              // 降低随机性
    .max_tokens(500)               // 限制回复长度
    .parallel_requests(true)        // 并发处理
    .build()
    .await?;
```

### Q: 如何处理大量并发请求？

**A**: 使用 Agent 池：

```rust
use lumosai::pool::AgentPool;

#[tokio::main]
async fn main() -> Result<()> {
    let pool = AgentPool::builder()
        .max_size(10)
        .min_size(2)
        .idle_timeout(Duration::from_secs(30))
        .build()
        .await?;
    
    // 从池中获取 Agent
    let agent = pool.acquire().await?;
    
    // 使用 Agent
    let response = agent.chat("Hello!").await?;
    
    // 自动归还到池中
    drop(agent); // 或者显式 pool.release(agent).await?
    
    Ok(())
}
```

### Q: 内存使用过高怎么办？

**A**: 内存管理策略：

```rust
let agent = Agent::builder()
    .name("memory_efficient_agent")
    .model("gpt-4")
    .memory_limit(10000)          // 限制内存使用
    .compression_enabled(true)     // 启用压缩
    .cleanup_interval(Duration::from_secs(300)) // 定期清理
    .build()
    .await?;
```

## 🔍 调试相关

### Q: 如何调试 Agent 的行为？

**A**: 启用调试模式：

```rust
let agent = Agent::builder()
    .name("debug_agent")
    .model("gpt-4")
    .debug_mode(true)
    .trace_level(TraceLevel::Verbose)
    .build()
    .await?;

// 获取执行轨迹
let trace = agent.chat_with_trace("Test message").await?;
for step in trace.steps {
    println!("{}: {} ({}ms)", step.step_type, step.description, step.duration);
}
```

### Q: 如何查看详细错误信息？

**A**: 使用错误诊断：

```rust
match agent.chat("Hello").await {
    Ok(response) => println!("Success: {}", response),
    Err(e) => {
        println!("Error: {}", e);
        
        // 获取详细错误信息
        if let Some(details) = e.details() {
            println!("Details: {}", details);
        }
        
        // 获取错误码
        if let Some(code) = e.code() {
            println!("Error code: {}", code);
        }
        
        // 获取建议
        if let Some(suggestion) = e.suggestion() {
            println!("Suggestion: {}", suggestion);
        }
    }
}
```

## 🌐 部署相关

### Q: 如何在生产环境部署？

**A**: 生产部署最佳实践：

```rust
// 1. 使用环境变量配置
let agent = Agent::builder()
    .name("production_agent")
    .model_from_env("LUMOS_MODEL")        // 从环境变量读取
    .api_key_from_env("LUMOS_API_KEY")   // 从环境变量读取
    .retry_policy(RetryPolicy::production())
    .audit_enabled(true)
    .build()
    .await?;

// 2. 配置健康检查
let health_check = HealthCheck::builder()
    .endpoint("/health")
    .checks(vec![
        Check::model_availability(),
        Check::api_connectivity(),
        Check::memory_usage(),
    ])
    .build();
```

### Q: 如何监控部署状态？

**A**: 使用内置监控：

```rust
use lumosai::telemetry::*;

let telemetry = Telemetry::builder()
    .prometheus_metrics(true)
    .jaeger_tracing(true)
    .log_level("info")
    .build()
    .await?;

// 获取指标
let metrics = telemetry.get_metrics().await?;
println!("Active agents: {}", metrics.active_agents);
println!("Requests per second: {}", metrics.requests_per_second);
println!("Average response time: {}ms", metrics.avg_response_time);
```

## 🔄 更新与迁移

### Q: 如何从旧版本升级？

**A**: 升级步骤：

1. **检查兼容性**: 查看 [CHANGELOG](../../CHANGELOG.md)
2. **更新依赖**: `cargo update lumosai`
3. **测试兼容性**: 运行现有测试
4. **迁移配置**: 根据迁移指南调整

### Q: 如何迁移数据？

**A**: 数据迁移工具：

```bash
# 导出现有数据
cargo run --bin data_exporter --format json

# 导入到新版本
cargo run --bin data_importer --format json --from old_version
```

## 🆘 获取更多帮助

### 官方支持渠道

- **📖 完整文档**: [文档中心](../README.md)
- **🐛 问题报告**: [GitHub Issues](https://github.com/louloulin/lumos.ai/issues)
- **💬 社区讨论**: [GitHub Discussions](https://github.com/louloulin/lumos.ai/discussions)

### 社区资源

- **Discord**: 实时讨论和帮助
- **Stack Overflow**: #lumosai 标签
- **Reddit**: r/LumosAI

### 商业支持

- **企业支持**: enterprise@lumosai.com
- **技术咨询**: consulting@lumosai.com
- **定制服务**: custom@lumosai.com

---

## 📝 持续更新

本 FAQ 会根据用户反馈持续更新。如果您遇到本文档未覆盖的问题，欢迎通过上述渠道联系我们。

**💡 提示**: 提问问题时请包含详细的错误信息和复现步骤，这样我们能更快地帮助您解决问题！