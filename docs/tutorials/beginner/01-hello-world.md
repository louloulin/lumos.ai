# Hello World - 您的第一个LumosAI Agent

## 🎯 学习目标

在这个5分钟的教程中，您将：
- 安装LumosAI
- 创建您的第一个AI Agent
- 生成第一个AI响应
- 理解LumosAI的基本概念

## 📋 前置要求

- Rust 1.70+ 已安装
- 基本的Rust编程知识
- 一个OpenAI API密钥 (可选，我们会提供Mock示例)

## 🚀 快速开始

### 1. 创建新项目

```bash
cargo new my-first-agent
cd my-first-agent
```

### 2. 添加依赖

编辑 `Cargo.toml`：

```toml
[dependencies]
lumosai_core = { path = "../lumosai_core" }  # 或使用发布版本
tokio = { version = "1.0", features = ["full"] }
```

### 3. 编写您的第一个Agent

编辑 `src/main.rs`：

```rust
use lumosai_core::prelude::*;
use lumosai_core::llm::MockLlmProvider;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 欢迎使用LumosAI!");
    
    // 创建一个Mock LLM提供者 (用于演示)
    let llm = Arc::new(MockLlmProvider::new(vec![
        "你好！我是您的AI助手，很高兴为您服务！".to_string(),
        "我可以帮助您解答问题、处理任务和提供建议。".to_string(),
        "有什么我可以帮助您的吗？".to_string(),
    ]));
    
    // 使用最简单的API创建Agent
    let agent = quick_agent("assistant", "你是一个友好的AI助手")
        .model(llm)
        .build()?;
    
    println!("✅ Agent创建成功!");
    
    // 生成响应
    let response = agent.generate("你好，请介绍一下自己").await?;
    
    println!("🤖 Agent回复: {}", response.content);
    
    Ok(())
}
```

### 4. 运行程序

```bash
cargo run
```

您应该看到类似的输出：

```
🚀 欢迎使用LumosAI!
✅ Agent创建成功!
🤖 Agent回复: 你好！我是您的AI助手，很高兴为您服务！
```

## 🎉 恭喜！

您已经成功创建了第一个LumosAI Agent！让我们来理解刚才发生了什么。

## 📖 代码解析

### 1. 导入必要的模块

```rust
use lumosai_core::prelude::*;
```

`prelude`模块包含了最常用的类型和函数，让您可以快速开始。

### 2. 创建LLM提供者

```rust
let llm = Arc::new(MockLlmProvider::new(vec![...]));
```

- `MockLlmProvider`: 用于演示的模拟LLM，返回预定义的响应
- `Arc`: Rust的原子引用计数，用于在多个地方共享LLM实例

### 3. 创建Agent

```rust
let agent = quick_agent("assistant", "你是一个友好的AI助手")
    .model(llm)
    .build()?;
```

- `quick_agent()`: 最简单的Agent创建函数
- 第一个参数: Agent的名称
- 第二个参数: Agent的指令/角色描述
- `.model()`: 设置使用的LLM
- `.build()`: 构建最终的Agent实例

### 4. 生成响应

```rust
let response = agent.generate("你好，请介绍一下自己").await?;
```

- `generate()`: 生成AI响应的异步方法
- 返回包含响应内容的结构体

## 🔄 使用真实的LLM

如果您有OpenAI API密钥，可以使用真实的LLM：

```rust
use lumosai_core::prelude::*;
use lumosai_core::llm::OpenAiProvider;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // 使用真实的OpenAI LLM
    let llm = Arc::new(OpenAiProvider::new("your-api-key-here"));
    
    let agent = quick_agent("assistant", "你是一个友好的AI助手")
        .model(llm)
        .build()?;
    
    let response = agent.generate("解释一下什么是人工智能").await?;
    println!("🤖 Agent回复: {}", response.content);
    
    Ok(())
}
```

## 🛠️ 故障排除

### 编译错误

如果遇到编译错误，请检查：
1. Rust版本是否为1.70+
2. 依赖路径是否正确
3. 是否启用了必要的features

### 运行时错误

如果程序运行时出错：
1. 检查API密钥是否正确 (如果使用真实LLM)
2. 确保网络连接正常
3. 查看错误信息获取具体原因

## 🎯 核心概念

通过这个简单的例子，您已经接触到了LumosAI的核心概念：

1. **Agent**: AI智能体，具有特定的角色和能力
2. **LLM Provider**: 大语言模型提供者，负责实际的AI推理
3. **Prelude**: 常用API的集合，简化导入
4. **异步编程**: 使用async/await处理AI响应

## 📚 下一步

现在您已经创建了第一个Agent，可以继续学习：

- [快速API入门](./02-quick-api.md) - 学习更多快速API功能
- [工具使用基础](./03-basic-tools.md) - 为Agent添加工具能力
- [API选择指南](../../api-choice-guide.md) - 了解不同的API选择

## 💡 小贴士

1. **从简单开始**: 先掌握`quick_agent()`，再学习更复杂的API
2. **多做实验**: 尝试不同的指令和输入
3. **查看示例**: 参考[示例代码库](../../../examples/)
4. **参与社区**: 在GitHub上提问和分享经验

继续您的LumosAI学习之旅！🚀
