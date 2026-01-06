# 🤖 Agent 基础

> 学习创建、配置和使用 AI Agent

## 📋 教程概述

本教程将教您：
- 创建您的第一个 AI Agent
- 配置 Agent 的行为和属性
- 处理对话和错误情况
- 管理对话历史和记忆

**预计时间**: 30分钟  
**难度**: ⭐ (初级)

## 🚀 准备工作

### 环境要求

- Rust 1.70+
- LumosAI 库
- LLM API 密钥

### 项目设置

```bash
# 创建新项目
cargo new agent_basics
cd agent_basics

# 添加依赖
cargo add lumosai tokio serde

# 创建 Cargo.toml 配置
cat > Cargo.toml << 'EOF'
[package]
name = "agent_basics"
version = "0.1.0"
edition = "2021"

[dependencies]
lumosai = "0.2.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }

# 设置环境变量
export OPENAI_API_KEY="your_openai_key"
# 或者使用其他提供商
export DEEPSEEK_API_KEY="your_deepseek_key"
EOF
```

## 第一部分：创建基础 Agent

### 1.1 简单 Agent 创建

让我们从最简单的 Agent 开始：

```rust
// src/main.rs
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🤖 创建第一个 Agent");
    
    // 创建简单的 Agent
    let agent = lumosai::agent::simple(
        "gpt-4",                    // 模型名称
        "你是一个友好的AI助手"          // 系统提示词
    ).await?;
    
    // 进行对话
    let response = agent.chat("你好！请介绍一下你自己。").await?;
    println!("Agent 回复: {}", response);
    
    Ok(())
}
```

**运行程序：**
```bash
cargo run
```

**预期输出：**
```
🤖 创建第一个 Agent
Agent 回复: 你好！我是一个AI助手，可以帮您解答问题、提供信息和协助完成各种任务。
```

### 1.2 理解核心概念

#### Agent 的基本结构

```rust
use lumosai::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Agent 是 LumosAI 的核心抽象
    let agent = lumosai::agent::simple("gpt-4", "你是一个友好的AI助手").await?;
    
    // 检查 Agent 信息
    println!("Agent 名称: {:?}", agent.name());
    println!("Agent 模型: {}", agent.model());
    println!("Agent 指令: {}", agent.get_instructions());
    
    Ok(())
}
```

## 第二部分：高级 Agent 配置

### 2.1 使用构建器模式

对于更复杂的配置，我们使用构建器模式：

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔧 创建高级 Agent");
    
    // 使用构建器创建 Agent
    let agent = Agent::builder()
        .name("研究助手")                    // Agent 名称
        .model("gpt-4")                      // 使用的模型
        .system_prompt("你是一个专业的研究助手，擅长学术分析和知识整理。")  // 系统提示词
        .max_tokens(2000)                    // 最大生成 token 数
        .temperature(0.7)                    // 温度参数 (0.0-2.0)
        .top_p(0.9)                          // 核采样参数
        .presence_penalty(0.1)              // 存在惩罚
        .frequency_penalty(0.1)             // 频率惩罚
        .build()
        .await?;
    
    println!("✅ 高级 Agent 创建成功");
    
    // 测试 Agent
    let response = agent.chat("请分析一下量子计算的基本原理。").await?;
    println!("分析结果: {}", response);
    
    Ok(())
}
```

### 2.2 配置参数详解

| 参数 | 类型 | 范围 | 作用 | 建议值 |
|------|------|------|------|--------|
| `temperature` | f64 | 0.0-2.0 | 控制生成随机性 | 创意任务: 0.8-1.2<br>事实任务: 0.1-0.3 |
| `max_tokens` | u32 | 1-8192 | 最大生成长度 | 短回答: 100-500<br>长回答: 1000-4000 |
| `top_p` | f64 | 0.0-1.0 | 核采样概率 | 0.9-0.95 |
| `presence_penalty` | f64 | -2.0-2.0 | 鼓励新话题 | 0.1-0.3 |
| `frequency_penalty` | f64 | -2.0-2.0 | 鼓励多样性 | 0.1-0.3 |

## 第三部分：对话管理

### 3.1 单轮对话

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = lumosai::agent::simple("gpt-4", "你是一个友好的AI助手").await?;
    
    // 生成回复（不保存历史）
    let response = agent.generate_simple("什么是机器学习？").await?;
    println!("回复: {}", response);
    
    // 再次询问（没有上下文）
    let response2 = agent.generate_simple("能详细解释一下吗？").await?;
    println!("回复2: {}", response2);
    
    Ok(())
}
```

