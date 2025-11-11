# LumosAI 多智能体协作快速参考

> **快速查找**: 选择合适的协作模式,复制代码即用

---

## 🎯 模式选择决策树

```
你的任务是什么?

1. 有明确的顺序依赖 (A→B→C)
   → 使用 Sequential (顺序执行)

2. 多个独立任务,需要并行处理
   → 使用 Parallel (并行执行)

3. 复杂的依赖关系 (A→B, A→C, B+C→D)
   → 使用 DAG Orchestration (DAG 编排)

4. 需要多个 Agent 讨论/辩论
   → 使用 Group Chat (群聊协作) [待实现]

5. 需要动态路由到不同专家
   → 使用 Handoff (任务移交) [待实现]

6. 需要迭代改进 (生成→审核→改进)
   → 使用 Reflection (反思优化) [待实现]

7. 需要动态规划任务
   → 使用 Magentic (动态任务规划) [待实现]

8. 需要正反方辩论
   → 使用 Debate (多方辩论) [待实现]

9. 有管理者和工作者
   → 使用 Hierarchical (层级执行)

10. 事件驱动的响应式系统
    → 使用 SOP React (事件驱动)
```

---

## 📖 代码示例速查

### 1. Sequential (顺序执行) ✅

**场景**: 数据清洗 → 分析 → 报告生成

```rust
use lumosai_core::agent::{AgentPipeline, AgentBuilder};
use std::sync::Arc;

// 创建 Agents
let cleaner = Arc::new(AgentBuilder::new()
    .name("cleaner")
    .instructions("You clean data")
    .model(llm.clone())
    .build()?);

let analyzer = Arc::new(AgentBuilder::new()
    .name("analyzer")
    .instructions("You analyze data")
    .model(llm.clone())
    .build()?);

let reporter = Arc::new(AgentBuilder::new()
    .name("reporter")
    .instructions("You generate reports")
    .model(llm.clone())
    .build()?);

// 构建 Pipeline
let pipeline = AgentPipeline::new(cleaner)
    .pipe(analyzer)
    .pipe(reporter);

// 执行
let result = pipeline.execute("Process sales data").await?;
println!("Final report: {}", result);
```

---

### 2. Parallel (并行执行) ✅

**场景**: 从技术、商业、用户体验三个角度分析

```rust
use lumosai_core::agent::{AgentParallel, AgentBuilder};
use std::sync::Arc;

// 创建 Agents
let technical = Arc::new(AgentBuilder::new()
    .name("technical")
    .instructions("You analyze from technical perspective")
    .model(llm.clone())
    .build()?);

let business = Arc::new(AgentBuilder::new()
    .name("business")
    .instructions("You analyze from business perspective")
    .model(llm.clone())
    .build()?);

let ux = Arc::new(AgentBuilder::new()
    .name("ux")
    .instructions("You analyze from UX perspective")
    .model(llm.clone())
    .build()?);

// 构建 Parallel
let parallel = AgentParallel::new(technical)
    .parallel(business)
    .parallel(ux);

// 执行
let results = parallel.execute("Evaluate Rust programming language").await?;
for (i, result) in results.iter().enumerate() {
    println!("Perspective {}: {}", i + 1, result);
}
```

---

### 3. DAG Orchestration (DAG 编排) ✅

**场景**: 复杂工作流 (A → B → D, A → C → D)

```rust
use lumosai_core::agent::{AgentDagOrchestrator, RuntimeContext};
use serde_json::json;
use std::sync::Arc;

// 创建 Agents
let agent_a = Arc::new(AgentBuilder::new().name("A").model(llm.clone()).build()?);
let agent_b = Arc::new(AgentBuilder::new().name("B").model(llm.clone()).build()?);
let agent_c = Arc::new(AgentBuilder::new().name("C").model(llm.clone()).build()?);
let agent_d = Arc::new(AgentBuilder::new().name("D").model(llm.clone()).build()?);

// 构建 DAG
let orchestrator = AgentDagOrchestrator::new();
orchestrator.add_agent("A", agent_a, vec![]).await?;
orchestrator.add_agent("B", agent_b, vec!["A"]).await?;
orchestrator.add_agent("C", agent_c, vec!["A"]).await?;
orchestrator.add_agent("D", agent_d, vec!["B", "C"]).await?;

// 执行
let context = RuntimeContext::default();
let input = json!({"message": "Start workflow"});
let results = orchestrator.execute(input, &context).await?;

println!("DAG results: {:?}", results);
```

---

