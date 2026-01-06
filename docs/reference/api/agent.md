# 📋 Agent API

**AI Agent创建、配置和管理接口**

## 🚀 快速创建

### `lumosai::agent::simple()`

最简单的Agent创建方式，适合快速原型开发。

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = lumosai::agent::simple("gpt-4", "You are helpful").await?;
    
    let response = agent.chat("Hello!").await?;
    println!("Agent: {}", response);
    
    Ok(())
}
```

**参数:**
- `model: &str` - 模型名称（支持 "gpt-4", "claude-3", "deepseek-chat" 等）
- `system_prompt: &str` - 系统提示词

**返回:** `Result<Agent>` - Agent实例

**支持的模型:**
- OpenAI: `gpt-4`, `gpt-4-turbo`, `gpt-3.5-turbo`
- Anthropic: `claude-3-opus`, `claude-3-sonnet`, `claude-3-haiku`
- DeepSeek: `deepseek-chat`, `deepseek-coder`
- 本地模型: 通过配置文件指定

---

## 🔧 高级配置

### `Agent::builder()`

使用构建器模式创建功能丰富的Agent。

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
        .tool(my_tool)
        .memory(my_memory)
        .build()
        .await?;
    
    Ok(())
}
```

### 构建器方法详解

| 方法 | 类型 | 描述 | 默认值 | 示例 |
|------|------|------|--------|------|
| `name(name: &str)` | 必需 | Agent名称 | - | `.name("客服助手")` |
| `model(model: &str)` | 必需 | LLM模型 | - | `.model("gpt-4")` |
| `system_prompt(prompt: &str)` | 必需 | 系统提示词 | - | `.system_prompt("你是一个专业的AI助手")` |
| `max_tokens(limit: u32)` | 可选 | 最大生成token数 | 2048 | `.max_tokens(1500)` |
| `temperature(temp: f64)` | 可选 | 生成随机性(0.0-2.0) | 1.0 | `.temperature(0.8)` |
| `top_p(n: f64)` | 可选 | 核采样参数 | 1.0 | `.top_p(0.9)` |
| `presence_penalty(penalty: f64)` | 可选 | 存在惩罚 | 0.0 | `.presence_penalty(0.1)` |
| `frequency_penalty(penalty: f64)` | 可选 | 频率惩罚 | 0.0 | `.frequency_penalty(0.1)` |
| `max_tool_calls(limit: u32)` | 可选 | 最大工具调用次数 | 10 | `.max_tool_calls(5)` |
| `tool(tool: impl Tool)` | 可选 | 添加工具 | 无 | `.tool(calculator_tool)` |
| `memory(memory: impl Memory)` | 可选 | 内存系统 | 无 | `.memory(conversation_memory)` |
| `rag(rag: impl RAG)` | 可选 | RAG系统 | 无 | `.rag(knowledge_base)` |

---

## 💬 Agent方法

### 对话接口

#### `chat(message: &str) -> Result<String>`

与Agent进行单轮对话。

```rust
let response = agent.chat("你好，请介绍一下LumosAI框架。").await?;
println!("Agent回复: {}", response);
```

#### `chat_with_context(message: &str, context: &str) -> Result<String>`

带上下文的对话，适用于特定场景。

```rust
let context = "用户是Python开发者，想了解AI框架选择。";
let response = agent.chat_with_context("推荐什么AI框架？", context).await?;
```

#### `stream_chat(message: &str) -> impl Stream<Item = Result<String>>`

流式对话，实时获取生成内容。

```rust
use futures::StreamExt;

let mut stream = agent.stream_chat("写一首关于编程的诗").await?;
while let Some(chunk) = stream.next().await {
    print!("{}", chunk?);
}
```

### 工具调用

#### `call_tool(tool_name: &str, params: Value) -> Result<Value>`

直接调用Agent的工具。

```rust
use serde_json::json;

let result = agent.call_tool("calculator", json!({
    "operation": "add",
    "a": 10,
    "b": 20
})).await?;

println!("计算结果: {}", result);
```

### 内存管理

#### `get_conversation_history() -> Vec<Message>`

获取对话历史。

```rust
let history = agent.get_conversation_history();
for message in history {
    println!("{}: {}", message.role, message.content);
}
```

#### `clear_memory()`

清空Agent内存。

```rust
agent.clear_memory().await?;
println!("Agent内存已清空");
```

---

## 🛠️ 工具集成

### 添加预定义工具

