# 教程 09：架构深度解析与完整用法指南

> 本教程对整个代码库进行系统性分析，全面覆盖核心模块与简化 API 的映射关系，并提供函数级注释的示例代码。内容包含 Agent、工具系统、内存与会话、事件系统、多 Agent 编排、RAG 管道与向量存储，以及端到端组合范式与迁移建议。

## 🧭 架构总览（模块分层与导出）
- 顶层库：`lumosai/`
  - `src/` 提供“简化 API”与教学演示入口：`agent.rs`、`session.rs`、`events.rs`、`orchestration.rs`、`rag.rs`、`vector.rs`、`prelude.rs`、`lib.rs`。
  - 设计目标：一行/少量代码完成“创建/组合/运行”，适合快速上手与示例。
- 核心库：`lumosai_core/`
  - 生产级实现与完整 Builder：`agent/`、`tool/`、`memory/`、`llm/`、`prelude.rs`、`workflow/`、`orchestration/`、`rag/`、`vector/`。
  - 设计目标：模块化、类型安全、可扩展，适合上线场景。
- 向量存储：`lumosai_vector/*` 与 `lumosai_vector_core/*`
  - 提供统一 `VectorStorage` trait 与多后端实现（Memory、Postgres、Qdrant、LanceDB、Weaviate）。
- 宏与 DSL：`lumos_macro/`
  - 提供宏 DSL（如 `rag_pipeline`、工具定义宏等），用于声明式配置与生成。
- UI 与 CLI：`lumosai_ui/`、`lumosai_cli/`
  - Web 控制台与项目脚手架，帮助集成、测试与部署。

## 🔗 简化 API 与核心实现映射
- 统一导出：`lumosai/src/prelude.rs` 与 `lumosai_core/src/prelude.rs`
  - 简化层重导出常用类型：`Agent`、`SimpleAgent`、`AgentBuilder`、`Tool`、`SessionManager`、`EventBus`、`VectorStorage`、`IndexConfig` 等。
- 模块映射（示例）：
  - `src/agent.rs` → `lumosai_core::agent::{AgentBuilder, simplified_api::Agent, ...}`
  - `src/session.rs` → `lumosai_core::agent::session::{SessionManager, MemorySessionStorage,...}`
  - `src/events.rs` → `lumosai_core::agent::events::{EventBus, EventHandler,...}`
  - `src/orchestration.rs` → `lumosai_core::agent::orchestration::{BasicOrchestrator, CollaborationTask,...}`
  - `src/rag.rs` → `lumosai_core::rag::{RagPipeline, RagPipelineBuilder, BasicRagPipeline}`
  - `src/vector.rs` → `lumosai_vector_core::traits::{VectorStorage}` 与多后端实现

## 🤖 Agent（构建、简化用法与专业化）
- 快速创建：`lumosai::agent::simple()`/`auto()`（教学演示）
- 生产级构建：`lumosai_core::agent::AgentBuilder` 与 `simplified_api::Agent`
- 专业化 Agent：`web_agent`、`file_agent`、`data_agent`（预置工具集）

```rust
use lumosai_core::prelude::*;
use std::sync::Arc;

#[tokio::main]
/// 创建专业化 Web Agent 并生成回复（生产实现）
async fn main() -> Result<()> {
    // 模型提供商（示例：测试/Mock，可替换为 openai/anthropic）
    let llm = lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc();

    // 使用简化 API 的构建器（生产级 AgentBuilder）
    let agent = Agent::builder()
        .name("web_helper")
        .instructions("你可以进行网页访问与解析")
        .model(llm)
        .max_tool_calls(8)
        .build()?;

    // 生成回复
    let resp = agent.generate("请概述LumosAI的核心能力").await?;
    println!("回复: {}", resp.content);
    Ok(())
}
```

## 🧰 工具系统（内置工具与便捷函数）
- 内置工具位于 `lumosai_core::tool::builtin::*`，通过 `prelude` 提供便捷创建函数：`file_reader()`、`file_writer()`、`calculator()`、`web_scraper()`、`http_request()`、`json_api()`、`url_validator()` 等。
- 专业化 Agent 的工具集组合：`web_agent()`、`file_agent()`、`data_agent()`。

```rust
use lumosai_core::prelude::*;

#[tokio::main]
/// 通过便捷函数添加常用工具到 Agent（生产实现）
async fn main() -> Result<()> {
    let llm = lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc();

    let agent = Agent::builder()
        .name("data_helper")
        .instructions("你可以执行数据处理与统计分析")
        .model(llm)
        .tools(vec![json_parser(), csv_parser(), calculator(), statistics()])
        .enable_smart_defaults()
        .build()?;

    let resp = agent.generate("解析并统计CSV的关键指标").await?;
    println!("回复: {}", resp.content);
    Ok(())
}
```

