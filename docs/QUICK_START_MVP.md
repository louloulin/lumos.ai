# 🚀 LumosAI 快速开始指南（MVP 版本）

> **目标**: 5 分钟内创建你的第一个 AI Agent 应用

---

## 📋 前置要求

- Rust 1.75+ （推荐使用最新稳定版）
- 一个 LLM API 密钥（OpenAI、Anthropic、Qwen 或 Zhipu）

---

## 🔧 安装

### 1. 创建新项目

```bash
cargo new my-lumosai-app
cd my-lumosai-app
```

### 2. 添加依赖

编辑 `Cargo.toml`：

```toml
[dependencies]
lumosai_core = { path = "../lumosai_core" }  # 或使用 crates.io 版本
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
```

### 3. 设置环境变量

创建 `.env` 文件：

```bash
# OpenAI
OPENAI_API_KEY=your_openai_api_key

# 或者使用 Zhipu AI（智谱）
ZHIPUAI_API_KEY=your_zhipu_api_key

# 或者使用 Qwen（通义千问）
DASHSCOPE_API_KEY=your_qwen_api_key
```

---

## 🎯 核心示例

### 示例 1: 最简单的 Agent（30 秒）

创建 `src/main.rs`：

```rust
use lumosai_core::prelude::*;
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建 LLM 提供商（使用测试提供商）
    let llm = create_test_zhipu_provider_arc();
    
    // 2. 创建 Agent
    let agent = Agent::builder()
        .name("assistant")
        .instructions("You are a helpful AI assistant")
        .model(llm)
        .build()?;
    
    // 3. 发送消息
    let response = agent.generate("Hello! What can you help me with?").await?;
    println!("Agent: {}", response);
    
    Ok(())
}
```

运行：

```bash
cargo run
```

---

### 示例 2: 带工具的 Agent（2 分钟）

```rust
use lumosai_core::prelude::*;
use lumosai_core::tool::{create_tool, ToolBuilder};
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建工具
    let calculator = create_tool(
        "calculator",
        "Performs basic arithmetic operations",
        |args: serde_json::Value| async move {
            let a = args["a"].as_f64().unwrap_or(0.0);
            let b = args["b"].as_f64().unwrap_or(0.0);
            let op = args["op"].as_str().unwrap_or("+");
            
            let result = match op {
                "+" => a + b,
                "-" => a - b,
                "*" => a * b,
                "/" => if b != 0.0 { a / b } else { 0.0 },
                _ => 0.0,
            };
            
            Ok(json!({ "result": result }))
        },
    );
    
    // 2. 创建 Agent 并注册工具
    let llm = create_test_zhipu_provider_arc();
    let agent = Agent::builder()
        .name("math_assistant")
        .instructions("You are a math assistant. Use the calculator tool for calculations.")
        .model(llm)
        .tool(calculator)
        .build()?;
    
    // 3. 使用 Agent
    let response = agent.generate("What is 123 + 456?").await?;
    println!("Agent: {}", response);
    
    Ok(())
}
```

---

### 示例 3: 多 Agent 协作（3 分钟）

```rust
use lumosai_core::prelude::*;
use lumosai_core::agent::{pipe, AgentPipeline};
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let llm = create_test_zhipu_provider_arc();
    
    // 1. 创建研究 Agent
    let researcher = Agent::builder()
        .name("researcher")
        .instructions("You are a researcher. Gather information and facts.")
        .model(llm.clone())
        .build()?;
    
    // 2. 创建写作 Agent
    let writer = Agent::builder()
        .name("writer")
        .instructions("You are a writer. Create engaging content based on research.")
        .model(llm.clone())
        .build()?;
    
    // 3. 创建编辑 Agent
    let editor = Agent::builder()
        .name("editor")
        .instructions("You are an editor. Polish and improve the content.")
        .model(llm)
        .build()?;
    
    // 4. 创建 Agent 流水线
    let pipeline: AgentPipeline = pipe![researcher, writer, editor];
    
    // 5. 执行流水线
    let result = pipeline.execute("Write a short article about AI").await?;
    println!("Final result: {}", result);
    
    Ok(())
}
```

