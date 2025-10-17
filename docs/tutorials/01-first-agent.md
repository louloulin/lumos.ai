# 教程 01: 创建您的第一个 Agent

## 🎯 学习目标

通过本教程，您将学会：
- 理解 Agent 的基本概念和架构
- 创建和配置您的第一个 AI Agent
- 与 Agent 进行基本交互
- 理解 Agent 的生命周期和状态管理

## 📋 前置要求

- 已完成 [安装指南](../quick-start/installation.md)
- 具备 Rust 基础语法知识
- 拥有有效的 OpenAI API 密钥

## 🤖 什么是 Agent？

在 LumosAI 中，Agent 是一个智能实体，它可以：
- 理解和处理自然语言
- 执行特定的任务和指令
- 使用工具和外部资源
- 维护对话历史和上下文
- 做出智能决策和推理

### Agent 的核心组件

```
┌─────────────────────────────────────┐
│              Agent                  │
├─────────────────────────────────────┤
│ • 名称和身份                        │
│ • 系统指令                          │
│ • LLM 模型                          │
│ • 工具集合                          │
│ • 内存系统                          │
│ • 配置参数                          │
└─────────────────────────────────────┘
```

## 🚀 创建第一个 Agent

### 步骤 1: 项目设置

创建新项目：

```bash
cargo new my-first-agent
cd my-first-agent
```

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
lumosai = "0.1.4"
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
```

### 步骤 2: 最简单的 Agent

创建 `src/main.rs`：

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建最简单的 Agent
    let agent = Agent::builder()
        .name("小助手")
        .instructions("你是一个友好的助手，用中文回答问题。")
        .model("gpt-3.5-turbo")
        .build()?;

    println!("Agent 创建成功: {}", agent.name());
    
    // 发送第一条消息
    let response = agent.generate("你好！").await?;
    println!("Agent: {}", response);

    Ok(())
}
```

运行程序：

```bash
export OPENAI_API_KEY="your-api-key"
cargo run
```

**预期输出：**
```
Agent 创建成功: 小助手
Agent: 你好！我是小助手，很高兴为您服务。有什么我可以帮助您的吗？
```

### 步骤 3: 配置 Agent 参数

