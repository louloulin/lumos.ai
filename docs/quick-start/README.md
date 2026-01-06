# LumosAI 快速开始指南

欢迎使用 LumosAI！这个 5 分钟快速开始指南将帮助您快速上手 LumosAI 的核心功能。

## 📋 前置要求

- Rust 1.75+
- 一个 OpenAI API 密钥（或其他支持的 LLM 提供商）

## 🚀 安装

### 1. 创建新项目

```bash
cargo new my-lumosai-app
cd my-lumosai-app
```

### 2. 添加依赖

在 `Cargo.toml` 中添加：

```toml
[dependencies]
lumosai = "0.1.4"
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
```

### 3. 设置环境变量

```bash
export OPENAI_API_KEY="your-api-key-here"
```

## 🎯 第一个 Agent

创建您的第一个 AI Agent：

```rust
use lumosai::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建一个简单的 Agent
    let agent = Agent::builder()
        .name("助手")
        .instructions("你是一个友好的助手，用中文回答问题。")
        .model("gpt-3.5-turbo")
        .build()?;

    // 发送消息
    let response = agent.generate("你好！请介绍一下自己。").await?;
    println!("Agent 回复: {}", response);

    Ok(())
}
```

## 🛠️ 添加工具

让 Agent 具备工具使用能力：

```rust
use lumosai::prelude::*;
use lumos_macro::tool;

// 使用宏定义工具
#[tool]
async fn calculate(operation: String, a: f64, b: f64) -> Result<f64> {
    match operation.as_str() {
        "add" => Ok(a + b),
        "subtract" => Ok(a - b),
        "multiply" => Ok(a * b),
        "divide" => {
            if b != 0.0 {
                Ok(a / b)
            } else {
                Err(Error::from("不能除以零"))
            }
        }
        _ => Err(Error::from("不支持的操作"))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 创建工具
    let calculator = CalculateTool::new();
    
    // 创建带工具的 Agent
    let agent = Agent::builder()
        .name("计算助手")
        .instructions("你是一个数学助手，可以帮助用户进行计算。")
        .model("gpt-3.5-turbo")
        .tool(calculator)
        .build()?;

    // 让 Agent 使用工具
    let response = agent.generate("请计算 15 + 27").await?;
    println!("Agent 回复: {}", response);

    Ok(())
}
```

## 💾 添加内存

为 Agent 添加记忆能力：

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建内存系统
    let memory = Memory::basic();
    
    // 创建带内存的 Agent
    let agent = Agent::builder()
        .name("记忆助手")
        .instructions("你是一个有记忆的助手，能记住之前的对话。")
        .model("gpt-3.5-turbo")
        .memory(memory)
        .build()?;

    // 多轮对话
    let response1 = agent.generate("我的名字是张三").await?;
    println!("第一轮: {}", response1);

    let response2 = agent.generate("你还记得我的名字吗？").await?;
    println!("第二轮: {}", response2);

    Ok(())
}
```

## 📚 RAG 系统

创建一个简单的 RAG（检索增强生成）系统：

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建 RAG 系统
    let rag = SimpleRag::builder()
        .embedding_provider("openai")
        .build()?;

    // 添加文档
    rag.add_document("LumosAI 是一个用 Rust 编写的 AI 应用开发框架。").await?;
    rag.add_document("它支持 Agent、RAG、工具集成等功能。").await?;

    // 创建带 RAG 的 Agent
    let agent = Agent::builder()
        .name("知识助手")
        .instructions("基于提供的文档回答问题。")
        .model("gpt-3.5-turbo")
        .rag(rag)
        .build()?;

    // 基于知识库回答问题
    let response = agent.generate("LumosAI 是什么？").await?;
    println!("Agent 回复: {}", response);

    Ok(())
}
```

## 🎉 恭喜！

您已经学会了 LumosAI 的基本用法：

- ✅ 创建基本 Agent
- ✅ 添加工具功能
- ✅ 使用内存系统
- ✅ 构建 RAG 系统

## 📖 下一步

- 查看 [教程系列](../tutorials/) 了解更多高级功能
- 浏览 [示例项目](../examples/) 获取灵感
- 阅读 [API 参考](../api-reference/) 了解详细接口
- 学习 [最佳实践](../best-practices/) 优化您的应用

## 🆘 需要帮助？

- 查看 [常见问题](../../docs/8_faq.md)
- 浏览 [GitHub Issues](https://github.com/lumosai/lumosai/issues)
- 加入我们的社区讨论

---

*LumosAI - 让 AI 应用开发变得简单而强大* 🌟
