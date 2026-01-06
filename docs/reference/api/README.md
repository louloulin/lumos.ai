# 📋 API Reference

**LumosAI 完整API文档 - 所有组件的详细接口说明**

## 🧭 API导航

### 核心组件API
- **[Agent API](agent.md)** - AI Agent创建和管理
- **[RAG API](rag.md)** - 检索增强生成系统
- **[Memory API](memory.md)** - 内存和上下文管理
- **[Tools API](tools.md)** - 工具系统和扩展
- **[Workflow API](workflow.md)** - 工作流编排

### 基础设施API
- **[Vector Storage API](vector-storage.md)** - 向量数据库操作
- **[LLM API](llm.md)** - 大语言模型接口
- **[Config API](config.md)** - 配置管理系统
- **[Auth API](auth.md)** - 认证和授权

## 🚀 快速API概览

### Agent API
```rust
use lumosai::prelude::*;

// 创建Agent
let agent = Agent::builder()
    .name("assistant")
    .model("gpt-4")
    .system_prompt("You are helpful")
    .build()
    .await?;

// 对话
let response = agent.chat("Hello").await?;
```

### RAG API
```rust
use lumosai::prelude::*;

// 创建RAG系统
let rag = lumosai::rag::builder()
    .storage(storage)
    .embedding_provider("openai")
    .build()
    .await?;

// 添加文档和搜索
rag.add_document("Your document content...").await?;
let results = rag.search("Query", 5).await?;
```

### Tools API
```rust
use lumosai::prelude::*;
use lumos_macro::tool;

#[tool]
async fn my_tool(param: String) -> Result<String> {
    // 工具实现
    Ok(format!("Processed: {}", param))
}

// 添加到Agent
let agent = Agent::builder()
    .tool(MyTool::new())
    .build()
    .await?;
```

---

## 📚 API分类

### 🤖 高级API (High-level APIs)
**简单易用的接口，适合快速开发**

| API | 描述 | 使用场景 |
|-----|------|----------|
| [`Agent::builder()`](agent.md#builder) | 快速创建Agent | 标准对话应用 |
| [`lumosai::rag::builder()`](rag.md#builder) | 快速创建RAG系统 | 知识问答应用 |
| [`lumosai::vector::memory()`](vector-storage.md#memory) | 内存向量存储 | 开发测试环境 |
| [`lumosai::simple()`](simple.md) | 一键创建Agent | 原型开发 |

### 🔧 低级API (Low-level APIs)  
**精细控制，适合复杂场景**

| API | 描述 | 使用场景 |
|-----|------|----------|
| [`AgentCore`](agent.md#core) | Agent核心逻辑 | 自定义Agent行为 |
| [`RagEngine`](rag.md#engine) | RAG引擎核心 | 自定义检索策略 |
| [`VectorStore`](vector-storage.md#traits) | 向量存储抽象 | 自定义存储后端 |
| [`ToolRegistry`](tools.md#registry) | 工具注册表 | 动态工具管理 |

### ⚙️ 配置API (Configuration APIs)
**系统配置和环境管理**

| API | 描述 | 链接 |
|-----|------|------|
| `LumosaiConfig` | 主配置结构 | [配置文档](../configuration/README.md) |
| `ConfigLoader` | 配置加载器 | [配置加载](../configuration/loading.md) |
| `Environment` | 环境变量管理 | [环境配置](../configuration/environment.md) |

---

## 🔍 API设计原则

### 1. 类型安全
所有API都使用Rust的类型系统确保编译时安全：

```rust
// 类型安全的Agent构建
let agent: Agent<OpenAIProvider> = Agent::builder()
    .model("gpt-4")  // 编译时检查模型名称
    .temperature(0.7) // 类型安全的参数
    .build()?;
```

### 2. 异步优先
所有I/O操作都是异步的，支持高并发：

```rust
// 异步API调用
let response = agent
    .chat("Hello")
    .await?;  // 非阻塞等待
```

### 3. 错误处理
完善的错误类型和处理机制：

```rust
use lumosai::error::{LumosaiError, Result};

fn handle_api_result() -> Result<()> {
    let response = agent.chat("Hello").await?;
    // 自动错误传播和处理
    Ok(())
}
```

### 4. 扩展性
基于trait的可扩展设计：

```rust
// 自定义向量存储
struct MyVectorStore;
impl VectorStore for MyVectorStore {
    // 实现必需的trait方法
}

// 与现有API无缝集成
let rag = RagEngine::new()
    .with_storage(MyVectorStore)
    .build();
```

---

## 📖 API使用指南

### 基础概念

**Provider Pattern (提供者模式)**
```rust
// 支持多种LLM提供商
let agent = Agent::builder()
    .provider(OpenAIProvider::new())  // OpenAI
    .provider(AnthropicProvider::new()) // 或Anthropic
    .build()?;
```

**Builder Pattern (构建器模式)**
```rust
// 流畅的API设计
let agent = Agent::builder()
    .name("assistant")
    .model("gpt-4")
    .temperature(0.7)
    .max_tokens(1000)
    .tool(my_tool)
    .memory(my_memory)
    .build()?;
```

**Trait-based Abstraction (基于trait的抽象)**
```rust
// 统一的接口设计
fn process_vector_store<V: VectorStore>(store: V) {
    // 适用于任何实现了VectorStore的类型
}
```

### 最佳实践

**1. 错误处理**
```rust
match agent.chat("Hello").await {
    Ok(response) => println!("Success: {}", response),
    Err(LumosaiError::ApiError(e)) => eprintln!("API Error: {}", e),
    Err(e) => eprintln!("Other Error: {}", e),
}
```

**2. 资源管理**
```rust
// 使用RAII管理资源
{
    let agent = Agent::builder().build().await?;
    let _response = agent.chat("Hello").await?;
} // agent自动清理
```

**3. 并发安全**
```rust
// Agent可以安全地在多个线程中使用
let agent = Arc::new(agent);
let mut handles = vec![];

for i in 0..10 {
    let agent_clone = Arc::clone(&agent);
    let handle = tokio::spawn(async move {
        agent_clone.chat(&format!("Message {}", i)).await
    });
    handles.push(handle);
}
```

---

## 🔗 相关资源

### 教程和示例
- [基础教程](../learn/tutorials/basics/README.md)
- [API示例集合](../learn/examples/README.md)
- [最佳实践](../learn/guides/best-practices.md)

### 配置参考
- [配置文档](../configuration/README.md)
- [环境变量](../configuration/environment.md)
- [模型配置](../configuration/models.md)

### 扩展开发
- [自定义工具开发](../contribute/tools.md)
- [向量存储扩展](../contribute/vector-stores.md)
- [LLM提供商集成](../contribute/llm-providers.md)

---

## 🆘 API支持

### 文档搜索
使用浏览器搜索功能 (Ctrl+F) 快速查找API:

- 搜索函数名: `chat`, `search`, `add_document`
- 搜索类型: `Agent`, `RagEngine`, `VectorStore`
- 搜索trait: `AgentProvider`, `VectorStore`, `Tool`

### 示例代码
每个API页面都包含：
- ✅ 可运行的示例代码
- 📝 参数说明
- ⚠️ 注意事项
- 🔗 相关API

### 问题报告
如果API文档有问题：
- [报告文档错误](https://github.com/louloulin/lumos.ai/issues/new?template=documentation.md)
- [API功能请求](https://github.com/louloulin/lumos.ai/issues/new?template=feature.md)
- [GitHub Discussions](https://github.com/louloulin/lumos.ai/discussions)

---

**开始探索LumosAI的强大API功能吧！** 🚀

*[← 返回文档首页](../README.md)*