让我们创建一个更详细的 Agent：

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = Agent::builder()
        .name("专业助手")
        .instructions(
            "你是一个专业的助手，具有以下特点：\
            1. 回答准确、详细\
            2. 语言正式但友好\
            3. 会主动提供相关建议\
            4. 遇到不确定的问题会诚实说明"
        )
        .model("gpt-3.5-turbo")
        .temperature(0.7)  // 控制创造性
        .max_tokens(500)   // 限制回复长度
        .build()?;

    // 测试不同类型的问题
    let questions = vec![
        "什么是人工智能？",
        "如何学习 Rust 编程？",
        "今天天气怎么样？",  // 测试不确定性
    ];

    for question in questions {
        println!("\n用户: {}", question);
        let response = agent.generate(question).await?;
        println!("Agent: {}", response);
        println!("{}", "-".repeat(50));
    }

    Ok(())
}
```

### 步骤 4: 多轮对话

实现持续对话功能：

```rust
use lumosai::prelude::*;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<()> {
    let agent = Agent::builder()
        .name("对话助手")
        .instructions("你是一个善于对话的助手，能够记住对话历史。")
        .model("gpt-3.5-turbo")
        .build()?;

    println!("🤖 对话助手已启动！输入 'quit' 退出。\n");

    loop {
        // 获取用户输入
        print!("用户: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        // 检查退出条件
        if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("退出") {
            println!("👋 再见！");
            break;
        }

        // 生成回复
        match agent.generate(input).await {
            Ok(response) => {
                println!("🤖 Agent: {}\n", response);
            }
            Err(e) => {
                println!("❌ 错误: {}\n", e);
            }
        }
    }

    Ok(())
}
```

### 步骤 5: Agent 状态检查

了解 Agent 的内部状态：

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = Agent::builder()
        .name("状态演示助手")
        .instructions("你是一个演示助手。")
        .model("gpt-3.5-turbo")
        .build()?;

    // 检查 Agent 基本信息
    println!("Agent 信息:");
    println!("  名称: {}", agent.name());
    println!("  模型: {}", agent.model());
    println!("  指令: {}", agent.instructions());

    // 发送消息并检查状态
    println!("\n发送消息...");
    let response = agent.generate("请介绍一下你自己").await?;
    println!("回复: {}", response);

    // 检查消息历史（如果有内存系统）
    if let Some(memory) = agent.memory() {
        let stats = memory.get_stats().await?;
        println!("\n内存统计:");
        println!("  消息数量: {}", stats.message_count);
        println!("  内存类型: {:?}", stats.memory_type);
    }

    Ok(())
}
```

## 🔧 Agent 配置详解

### 核心参数

| 参数 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `name` | String | "Agent" | Agent 的名称标识 |
| `instructions` | String | "" | 系统指令，定义 Agent 行为 |
| `model` | String | "gpt-3.5-turbo" | 使用的 LLM 模型 |
| `temperature` | f32 | 0.7 | 创造性控制 (0.0-2.0) |
| `max_tokens` | u32 | 1000 | 最大回复长度 |

### 高级配置

```rust
let agent = Agent::builder()
    .name("高级助手")
    .instructions("详细的系统指令...")
    .model("gpt-4")
    .temperature(0.8)
    .max_tokens(2000)
    .top_p(0.9)           // 核采样参数
    .frequency_penalty(0.1) // 频率惩罚
    .presence_penalty(0.1)  // 存在惩罚
    .build()?;
```

## 🎯 实践练习

### 练习 1: 专业角色 Agent

创建一个特定角色的 Agent（如医生、律师、教师）：

```rust
let doctor_agent = Agent::builder()
    .name("AI医生助手")
    .instructions(
        "你是一个医学助手，具有丰富的医学知识。\
        重要提醒：\
        1. 你只提供一般性医学信息，不能替代专业医生诊断\
        2. 对于严重症状，建议立即就医\
        3. 回答要准确、负责任"
    )
    .model("gpt-3.5-turbo")
    .temperature(0.3)  // 降低创造性，提高准确性
    .build()?;
```

### 练习 2: 多语言 Agent

创建支持多语言的 Agent：

```rust
let multilingual_agent = Agent::builder()
    .name("多语言助手")
    .instructions(
        "你是一个多语言助手，能够用多种语言交流。\
        请根据用户使用的语言来回复，如果用户用中文提问就用中文回答，\
        用英文提问就用英文回答。"
    )
    .model("gpt-3.5-turbo")
    .build()?;
```

### 练习 3: 情感感知 Agent

创建能够感知和回应情感的 Agent：

```rust
let emotional_agent = Agent::builder()
    .name("情感助手")
    .instructions(
        "你是一个具有情感智能的助手。\
        请注意用户的情感状态，并给出适当的回应：\
        - 如果用户情绪低落，给予安慰和鼓励\
        - 如果用户兴奋，分享他们的喜悦\
        - 如果用户困惑，耐心解释和指导\
        - 始终保持同理心和温暖"
    )
    .model("gpt-3.5-turbo")
    .temperature(0.8)
    .build()?;
```

## ✅ 总结

通过本教程，您学会了：

- ✅ Agent 的基本概念和组成
- ✅ 使用 Builder 模式创建 Agent
- ✅ 配置 Agent 的各种参数
- ✅ 实现单次和多轮对话
- ✅ 检查 Agent 的状态和信息
- ✅ 创建不同类型的专业 Agent

### 关键要点

1. **Builder 模式**: LumosAI 使用 Builder 模式创建 Agent，提供灵活的配置选项
2. **系统指令**: 通过 `instructions` 定义 Agent 的行为和个性
3. **参数调优**: `temperature`、`max_tokens` 等参数影响 Agent 的回复质量
4. **错误处理**: 始终使用 `Result` 类型处理可能的错误

## 🔗 下一步

现在您已经掌握了 Agent 的基础，可以继续学习：

- [教程 02: 工具集成详解](./02-tool-integration.md) - 为 Agent 添加工具能力
- [教程 03: 内存系统深入](./03-memory-systems.md) - 让 Agent 具备记忆能力
- [自定义工具开发](./custom-tools.md) - 开发专属工具

## 🆘 常见问题

**Q: Agent 回复很慢怎么办？**
A: 检查网络连接和 API 密钥，考虑使用更快的模型如 `gpt-3.5-turbo`。

**Q: 如何让 Agent 回复更准确？**
A: 降低 `temperature` 参数，完善 `instructions` 指令。

**Q: Agent 不记住之前的对话？**
A: 需要添加内存系统，参见下一个教程。

---

*恭喜完成第一个教程！继续探索 LumosAI 的强大功能吧！* 🎉