## 💾 内存与会话（Session 管理）
- 简化层：`src/session.rs` 提供 `create()`、`create_with_storage()`、`load()`、`list_user_sessions()` 与 `SessionBuilder`。
- 核心层：`lumosai_core::agent::session::{SessionManager, MemorySessionStorage,...}`。

```rust
use lumosai::prelude::*;
use std::sync::Arc;

#[tokio::main]
/// 使用简化会话 API：创建、添加消息、保存与查询历史
async fn main() -> Result<()> {
    // 创建会话（默认内存存储）
    let session = lumosai::session::create("agent_alpha", Some("user_42")).await?;

    /// 添加一条消息并保存
    session.add_message(Message { role: Role::User, content: "你好".into(), metadata: None, name: None }).await?;
    session.save().await?;

    /// 列出用户会话（便捷方法）
    let storage = Arc::new(lumosai::session::MemorySessionStorage::new());
    let list = lumosai::session::list_user_sessions("user_42", storage, Some(10)).await?;
    println!("会话数量: {}", list.len());
    Ok(())
}
```

## 📣 事件系统（Event Bus 与处理器）
- 简化层：`src/events.rs` 提供 `create_bus()`、`publish()`、`subscribe()`、`register_log_handler()`、`register_metrics_handler()`、`filter()`。
- 核心层：`lumosai_core::agent::events::{EventBus, EventHandler, LogEventHandler, MetricsEventHandler, EventFilter}`。

```rust
use lumosai::prelude::*;

#[tokio::main]
/// 创建事件总线并发布/订阅事件（简化 API）
async fn main() -> Result<()> {
    let bus = lumosai::events::create_bus(1000);

    /// 发布一个 Agent 启动事件
    lumosai::events::publish(&bus, "agent_started", serde_json::json!({"agent_id": "agent_001"})).await?;

    /// 订阅事件并接收
    let mut rx = lumosai::events::subscribe(&bus);
    tokio::spawn(async move {
        while let Ok(evt) = rx.recv().await {
            println!("收到事件: {:?}", evt);
        }
    });

    Ok(())
}
```

## 🤝 编排协作（Sequential/Parallel/Pipeline）
- 简化层：`src/orchestration.rs` 提供 `task()`、`sequential()`、`parallel()`、`execute()` 与 `TaskBuilder`。
- 核心层：`lumosai_core::agent::orchestration::{AgentOrchestrator, BasicOrchestrator, CollaborationTask, OrchestrationPattern}`。

```rust
use lumosai::prelude::*;

#[tokio::main]
/// 创建协作任务并以顺序模式执行（简化 API）
async fn main() -> Result<()> {
    let researcher = lumosai::agent::simple("gpt-4", "You are a researcher").await?;
    let writer = lumosai::agent::simple("gpt-4", "You are a writer").await?;

    /// 创建协作任务（顺序模式）
    let task = lumosai::orchestration::task()
        .name("Research and Write")
        .agent(researcher)
        .agent(writer)
        .pattern(Pattern::Sequential)
        .build();

    /// 执行协作
    let results = lumosai::orchestration::execute(task).await?;
    println!("执行状态: {:?}", results.status);
    Ok(())
}
```

## 📚 RAG 系统（简化 API 与生产管道）
- 简化层：`src/rag.rs` 提供 `rag::simple()`、`rag::auto()`、`rag::builder()` 与 `RagSystem`（教学占位实现）。
- 核心层：`lumosai_core::rag::{RagPipeline, RagPipelineBuilder, BasicRagPipeline}` 与 `lumosai_vector` 多后端。

```rust
use lumosai_core::prelude::*;
use lumosai_core::rag::{RagPipelineBuilder, DocumentSource};

#[tokio::main]
/// 使用生产级 RagPipelineBuilder 构建管道并查询
async fn main() -> Result<()> {
    // 1) 构建管道并添加数据源
    let pipeline = RagPipelineBuilder::new("kb")
        .add_source(DocumentSource::from_text("LumosAI 支持 Agent、工具与 RAG 集成。"))
        .build()
        .await?;

    // 2) 执行查询
    let result = pipeline.query("介绍 LumosAI 的核心能力", 5).await?;
    println!("相关文档数量: {}", result.documents.len());
    Ok(())
}
```

