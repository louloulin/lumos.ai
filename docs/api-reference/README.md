# 🔧 API 参考

> LumosAI 完整 API 接口文档

## 概览

LumosAI 提供了简洁而强大的 API，支持 Agent 创建、RAG 系统、向量存储、工作流编排等核心功能。

### 核心 API 结构

```rust
use lumosai::prelude::*;

// 核心 API
lumosai::agent::simple()           // 创建简单 Agent
lumosai::vector::memory()         // 创建向量存储
lumosai::rag::simple()            // 创建 RAG 系统
lumosai::workflow::task()         // 创建工作流任务
```

---

## 🤖 Agent API

### 快速创建

#### `lumosai::agent::simple()`

创建一个简单的 AI Agent。

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = lumosai::agent::simple("gpt-4", "You are helpful").await?;
    
    let response = agent.chat("Hello!").await?;
    println!("{}", response);
    
    Ok(())
}
```

**参数:**
- `model: &str` - 模型名称（支持 "gpt-4", "claude-3", "deepseek-chat" 等）
- `system_prompt: &str` - 系统提示词

**返回:** `Result<Agent>` - Agent 实例

### AgentBuilder

#### `Agent::builder()`

使用构建器模式创建复杂的 Agent。

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = Agent::builder()
        .name("assistant")
        .model("gpt-4")
        .system_prompt("You are a helpful AI assistant")
        .max_tokens(1000)
        .temperature(0.7)
        .build()
        .await?;
    
    Ok(())
}
```

**构建器方法:**

| 方法 | 类型 | 描述 | 默认值 |
|------|------|------|--------|
| `name(name: &str)` | 必需 | Agent 名称 | - |
| `model(model: &str)` | 必需 | LLM 模型 | - |
| `system_prompt(prompt: &str)` | 必需 | 系统提示词 | - |
| `max_tokens(limit: u32)` | 可选 | 最大生成 token 数 | 2048 |
| `temperature(temp: f64)` | 可选 | 生成随机性 (0.0-2.0) | 1.0 |
| `max_tool_calls(limit: u32)` | 可选 | 最大工具调用次数 | 10 |

### Agent 方法

#### `chat(message: &str)`

与 Agent 进行对话。

```rust
let response = agent.chat("Hello, how are you?").await?;
```

**参数:**
- `message: &str` - 用户消息

**返回:** `Result<String>` - Agent 回复

#### `generate_simple(prompt: &str)`

生成简单回复（不保存对话历史）。

```rust
let response = agent.generate_simple("Explain quantum computing").await?;
```

---

## 🧠 RAG API

### 快速创建

#### `lumosai::rag::simple()`

创建一个 RAG（检索增强生成）系统。

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建向量存储
    let storage = lumosai::vector::memory().await?;
    
    // 创建 RAG 系统
    let rag = lumosai::rag::simple(storage, "openai").await?;
    
    // 添加文档
    rag.add_document("Rust is a systems programming language.").await?;
    
    // 搜索并生成
    let response = rag.search_and_generate("What is Rust?", 5).await?;
    println!("{}", response);
    
    Ok(())
}
```

**参数:**
- `storage: VectorStorage` - 向量存储实例
- `embedding_provider: &str` - 嵌入模型提供商

**返回:** `Result<RagEngine>` - RAG 引擎实例

### RAGBuilder

#### `lumosai::rag::builder()`

使用构建器创建自定义 RAG 系统。

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let storage = lumosai::vector::memory().await?;
    
    let rag = lumosai::rag::builder()
        .storage(storage)
        .embedding_provider("openai")
        .chunk_size(1000)
        .chunk_overlap(200)
        .build()
        .await?;
    
    Ok(())
}
```

**构建器方法:**

| 方法 | 类型 | 描述 | 默认值 |
|------|------|------|--------|
| `storage(storage: VectorStorage)` | 必需 | 向量存储 | - |
| `embedding_provider(provider: &str)` | 必需 | 嵌入模型 | - |
| `chunk_size(size: usize)` | 可选 | 文档块大小 | 1000 |
| `chunk_overlap(overlap: usize)` | 可选 | 块重叠大小 | 200 |

### RAG 方法

#### `add_document(content: &str)`

添加文档到 RAG 系统。

```rust
rag.add_document("This is a document about AI.").await?;
```

#### `search(query: &str, limit: usize)`

搜索相关文档。

```rust
let results = rag.search("artificial intelligence", 5).await?;
```

**参数:**
- `query: &str` - 搜索查询
- `limit: usize` - 结果数量限制

**返回:** `Result<Vec<SearchResult>>` - 搜索结果列表

#### `search_and_generate(query: &str, limit: usize)`

搜索并基于搜索结果生成回答。

```rust
let response = rag.search_and_generate("What is AI?", 5).await?;
```

---

## 💾 向量存储 API

### 内存向量存储

#### `lumosai::vector::memory()`

