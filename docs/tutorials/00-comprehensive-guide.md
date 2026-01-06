# 教程 00：LumosAI 全面指南（从简化 API 到生产级架构）

> 本教程系统性覆盖 LumosAI 的核心模块与用法：Agent、工具、内存与会话、事件系统、向量存储、RAG、编排协作与测试构建。所有示例以中文呈现，并在函数级添加注释。

## 🧭 总览与架构
- 模块分层：
  - `src/` 提供简化 API（快速上手、示例为主），如 `agent::simple()`、`rag::simple()`、`vector::memory()`。
  - `lumosai_core/` 提供生产级实现（完整的 Builder、工具系统、内存系统、会话持久化、事件与编排等）。
  - `lumosai_vector/*` 提供不同向量存储后端（内存、Postgres、Qdrant、LanceDB 等）。
- 开发建议：教程示例多使用 `use lumosai::prelude::*` 重导出，简化引入路径。生产环境建议使用 `lumosai_core` 的 Builder 与配置能力。
- 重要说明：当前 `src/agent.rs` 与 `src/rag.rs` 的简化实现为教学/占位实现，默认返回模拟数据；集成真实 LLM/RAG 时应使用 `lumosai_core` 提供的实现与接口。

## ⚙️ 安装与工程配置
- 添加依赖与默认特性：
  ```toml
  [dependencies]
  lumosai = "0.2.0"
  tokio = { version = "1", features = ["full"] }
  serde = { version = "1", features = ["derive"] }
  serde_json = "1"
  ```
- 向量存储特性（根据需要启用）：
  - `vector-memory`（默认启用）：内存向量存储，适合开发、单机快速原型。
  - `vector-qdrant`、`vector-postgres`、`vector-lancedb`：外部存储，适合生产检索与扩展。

## 🤖 Agent：从简化到 Builder
### 快速上手（简化 API）
```rust
use lumosai::prelude::*;

#[tokio::main]
/// 启动一个简化 Agent 并进行一次对话
async fn main() -> Result<()> {
    // 注意：简化 Agent 为教学占位实现，返回示例文本
    let agent = lumosai::agent::simple("gpt-4o-mini", "你是一个中文助手").await?;

    let reply = agent.chat("你好，帮我总结一下这段文本").await?;
    println!("Agent 回复: {}", reply);
    Ok(())
}
```

### 生产建议（使用 `AgentBuilder`）
```rust
use lumosai::prelude::*;
use std::sync::Arc;

#[tokio::main]
/// 使用生产级 Builder 创建 Agent，并配置模型与工具
async fn main() -> Result<()> {
    // 示例：使用测试 LLM 或实际 LLM 提供商（此处仅展示接口）
    let builder = lumosai_core::agent::AgentBuilder::new()
        .name("中文助手")
        .instructions("你是一个专业的中文助手，回答准确简洁")
        .model_name("gpt-4o");

    let agent = builder.build()?; // 生产中建议提前初始化模型与工具
    let resp = agent.generate("请用要点说明 Rust 的优势").await?;
    println!("回复: {}", resp);
    Ok(())
}
```

## 🧰 工具集成（Tools）
- 方法：`AgentBuilder::tool(...)`、`tools(...)`、`with_web_tools()`。
- 用途：扩展 Agent 能力，如 HTTP 请求、网页抓取、计算、文件操作等。
```rust
use lumosai::prelude::*;
use std::sync::Arc;

#[tokio::main]
/// 为 Agent 添加工具以增强外部能力
async fn main() -> Result<()> {
    // 直接引入内置 web 工具集
    let agent = lumosai_core::agent::AgentBuilder::new()
        .name("工具助手")
        .instructions("善用工具解决问题")
        .with_web_tools()
        .model_name("gpt-4o")
        .build()?;

    let resp = agent.generate("抓取 https://example.com 的标题").await?;
    println!("工具调用结果: {}", resp);
    Ok(())
}
```

## 🧠 内存与会话（Memory & Session）
### 工作内存（Working Memory）配置
```rust
use lumosai::prelude::*;
use lumosai_core::memory::WorkingMemoryConfig;

#[tokio::main]
/// 配置 Agent 的工作内存以支持上下文与临时存储
async fn main() -> Result<()> {
    let wm = WorkingMemoryConfig {
        capacity: Some(128),
        strategy: Some("sliding_window".to_string()),
        ..Default::default()
    };

    let agent = lumosai_core::agent::AgentBuilder::new()
        .name("记忆助手")
        .instructions("维护上下文并进行总结")
        .working_memory(wm)
        .model_name("gpt-4o")
        .build()?;

    let resp = agent.generate("上下文记忆演示：请逐步总结输入").await?;
    println!("回复: {}", resp);
    Ok(())
}
```

