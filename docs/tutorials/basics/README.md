# 📚 基础教程

> 从零开始学习 LumosAI 的核心功能

基础教程面向 LumosAI 新用户，通过循序渐进的实践帮助您快速掌握框架的核心概念和基本用法。

## 🎯 教程路径

```
入门 → Agent 基础 → RAG 基础 → 工具集成 → 进阶功能
  ↓       ↓         ↓         ↓         ↓
理解概念  创建Agent  知识检索  扩展能力  实际应用
```

## 教程列表

### 1. [Agent 基础](agent-basics.md)
- **学习目标**: 掌握 Agent 的创建、配置和基本使用
- **包含内容**: 
  - Agent 生命周期管理
  - 对话处理和记忆管理
  - 错误处理和调试技巧
- **预计时间**: 30分钟
- **难度**: ⭐

### 2. [RAG 基础](rag-basics.md)
- **学习目标**: 学会构建和使用检索增强生成系统
- **包含内容**:
  - 向量存储配置和使用
  - 文档处理和分块策略
  - 检索优化和结果分析
- **预计时间**: 45分钟
- **难度**: ⭐⭐

### 3. [工具集成](tool-integration.md)
- **学习目标**: 掌握工具系统的使用和自定义开发
- **包含内容**:
  - 内置工具的使用方法
  - 自定义工具开发
  - 工具链组合和工作流
- **预计时间**: 60分钟
- **难度**: ⭐⭐

## 🚀 开始之前

### 前置要求

- **Rust 环境**: 1.70+ 版本
- **API 密钥**: 至少一个 LLM 提供商的 API 密钥
- **基础概念**: 了解异步编程和基本的 API 使用

### 环境准备

```bash
# 1. 创建新项目
cargo new lumosai-learning
cd lumosai-learning

# 2. 添加依赖
cargo add lumosai tokio

# 3. 设置环境变量
export OPENAI_API_KEY="your_key_here"
# 或者
export DEEPSEEK_API_KEY="your_key_here"
```

### 基础代码模板

```rust
// src/main.rs
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 你的 LumosAI 代码将写在这里
    println!("🤖 Welcome to LumosAI!");
    
    Ok(())
}
```

## 🎓 学习建议

### 最佳实践

1. **循序渐进**: 按照教程顺序学习，不要跳跃
2. **动手实践**: 每个教程都包含可运行的代码示例
3. **实验探索**: 在基础代码上进行修改和实验
4. **问题解决**: 遇到问题时参考 FAQ 和 API 文档

### 学习资源

- **[API 参考](../api-reference/)** - 完整的接口文档
- **[核心概念](../concepts/)** - 深入理解设计理念
- **[示例代码](../../examples/)** - 更多实用示例
- **[故障排除](../resources/faq.md)** - 常见问题解决

### 社区支持

- **GitHub Issues**: 报告问题和功能请求
- **Discord 社区**: 实时讨论和帮助
- **论坛**: 深度技术讨论

## 📊 学习路径

### 初学者路径 (0-1个月)
```
Week 1: Agent 基础 → 熟悉基本操作
Week 2: RAG 基础 → 掌握知识检索
Week 3: 工具集成 → 扩展系统能力
Week 4: 实践项目 → 构建第一个应用
```

### 开发者路径 (1-3个月)
```
Month 1: 完成基础教程 → 建立扎实基础
Month 2: 进阶教程学习 → 掌握高级功能
Month 3: 项目实践 → 独立开发应用
```

### 专家路径 (3-6个月)
```
Month 1-2: 基础 + 进阶 → 全面掌握
Month 3-4: 深度定制 → 扩展和优化
Month 5-6: 架构设计 → 大型应用开发
```

## 🔧 实用工具

### 开发工具

```bash
# 代码格式化
cargo fmt

# 代码检查
cargo clippy

# 运行测试
cargo test

# 生成文档
cargo doc
```

### 调试技巧

```rust
// 启用详细日志
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("Starting LumosAI application");
    
    // 你的代码
    
    Ok(())
}
```

### 性能测试

```rust
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    let start = Instant::now();
    
    // 执行操作
    let response = agent.chat("Hello").await?;
    
    let duration = start.elapsed();
    println!("Response time: {:?}", duration);
    
    Ok(())
}
```

---

## 🎉 开始学习

准备好了吗？让我们开始第一个教程：

- 👉 **[Agent 基础教程](agent-basics.md)** - 从最基础的 Agent 创建开始

如果您已经有一些经验，也可以直接跳转到感兴趣的教程。

需要帮助？查看我们的 [FAQ](../resources/faq.md) 或联系社区支持。

**🌟 享受学习 LumosAI 的旅程！**