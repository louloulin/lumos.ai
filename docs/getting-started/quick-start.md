# 🚀 快速开始

> 5分钟体验 LumosAI 的强大功能

## 前置要求

- **Rust**: 1.70+ ([安装指南](https://www.rust-lang.org/tools/install))
- **操作系统**: Linux, macOS, 或 Windows

## 第一步：安装 LumosAI

在你的 Rust 项目中添加 LumosAI：

```bash
# 新建项目
cargo new my_ai_app
cd my_ai_app

# 添加 LumosAI 依赖
cargo add lumosai
```

或者手动编辑 `Cargo.toml`：

```toml
[dependencies]
lumosai = "0.2.0"
tokio = { version = "1.0", features = ["full"] }
```

## 第二步：创建你的第一个 AI Agent

创建 `src/main.rs` 文件：

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🤖 创建第一个 AI Agent");
    
    // 创建 AI Agent
    let agent = Agent::builder()
        .name("assistant")
        .model("gpt-4")  // 或其他支持的模型
        .system_prompt("你是一个友好的AI助手，总是乐于帮助用户。")
        .build()
        .await?;

    // 开始对话
    let response = agent.chat("你好！请介绍一下你自己。").await?;
    
    println!("Agent 回复: {}", response);
    
    Ok(())
}
```

## 第三步：运行应用

```bash
# 设置环境变量（如果需要）
export OPENAI_API_KEY="your_api_key_here"

# 运行应用
cargo run
```

**预期输出：**
```
🤖 创建第一个 AI Agent
Agent 回复: 你好！我是一个友好的AI助手，我可以帮助您解答问题、提供信息和协助完成各种任务。
```

## 🎯 接下来做什么？

### 尝试更多功能

#### 1. 添加 RAG（检索增强生成）
```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建向量存储
    let storage = lumosai::vector::memory().await?;
    
    // 创建 RAG 系统
    let rag = lumosai::rag::builder()
        .storage(storage)
        .embedding_provider("openai")
        .build()
        .await?;
    
    // 添加文档
    rag.add_document("Rust 是一门系统编程语言，注重安全、速度和并发。").await?;
    
    // 搜索相关内容
    let results = rag.search("什么是 Rust？", 3).await?;
    
    println!("找到 {} 个相关文档", results.len());
    
    Ok(())
}
```

#### 2. 多 Agent 协作
```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建专业 Agent
    let researcher = Agent::builder()
        .name("研究员")
        .model("gpt-4")
        .system_prompt("你是一个专业的研究员，擅长收集和分析信息。")
        .build()
        .await?;
    
    let writer = Agent::builder()
        .name("写手")
        .model("gpt-4")
        .system_prompt("你是一个专业的写手，擅长将信息整理成清晰的文章。")
        .build()
        .await?;
    
    // 协作完成任务
    let research_result = researcher.chat("研究人工智能的最新发展。").await?;
    let final_article = writer.chat(&format!("基于以下信息写一篇文章：{}", research_result)).await?;
    
    println!("最终文章：{}", final_article);
    
    Ok(())
}
```

#### 3. 工具集成
```rust
use lumosai::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建带工具的 Agent
    let agent = Agent::builder()
        .name("助手")
        .model("gpt-4")
        .system_prompt("你是一个可以调用工具的助手。")
        .build()
        .await?;
    
    // 添加工具
    agent.add_tool("get_weather", "获取当前天气", vec![
        ("location", "城市名称", "string")
    ]).await?;
    
    // 使用工具
    let response = agent.chat("北京今天天气怎么样？").await?;
    
    println!("回复：{}", response);
    
    Ok(())
}
```

## 📚 学习路径

### 🆕 初学者
1. [Agent 基础教程](../tutorials/basics/agent-basics.md) - 深入了解 Agent 系统
2. [RAG 基础教程](../tutorials/basics/rag-basics.md) - 构建知识问答系统
3. [工具集成教程](../tutorials/basics/tool-integration.md) - 扩展 Agent 能力

### 🚀 进阶用户
1. [多 Agent 协作](../tutorials/intermediate/multi-agent.md) - 团队协作模式
2. [自定义工具开发](../tutorials/intermediate/custom-tools.md) - 创建专业工具
3. [性能优化](../tutorials/intermediate/performance.md) - 提升系统性能

### 🏗️ 企业用户
1. [企业部署指南](../tutorials/advanced/enterprise.md) - 生产环境部署
2. [监控与日志](../tutorials/advanced/monitoring.md) - 系统运维
3. [安全最佳实践](../guides/security/) - 安全配置

## 🔧 环境变量配置

根据你使用的 LLM 提供商，设置相应的环境变量：

```bash
# OpenAI
export OPENAI_API_KEY="your_openai_key"

# Anthropic Claude
export ANTHROPIC_API_KEY="your_anthropic_key"

# DeepSeek
export DEEPSEEK_API_KEY="your_deepseek_key"

# Zhipu AI
export ZHIPUAI_API_KEY="your_zhipuai_key"
```

## ❓ 常见问题

### Q: 如何在没有 API 密钥的情况下测试？
A: 使用 Mock 提供商进行开发测试：
```rust
use lumosai_core::llm::MockLlmProvider;
use std::sync::Arc;

let mock_provider = Arc::new(MockLlmProvider::new(vec![
    "这是一个测试回复".to_string()
]));

let agent = Agent::builder()
    .model(mock_provider)
    .build()
    .await?;
```

### Q: 支持哪些 LLM 提供商？
A: 目前支持 OpenAI、Anthropic Claude、DeepSeek、Zhipu AI 等主流提供商。

### Q: 如何处理长对话？
A: LumosAI 自动管理对话历史，你也可以手动控制：
```rust
agent.set_memory_limit(1000)?; // 限制记忆长度
```

## 🆘 获取帮助

- **📖 完整文档**: [文档中心](../README.md)
- **💡 示例代码**: [examples/](../../examples/)
- **🐛 问题反馈**: [GitHub Issues](https://github.com/louloulin/lumos.ai/issues)
- **❓ FAQ**: [常见问题](../resources/faq.md)

---

## 🎉 成功！

你已经成功创建了第一个 LumosAI 应用！接下来可以：

1. **探索更多功能**: 查看 [教程](../tutorials/) 了解更多
2. **查看示例**: 浏览 [examples/](../../examples/) 获取灵感
3. **阅读 API 文档**: [API 参考](../api-reference/README.md) 了解详细信息

**🌟 享受构建 AI 应用的乐趣！**