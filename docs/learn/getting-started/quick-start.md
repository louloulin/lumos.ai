# ⚡ 快速开始

**5分钟创建你的第一个AI Agent**

本指南将带你快速体验LumosAI的核心功能，创建一个能够对话的AI Agent。

## ⏱️ 预计时间: 5-10分钟

## 📋 前置条件

在开始之前，请确保你已经：
- ✅ 完成了[环境安装](installation.md)
- ✅ 准备好了AI模型的API密钥

---

## 🚀 第一步：创建项目

### 1. 初始化Rust项目
```bash
cargo new my-lumosai-app
cd my-lumosai-app
```

### 2. 添加依赖
在 `Cargo.toml` 中添加LumosAI：

```toml
[dependencies]
lumosai = "0.2.0"
tokio = { version = "1.0", features = ["full"] }
```

### 3. 设置API密钥
```bash
# OpenAI API (推荐)
export OPENAI_API_KEY="your-api-key-here"

# 或者其他支持的模型API
export ANTHROPIC_API_KEY="your-api-key-here"
```

---

## 🤖 第二步：创建简单Agent

创建你的第一个AI Agent：

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 🤖 创建一个简单的Agent
    let agent = Agent::builder()
        .name("助手")
        .system_prompt("你是一个友好的AI助手，用中文回答问题。")
        .model("gpt-4")
        .build()
        .await?;

    // 💬 开始对话
    let response = agent.chat("你好！请介绍一下自己。").await?;
    println!("Agent: {}", response);

    Ok(())
}
```

**运行它：**
```bash
cargo run
```

**预期输出：**
```
Agent: 你好！我是一个AI助手，可以帮助你解答问题、提供信息和建议...
```

---

## 🛠️ 第三步：添加工具能力

让Agent具备实际操作能力：

```rust
use lumosai::prelude::*;
use lumos_macro::tool;

// 🛠️ 定义一个工具
#[tool]
async fn get_current_time() -> Result<String> {
    let now = chrono::Utc::now();
    Ok(now.format("%Y-%m-%d %H:%M:%S").to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    // 🔧 创建工具实例
    let time_tool = GetCurrentTimeTool::new();
    
    // 🤖 创建带工具的Agent
    let agent = Agent::builder()
        .name("智能助手")
        .system_prompt("你是一个功能丰富的助手，可以获取当前时间。")
        .model("gpt-4")
        .tool(time_tool)
        .build()
        .await?;

    // 💬 测试工具使用
    let response = agent.chat("现在几点了？").await?;
    println!("Agent: {}", response);

    Ok(())
}
```

---

## 🧠 第四步：添加RAG能力

给Agent添加知识检索功能：

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 📦 创建向量存储
    let storage = lumosai::vector::memory().await?;

    // 🧠 创建RAG系统
    let rag = lumosai::rag::builder()
        .storage(storage)
        .embedding_provider("openai")
        .chunking_strategy("recursive")
        .build()
        .await?;

    // 📄 添加文档
    rag.add_document("LumosAI是一个基于Rust的AI框架。").await?;
    rag.add_document("它支持Agent、RAG、工具集成等功能。").await?;

    // 🔍 搜索相关内容
    let results = rag.search("LumosAI的特点", 3).await?;
    println!("找到 {} 个相关文档", results.len());

    // 🤖 创建基于RAG的Agent
    let agent = Agent::builder()
        .name("知识助手")
        .system_prompt("基于提供的文档回答问题。")
        .model("gpt-4")
        .rag(rag)
        .build()
        .await?;

    // 💬 基于知识库回答
    let response = agent.chat("LumosAI有什么功能？").await?;
    println!("Agent: {}", response);

    Ok(())
}
```

---

## 🎯 恭喜！

你已经成功创建了第一个LumosAI应用！🎉

你学会了：
- ✅ 创建AI Agent
- ✅ 添加工具能力  
- ✅ 集成RAG知识库
- ✅ 实现智能对话

---

## 🔥 下一步

完成了快速开始？这里有很多有趣的后续内容：

### 🎓 继续学习
- **[基础教程](../tutorials/basics/README.md)** - 深入理解核心概念
- **[API文档](../../reference/api/README.md)** - 完整的接口文档
- **[示例集合](../examples/README.md)** - 更多实用示例

### 🚀 进阶功能
- **[多Agent协作](../tutorials/advanced/multi-agent.md)** - 构建Agent团队
- **[工作流自动化](../tutorials/advanced/workflows.md)** - 复杂任务编排
- **[企业级功能](../guides/enterprise.md)** - 生产环境部署

### 💡 实用资源
- **[配置参考](../../reference/configuration/README.md)** - 详细的配置选项
- **[最佳实践](../guides/best-practices.md)** - 开发经验总结
- **[故障排除](troubleshooting.md)** - 常见问题解决

---

## 🆘 需要帮助？

遇到问题？这里有一些解决方案：

### 🔧 常见问题
- **API密钥问题** → 检查环境变量设置
- **网络连接** → 确保能访问AI模型API
- **依赖编译** → 运行 `cargo update` 更新依赖

### 📖 更多资源
- [故障排除指南](troubleshooting.md)
- [常见问题FAQ](../../community/faq.md)
- [GitHub Issues](https://github.com/louloulin/lumos.ai/issues)

### 💬 社区支持
- [GitHub Discussions](https://github.com/louloulin/lumos.ai/discussions)
- [技术交流群](../../community/README.md)

---

**继续你的LumosAI学习之旅！** 🚀

*[← 返回 Getting Started](README.md)*