创建内存向量存储（适合开发和测试）。

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let storage = lumosai::vector::memory().await?;
    
    // 添加向量
    let vector = vec![0.1, 0.2, 0.3, 0.4];
    let id = storage.add_vector(vector).await?;
    
    // 搜索相似向量
    let query = vec![0.1, 0.2, 0.3, 0.4];
    let results = storage.search(&query, 5).await?;
    
    Ok(())
}
```

**返回:** `Result<MemoryVectorStorage>` - 内存向量存储实例

### VectorStorage Trait

所有向量存储都必须实现 `VectorStorage` trait：

```rust
#[async_trait]
pub trait VectorStorage: Send + Sync {
    async fn add_vector(&self, vector: Vec<f32>) -> Result<String>;
    async fn add_vector_with_metadata(&self, vector: Vec<f32>, metadata: serde_json::Value) -> Result<String>;
    async fn search(&self, query: &[f32], limit: usize) -> Result<Vec<SearchResult>>;
    async fn delete_vector(&self, id: &str) -> Result<()>;
    async fn update_vector(&self, id: &str, vector: Vec<f32>) -> Result<()>;
}
```

---

## 🔄 工作流 API

### 任务创建

#### `lumosai::workflow::task()`

创建工作流任务。

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = lumosai::agent::simple("gpt-4", "You are a researcher").await?;
    
    let task = lumosai::workflow::task()
        .name("Research Task")
        .description("Research about AI trends")
        .agent(agent)
        .build();
    
    // 执行任务
    let result = task.execute().await?;
    println!("{}", result);
    
    Ok(())
}
```

### TaskBuilder

```rust
let task = lumosai::workflow::task()
    .name("My Task")
    .description("Task description")
    .agent(agent)
    .input_data(json!({"query": "AI trends"}))
    .build();
```

---

## 🛠️ 工具 API

### 添加工具

#### `agent.add_tool()`

为 Agent 添加工具。

```rust
use lumosai::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = Agent::builder()
        .name("assistant")
        .model("gpt-4")
        .build()
        .await?;
    
    // 添加自定义工具
    agent.add_tool(
        "get_weather",
        "Get current weather for a location",
        vec![
            ("location", "City name", "string"),
        ]
    ).await?;
    
    // Agent 现在可以调用 get_weather 工具
    let response = agent.chat("What's the weather in Tokyo?").await?;
    
    Ok(())
}
```

### 自定义工具实现

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
        "Get current weather for a location"
    }
    
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "location".to_string(),
                description: "City name".to_string(),
                parameter_type: "string".to_string(),
                required: true,
            }
        ]
    }
    
    async fn execute(&self, args: Value) -> Result<Value> {
        let location = args["location"].as_str().unwrap_or("Unknown");
        
        // 这里实现实际的天气查询逻辑
        let weather = json!({
            "location": location,
            "temperature": "22°C",
            "condition": "Sunny"
        });
        
        Ok(weather)
    }
}

// 使用自定义工具
let weather_tool = Box::new(WeatherTool);
agent.add_custom_tool(weather_tool).await?;
```

---

## 📊 错误处理

LumosAI 使用统一的 `Result<T>` 类型进行错误处理：

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    match lumosai::agent::simple("gpt-4", "Hello").await {
        Ok(agent) => {
            println!("Agent created successfully");
            // 使用 agent...
        }
        Err(e) => {
            eprintln!("Failed to create agent: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}
```

### 常见错误类型

- `ConfigError` - 配置错误
- `ProviderError` - LLM 提供商错误
- `StorageError` - 存储相关错误
- `NetworkError` - 网络连接错误
- `ValidationError` - 输入验证错误

---

## 🔗 类型参考

### 核心类型

```rust
// Agent 相关
pub struct Agent { /* ... */ }
pub struct AgentBuilder { /* ... */ }

// RAG 相关
pub struct RagEngine { /* ... */ }
pub struct RagBuilder { /* ... */ }
pub struct SearchResult { /* ... */ }

// 向量存储
#[async_trait]
pub trait VectorStorage { /* ... */ }
pub struct MemoryVectorStorage { /* ... */ }

// 工具相关
#[async_trait]
pub trait Tool { /* ... */ }
pub struct ToolParameter { /* ... */ }

// 工作流
pub struct Task { /* ... */ }
pub struct TaskBuilder { /* ... */ }
```

### 便捷类型别名

```rust
pub type Result<T> = std::result::Result<T, Error>;
pub type Message = lumosai_core::llm::Message;
pub type Role = lumosai_core::llm::Role;
```

---

## 📚 更多示例

- [完整示例集合](../../../examples/)
- [Agent 教程](../tutorials/basics/agent-basics.md)
- [RAG 教程](../tutorials/basics/rag-basics.md)
- [工具集成教程](../tutorials/basics/tool-integration.md)

---

## 🆘 获取帮助

- **📖 完整文档**: [文档中心](../README.md)
- **💡 示例代码**: [examples/](../../../examples/)
- **🐛 问题反馈**: [GitHub Issues](https://github.com/louloulin/lumos.ai/issues)
- **❓ FAQ**: [常见问题](../resources/faq.md)

---

> **💡 提示**: 所有 API 都是异步的，请确保在 `#[tokio::main]` 函数中调用。