### 3.2 多轮对话

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = lumosai::agent::simple("gpt-4", "你是一个友好的AI助手").await?;
    
    println!("💬 开始多轮对话");
    
    // 第一轮对话
    let response1 = agent.chat("我叫张三，今年25岁。").await?;
    println!("用户: 我叫张三，今年25岁");
    println!("Agent: {}", response1);
    
    // 第二轮对话（会记住之前的上下文）
    let response2 = agent.chat("我的爱好是编程和阅读。").await?;
    println!("用户: 我的爱好是编程和阅读");
    println!("Agent: {}", response2);
    
    // 第三轮对话（测试记忆）
    let response3 = agent.chat("请告诉我我的名字和年龄。").await?;
    println!("用户: 请告诉我我的名字和年龄");
    println!("Agent: {}", response3);
    
    Ok(())
}
```

### 3.3 对话历史管理

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = lumosai::agent::simple("gpt-4", "你是一个友好的AI助手").await?;
    
    // 进行一些对话
    agent.chat("你好！").await?;
    agent.chat("我喜欢编程").await?;
    agent.chat("我正在学习 Rust").await?;
    
    // 查看对话历史
    let history = agent.get_conversation_history().await?;
    println!("对话历史 ({} 条消息):", history.len());
    
    for (i, message) in history.iter().enumerate() {
        println!("  {}: {} - {}", i + 1, message.role(), message.content());
    }
    
    // 清除历史记录
    agent.clear_conversation_history().await?;
    println!("✅ 历史记录已清除");
    
    Ok(())
}
```

## 第四部分：错误处理