## 📦 向量存储（Memory/Postgres/Qdrant/Weaviate/LanceDB）
- 简化层：`src/vector.rs` 提供一行创建函数：`memory()`、`auto()`、`postgres(url)`、`qdrant(url)`、`weaviate(url)` 与 `VectorStorageBuilder`。
- 核心层：`lumosai_vector_core::traits::VectorStorage` 与多实现；索引配置 `IndexConfig` 与 `Document` 类型。

```rust
use lumosai::prelude::*;

#[tokio::main]
/// 一行代码创建内存向量存储并插入/检索文档（简化 API）
async fn main() -> Result<()> {
    // 创建向量存储（内存）
    let storage = lumosai::vector::memory().await?;

    /// 插入文档并创建索引
    storage.create_index("docs", IndexConfig::default()).await?;
    storage.upsert_documents("docs", vec![
        Document { id: "d1".into(), content: "LumosAI 是一个模块化 AI 框架".into(), metadata: None },
        Document { id: "d2".into(), content: "它支持 Agent、RAG 和工具".into(), metadata: None },
    ]).await?;

    /// 执行搜索
    let results = storage.search("docs", "什么是 LumosAI?", 3).await?;
    println!("命中数量: {}", results.len());
    Ok(())
}
```

## 🔧 端到端组合（Agent + 会话 + RAG + 向量 + 事件）
```rust
use lumosai::prelude::*;
use std::sync::Arc;

#[tokio::main]
/// 组合简化 API：创建Agent、持久化会话、构建RAG、写入向量并订阅事件
async fn main() -> Result<()> {
    // 1) Agent
    let agent = lumosai::agent::simple("gpt-4", "You are a helpful assistant").await?;

    // 2) 会话（持久化消息）
    let session = lumosai::session::create("assistant", Some("user_123")).await?;
    session.add_message(Message { role: Role::User, content: "你好！".into(), metadata: None, name: None }).await?;

    // 3) 向量存储
    let storage = lumosai::vector::memory().await?;

    // 4) RAG（简化管道）
    let rag = lumosai::rag::simple(storage.clone(), "openai").await?;
    rag.add_document("LumosAI 是一个高性能、类型安全的Rust AI框架").await?;

    // 5) 事件总线
    let bus = lumosai::events::create_bus(1000);
    lumosai::events::register_log_handler(&bus).await?;
    lumosai::events::publish(&bus, "agent_started", serde_json::json!({"agent_id": "assistant"})).await?;

    println!("✅ 端到端组合完成");
    Ok(())
}
```

## 🖥️ macOS 环境与配置建议
- 环境变量：
  - `OPENAI_API_KEY`、`ANTHROPIC_API_KEY`、`DEEPSEEK_API_KEY` 等按需设置。
  - 向量后端：`QDRANT_URL`、`WEAVIATE_URL`、`POSTGRES_URL`。
- 常用命令：
  - `export OPENAI_API_KEY="your-key"`
  - `cargo run --package lumosai_examples --bin test_prelude_comprehensive`
  - `cargo test -p lumosai_core`

## 🚚 迁移策略与最佳实践
- 教学 → 生产：
  - 简化层 `src/*` 适合快速演示；上线建议切换到 `lumosai_core` 的 Builder/trait，实现更强的可控与观测性。
- 模块解耦与组合：
  - Agent 与 RAG 管道解耦，通过 Builder 或运行时注入组合，保证可替换性（嵌入提供商、重排序、向量后端）。
- 配置与可观测性：
  - 使用 `EventBus` 与指标处理器采集关键事件，结合 `Telemetry` 模块进行端到端观测。
- 测试策略：
  - 以模块粒度进行单测（Agent、工具、RAG、向量），搭配端到端集成测试确保组合行为正确。

## 📎 接口速查（精选）
- Agent：`Agent::builder()`、`Agent::new()`、`web_agent()`、`file_agent()`、`data_agent()`
- 会话：`SessionManager::new(storage)`、`create()`、`load()`、`list_user_sessions()`
- 事件：`EventBus::new(cap)`、`publish()`、`subscribe()`、`register_log_handler()`
- 编排：`task()`、`execute()`、`sequential()`、`parallel()`、`OrchestrationPattern`
- RAG：`RagPipelineBuilder::new(name)`、`add_source()`、`query()`
- 向量：`vector::memory()`、`vector::postgres(url)`、`vector::qdrant(url)`、`IndexConfig`

---

> 结语：本教程已系统性梳理简化 API 与核心实现的对应关系，并给出端到端组合示例。后续可根据业务场景在 `lumosai_core` 层增强工具、内存、RAG 与编排能力，以获得生产级的稳定性与性能。