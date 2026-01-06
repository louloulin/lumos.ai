# 🤖 Agent基础概念

**30分钟掌握LumosAI Agent的核心概念和创建方法**

## 🎯 学习目标

完成本教程后，你将能够：
- ✅ 理解Agent的概念和核心组件
- ✅ 创建和配置你的第一个Agent
- ✅ 掌握Agent的基本交互方法
- ✅ 了解不同类型的应用场景

## ⏱️ 预计时间: 30分钟
## 📋 前置要求: Rust基础语法

---

## 🧠 什么是Agent？

### Agent定义

**Agent**（智能体）是一个能够：
- **理解**: 理解用户的意图和上下文
- **推理**: 基于知识进行逻辑推理
- **行动**: 执行工具调用和生成响应
- **学习**: 从交互中积累经验和知识

### 核心特性

```rust
// LumosAI Agent的核心特性
use lumosai::prelude::*;

struct Agent {
    name: String,           // 🏷️ 身份标识
    model: String,          // 🧠 大脑模型
    system_prompt: String,  // 📋 行为指令
    tools: Vec<Box<dyn Tool>>, // 🛠️ 能力工具
    memory: Box<dyn Memory>,   // 💾 记忆系统
    rag: Option<Box<dyn RAG>>,  // 📚 知识库
}
```

**与传统聊天机器人的区别**:

| 特性 | 聊天机器人 | LumosAI Agent |
|------|-----------|--------------|
| 上下文记忆 | 短期 | 持久化、可配置 |
| 工具能力 | 无 | 丰富扩展 |
| 知识检索 | 无 | RAG集成 |
| 多模态 | 有限 | 支持多种输入 |
| 自定义能力 | 低 | 高度可定制 |

---

## 🚀 创建你的第一个Agent

### 步骤1: 基础设置

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 开始创建第一个Agent");
    
    // 创建简单的Agent
    let agent = lumosai::agent::simple(
        "gpt-3.5-turbo", 
        "你是一个友好的AI助手，用中文回答问题。"
    ).await?;
    
    println!("✅ Agent创建成功: {}", agent.name());
    Ok(())
}
```

### 步骤2: 基础对话

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let agent = lumosai::agent::simple(
        "gpt-3.5-turbo",
        "你是一个专业的技术顾问"
    ).await?;
    
    // 发送消息
    let response = agent.chat("什么是LumosAI框架？").await?;
    println!("🤖 Agent: {}", response);
    
    Ok(())
}
```

**运行示例**:
```bash
# 设置API密钥
export OPENAI_API_KEY="your-key"

# 运行程序
cargo run

# 预期输出
🤖 Agent: LumosAI是一个基于Rust的企业级AI应用开发框架...
```

---

## 🔧 高级Agent配置

### 使用Builder模式

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = Agent::builder()
        .name("技术专家")
        .model("gpt-4")
        .system_prompt("你是一位资深的软件架构师，具有15年的开发经验。")
        .max_tokens(1500)           // 限制回答长度
        .temperature(0.7)           // 控制创造性
        .top_p(0.9)                 // 核采样参数
        .presence_penalty(0.1)      // 鼓励新话题
        .frequency_penalty(0.1)     // 减少重复
        .build()
        .await?;
    
    Ok(())
}
```

### 参数详解

| 参数 | 作用 | 推荐值 | 说明 |
|------|------|--------|------|
| `temperature` | 控制随机性 | 0.7-1.0 | 0=确定性，1.0=高随机性 |
| `max_tokens` | 最大输出长度 | 1500-2048 | 控制回答长度 |
| `top_p` | 核采样 | 0.9-1.0 | 限制低概率词汇 |
| `presence_penalty` | 存在惩罚 | 0.1-0.5 | 鼓励新话题 |
| `frequency_penalty` | 频率惩罚 | 0.1-0.5 | 减少重复内容 |

---

## 💬 Agent交互模式

### 1. 简单对话

```rust
let response = agent.chat("你好，请介绍一下自己").await?;
println!("Agent: {}", response);
```

### 2. 上下文对话

```rust
let context = "用户是Python开发者，想学习Rust";
let response = agent.chat_with_context(
    "我应该如何开始学习Rust？",
    context
).await?;
```

### 3. 流式对话（实时响应）

```rust
use futures::StreamExt;