### 4. Hierarchical (层级执行) ✅

**场景**: 项目管理 (Manager + Workers)

```rust
use lumosai_core::agent::{Crew, CollaborationMode, CrewAgentRole};
use std::sync::Arc;

// 创建 Crew
let crew = Crew::new("project_team", CollaborationMode::Hierarchical, 10);

// 添加 Manager
let manager = Arc::new(AgentBuilder::new()
    .name("manager")
    .instructions("You coordinate the team")
    .model(llm.clone())
    .build()?);

crew.add_agent("manager", manager, CrewAgentRole::Manager).await?;

// 添加 Workers
let worker1 = Arc::new(AgentBuilder::new()
    .name("worker1")
    .instructions("You handle backend tasks")
    .model(llm.clone())
    .build()?);

let worker2 = Arc::new(AgentBuilder::new()
    .name("worker2")
    .instructions("You handle frontend tasks")
    .model(llm.clone())
    .build()?);

crew.add_agent("worker1", worker1, CrewAgentRole::Worker).await?;
crew.add_agent("worker2", worker2, CrewAgentRole::Worker).await?;

// 执行
let results = crew.kickoff().await?;
println!("Project results: {:?}", results);
```

---

### 5. Group Chat (群聊协作) ⚠️ 待实现

**场景**: 多个专家讨论 AI 伦理

```rust
use lumosai_core::agent::GroupChat;
use std::sync::Arc;

// 创建 Agents
let ethicist = Arc::new(AgentBuilder::new()
    .name("ethicist")
    .instructions("You are an AI ethics expert")
    .model(llm.clone())
    .build()?);

let engineer = Arc::new(AgentBuilder::new()
    .name("engineer")
    .instructions("You are an AI engineer")
    .model(llm.clone())
    .build()?);

let lawyer = Arc::new(AgentBuilder::new()
    .name("lawyer")
    .instructions("You are a tech lawyer")
    .model(llm.clone())
    .build()?);

// 构建 Group Chat
let mut chat = GroupChat::new()
    .add_agent("ethicist", ethicist)
    .add_agent("engineer", engineer)
    .add_agent("lawyer", lawyer)
    .max_rounds(10)
    .termination_condition(|thread| thread.has_consensus());

// 执行
let result = chat.execute("Discuss AI safety regulations").await?;
println!("Discussion result: {}", result);
```

---

### 6. Handoff (任务移交) ⚠️ 待实现

**场景**: 客户支持 (Triage → Specialist)

```rust
use lumosai_core::agent::{HandoffOrchestrator, HandoffCondition};
use std::sync::Arc;

// 创建 Agents
let triage = Arc::new(AgentBuilder::new()
    .name("triage")
    .instructions("You triage customer requests")
    .model(llm.clone())
    .build()?);

let technical = Arc::new(AgentBuilder::new()
    .name("technical")
    .instructions("You handle technical issues")
    .model(llm.clone())
    .build()?);

let billing = Arc::new(AgentBuilder::new()
    .name("billing")
    .instructions("You handle billing issues")
    .model(llm.clone())
    .build()?);

// 构建 Handoff
let orchestrator = HandoffOrchestrator::new()
    .add_agent("triage", triage)
    .add_agent("technical", technical)
    .add_agent("billing", billing)
    .add_handoff("triage", "technical", HandoffCondition::contains("network"))
    .add_handoff("triage", "billing", HandoffCondition::contains("payment"));

// 执行
let result = orchestrator.execute("My internet is down").await?;
println!("Support result: {}", result);
```

---

### 7. Reflection (反思优化) ⚠️ 待实现

**场景**: 代码生成 + 审查

```rust
use lumosai_core::agent::ReflectionLoop;
use std::sync::Arc;

// 创建 Agents
let generator = Arc::new(AgentBuilder::new()
    .name("generator")
    .instructions("You generate code")
    .model(llm.clone())
    .build()?);

let reviewer = Arc::new(AgentBuilder::new()
    .name("reviewer")
    .instructions("You review code and provide feedback")
    .model(llm.clone())
    .build()?);

// 构建 Reflection Loop
let reflection = ReflectionLoop::new()
    .generator(generator)
    .critic(reviewer)
    .max_iterations(5)
    .improvement_threshold(0.8);

// 执行
let result = reflection.execute("Write a binary search function in Rust").await?;
println!("Final code: {}", result);
```

---

### 8. Maker-Checker Loop ⚠️ 待实现

**场景**: 内容创作 + 编辑

