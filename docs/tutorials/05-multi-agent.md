# 教程 05：多 Agent 编排与协作

本教程介绍如何使用 LumosAI 的简化编排 API 快速实现多 Agent 的协作，包括顺序（Sequential）、并行（Parallel）与管道（Pipeline）三种模式，并给出可直接运行的示例代码与最佳实践建议。

- 教学 API 文件：`src/orchestration.rs`
- 关键便捷函数：`task()`、`execute()`、`sequential()`、`parallel()`
- 结果类型：`OrchestrationResult { task_id, results, execution_time_ms, status }`

> 说明：本教程使用简化版编排接口，适合学习与原型开发。生产场景可迁移到 `lumosai_core::agent::orchestration` 下的完整编排器（含事件总线与状态管理），详见文末“进阶：连接生产级编排器”。

---

## 顺序模式（Sequential）

适用场景：步骤明确、线性推进，如“调研 → 结构 → 撰写”。

```rust
use lumosai::prelude::*;
use serde_json::json;

/// 演示顺序模式编排：两个 Agent 依次处理任务
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 准备两个简化 Agent（模型与角色）
    let researcher = lumosai::agent::simple("gpt-4", "You are a researcher").await?;
    let writer = lumosai::agent::simple("gpt-4", "You are a writer").await?;

    // 顺序执行：按添加先后依次处理
    let result = lumosai::orchestration::sequential(
        "Research and Write",
        vec![researcher, writer],
        json!({"topic": "AI 在医疗中的应用"})
    ).await?;

    // 输出结果（简化实现中，每个 agent_i 会生成一条字符串）
    println!("task_id={} status={} results={:?}", result.task_id, result.status, result.results);
    Ok(())
}
```

---

## 并行模式（Parallel）

适用场景：独立子任务可以同时执行，如“分析 + 评审”。

```rust
use lumosai::prelude::*;
use serde_json::json;

/// 演示并行模式：两个 Agent 并发处理同一输入
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let analyst = lumosai::agent::simple("gpt-4", "You are an analyst").await?;
    let critic = lumosai::agent::simple("gpt-4", "You are a critic").await?;

    let result = lumosai::orchestration::parallel(
        "Analyze and Critique",
        vec![analyst, critic],
        json!({"content": "请对以下段落进行分析与评审"})
    ).await?;

    println!("并行结果: {:?}", result.results);
    Ok(())
}
```

---

## 管道模式（Pipeline）

适用场景：串行分阶段，但每阶段有明确职责（例如：提取要点 → 扩写 → 校对）。

```rust
use lumosai::prelude::*;
use serde_json::json;

/// 演示管道模式：TaskBuilder 设置为 Pipeline 并执行
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let extractor = lumosai::agent::simple("gpt-4", "You extract key points").await?;
    let expander = lumosai::agent::simple("gpt-4", "You expand content").await?;
    let proofreader = lumosai::agent::simple("gpt-4", "You proofread").await?;

    // 使用任务构建器定义管道
    let task = lumosai::orchestration::task()
        .name("Pipeline Writing")
        .description("Extract → Expand → Proofread")
        .agents(vec![extractor, expander, proofreader])
        .pattern(lumosai_core::agent::orchestration::OrchestrationPattern::Pipeline)
        .input(json!({"draft": "这是一段需要处理的草稿文本"}))
        .timeout(30)
        .build();

    let result = lumosai::orchestration::execute(task).await?;
    println!("管道结果: {:?}", result.results);
    Ok(())
}
```

---

## 通用任务构建：`task()` + `execute()`

当需要自定义模式或添加描述、超时等元信息时，使用 `TaskBuilder`：

```rust
use lumosai::prelude::*;
use serde_json::json;

/// 通用示例：自定义描述与超时，并以并行模式执行
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a1 = lumosai::agent::simple("gpt-4", "You summarize").await?;
    let a2 = lumosai::agent::simple("gpt-4", "You verify").await?;

    let task = lumosai::orchestration::task()
        .name("Summarize & Verify")
        .description("并行进行：摘要与事实核查")
        .agents(vec![a1, a2])
        .pattern(lumosai_core::agent::orchestration::OrchestrationPattern::Parallel)
        .input(json!({"article": "长文内容……"}))
        .timeout(20)
        .build();

    let result = lumosai::orchestration::execute(task).await?;
    println!("结果: {:?}", result.results);
    Ok(())
}
```

---

## 结果结构：`OrchestrationResult`

简化编排返回：

```rust
/// 编排结果结构（教学版）
/// task_id: 任务唯一标识
/// results: 每个 agent_i 的输出（教学版为字符串或 JSON）
/// execution_time_ms: 执行耗时
/// status: 执行状态（"completed" 等）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrchestrationResult {
    pub task_id: String,
    pub results: std::collections::HashMap<String, serde_json::Value>,
    pub execution_time_ms: u64,
    pub status: String,
}
```

---

## 最佳实践

- 输入结构化：将上下文统一打包到 `serde_json::Value`，例如 `{ "topic": "...", "constraints": [..] }`。
- 角色清晰：为不同阶段使用明确的系统提示（如“检索员”“整合员”“校对员”）。
- 超时与描述：通过 `.timeout(..)` 与 `.description(..)` 标注任务元信息，便于日志与排查。
- 可测试：对结果中的每个 `agent_i` 进行断言，保持示例可回归。

---

## 进阶：连接生产级编排器

当需要状态、事件与复杂模式（如投票、条件分支）时，迁移到 `lumosai_core::agent::orchestration`：

```rust
use std::sync::Arc;
use lumosai_core::agent::events::EventBus;
use lumosai_core::agent::orchestration::{BasicOrchestrator, OrchestrationPattern, AgentOrchestrator};
use lumosai_core::Agent; // 生产级 Agent trait
use serde_json::json;

/// 演示如何初始化事件总线与基础编排器（生产版）
#[tokio::main]
async fn main() -> lumosai_core::Result<()> {
    let event_bus = Arc::new(EventBus::new(100));
    let orchestrator = BasicOrchestrator::new(event_bus.clone());

    // 生产版需要注册到会话，并以 AgentId 驱动
    // 省略完整示例，详见 docs/MULTI_AGENT_QUICK_REFERENCE.md

    Ok(())
}
```

> 提示：教学 API 与生产 API 的数据结构不同（如参与者标识、状态管理、事件流等）。从教学版迁移到生产版时，先在单测中固定输入输出，再替换编排与 Agent 抽象。

---

## 参考

- 源码：`src/orchestration.rs`
- 速查：`docs/MULTI_AGENT_QUICK_REFERENCE.md`
- 完整 API：`lumosai_core/src/agent/orchestration.rs`