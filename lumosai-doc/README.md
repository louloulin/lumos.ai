# Lumosai 技术文档

这是 Lumosai 项目的技术文档。Lumosai 是一个用 Rust 实现的 AI Agent 框架，专注于性能、安全性和可扩展性。

## 文档目录

1. [项目概述](./01_overview.md) - 项目的基本介绍、目标和核心功能
2. [系统架构](./02_architecture.md) - 系统的整体架构设计和组件关系
3. [技术栈](./03_tech_stack.md) - 项目使用的关键技术和依赖
4. [核心组件](./04_core_components.md) - 核心组件详细介绍和实现
5. [API参考](./05_api_reference.md) - API接口详细说明和用法
6. [开发指南](./06_development_guide.md) - 开发环境配置和开发流程
7. [部署指南](./07_deployment_guide.md) - 不同部署模式的配置和操作指南
8. [常见问题](./08_faq.md) - 常见问题解答

## 快速开始

### 安装

添加依赖到你的`Cargo.toml`：

```toml
[dependencies]
lumosai_core = "0.1.0"
```

若要使用宏功能，启用`macros`特性：

```toml
[dependencies]
lumosai_core = { version = "0.1.0", features = ["macros"] }
lumos_macro = "0.1.0"
```

### 基础使用示例

```rust
use lumosai_core::{Result, Error};
use lumosai_core::agent::{Agent, SimpleAgent};
use lumosai_core::tool::{Tool, FunctionTool};
use lumosai_core::llm::{LlmProvider, OpenAiAdapter};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建LLM适配器
    let llm = Arc::new(OpenAiAdapter::new(
        "your-api-key",
        "gpt-4",
    ));
    
    // 创建代理
    let mut agent = SimpleAgent::new(
        "math_helper",
        "你是一个擅长数学的助手。",
        llm,
    );
    
    // 运行代理
    let response = agent.run("计算 (15 + 27) * 2").await?;
    println!("代理回答: {}", response);
    
    Ok(())
}
```

## 贡献

欢迎贡献代码、报告问题或提出改进建议。请参考[开发指南](./06_development_guide.md)了解更多信息。

## 许可证

Lumosai 项目基于 MIT 许可证开源。详情请参阅 [LICENSE](../LICENSE) 文件。 