---

### 示例 4: 使用 Memory（4 分钟）

```rust
use lumosai_core::prelude::*;
use lumosai_core::memory::{create_working_memory, WorkingMemoryConfig};
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建内存
    let memory = create_working_memory(WorkingMemoryConfig {
        max_messages: 10,
        max_tokens: 4000,
    });
    
    // 2. 创建带内存的 Agent
    let llm = create_test_zhipu_provider_arc();
    let agent = Agent::builder()
        .name("assistant")
        .instructions("You are a helpful assistant with memory")
        .model(llm)
        .memory(memory)
        .build()?;
    
    // 3. 多轮对话
    let response1 = agent.generate("My name is Alice").await?;
    println!("Agent: {}", response1);
    
    let response2 = agent.generate("What's my name?").await?;
    println!("Agent: {}", response2);  // 应该记得名字是 Alice
    
    Ok(())
}
```

---

### 示例 5: 简单的 Workflow（5 分钟）

```rust
use lumosai_core::prelude::*;
use lumosai_core::workflow::{DagWorkflow, DagWorkflowBuilder};
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let llm = create_test_zhipu_provider_arc();
    
    // 1. 创建 Agents
    let planner = Agent::builder()
        .name("planner")
        .instructions("Create a plan")
        .model(llm.clone())
        .build()?;
    
    let executor = Agent::builder()
        .name("executor")
        .instructions("Execute the plan")
        .model(llm.clone())
        .build()?;
    
    let reviewer = Agent::builder()
        .name("reviewer")
        .instructions("Review the execution")
        .model(llm)
        .build()?;
    
    // 2. 创建 DAG Workflow
    let mut workflow = DagWorkflowBuilder::new("task_workflow")
        .description("A simple task workflow")
        .build();
    
    // 3. 添加节点和依赖
    workflow.add_agent_node("plan", planner)?;
    workflow.add_agent_node("execute", executor)?;
    workflow.add_agent_node("review", reviewer)?;
    
    workflow.add_dependency("execute", "plan")?;  // execute 依赖 plan
    workflow.add_dependency("review", "execute")?;  // review 依赖 execute
    
    // 4. 执行 Workflow
    let result = workflow.execute("Complete a project").await?;
    println!("Workflow result: {:?}", result);
    
    Ok(())
}
```

---

## 📚 下一步

恭喜！你已经掌握了 LumosAI 的核心功能。接下来你可以：

1. **探索更多示例**: 查看 `lumosai_examples/examples/` 目录
2. **阅读 API 文档**: 运行 `cargo doc --open`
3. **学习高级功能**: 
   - RAG 系统（文档检索）
   - 资源池优化（连接池、对象池）
   - 缓存机制（多层缓存）
   - 性能监控（资源监控）

4. **加入社区**: 
   - GitHub: https://github.com/louloulin/lumos.ai
   - 文档: https://docs.rs/lumosai

---

## 🐛 常见问题

### Q: 如何使用真实的 LLM 提供商？

A: 替换测试提供商为真实提供商：

```rust
// OpenAI
use lumosai_core::llm::OpenAIProvider;
let llm = OpenAIProvider::from_env()?;

// Zhipu AI
use lumosai_core::llm::ZhipuProvider;
let llm = ZhipuProvider::from_env()?;

// Qwen
use lumosai_core::llm::QwenProvider;
let llm = QwenProvider::from_env()?;
```

### Q: 如何处理错误？

A: LumosAI 使用 `Result` 类型处理错误：

```rust
match agent.generate("Hello").await {
    Ok(response) => println!("Success: {}", response),
    Err(e) => eprintln!("Error: {}", e),
}
```

### Q: 如何调试？

A: 启用日志：

```bash
RUST_LOG=debug cargo run
```

---

## 🎉 总结

你已经学会了：
- ✅ 创建基本的 Agent
- ✅ 使用工具扩展 Agent 能力
- ✅ 多 Agent 协作
- ✅ 使用 Memory 实现上下文记忆
- ✅ 使用 Workflow 编排复杂任务

现在你可以开始构建自己的 AI 应用了！🚀

