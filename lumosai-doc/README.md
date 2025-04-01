# Lumosai 技术文档

![Lumosai版本](https://img.shields.io/badge/版本-0.1.0-blue)
![Rust版本](https://img.shields.io/badge/Rust-1.70+-orange)
![许可证](https://img.shields.io/badge/许可-MIT-green)
![构建状态](https://img.shields.io/badge/构建-通过-brightgreen)

Lumosai 是一个开源的分布式 AI 代理框架，使用 Rust 语言开发，专注于性能、安全性和可扩展性。它支持创建智能代理，这些代理可以执行复杂任务、进行协作，并能在分布式网络中运行。

## 核心特性

- **高性能**: 使用 Rust 语言开发，提供卓越的运行时性能和内存安全性
- **去中心化**: 支持 P2P 网络架构，实现代理的分布式协作
- **多模态**: 原生支持文本、图像、音频和视频的输入和输出处理
- **本地优先**: 支持本地部署和运行 AI 模型，保护数据隐私
- **跨平台**: 支持在桌面、服务器和嵌入式设备上运行
- **可扩展**: 模块化设计，支持自定义组件和扩展功能

## 文档目录

1. [项目概述](./01_overview.md) - 项目的基本介绍、目标和核心功能
2. [系统架构](./02_architecture.md) - 系统的整体架构设计和组件关系
3. [技术栈](./03_tech_stack.md) - 项目使用的关键技术和依赖
4. [核心组件](./04_core_components.md) - 核心组件详细介绍和实现
5. [API参考](./05_api_reference.md) - API接口详细说明和用法
6. [开发指南](./06_development_guide.md) - 开发环境配置和开发流程
7. [部署指南](./07_deployment_guide.md) - 不同部署模式的配置和操作指南
8. [常见问题](./08_faq.md) - 常见问题解答

## 系统要求

- **操作系统**: Linux, macOS, Windows
- **Rust 版本**: 1.70 或更高
- **内存**: 最小 4GB，推荐 8GB 或更多
- **存储**: 最小 1GB 可用空间，实际需求取决于模型大小和数据量

## 快速开始

### 安装

#### 通过 Cargo 安装 CLI 工具

```bash
# 安装 Lumosai CLI
cargo install lumosai_cli

# 验证安装
lumosai --version
```

#### 添加库依赖到项目

在你的`Cargo.toml`中添加依赖：

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

高级配置，启用所有功能：

```toml
[dependencies]
lumosai_core = { version = "0.1.0", features = ["macros", "distributed", "multi-modal", "local-models"] }
lumosai_distributed = "0.1.0"
lumosai_tools = "0.1.0"
```

### 创建第一个代理

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

### 创建带工具的代理

```rust
use lumosai_core::{Result, Error};
use lumosai_core::agent::{Agent, ReActAgent};
use lumosai_core::tool::{Tool, FunctionTool};
use lumosai_core::llm::{LlmProvider, OpenAiAdapter};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct CalculatorParams {
    operation: String,
    a: f64,
    b: f64,
}

#[derive(Serialize, Deserialize)]
struct CalculatorResult {
    result: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 创建LLM适配器
    let llm = Arc::new(OpenAiAdapter::new(
        "your-api-key",
        "gpt-4",
    ));
    
    // 创建计算器工具
    let calculator = FunctionTool::new(
        "calculator",
        "执行基本的数学计算",
        |params: CalculatorParams| async move {
            let result = match params.operation.as_str() {
                "add" => params.a + params.b,
                "subtract" => params.a - params.b,
                "multiply" => params.a * params.b,
                "divide" => params.a / params.b,
                _ => return Err(Error::Tool("不支持的操作".into())),
            };
            
            Ok(CalculatorResult { result })
        },
    );
    
    // 创建支持工具的代理
    let mut agent = ReActAgent::new(
        "math_solver",
        "你是一个数学解题助手，可以解决各种数学问题。",
        llm,
    );
    
    // 添加工具到代理
    agent.add_tool(Arc::new(calculator));
    
    // 运行代理
    let response = agent.run("求解方程 3x + 7 = 25").await?;
    println!("代理回答: {}", response);
    
    Ok(())
}
```

### 使用 JavaScript/TypeScript 客户端库

```typescript
import { LumosClient, Agent } from '@lumosai/client-js';

async function main() {
  // 创建客户端连接
  const client = new LumosClient({
    apiKey: process.env.LUMOSAI_API_KEY,
    serverUrl: 'https://api.lumosai.org'
  });
  
  // 创建代理
  const agent = await client.createAgent({
    name: "assistant",
    description: "一个通用的助手，可以回答问题和执行任务。"
  });
  
  // 执行查询
  const result = await agent.execute({
    query: "总结量子计算的最新进展"
  });
  
  console.log("代理回答:", result.output);
}

main().catch(console.error);
```

## 部署选项

Lumosai 提供多种部署选项：

### 单节点部署

适合个人使用或小型团队：

```bash
# 启动单个 Lumosai 节点
lumosai start --config config.toml
```

### 分布式部署

适合大规模应用和高可用性需求：

```bash
# 启动引导节点
lumosai node start --bootstrap --addr 0.0.0.0:7000

# 启动普通节点并连接到引导节点
lumosai node start --connect /ip4/192.168.1.100/tcp/7000/p2p/QmBootstrapNodeId
```

### Docker 部署

使用 Docker 快速部署：

```bash
# 拉取官方镜像
docker pull lumosai/server:latest

# 启动服务
docker run -d -p 3000:3000 -v ./data:/app/data lumosai/server:latest
```

详细部署指南请参阅[部署文档](./07_deployment_guide.md)。

## 社区和支持

- [GitHub 仓库](https://github.com/lumosai/lumosai) - 代码仓库和问题跟踪
- [官方网站](https://www.lumosai.org) - 项目官方网站和最新动态
- [社区论坛](https://community.lumosai.org) - 讨论和问答
- [Discord 频道](https://discord.gg/lumosai) - 实时交流和社区活动
- [微信公众号](https://www.lumosai.org/wechat) - Lumosai开发者社区

## 路线图

- **短期计划 (0-3 个月)**
  - 增强多模态处理能力
  - 优化边缘设备上的模型推理性能
  - 扩展工具生态系统

- **中期计划 (3-6 个月)**
  - 改进 P2P 网络的安全机制
  - 增加企业级功能和多租户支持
  - 开发更多垂直领域适配器

- **长期计划 (6+ 个月)**
  - 实现代理自主学习和进化能力
  - 建立去中心化的模型共享市场
  - 开发更完善的评估和基准测试框架

## 贡献

欢迎贡献代码、报告问题或提出改进建议。参与贡献的步骤：

1. Fork 项目仓库
2. 创建您的特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交您的更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 提交 Pull Request

详细信息请参考[开发指南](./06_development_guide.md)。

## 引用

如果您在学术工作中使用了 Lumosai，请使用以下格式引用：

```
Zhang, L., et al. (2023). Lumosai: A Distributed AI Agent Framework.
GitHub repository: https://github.com/lumosai/lumosai
```

## 许可证

Lumosai 项目基于 MIT 许可证开源。详情请参阅 [LICENSE](../LICENSE) 文件。 