### 4.1 基本错误处理

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    match lumosai::agent::simple("gpt-4", "友好的AI助手").await {
        Ok(agent) => {
            println!("✅ Agent 创建成功");
            
            // 尝试对话
            match agent.chat("你好！").await {
                Ok(response) => println!("✅ 对话成功: {}", response),
                Err(e) => eprintln!("❌ 对话失败: {}", e),
            }
        }
        Err(e) => {
            eprintln!("❌ Agent 创建失败: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}
```

### 4.2 完整的错误处理

```rust
use lumosai::prelude::*;
use thiserror::Error;

#[derive(Debug, Error)]
enum AgentError {
    #[error("创建失败: {0}")]
    CreationFailed(String),
    
    #[error("对话失败: {0}")]
    ChatFailed(String),
    
    #[error("配置错误: {0}")]
    ConfigurationError(String),
}

#[tokio::main]
async fn main() -> Result<(), AgentError> {
    println!("🛡️ 错误处理示例");
    
    // 创建 Agent
    let agent = lumosai::agent::simple("gpt-4", "友好的AI助手")
        .await
        .map_err(|e| AgentError::CreationFailed(e.to_string()))?;
    
    // 尝试多次对话
    let questions = vec![
        "你好！",
        "今天天气怎么样？", // Agent 可能无法回答
        "什么是Rust编程语言？",
    ];
    
    for question in questions {
        println!("问题: {}", question);
        
        match agent.chat(question).await {
            Ok(response) => {
                println!("回答: {}", response);
            }
            Err(e) => {
                println!("错误: 无法回答此问题");
                println!("详情: {}", e);
                
                // 提供备用回答
                println!("备用: 对不起，我无法回答这个问题。");
            }
        }
        
        println!("{}", "-".repeat(50));
    }
    
    Ok(())
}
```

## 第五部分：实用技巧

### 5.1 提示词工程

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("✍️ 提示词工程示例");
    
    // 好的提示词设计
    let expert_agent = Agent::builder()
        .name("技术专家")
        .model("gpt-4")
        .system_prompt(r#"
你是一个资深的技术专家。请遵循以下原则回答问题：

1. 提供准确、详细的技术信息
2. 使用结构化的回答格式
3. 包含实用的代码示例
4. 解释技术概念的核心原理
5. 提供最佳实践建议

如果遇到不确定的问题，请诚实地说明你的知识局限。
        "#.trim())
        .temperature(0.3)  // 较低温度，保证准确性
        .build()
        .await?;
    
    let question = "如何优化 Rust 程序的性能？";
    let response = expert_agent.chat(question).await?;
    
    println!("问题: {}", question);
    println!("专家回答:");
    println!("{}", response);
    
    Ok(())
}
```

### 5.2 性能监控

```rust
use lumosai::prelude::*;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    println!("⚡ 性能监控示例");
    
    let agent = lumosai::agent::simple("gpt-4", "友好的AI助手").await?;
    
    let test_questions = vec![
        "什么是人工智能？",
        "Rust 有哪些主要特性？",
        "如何学习编程？",
    ];
    
    for (i, question) in test_questions.iter().enumerate() {
        println!("问题 {}: {}", i + 1, question);
        
        let start_time = Instant::now();
        
        match agent.chat(question).await {
            Ok(response) => {
                let duration = start_time.elapsed();
                let word_count = response.split_whitespace().count();
                let words_per_second = word_count as f64 / duration.as_secs_f64();
                
                println!("✅ 回复成功");
                println!("   时间: {:?}", duration);
                println!("   字数: {}", word_count);
                println!("   速度: {:.1} 词/秒", words_per_second);
                println!("   回复: {}", response.chars().take(100).collect::<String>() + "...");
            }
            Err(e) => {
                println!("❌ 回复失败: {}", e);
            }
        }
        
        println!("{}", "-".repeat(60));
    }
    
    Ok(())
}
```

### 5.3 批量处理

```rust
use lumosai::prelude::*;
use futures::stream::{self, StreamExt};

#[tokio::main]
async fn main() -> Result<()> {
    println!("📦 批量处理示例");
    
    let agent = lumosai::agent::simple("gpt-4", "友好的AI助手").await?;
    
    let questions = vec![
        "什么是机器学习？",
        "Rust 的优势是什么？",
        "如何编写高质量的代码？",
    ];
    
    // 使用 futures 进行并发处理
    let responses = stream::iter(questions)
        .map(|q| async move {
            let start = std::time::Instant::now();
            let result = agent.chat(q).await;
            let duration = start.elapsed();
            (q, result, duration)
        })
        .buffer_unordered(2)  // 同时处理最多2个请求
        .collect::<Vec<_>>()
        .await;
    
    for (question, result, duration) in responses {
        println!("问题: {}", question);
        println!("时间: {:?}", duration);
        
        match result {
            Ok(response) => println!("回复: {}", response),
            Err(e) => println!("错误: {}", e),
        }
        
        println!("{}", "-".repeat(40));
    }
    
    Ok(())
}
```

## 🎯 总结

通过本教程，您学会了：

✅ **Agent 创建**: 使用简单 API 和构建器模式创建 Agent  
✅ **配置管理**: 设置温度、token 限制等参数  
✅ **对话处理**: 单轮和多轮对话管理  
✅ **错误处理**: 健壮的错误处理机制  
✅ **性能监控**: 监控响应时间和性能指标  
✅ **实用技巧**: 提示词工程和批量处理

## 🔗 下一步学习

继续学习更多 LumosAI 功能：

- 📚 [RAG 基础](rag-basics.md) - 学习知识检索系统
- 🔧 [工具集成](tool-integration.md) - 扩展 Agent 能力
- 🏗️ [架构概览](../concepts/architecture.md) - 深入理解系统设计
- 🔍 [API 参考](../api-reference/agents.md) - 查看完整 API 文档

## 💡 练习建议

1. **创建不同角色的 Agent**: 编程助手、写作助手、分析助手
2. **尝试不同的配置参数**: 观察温度和其他参数对回复的影响
3. **实现一个简单的对话应用**: 结合错误处理和用户输入
4. **性能优化**: 测试不同模型的响应时间和质量

**🌟 恭喜完成 Agent 基础教程！**