### 会话管理（简化 API）
```rust
use lumosai::prelude::*;

#[tokio::main]
/// 创建并使用一个简化会话，持久化消息历史
async fn main() -> Result<()> {
    let session = lumosai::session::create("my_agent", Some("user_123")).await?;

    /// 插入一条用户消息
    let msg = Message { role: Role::User, content: "你好！".to_string(), metadata: None, name: None };
    session.add_message(msg).await?;

    /// 获取历史消息用于上下文
    let history = session.get_messages().await?;
    println!("历史消息条数: {}", history.len());
    Ok(())
}
```

## 📣 事件系统（Event Bus）
```rust
use lumosai::prelude::*;

#[tokio::main]
/// 创建事件总线并发布/订阅事件
async fn main() -> Result<()> {
    let bus = lumosai::events::create_bus(1000);

    /// 发布一个 Agent 启动事件
    lumosai::events::publish(&bus, "agent_started", serde_json::json!({
        "agent_id": "agent_001"
    })).await?;

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

## 📦 向量存储（Vector Storage）
- 简化入口：`vector::memory()`、`vector::auto()`、`vector::builder()`。
- 用途：为 RAG 管道提供嵌入索引与近似搜索。
```rust
use lumosai::prelude::*;

#[tokio::main]
/// 创建内存向量存储并进行基本索引/查询（示例）
async fn main() -> Result<()> {
    let store = lumosai::vector::memory().await?; // 默认内存存储

    /// 创建索引并插入文档（具体 API 以实际 VectorStorage trait 为准）
    store.create_index("docs").await?;
    store.insert("docs", "doc_1", "这是一个关于 Rust 的文档").await?;

    let results = store.search("docs", "Rust", 5).await?;
    println!("检索结果数量: {}", results.len());
    Ok(())
}
```

## 🔎 RAG 系统（检索增强生成）
- 简化入口：`rag::simple()`、`rag::auto()`、`rag::builder()`。
- 说明：`src/rag.rs` 的 `SimpleRagImpl` 为教学实现（内存与关键词检索），生产场景建议使用 `lumosai_core` 的管道与 `lumosai_vector` 后端。
```rust
use lumosai::prelude::*;

#[tokio::main]
/// 使用简化 RAG 管道进行示例问答
async fn main() -> Result<()> {
    let rag = lumosai::rag::simple().await?; // 教学实现：内存管道

    /// 添加文档并进行检索问答
    rag.add_document("doc_1", "LumosAI 是一个模块化 AI 框架，用于构建 Agent 与 RAG").await?;
    let answer = rag.answer("什么是 LumosAI？").await?;
    println!("RAG 回答: {}", answer);
    Ok(())
}
```

## 🤝 多 Agent 编排（Orchestration）
```rust
use lumosai::prelude::*;

#[tokio::main]
/// 使用简化编排实现顺序协作（Sequential）
async fn main() -> Result<()> {
    let a1 = lumosai::agent::simple("gpt-4o-mini", "你是研究员").await?;
    let a2 = lumosai::agent::simple("gpt-4o-mini", "你是写作者").await?;

    let result = lumosai::orchestration::sequential(
        "研究与写作", vec![a1, a2], serde_json::json!({"topic": "AI 医疗"})
    ).await?;

    println!("任务状态: {}", result.status);
    Ok(())
}
```

## 🧪 测试与验证
- 运行构建与测试：
  - `cargo check`：快速类型与依赖检查
  - `cargo test -p lumosai_core`：核心功能单测
  - `cargo test`：工作区全部测试
- 建议：对你新增的模块示例，可添加最小化单测或示例运行脚本。

## ⚠️ 注意事项与最佳实践
- 简化模块为教学占位，请勿在生产使用其默认返回；生产用法请转 `lumosai_core`。
- 明确区分「教学代码」与「生产代码」目录与依赖。
- 启用向量存储特性时，请确保正确配置服务端（如 Qdrant/Postgres）。
- 工具调用需进行安全审查与参数校验，避免不受控外部调用。
- 事件总线建议在生产中结合日志与指标处理器（`LogEventHandler`、`MetricsEventHandler`）。

## 🔗 进一步阅读
- `docs/QUICK_START.md`、`docs/USER_GUIDE.md`
- `docs/VECTOR_DATABASES.md`（向量数据库详解与配置）
- `docs/api-reference/README.md`（API 参考总览）
- `lumosai_core/src/agent/builder.rs`（AgentBuilder 全部方法与示例）

---

如需针对你的业务场景定制教程，请提出需求或示例输入，我会补充对应章节与代码。