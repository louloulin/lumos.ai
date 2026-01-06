# 教程 03：内存系统与会话管理（Memory & Session）

> 本教程讲解两类内存：对话记忆（MemoryConfig/WorkingMemoryConfig）与会话存储（SessionStorage/MemorySessionStorage）。并提供简化 API 与生产级用法的示例，所有示例含函数级注释。

## 🧠 工作内存（Working Memory）
- 作用：控制对话上下文的保留策略与容量（如滑动窗口）。
- 入口：`lumosai_core::memory::WorkingMemoryConfig` 在 `AgentBuilder` 中通过 `.working_memory(...)` 配置。

```rust
use lumosai::prelude::*;
use lumosai_core::memory::WorkingMemoryConfig;

#[tokio::main]
/// 为 Agent 配置工作内存，用于维护有限长度的上下文
async fn main() -> Result<()> {
    let wm = WorkingMemoryConfig {
        enabled: true,
        max_capacity: Some(200),
        template: None,
        content_type: None,
    };

    let agent = lumosai_core::agent::AgentBuilder::new()
        .name("上下文助手")
        .instructions("根据历史上下文进行回答")
        .model_name("gpt-4o")
        .working_memory(wm)
        .build()?;

    let resp = agent.generate("请根据之前的对话继续回答").await?;
    println!("回复: {}", resp);
    Ok(())
}
```

## 💾 会话存储（Session Storage）
- 说明：会话用于记录消息历史、标题与状态，支持加载、保存与列出。
- 简化 API：`lumosai::session` 提供 `create`、`load`、`list_user_sessions`。
- 存储实现：默认内存存储 `MemorySessionStorage`，可替换其他后端。

```rust
use lumosai::prelude::*;
use std::sync::Arc;

#[tokio::main]
/// 使用简化会话 API：创建/保存/查询会话
async fn main() -> Result<()> {
    // 创建会话（默认内存存储）
    let session = lumosai::session::create("agent_alpha", Some("user_42")).await?;

    /// 添加一条消息并保存
    session.add_message(Message { role: Role::User, content: "你好".into(), metadata: None, name: None }).await?;
    session.save().await?;

    /// 列出用户会话
    let list = lumosai::session::list_user_sessions("user_42").await?;
    println!("会话数量: {}", list.len());
    Ok(())
}
```

### 自定义存储：`MemorySessionStorage`
```rust
use lumosai::prelude::*;
use std::sync::Arc;

#[tokio::main]
/// 使用自定义存储创建会话（演示内存存储注入）
async fn main() -> Result<()> {
    let storage = Arc::new(lumosai::session::MemorySessionStorage::new());
    let session = lumosai::session::create_with_storage("agent_beta", Some("user_007"), storage).await?;

    /// 设置标题与状态
    session.set_title(Some("第一次对话")).await?;
    session.set_state(Some(serde_json::json!({"phase": 1}))).await?;

    /// 再次保存
    session.save().await?;
    Ok(())
}
```

## 🧩 与 Agent 联动（上下文对话）
```rust
use lumosai::prelude::*;

#[tokio::main]
/// 将会话历史作为上下文传给 Agent（简化演示）
async fn main() -> Result<()> {
    let agent = lumosai::agent::simple("gpt-4o-mini", "你是总结助手").await?;
    let session = lumosai::session::create("summarizer", Some("user_88")).await?;

    /// 累积对话
    session.add_message(Message { role: Role::User, content: "请记住以下要点：安全、性能、可维护性".into(), metadata: None, name: None }).await?;
    session.add_message(Message { role: Role::User, content: "现在根据要点生成会议纪要".into(), metadata: None, name: None }).await?;

    /// 从会话提取上下文并传给 Agent（示例中简化为拼接文本）
    let history = session.get_messages().await?;
    let ctx = history.iter().map(|m| m.content.clone()).collect::<Vec<_>>().join("\n");

    let reply = agent.chat_with_context("生成纪要", &ctx).await?;
    println!("纪要: {}", reply);
    Ok(())
}
```

## ✅ 建议与实践
- 将短期上下文交由 `WorkingMemory` 管理，将长期历史交由 `SessionStorage` 持久化。
- 为会话设置结构化 `state`（如 JSON）以便工作流编排与状态机管理。
- 当对话较长时，使用滑动窗口或摘要策略减少上下文长度。
- 在生产中替换 `MemorySessionStorage` 为数据库/外部存储实现，保障可靠性。

---

如果你希望把会话与事件总线、编排任务联动，请告诉我目标流程，我将提供完整的示例与校验策略。