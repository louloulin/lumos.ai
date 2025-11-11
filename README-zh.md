# 🌟 LumosAI（中文）

<div align="center">

**一个用 Rust 构建的企业级 AI 框架，用于快速打造智能应用**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/louloulin/lumos.ai)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/lumosai)

[📖 英文文档](docs/index.md) | [🌏 中文文档索引](docs/README_CN.md) | [🚀 快速开始](docs/quick-start/README.md) | [💡 示例](#示例) | [🤝 参与贡献](#参与贡献)

</div>

---

## 目录
- [特色能力](#特色能力)
- [快速开始](#快速开始)
- [系统要求](#系统要求)
- [示例](#示例)
- [架构](#架构)
- [工作区总览](#工作区总览)
- [特性开关](#特性开关)
- [文档](#文档)
- [更新日志与版本发布](#更新日志与版本发布)
- [参与贡献](#参与贡献)
- [社区与支持](#社区与支持)
- [许可协议](#许可协议)

## ✨ 特色能力

### 🤖 智能 Agent 系统
- 多模型支持：OpenAI、Anthropic、本地模型
- 专业化 Agent：研究、写作、分析与自定义角色
- 工具集成：可扩展工具系统与内置工具
- 会话记忆：持久化上下文与历史记录

### 🧠 先进 RAG 系统
- 文档处理：PDF、文本、Markdown、网页内容
- 智能分块：递归、语义与自定义策略
- 向量存储：Memory、PostgreSQL、Qdrant、Weaviate
- 混合检索：语义检索 + 关键词匹配

### 🔄 工作流编排
- 多 Agent 协作：串行、并行与条件分支
- 任务管理：复杂任务拆解与执行
- 事件驱动：实时事件处理与路由
- 错误处理：重试与回退策略

### 🛡️ 企业级安全
- 认证：JWT、OAuth2、API Key、多因子认证
- 授权：RBAC 细粒度权限控制
- 多租户：租户隔离与自定义配置
- 审计：安全与合规日志

### 📊 监控与可观测性
- 实时指标：性能、使用率与健康监控
- 分布式追踪：跨 Agent 交互的请求追踪
- 自定义看板：Grafana、Prometheus 集成
- 告警：系统异常智能告警

### ⚡ 高性能
- Rust 性能：内存安全与零开销抽象
- Async/Await：高并发异步 I/O
- 缓存：多层智能缓存
- 可扩展性：横向扩展与负载均衡

---

## 🚀 快速开始

### 安装依赖

将 LumosAI 添加到你的 `Cargo.toml`：

```toml
[dependencies]
lumosai = "0.2.0"
tokio = { version = "1.0", features = ["full"] }
```

默认启用内存向量存储（`vector-memory`），无需额外配置即可运行 RAG 与向量示例。

### 基础用法

```rust
use lumosai::prelude::*;

/// 程序入口：演示 Agent 创建与对话
#[tokio::main]
async fn main() -> Result<()> {
    // 🤖 创建一个简单的 Agent
    let agent = Agent::builder()
        .name("assistant")
        .model("gpt-4")
        .system_prompt("You are a helpful AI assistant")
        .build()
        .await?;

    // 💬 进行对话
    let response = agent.chat("Hello, how are you?").await?;
    println!("Agent: {}", response);

    Ok(())
}
```

### 进阶示例：RAG 系统

```rust
use lumosai::prelude::*;

/// 程序入口：演示 RAG 系统创建与检索
#[tokio::main]
async fn main() -> Result<()> {
    // 📦 创建向量存储
    let storage = lumosai::vector::memory().await?;

    // 🧠 创建 RAG 系统
    let rag = lumosai::rag::builder()
        .storage(storage)
        .embedding_provider("openai")
        .chunking_strategy("recursive")
        .build()
        .await?;

    // 📄 添加文档
    rag.add_document("AI is transforming industries...").await?;

    // 🔍 搜索与生成
    let results = rag.search("What is AI?", 5).await?;
    println!("Found {} relevant documents", results.len());

    Ok(())
}
```

---

## 🧩 系统要求
- `Rust` >= `1.70`（工作区使用 Rust 2021 edition）
- `tokio` 1.x（异步运行时）
- 支持 macOS、Linux、Windows（当前主要在 macOS 验证）
- 可选：如启用集成，需具备外部模型提供商访问权限

## 💡 示例

我们的示例套件覆盖真实使用场景：

| 示例 | 描述 | 复杂度 |
|------|------|--------|
| [🤖 基础 Agent](examples/basic_agent.rs) | Agent 创建与对话 | ⭐ |
| [🧠 RAG 系统](examples/rag_system.rs) | 文档处理与检索 | ⭐⭐ |
| [🛠️ 工具集成](examples/tool_integration.rs) | Agent 绑定工具 | ⭐⭐ |
| [💾 记忆系统](examples/memory_system.rs) | 会话记忆与上下文 | ⭐⭐ |
| [📊 向量存储](examples/vector_storage.rs) | 向量数据库操作 | ⭐⭐ |
| [🌊 流式响应](examples/streaming_response.rs) | 实时流式输出 | ⭐⭐⭐ |
| [👥 多 Agent 工作流](examples/multi_agent_workflow.rs) | 协作与编排模式 | ⭐⭐⭐ |
| [🚀 完整 API 演示](examples/simplified_api_complete_demo.rs) | 全功能演示 | ⭐⭐⭐⭐⭐ |

### 运行示例

```bash
# 基础 Agent 示例
cargo run --example basic_agent

# RAG 系统示例
cargo run --example rag_system

# 多 Agent 协作
cargo run --example multi_agent_workflow

# 完整 API 演示
cargo run --example simplified_api_complete_demo
```

---

## 🏗️ 架构

LumosAI 采用模块化分层架构，强调可扩展与可维护性：

```
┌─────────────────────────────────────────────────────────────┐
│                    应用层                                  │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │
│  │   Web UI    │ │     CLI     │ │     自定义应用          │ │
│  └─────────────┘ └─────────────┘ └─────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                     API 层                                  │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │
│  │  REST API   │ │  GraphQL    │ │      WebSocket API      │ │
│  └─────────────┘ └─────────────┘ └─────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                   服务层                                    │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │
│  │   Agents    │ │  Workflows  │ │      Authentication     │ │
│  │   Memory    │ │     RAG     │ │       Monitoring        │ │
│  │   Tools     │ │   Events    │ │       Security          │ │
│  └─────────────┘ └─────────────┘ └─────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                    核心层                                   │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │
│  │   Traits    │ │   Types     │ │       Utilities         │ │
│  │   Errors    │ │   Config    │ │       Macros            │ │
│  └─────────────┘ └─────────────┘ └─────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                基础设施层                                   │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │
│  │  Databases  │ │   Storage   │ │      External APIs      │ │
│  │   Cache     │ │   Queues    │ │       Providers         │ │
│  └─────────────┘ └─────────────┘ └─────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

## 🧭 工作区总览
项目为 Rust 工作区，模块化拆分如下：

- `lumosai_core` — 核心 Traits、类型、配置与简化 API
- `lumosai_vector` — 统一向量存储层（默认启用内存后端）
- `lumosai_rag` — RAG 引擎与流水线
- `lumosai_cli` — 命令行工具与示例运行器
- `lumosai_evals` — 评测与基准工具（可选）
- `lumosai_network` — 网络与集成工具（可选）
- `lumosai_mcp` — MCP 集成（可选）
- `lumosai_auth` — 认证与安全基础能力
- `lumosai_enterprise` — 企业级功能（RBAC、多租户、审计）
- `lumosai_security` — 安全策略与工具
- `lumosai_telemetry` — 监控与可观测性
- `lumosai_voice` — 语音接口与音频处理
- `lumosai_bindings` — 语言绑定（Python/Node）与互操作
- `lumos_macro` — 过程宏与 DSL 支持
- `lumosai_derive` — 常用模式的派生宏
- `lumosai_multimodal` — 多模态处理工具
- `lumosai_examples` — 示例与演示集合

更多成员与暂时排除的包详见 `Cargo.toml`。

---

## 🧱 特性开关
通过 Cargo 特性开关灵活裁剪构建（摘自 `Cargo.toml`）：

- `default = ["integrations", "vector-memory"]`
- `integrations` — 启用 HTTP 客户端集成（如基于 `reqwest` 的远端 API）
- `vector-memory` — 启用内存向量存储（`lumosai_vector/memory`）
- （暂未启用）`vector-qdrant`、`vector-weaviate`、`vector-postgres` — 其他向量数据库

在你的 `Cargo.toml` 中启用/禁用：

```toml
[dependencies]
lumosai = { version = "0.2.0", features = ["integrations", "vector-memory"] }
```

## 📚 文档

- [📖 文档索引](docs/index.md) — 英文索引入口
- [🌏 中文文档索引](docs/README_CN.md) — 中文概览入口
- [🚀 快速开始](docs/quick-start/README.md) — 规范入口
- [🧠 向量数据库集成](docs/VECTOR_DATABASES.md)
- [📊 向量 API 参考](docs/vector_api_reference.md)

---

## 📜 更新日志与版本发布
- 发布记录：`docs/releases/`
- 更新历史：`docs/updates/`
- 发布配置：`release.toml`

遵循语义化版本管理。详情参阅 `docs/RELEASE_GUIDE.md`。

## 🤝 参与贡献

欢迎任何形式的贡献！包括修复问题、添加功能、改进文档与示例等。

```bash
# 克隆仓库
git clone https://github.com/louloulin/lumos.ai.git
cd lumosai

# 构建
cargo build

# 测试
cargo test

# 运行示例
cargo run --example basic_agent

# 代码质量
cargo clippy
cargo fmt --check
```

---

## 🌟 社区与支持

- **💬 Discord**：加入我们的社区讨论
- **🐛 Issues**：在 GitHub Issues 提交问题与建议
- **📖 文档**：查看完整文档与示例

---

## 📄 许可协议

本项目基于 **MIT License** 开源，详见 `LICENSE` 文件。

<div align="center">

**如果 LumosAI 对你有帮助，欢迎点亮 GitHub Star！**

[⭐ Star](https://github.com/louloulin/lumos.ai) | [🐛 报告问题](https://github.com/louloulin/lumos.ai/issues) | [💡 请求功能](https://github.com/louloulin/lumos.ai/issues)

</div>