let mut stream = agent.stream_chat("写一首关于编程的诗").await?;
print!("🤖 Agent: ");
while let Some(chunk) = stream.next().await {
    print!("{}", chunk?);
    std::io::stdout().flush().unwrap();
}
println!();
```

### 4. 批量对话（提高效率）

```rust
let questions = vec![
    "什么是Rust?",
    "Rust有什么优势?",
    "如何学习Rust?"
];

for question in questions {
    let response = agent.chat(question).await?;
    println!("问: {}\n答: {}\n", question, response);
}
```

---

## 🎭 Agent角色设计

### 定义专业的Agent角色

```rust
// 技术顾问Agent
let tech_advisor = Agent::builder()
    .name("技术顾问")
    .system_prompt(r#"
你是一位资深的技术顾问，具有以下特点：
1. 专业：对各种技术有深入理解
2. 客观：基于事实提供建议
3. 实用：给出可执行的建议
4. 简洁：用简单语言解释复杂概念
    "#)
    .build()
    .await?;

// 创意助手Agent
let creative_assistant = Agent::builder()
    .name("创意助手")
    .system_prompt(r#"
你是一位富有创造力的助手：
1. 思维开放：接受各种创意想法
2. 联想丰富：能够从不同角度思考
3. 鼓励创新：帮助用户突破思维定势
4. 注重细节：将创意具体化
    "#)
    .build()
    .await?;
```

### 实际应用示例

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 创建不同角色的Agent
    let tech_agent = Agent::builder()
        .name("技术专家")
        .system_prompt("你是技术专家，提供专业技术咨询")
        .temperature(0.3)  // 更确定性
        .build()
        .await?;
        
    let creative_agent = Agent::builder()
        .name("创意专家")
        .system_prompt("你是创意专家，提供创新想法")
        .temperature(0.9)  // 更有创造性
        .build()
        .await?;
    
    // 同一个问题，不同角色的回答
    let question = "如何设计一个更好的用户界面？";
    
    println!("🔧 技术专家的回答:");
    let tech_answer = tech_agent.chat(question).await?;
    println!("{}", tech_answer);
    
    println!("\n💡 创意专家的回答:");
    let creative_answer = creative_agent.chat(question).await?;
    println!("{}", creative_answer);
    
    Ok(())
}
```

---

## 🔍 Agent状态管理

### 获取Agent信息

```rust
// 获取Agent基本信息
println!("Agent名称: {}", agent.name());
println!("Agent模型: {}", agent.model());
println!("Agent提示词: {}", agent.system_prompt());

// 获取对话历史
let history = agent.get_conversation_history();
println!("对话轮数: {}", history.len());

for (i, message) in history.iter().enumerate() {
    println!("{}. {}: {}", i+1, message.role, message.content);
}
```

### 内存管理

```rust
// 清空对话历史
agent.clear_memory().await?;
println!("对话历史已清空");

// 设置内存限制
let limited_agent = Agent::builder()
    .name("有限记忆Agent")
    .memory(ConversationMemory::with_max_history(5))
    .build()
    .await?;
```

---

## 🛠️ 错误处理

### 完整的错误处理

```rust
use lumosai::error::LumosaiError;

async fn safe_chat(agent: &Agent, message: &str) -> Result<String> {
    match agent.chat(message).await {
        Ok(response) => {
            println!("✅ 成功获得回答");
            Ok(response)
        }
        Err(LumosaiError::APIError(msg)) => {
            eprintln!("❌ API错误: {}", msg);
            Ok("抱歉，API调用出现问题，请稍后重试".to_string())
        }
        Err(LumosaiError::NetworkError(err)) => {
            eprintln!("❌ 网络错误: {}", err);
            Ok("网络连接出现问题，请检查网络设置".to_string())
        }
        Err(LumosaiError::ValidationError(msg)) => {
            eprintln!("❌ 参数错误: {}", msg);
            Ok("输入参数有误，请检查输入内容".to_string())
        }
        Err(err) => {
            eprintln!("❌ 未知错误: {}", err);
            Ok("系统出现未知错误，请联系技术支持".to_string())
        }
    }
}
```

### 重试机制

```rust
use tokio::time::{sleep, Duration};

async fn chat_with_retry(agent: &Agent, message: &str, max_retries: u32) -> Result<String> {
    for attempt in 1..=max_retries {
        match agent.chat(message).await {
            Ok(response) => return Ok(response),
            Err(_) if attempt < max_retries => {
                println!("⏳ 第{}次尝试失败，等待重试...", attempt);
                sleep(Duration::from_secs(2_u64.pow(attempt))).await;
            }
            Err(e) => return Err(e),
        }
    }
    Err(LumosaiError::APIError("重试失败".to_string()))
}
```

---

## 🎯 实战练习

### 练习1: 创建客服Agent

```rust
// 🎯 任务：创建一个电商客服Agent
#[tokio::main]
async fn main() -> Result<()> {
    let agent = Agent::builder()
        .name("电商客服")
        .system_prompt(r#"
你是一个专业的电商客服助手：
1. 友好热情，始终以客户为中心
2. 了解产品信息和常见问题
3. 能够处理订单查询、退换货等业务
4. 在无法解决问题时，引导客户联系人工客服
        "#)
        .temperature(0.8)  // 友善且专业
        .build()
        .await?;
    
    // 测试对话
    let test_questions = vec![
        "我想查询订单状态",
        "产品有什么质量保证？",
        "如何办理退换货？",
        "你们的工作时间是什么时候？"
    ];
    
    for question in test_questions {
        println!("👤 客户: {}", question);
        let response = agent.chat(question).await?;
        println!("🤖 客服: {}\n", response);
    }
    
    Ok(())
}
```

### 练习2: 创建学习助手Agent

```rust
// 🎯 任务：创建一个个性化学习助手
#[tokio::main]
async fn main() -> Result<()> {
    let agent = Agent::builder()
        .name("学习助手")
        .system_prompt(r#"
你是一个个性化的学习助手：
1. 了解用户的学习目标和进度
2. 提供个性化的学习建议
3. 能够解释复杂概念
4. 提供练习题目和反馈
        "#)
        .memory(ConversationMemory::with_max_history(10))  // 记住学习历史
        .build()
        .await?;
    
    // 模拟学习对话
    let learning_conversation = vec![
        "我想学习编程，应该从哪里开始？",
        "Python和Rust有什么区别？",
        "你能给我推荐一些学习资源吗？",
        "我应该如何练习编程？"
    ];
    
    for (i, question) in learning_conversation.iter().enumerate() {
        println!("📚 第{}轮学习", i+1);
        println!("👤 学员: {}", question);
        
        let response = agent.chat(question).await?;
        println!("🤖 助手: {}\n", response);
    }
    
    Ok(())
}
```

---

## 📊 性能监控

### 响应时间监控

```rust
use std::time::Instant;

async fn timed_chat(agent: &Agent, message: &str) -> Result<String> {
    let start = Instant::now();
    
    let response = agent.chat(message).await?;
    
    let duration = start.elapsed();
    println!("⏱️ 响应时间: {:?}", duration);
    
    Ok(response)
}
```

### 使用量统计

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// 简单的使用统计
struct UsageStats {
    total_requests: u32,
    total_tokens: u32,
    average_response_time: f64,
}

impl UsageStats {
    fn new() -> Self {
        Self {
            total_requests: 0,
            total_tokens: 0,
            average_response_time: 0.0,
        }
    }
    
    fn record_request(&mut self, response_time: f64, tokens: u32) {
        self.total_requests += 1;
        self.total_tokens += tokens;
        self.average_response_time = 
            (self.average_response_time * (self.total_requests - 1) as f64 + response_time) 
            / self.total_requests as f64;
    }
}
```

---

## ✅ 学习检查

### 关键概念回顾

1. **Agent核心组件**:
   - 🏷️ 身份和角色定义
   - 🧠 大语言模型选择
   - 🛠️ 工具和扩展能力
   - 💾 记忆和上下文管理

2. **Agent创建方法**:
   - `lumosai::agent::simple()` - 快速创建
   - `Agent::builder()` - 高级配置

3. **交互模式**:
   - 简单对话、上下文对话、流式对话、批量处理

### 实践验证

完成以下任务来验证你的学习成果：

- [ ] 创建一个具有特定角色的Agent
- [ ] 实现不同参数配置的对比
- [ ] 添加完整的错误处理
- [ ] 实现基本的使用统计

### 下一步学习

掌握了Agent基础后，你可以继续学习：
- [RAG系统入门](./rag-basics.md) - 添加知识检索能力
- [工具集成详解](./tool-integration.md) - 扩展Agent功能
- [内存系统使用](./memory-systems.md) - 管理对话上下文

---

**🎉 恭喜！你已经掌握了LumosAI Agent的基础概念！**

*[← 返回教程导航](README.md)*