# 📚 LumosAI 用户指南

> **完整的 LumosAI 使用手册，从基础到高级**

---

## 目录

1. [简介](#1-简介)
2. [核心概念](#2-核心概念)
3. [Agent 系统](#3-agent-系统)
4. [工具系统 (Tools)](#4-工具系统-tools)
5. [记忆系统 (Memory)](#5-记忆系统-memory)
6. [工作流 (Workflow)](#6-工作流-workflow)
7. [RAG 系统](#7-rag-系统)
8. [LLM 提供商](#8-llm-提供商)
9. [Multi-Agent 协作](#9-multi-agent-协作)
10. [高级特性](#10-高级特性)
11. [性能优化](#11-性能优化)
12. [错误处理](#12-错误处理)
13. [测试和调试](#13-测试和调试)

---

## 1. 简介

### 1.1 什么是 LumosAI？

LumosAI 是一个用 Rust 编写的企业级 AI 应用框架，提供以下核心功能：

- **Agent 系统**: 创建智能 AI 代理
- **工具集成**: 让 AI 能够调用外部工具和 API
- **记忆管理**: 支持短期和长期记忆
- **工作流编排**: 多步骤任务自动化
- **RAG**: 检索增强生成，基于知识库的问答
- **Multi-Agent**: 多个 Agent 协同工作

### 1.2 设计理念

LumosAI 的设计遵循以下原则：

- **类型安全**: 充分利用 Rust 的类型系统保证安全性
- **异步优先**: 基于 Tokio 的高性能异步运行时
- **模块化**: 可插拔的组件设计
- **渐进式 API**: Level 1 (简单) → Level 2 (标准) → Level 3 (高级)
- **生产就绪**: 企业级功能（认证、审计、监控）

### 1.3 适用场景

LumosAI 适合以下应用场景：

- 智能客服和对话系统
- 知识库问答系统
- 自动化工作流
- 数据分析和处理
- 内容生成和编辑
- 多模态应用（文本、图像、语音）

---

## 2. 核心概念

### 2.1 Agent

Agent 是 LumosAI 的核心抽象，代表一个具有特定能力的 AI 实体。

**关键特性**:
- 系统提示词 (Instructions)
- LLM 模型配置
- 工具集合
- 记忆系统
- 配置参数（温度、最大 tokens 等）

**生命周期**:
```
创建 → 配置 → 执行 → 响应 → 清理
```

### 2.2 Tool

Tool 是 Agent 可以调用的外部功能，如 API、数据库查询、计算等。

**工具类型**:
- **同步工具**: 简单的计算和转换
- **异步工具**: API 调用、I/O 操作
- **有状态工具**: 需要保持状态的工具

### 2.3 Memory

Memory 提供 Agent 的记忆能力，使其能够记住历史对话和上下文。

**记忆类型**:
- **WorkingMemory**: 短期工作记忆（对话历史）
- **SemanticMemory**: 语义记忆（向量检索）
- **BasicMemory**: 简单键值对存储
- **UnifiedMemory**: 统一的记忆接口

### 2.4 Workflow

Workflow 用于编排多步骤的复杂任务。

**执行模式**:
- **顺序执行**: 步骤按顺序依次执行
- **并行执行**: 多个步骤同时执行
- **DAG 执行**: 基于依赖关系的智能调度

### 2.5 LLM Provider

LLM Provider 是与大语言模型的接口抽象。

**支持的提供商**:
- OpenAI (GPT-3.5, GPT-4)
- Anthropic (Claude)
- Qwen (通义千问)
- Zhipu AI (智谱 GLM)
- DeepSeek
- Ollama (本地模型)
- 以及更多...

---

## 3. Agent 系统

### 3.1 创建 Agent

#### 3.1.1 使用 AgentBuilder（推荐）

```rust
use lumosai_core::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let llm = create_test_zhipu_provider_arc();
    
    let agent = AgentBuilder::new()
        .name("assistant")
        .instructions("You are a helpful AI assistant")
        .model(llm)
        .temperature(0.7)
        .max_tokens(2000)
        .build()?;
    
    Ok(())
}
```

#### 3.1.2 配置选项

**必需配置**:
- `name`: Agent 名称
- `instructions`: 系统提示词
- `model`: LLM 提供商

**可选配置**:
- `temperature`: 生成温度 (0.0 - 2.0)
- `max_tokens`: 最大生成 tokens
- `tools`: 工具列表
- `memory`: 记忆配置
- `max_tool_calls`: 最大工具调用次数

### 3.2 使用 Agent

#### 3.2.1 简单对话

```rust
// 发送单条消息
let response = agent.generate_simple("Hello!").await?;
println!("Response: {}", response);
```

#### 3.2.2 流式响应

```rust
use futures::StreamExt;

let mut stream = agent.generate_stream("Tell me a story").await?;

while let Some(chunk) = stream.next().await {
    match chunk {
        Ok(text) => print!("{}", text),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

#### 3.2.3 带选项的生成

```rust
use lumosai_core::agent::types::AgentGenerateOptions;

let options = AgentGenerateOptions {
    temperature: Some(0.5),
    max_tokens: Some(1000),
    ..Default::default()
};

let response = agent.generate_with_options("Question", options).await?;
```

### 3.3 Agent 配置

#### 3.3.1 动态配置

```rust
use lumosai_core::agent::dynamic_config::DynamicArgument;

// 基于运行时上下文动态调整温度
let temperature = DynamicArgument::Dynamic(Box::new(|context| {
    Box::pin(async move {
        let temp = if context.complexity == ComplexityLevel::Expert {
            0.3 // 专家任务需要更精确
        } else {
            0.7 // 普通任务更有创造性
        };
        Ok(temp)
    })
}));

let agent = AgentBuilder::new()
    .name("dynamic_agent")
    .instructions("You are an adaptive assistant")
    .model(llm)
    .dynamic_temperature(temperature)
    .build()?;
```

#### 3.3.2 Agent 模板

```rust
// 创建可复用的 Agent 模板
fn create_customer_service_agent(llm: Arc<dyn LlmProvider>) -> Result<BasicAgent> {
    AgentBuilder::new()
        .name("customer_service")
        .instructions(
            "You are a customer service representative. \
             Be polite, helpful, and professional."
        )
        .model(llm)
        .temperature(0.5)
        .max_tokens(1500)
        .build()
}

// 使用模板
let agent1 = create_customer_service_agent(llm.clone())?;
let agent2 = create_customer_service_agent(llm.clone())?;
```

---

## 4. 工具系统 (Tools)

### 4.1 创建工具

#### 4.1.1 使用宏定义工具（推荐）

```rust
use lumosai_core::tool;
use serde_json::json;

/// 计算器工具 - 执行基本的数学运算
///
/// 参数:
/// - operation: 运算类型 (add, subtract, multiply, divide)
/// - a: 第一个数字
/// - b: 第二个数字
#[tool(name = "calculator", description = "执行数学计算")]
async fn calculator(operation: String, a: f64, b: f64) -> Result<Value> {
    let result = match operation.as_str() {
        "add" => a + b,
        "subtract" => a - b,
        "multiply" => a * b,
        "divide" => {
            if b == 0.0 {
                return Err(Error::invalid_input("除数不能为零"));
            }
            a / b
        }
        _ => return Err(Error::invalid_input("不支持的运算")),
    };
    
    Ok(json!({ "result": result }))
}
```

#### 4.1.2 手动创建工具

```rust
use lumosai_core::tool::{ToolBuilder, Tool};

let weather_tool = ToolBuilder::new()
    .name("get_weather")
    .description("获取指定城市的天气信息")
    .parameter("city", "城市名称", true)
    .execute(|args| async move {
        let city = args["city"].as_str().unwrap_or("北京");
        // 调用天气 API
        Ok(json!({
            "city": city,
            "temperature": 25,
            "condition": "晴朗"
        }))
    })
    .build()?;
```

### 4.2 注册工具到 Agent

```rust
let agent = AgentBuilder::new()
    .name("tool_agent")
    .instructions("You can use tools to help users")
    .model(llm)
    .tool(calculator)
    .tool(weather_tool)
    .build()?;
```

### 4.3 高级工具特性

#### 4.3.1 工具验证

```rust
#[tool(name = "email_sender", description = "发送邮件")]
async fn send_email(to: String, subject: String, body: String) -> Result<Value> {
    // 验证邮箱格式
    if !to.contains('@') {
        return Err(Error::invalid_input("无效的邮箱地址"));
    }
    
    // 发送邮件逻辑...
    Ok(json!({ "status": "sent" }))
}
```

#### 4.3.2 工具超时和重试

```rust
use lumosai_core::tool::ToolExecutionOptions;

let options = ToolExecutionOptions {
    timeout: Some(Duration::from_secs(5)),
    max_retries: 3,
    retry_delay: Duration::from_millis(1000),
};

let result = tool.execute_with_options(args, options).await?;
```

---

## 5. 记忆系统 (Memory)

### 5.1 WorkingMemory（工作记忆）

WorkingMemory 用于保存对话历史。

```rust
use lumosai_core::memory::{create_working_memory, WorkingMemoryConfig};

let memory = create_working_memory(WorkingMemoryConfig {
    max_messages: 10,      // 最多保存 10 条消息
    max_tokens: 4000,      // 最多 4000 tokens
});

// 创建带记忆的 Agent
let agent = AgentBuilder::new()
    .name("memory_agent")
    .instructions("You remember our conversation")
    .model(llm)
    .working_memory(memory)
    .build()?;

// 多轮对话
agent.generate_simple("My name is Alice").await?;
agent.generate_simple("What's my name?").await?;  // 应该记得名字
```

### 5.2 SemanticMemory（语义记忆）

SemanticMemory 基于向量检索，用于存储和检索语义相关的信息。

```rust
use lumosai_core::memory::{SemanticMemory, MemoryConfig};

let semantic_memory = SemanticMemory::new(MemoryConfig {
    vector_store: "memory".to_string(),
    embedding_model: "text-embedding-ada-002".to_string(),
    similarity_threshold: 0.7,
    max_results: 5,
})?;

// 存储记忆
semantic_memory.add("用户喜欢吃披萨").await?;
semantic_memory.add("用户住在北京").await?;

// 检索相关记忆
let memories = semantic_memory.search("用户的饮食偏好", 3).await?;
```

### 5.3 UnifiedMemory（统一记忆）

结合多种记忆类型的统一接口。

```rust
use lumosai_core::memory::{UnifiedMemory, UnifiedMemoryConfig};

let unified_memory = UnifiedMemory::new(UnifiedMemoryConfig {
    enable_working: true,
    enable_semantic: true,
    enable_persistent: true,
    ..Default::default()
})?;

let agent = AgentBuilder::new()
    .name("unified_memory_agent")
    .instructions("You have perfect memory")
    .model(llm)
    .unified_memory(unified_memory)
    .build()?;
```

---

## 6. 工作流 (Workflow)

### 6.1 基础工作流

#### 6.1.1 顺序执行

```rust
use lumosai_core::workflow::{DagWorkflowBuilder, WorkflowStep};

let workflow = DagWorkflowBuilder::new("sequential".to_string())
    .description("顺序执行工作流".to_string())
    .build();

// 添加步骤
workflow.add_root_node(
    "step1".to_string(),
    "第一步".to_string(),
    WorkflowStep::new("step1".to_string(), "处理数据".to_string()),
).await?;

workflow.add_node(
    "step2".to_string(),
    "第二步".to_string(),
    vec!["step1".to_string()],  // 依赖 step1
    WorkflowStep::new("step2".to_string(), "分析结果".to_string()),
).await?;

// 执行工作流
let context = RuntimeContext::new();
let result = workflow.execute(json!({"input": "data"}), &context).await?;
```

#### 6.1.2 并行执行

```rust
// 创建并行步骤
workflow.add_root_node("step_a", "并行任务 A", step_a).await?;
workflow.add_root_node("step_b", "并行任务 B", step_b).await?;
workflow.add_root_node("step_c", "并行任务 C", step_c).await?;

// step_a, step_b, step_c 将并行执行

// 添加汇总步骤
workflow.add_node(
    "summary",
    "汇总结果",
    vec!["step_a".to_string(), "step_b".to_string(), "step_c".to_string()],
    summary_step,
).await?;
```

### 6.2 DAG 工作流

DAG (Directed Acyclic Graph) 工作流支持复杂的依赖关系。

```rust
use lumosai_core::workflow::{DagWorkflowBuilder, DagWorkflow};

let workflow = DagWorkflowBuilder::new("complex_dag".to_string())
    .description("复杂 DAG 工作流".to_string())
    .max_concurrency(4)  // 最多 4 个步骤并行
    .build();

// 构建 DAG
//     A
//    / \
//   B   C
//    \ /
//     D

workflow.add_root_node("A", "开始", step_a).await?;
workflow.add_node("B", "分支 B", vec!["A"], step_b).await?;
workflow.add_node("C", "分支 C", vec!["A"], step_c).await?;
workflow.add_node("D", "汇总", vec!["B", "C"], step_d).await?;

let result = workflow.execute(json!({}), &context).await?;
```

### 6.3 工作流状态管理

```rust
use lumosai_core::workflow::{Workflow, WorkflowStatus};

// 执行工作流
let run_id = "run-123";
workflow.execute_with_id(run_id, input, &context).await?;

// 查询状态
let status = workflow.get_status(run_id).await?;
match status {
    WorkflowStatus::Running => println!("运行中"),
    WorkflowStatus::Completed(result) => println!("完成: {:?}", result),
    WorkflowStatus::Failed(error) => println!("失败: {}", error),
    WorkflowStatus::Suspended => println!("暂停"),
    WorkflowStatus::NotFound => println!("未找到"),
}

// 暂停工作流
workflow.suspend(run_id).await?;

// 恢复工作流
workflow.resume(run_id, None).await?;
```

---

## 7. RAG 系统

### 7.1 基础 RAG

```rust
use lumosai_core::rag::{RagPipeline, RagConfig};

// 创建 RAG 系统
let rag = RagPipeline::builder()
    .embedding_provider("openai")
    .vector_store("memory")
    .chunk_size(500)
    .chunk_overlap(50)
    .build()
    .await?;

// 添加文档
rag.add_document("LumosAI 是一个 Rust AI 框架").await?;
rag.add_document("它支持 Agent、RAG、Workflow 等功能").await?;

// 检索
let results = rag.retrieve("LumosAI 的功能", 3).await?;
for result in results {
    println!("相关性: {:.2}, 内容: {}", result.score, result.content);
}
```

### 7.2 RAG + Agent

```rust
// 创建带 RAG 的 Agent
let agent = AgentBuilder::new()
    .name("rag_agent")
    .instructions("回答问题时使用知识库")
    .model(llm)
    .rag(rag)
    .build()?;

// Agent 会自动使用 RAG 检索相关信息
let response = agent.generate_simple("LumosAI 支持哪些功能？").await?;
```

### 7.3 高级 RAG 特性

#### 7.3.1 混合检索

```rust
use lumosai_core::rag::{HybridRetriever, RetrieverConfig};

let retriever = HybridRetriever::new(RetrieverConfig {
    vector_weight: 0.7,    // 向量检索权重
    keyword_weight: 0.3,   // 关键词检索权重
    enable_rerank: true,   // 启用重排序
    ..Default::default()
})?;
```

#### 7.3.2 文档分块策略

```rust
use lumosai_core::rag::chunking::{ChunkingStrategy, RecursiveCharacterSplitter};

// 递归字符分割器
let splitter = RecursiveCharacterSplitter::new(
    500,   // chunk_size
    50,    // chunk_overlap
    vec!["\n\n", "\n", " ", ""],  // 分割符优先级
);

let chunks = splitter.split_text("长文本...")?;
```

---

## 8. LLM 提供商

### 8.1 配置 LLM 提供商

#### 8.1.1 OpenAI

```rust
use lumosai_core::llm::{openai, LlmProvider};

let llm = openai("gpt-4")?
    .temperature(0.7)
    .max_tokens(2000)
    .build()?;
```

#### 8.1.2 Anthropic (Claude)

```rust
use lumosai_core::llm::anthropic;

let llm = anthropic("claude-3-sonnet")?
    .temperature(0.5)
    .max_tokens(4000)
    .build()?;
```

#### 8.1.3 Qwen（通义千问）

```rust
use lumosai_core::llm::qwen;

let llm = qwen("qwen-turbo")?
    .api_key(&std::env::var("DASHSCOPE_API_KEY")?)
    .build()?;
```

### 8.2 切换 LLM 提供商

```rust
// 创建多个提供商
let openai_llm = openai("gpt-4")?.build()?;
let claude_llm = anthropic("claude-3")?.build()?;

// 根据需求选择
let llm = if task_needs_reasoning {
    Arc::new(openai_llm) as Arc<dyn LlmProvider>
} else {
    Arc::new(claude_llm) as Arc<dyn LlmProvider>
};
```

### 8.3 自定义 LLM 提供商

```rust
use lumosai_core::llm::{LlmProvider, GenerateRequest, GenerateResponse};
use async_trait::async_trait;

struct MyLlmProvider {
    api_key: String,
    model: String,
}

#[async_trait]
impl LlmProvider for MyLlmProvider {
    async fn generate(&self, request: &GenerateRequest) -> Result<GenerateResponse> {
        // 实现你的 LLM 调用逻辑
        todo!()
    }
    
    // 实现其他必需方法...
}
```

---

## 9. Multi-Agent 协作

### 9.1 Agent 流水线

```rust
use lumosai_core::agent::{pipe, AgentPipeline};

// 创建多个专业化的 Agent
let researcher = AgentBuilder::new()
    .name("researcher")
    .instructions("You research and gather information")
    .model(llm.clone())
    .build()?;

let writer = AgentBuilder::new()
    .name("writer")
    .instructions("You write engaging content")
    .model(llm.clone())
    .build()?;

let editor = AgentBuilder::new()
    .name("editor")
    .instructions("You polish and improve content")
    .model(llm)
    .build()?;

// 创建流水线
let pipeline: AgentPipeline = pipe![researcher, writer, editor];

// 执行流水线
let result = pipeline.execute("Write an article about AI").await?;
```

### 9.2 Agent 团队协作

```rust
use lumosai_core::agent::collaboration::{AgentTeam, TeamConfig};

let team = AgentTeam::new(TeamConfig {
    name: "content_team".to_string(),
    strategy: CollaborationStrategy::Hierarchical,  // 层级协作
    max_iterations: 5,
});

// 添加团队成员
team.add_member("researcher", researcher).await?;
team.add_member("writer", writer).await?;
team.add_member("editor", editor).await?;

// 执行团队任务
let result = team.execute_task("创建一篇技术博客").await?;
```

### 9.3 Agent 共识机制

```rust
use lumosai_core::agent::collaboration::ConsensusMode;

// 创建需要共识的团队
let team = AgentTeam::new(TeamConfig {
    name: "review_team".to_string(),
    strategy: CollaborationStrategy::Consensus(ConsensusMode::Majority),
    ..Default::default()
});

// 多个 Agent 投票达成共识
let decision = team.reach_consensus("Is this content appropriate?").await?;
```

---

## 10. 高级特性

### 10.1 流式响应

```rust
use futures::StreamExt;

// 流式生成
let mut stream = agent.generate_stream("Tell me a long story").await?;

while let Some(chunk) = stream.next().await {
    match chunk {
        Ok(text) => {
            print!("{}", text);
            std::io::Write::flush(&mut std::io::stdout())?;
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### 10.2 函数调用

```rust
// LLM 会自动识别何时调用工具
let agent = AgentBuilder::new()
    .name("function_agent")
    .instructions("Use functions when appropriate")
    .model(llm)
    .tool(calculator)
    .tool(weather_tool)
    .enable_function_calling(true)
    .build()?;

// Agent 会自动调用合适的工具
let response = agent.generate_simple("What's 123 * 456?").await?;
```

### 10.3 并发控制

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

// 限制并发请求数
let semaphore = Arc::new(Semaphore::new(5));

let mut handles = vec![];
for i in 0..100 {
    let agent = agent.clone();
    let semaphore = semaphore.clone();
    
    let handle = tokio::spawn(async move {
        let _permit = semaphore.acquire().await.unwrap();
        agent.generate_simple(&format!("Query {}", i)).await
    });
    
    handles.push(handle);
}

// 等待所有任务完成
let results = futures::future::join_all(handles).await;
```

---

## 11. 性能优化

### 11.1 缓存

```rust
use lumosai_core::cache::{LruCache, CacheConfig};

// 启用响应缓存
let cache = LruCache::new(CacheConfig {
    max_size: 1000,
    ttl: Duration::from_secs(3600),
});

let agent = AgentBuilder::new()
    .name("cached_agent")
    .instructions("You can cache responses")
    .model(llm)
    .cache(cache)
    .build()?;

// 相同的请求会从缓存返回
let response1 = agent.generate_simple("What is AI?").await?;
let response2 = agent.generate_simple("What is AI?").await?;  // 从缓存返回
```

### 11.2 批处理

```rust
// 批量处理请求
let queries = vec![
    "Query 1",
    "Query 2",
    "Query 3",
];

let results = agent.batch_generate(queries).await?;
```

### 11.3 连接池

```rust
use lumosai_core::pool::{ConnectionPool, PoolConfig};

// 创建 LLM 连接池
let pool = ConnectionPool::new(PoolConfig {
    max_size: 10,
    min_size: 2,
    idle_timeout: Duration::from_secs(60),
})?;
```

---

## 12. 错误处理

### 12.1 错误类型

```rust
use lumosai_core::error::Error;

match agent.generate_simple("Hello").await {
    Ok(response) => println!("{}", response),
    Err(Error::LlmError(e)) => eprintln!("LLM 错误: {}", e),
    Err(Error::NetworkError(e)) => eprintln!("网络错误: {}", e),
    Err(Error::InvalidInput(e)) => eprintln!("输入错误: {}", e),
    Err(e) => eprintln!("其他错误: {}", e),
}
```

### 12.2 重试机制

```rust
use lumosai_core::retry::{RetryPolicy, ExponentialBackoff};

let retry_policy = RetryPolicy {
    max_retries: 3,
    backoff: ExponentialBackoff {
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(5),
        multiplier: 2.0,
    },
};

let agent = AgentBuilder::new()
    .name("resilient_agent")
    .instructions("I retry on failure")
    .model(llm)
    .retry_policy(retry_policy)
    .build()?;
```

### 12.3 降级策略

```rust
// 主提供商失败时降级到备用提供商
let primary_llm = openai("gpt-4")?.build()?;
let fallback_llm = openai("gpt-3.5-turbo")?.build()?;

let response = match agent_with_primary.generate_simple("Hello").await {
    Ok(res) => res,
    Err(_) => {
        // 降级到备用提供商
        let fallback_agent = AgentBuilder::new()
            .name("fallback")
            .instructions("Same as primary")
            .model(Arc::new(fallback_llm))
            .build()?;
        fallback_agent.generate_simple("Hello").await?
    }
};
```

---

## 13. 测试和调试

### 13.1 测试 Agent

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use lumosai_core::llm::test_helpers::create_mock_llm;

    #[tokio::test]
    async fn test_agent_creation() {
        let llm = create_mock_llm("Test response");
        
        let agent = AgentBuilder::new()
            .name("test_agent")
            .instructions("Test")
            .model(Arc::new(llm))
            .build();
        
        assert!(agent.is_ok());
    }
    
    #[tokio::test]
    async fn test_agent_response() {
        let llm = create_mock_llm("Hello from test");
        let agent = AgentBuilder::new()
            .name("test")
            .instructions("Test")
            .model(Arc::new(llm))
            .build()
            .unwrap();
        
        let response = agent.generate_simple("Hi").await.unwrap();
        assert!(response.contains("Hello"));
    }
}
```

### 13.2 日志和监控

```rust
use tracing::{info, warn, error};

// 启用结构化日志
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .init();

// 在代码中添加日志
let response = agent.generate_simple("Hello").await?;
info!(agent_name = agent.name(), response_len = response.len(), "Agent responded");
```

### 13.3 性能分析

```rust
use std::time::Instant;

let start = Instant::now();
let response = agent.generate_simple("Query").await?;
let duration = start.elapsed();

println!("请求耗时: {:?}", duration);
```

---

## 总结

LumosAI 提供了一个强大而灵活的 AI 应用开发框架。通过本指南，你应该能够：

✅ 创建和配置 Agent
✅ 使用工具扩展 Agent 能力
✅ 实现记忆系统
✅ 构建复杂的工作流
✅ 集成 RAG 系统
✅ 实现 Multi-Agent 协作
✅ 优化性能
✅ 处理错误
✅ 测试和调试

**下一步**:
- 查看 [API 文档](https://docs.rs/lumosai_core)
- 阅读 [最佳实践指南](BEST_PRACTICES.md)
- 参考 [示例代码](../lumosai_examples/examples/)
- 加入 [社区讨论](https://github.com/your-org/lumosai/discussions)

**需要帮助？**
- 📖 文档: [docs/](.)
- 🐛 问题: [GitHub Issues](https://github.com/your-org/lumosai/issues)
- 💬 讨论: [GitHub Discussions](https://github.com/your-org/lumosai/discussions)

Happy Coding! 🚀