```rust
use lumosai::prelude::*;
use lumosai::tools::{CalculatorTool, WeatherTool, WebSearchTool};

#[tokio::main]
async fn main() -> Result<()> {
    let agent = Agent::builder()
        .name("全能助手")
        .model("gpt-4")
        .system_prompt("你是一个功能丰富的AI助手")
        .tool(CalculatorTool::new())
        .tool(WeatherTool::new())
        .tool(WebSearchTool::new())
        .build()
        .await?;
    
    // Agent现在可以使用计算器、天气查询、网络搜索等工具
    Ok(())
}
```

### 自定义工具

```rust
use lumosai::prelude::*;
use lumos_macro::tool;

#[tool]
async fn get_server_status(server_id: String) -> Result<String> {
    // 模拟检查服务器状态
    Ok(format!("服务器 {} 运行正常", server_id))
}

#[tokio::main]
async fn main() -> Result<()> {
    let agent = Agent::builder()
        .name("运维助手")
        .tool(GetServerStatusTool::new())
        .build()
        .await?;
    
    let response = agent.chat("检查server-001的状态").await?;
    println!("{}", response);
    
    Ok(())
}
```

---

## 🔧 配置选项

### 环境变量配置

```bash
export OPENAI_API_KEY="your-openai-key"
export ANTHROPIC_API_KEY="your-anthropic-key"
export DEEPSEEK_API_KEY="your-deepseek-key"
```

### 代码配置

```rust
use lumosai::config::{AgentConfig, LLMConfig};

let config = AgentConfig {
    name: "my-agent".to_string(),
    llm: LLMConfig {
        provider: "openai".to_string(),
        model: "gpt-4".to_string(),
        api_key: std::env::var("OPENAI_API_KEY")?,
        temperature: Some(0.7),
        max_tokens: Some(1500),
    },
    tools: vec![],
    memory: None,
};

let agent = Agent::from_config(config).await?;
```

---

## ⚠️ 错误处理

### 常见错误类型

```rust
use lumosai::error::{LumosaiError, Result};

match agent.chat("Hello").await {
    Ok(response) => println!("成功: {}", response),
    Err(LumosaiError::APIError(msg)) => {
        eprintln!("API调用失败: {}", msg);
    },
    Err(LumosaiError::NetworkError(err)) => {
        eprintln!("网络错误: {}", err);
    },
    Err(LumosaiError::ValidationError(msg)) => {
        eprintln!("参数验证失败: {}", msg);
    },
    Err(err) => {
        eprintln!("其他错误: {}", err);
    }
}
```

### 重试机制

```rust
use tokio::time::{sleep, Duration};

async fn retry_chat(agent: &Agent, message: &str, max_retries: u32) -> Result<String> {
    for attempt in 1..=max_retries {
        match agent.chat(message).await {
            Ok(response) => return Ok(response),
            Err(_) if attempt < max_retries => {
                sleep(Duration::from_secs(2 * attempt as u64)).await;
            },
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}
```

---

## 🚀 性能优化

### 并发使用

```rust
use std::sync::Arc;
use tokio::task::JoinSet;

let agent = Arc::new(agent);
let mut tasks = JoinSet::new();

// 并发处理多个请求
for i in 0..10 {
    let agent_clone = Arc::clone(&agent);
    tasks.spawn(async move {
        agent_clone.chat(&format!("消息 {}", i)).await
    });
}

while let Some(result) = tasks.join_next().await {
    match result {
        Ok(Ok(response)) => println!("成功: {}", response),
        Ok(Err(e)) => eprintln!("错误: {}", e),
        Err(e) => eprintln!("任务错误: {}", e),
    }
}
```

### 内存优化

```rust
// 限制对话历史长度
let agent = Agent::builder()
    .name("助手")
    .memory(ConversationMemory::with_max_history(20))
    .build()
    .await?;

// 定期清理内存
if agent.get_conversation_history().len() > 50 {
    agent.clear_memory().await?;
}
```

---

## 🔗 相关API

- **[RAG API](rag.md)** - 检索增强生成
- **[Memory API](memory.md)** - 内存管理系统
- **[Tools API](tools.md)** - 工具系统
- **[LLM API](llm.md)** - 大语言模型接口

---

## 📖 更多示例

- [基础Agent示例](../../learn/examples/agent-basics.md)
- [工具集成示例](../../learn/examples/tool-integration.md)
- [内存管理示例](../../learn/examples/memory-system.md)
- [高级Agent配置示例](../../learn/examples/advanced-agent.md)