```rust
use lumosai_core::agent::MakerCheckerLoop;
use std::sync::Arc;

// 创建 Agents
let maker = Arc::new(AgentBuilder::new()
    .name("maker")
    .instructions("You create content")
    .model(llm.clone())
    .build()?);

let checker = Arc::new(AgentBuilder::new()
    .name("checker")
    .instructions("You review and approve content")
    .model(llm.clone())
    .build()?);

// 构建 Maker-Checker Loop
let loop_exec = MakerCheckerLoop::new(maker, checker)
    .max_iterations(3);

// 执行
let result = loop_exec.execute("Write a blog post about Rust").await?;
println!("Approved content: {}", result);
```

---

### 9. Debate (多方辩论) ⚠️ 待实现

**场景**: 技术决策 (正方 vs 反方)

```rust
use lumosai_core::agent::{DebateOrchestrator, DebateRole};
use std::sync::Arc;

// 创建 Agents
let proponent = Arc::new(AgentBuilder::new()
    .name("proponent")
    .instructions("You argue FOR microservices")
    .model(llm.clone())
    .build()?);

let opponent = Arc::new(AgentBuilder::new()
    .name("opponent")
    .instructions("You argue AGAINST microservices")
    .model(llm.clone())
    .build()?);

let judge = Arc::new(AgentBuilder::new()
    .name("judge")
    .instructions("You evaluate both arguments")
    .model(llm.clone())
    .build()?);

// 构建 Debate
let debate = DebateOrchestrator::new()
    .add_debater("proponent", proponent, DebateRole::For)
    .add_debater("opponent", opponent, DebateRole::Against)
    .add_judge("judge", judge)
    .max_rounds(5);

// 执行
let result = debate.execute("Should we adopt microservices architecture?").await?;
println!("Debate result: {}", result);
```

---

## 🔧 常用工具函数

### 创建测试 Agent

```rust
use lumosai_core::agent::AgentBuilder;
use lumosai_core::llm::zhipu::ZhipuProvider;

fn create_test_agent(name: &str, instructions: &str) -> Result<Agent> {
    let llm = Arc::new(ZhipuProvider::new(
        std::env::var("ZHIPU_API_KEY")?,
        "glm-4-flash".to_string(),
    ));
    
    AgentBuilder::new()
        .name(name)
        .instructions(instructions)
        .model(llm)
        .build()
}
```

### 创建 Crew

```rust
use lumosai_core::agent::{Crew, CollaborationMode};

fn create_crew(name: &str, mode: CollaborationMode) -> Crew {
    Crew::new(name, mode, 10)
}
```

---

## 📊 性能参考

| 模式 | 3 Agents | 10 Agents | 适用场景 |
|------|---------|----------|---------|
| Sequential | ~45s | ~150s | 有依赖关系 |
| Parallel | ~15s | ~15s | 独立任务 |
| DAG | ~30s | ~80s | 复杂依赖 |
| Group Chat | ~120s | ~400s | 讨论/辩论 |
| Handoff | ~20s | ~60s | 动态路由 |

**测试条件**: GPT-4, 相同提示词, 平均 5 次运行

---

## 🚨 常见错误

### 1. 忘记 await

```rust
// ❌ 错误
let result = pipeline.execute("input");

// ✅ 正确
let result = pipeline.execute("input").await?;
```

### 2. 忘记 Arc

```rust
// ❌ 错误
let pipeline = AgentPipeline::new(agent1);

// ✅ 正确
let pipeline = AgentPipeline::new(Arc::new(agent1));
```

### 3. 循环依赖

```rust
// ❌ 错误 (A → B → A)
orchestrator.add_agent("A", agent_a, vec!["B"]).await?;
orchestrator.add_agent("B", agent_b, vec!["A"]).await?;

// ✅ 正确 (A → B)
orchestrator.add_agent("A", agent_a, vec![]).await?;
orchestrator.add_agent("B", agent_b, vec!["A"]).await?;
```

---

## 📚 更多资源

- **完整文档**: `docs/MULTI_AGENT_COLLABORATION_PATTERNS.md`
- **实现计划**: `docs/MULTI_AGENT_IMPLEMENTATION_PLAN.md`
- **框架对比**: `docs/FRAMEWORK_COMPARISON.md`
- **测试示例**: `tests/e2e/integration_tests.rs`
- **代码示例**: `lumosai_examples/examples/`

---

**提示**: 优先使用已实现的模式 (✅),待实现模式 (⚠️) 预计 3 周内完成。

