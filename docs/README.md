# 📚 LumosAI 文档中心

> 🚀 **企业级 AI 应用开发框架** - 基于 Rust 的高性能、类型安全、可扩展的 AI 框架

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Documentation](https://img.shields.io/badge/docs-latest-brightgreen.svg)](docs/index.md)
[![Crates.io](https://img.shields.io/crates/v/lumosai.svg)](https://crates.io/crates/lumosai)

## 🚀 快速开始

### 新手入门
如果你是 LumosAI 新手，推荐按以下路径学习：

1. **[安装指南](getting-started/installation.md)** - 5分钟完成环境搭建
2. **[快速体验](getting-started/quick-start.md)** - 创建你的第一个 AI Agent
3. **[基础教程](tutorials/basics/)** - 掌握核心概念和用法

### 快速链接
- [🎯 5分钟快速开始](getting-started/quick-start.md)
- [📖 核心概念](concepts/README.md)
- [🔧 API 参考](api-reference/README.md)
- [💡 示例代码](resources/examples/README.md)

---

## 📋 文档导航

### 🚀 Getting Started (快速开始)
适合新用户，快速上手 LumosAI

- [**安装指南**](getting-started/installation.md) - 详细安装说明
- [**快速开始**](getting-started/quick-start.md) - 5分钟体验 LumosAI
- [**第一个 Agent**](getting-started/first-agent.md) - 创建并运行 AI Agent
- [**故障排除**](getting-started/troubleshooting.md) - 常见问题解决方案

### 💡 Concepts (核心概念)
理解 LumosAI 的核心架构和设计理念

- [**架构概览**](concepts/architecture.md) - 系统整体架构
- [**Agent 系统**](concepts/agents.md) - AI Agent 详解
- [**RAG 引擎**](concepts/rag.md) - 检索增强生成
- [**内存系统**](concepts/memory.md) - 对话与内存管理
- [**工作流**](concepts/workflows.md) - 多Agent协作流程

### 📚 Tutorials (教程)
循序渐进的实践教程，从基础到高级

#### 基础教程
- [**Agent 基础**](tutorials/basics/agent-basics.md) - 创建和配置 Agent
- [**RAG 基础**](tutorials/basics/rag-basics.md) - 构建知识问答系统
- [**工具集成**](tutorials/basics/tool-integration.md) - 扩展 Agent 能力

#### 进阶教程
- [**多 Agent 协作**](tutorials/intermediate/multi-agent.md) - Agent 团队协作
- [**自定义工具**](tutorials/intermediate/custom-tools.md) - 开发自定义工具
- [**性能优化**](tutorials/intermediate/performance.md) - 提升系统性能

#### 高级教程
- [**企业部署**](tutorials/advanced/enterprise.md) - 生产环境部署
- [**监控日志**](tutorials/advanced/monitoring.md) - 系统监控与日志
- [**扩展开发**](tutorials/advanced/extensions.md) - 框架扩展开发

### 🔧 API Reference (API参考)
完整的 API 文档和接口说明

- [**API 概览**](api-reference/README.md) - API 使用指南
- [**Agent API**](api-reference/agents.md) - Agent 相关接口
- [**RAG API**](api-reference/rag.md) - RAG 相关接口
- [**Memory API**](api-reference/memory.md) - 内存管理接口
- [**Tools API**](api-reference/tools.md) - 工具系统接口
- [**Workflow API**](api-reference/workflows.md) - 工作流接口

### 📋 Guides (专题指南)
特定主题的深度指南

- [**部署指南**](guides/deployment/) - 多环境部署方案
- [**安全指南**](guides/security/) - 安全最佳实践
- [**集成指南**](guides/integration/) - 第三方系统集成
- [**迁移指南**](guides/migration/) - 从其他框架迁移

### 📦 Resources (资源)
实用工具和参考资料

- [**示例代码**](resources/examples/) - 丰富的示例集合
- [**项目模板**](resources/templates/) - 快速启动模板
- [**术语表**](resources/glossary.md) - 专业术语解释
- [**常见问题**](resources/faq.md) - FAQ 解答

### 🤝 Contributing (贡献)
参与 LumosAI 开发和文档贡献

- [**开发环境**](contributing/development.md) - 搭建开发环境
- [**文档贡献**](contributing/documentation.md) - 文档贡献指南
- [**发布流程**](contributing/release-process.md) - 版本发布流程

---

## 🎯 按使用场景导航

### 我想要...
- **快速体验** → [快速开始](getting-started/quick-start.md)
- **构建 AI Agent** → [Agent 基础教程](tutorials/basics/agent-basics.md)
- **添加知识库** → [RAG 基础教程](tutorials/basics/rag-basics.md)
- **部署到生产** → [企业部署指南](tutorials/advanced/enterprise.md)
- **查找 API** → [API 参考](api-reference/README.md)
- **解决问题** → [故障排除](getting-started/troubleshooting.md)

### 我是...
- **🆕 新手** → [安装指南](getting-started/installation.md) → [快速开始](getting-started/quick-start.md)
- **👨‍💻 开发者** → [核心概念](concepts/README.md) → [API 参考](api-reference/README.md)
- **🏗️ 架构师** → [架构概览](concepts/architecture.md) → [部署指南](guides/deployment/)
- **🔧 运维** → [监控指南](tutorials/advanced/monitoring.md) → [故障排除](getting-started/troubleshooting.md)

---

## 📊 项目信息

### 版本信息
- **当前版本**: v0.2.0
- **Rust 版本**: 1.70+
- **支持平台**: Linux, macOS, Windows

### 核心特性
- 🤖 **智能 Agent 系统** - 支持多种 LLM 提供商
- 🧠 **高级 RAG 引擎** - 检索增强生成
- 🔄 **工作流编排** - 多Agent协作
- 💾 **向量存储** - 多种向量数据库支持
- 🛡️ **企业级安全** - 认证、授权、审计
- ⚡ **高性能** - 基于 Rust 的零成本抽象

### 快速命令
```bash
# 安装 LumosAI
cargo add lumosai

# 创建简单 Agent
use lumosai::prelude::*;

let agent = lumosai::agent::simple("gpt-4", "You are helpful").await?;

# 开始对话
let response = agent.chat("Hello!").await?;
```

---

## 🔗 外部链接

- **GitHub 仓库**: https://github.com/louloulin/lumos.ai
- **Crates.io**: https://crates.io/crates/lumosai
- **API 文档**: https://docs.rs/lumosai
- **更新日志**: [CHANGELOG.md](CHANGELOG.md)

---

## 🆘 获取帮助

遇到问题？这里有多种方式获取帮助：

- **📖 查看文档**: 浏览本文档中心
- **🔍 搜索**: 使用页面搜索功能
- **❓ FAQ**: 查看[常见问题](resources/faq.md)
- **🐛 报告问题**: 在 GitHub 提交 Issue
- **💬 社区讨论**: 参与社区交流

---

<div align="center">

**🌟 如果 LumosAI 对你有帮助，请给我们一个 Star！**

[GitHub](https://github.com/louloulin/lumos.ai) | [Crates.io](https://crates.io/crates/lumosai) | [API Docs](https://docs.rs/lumosai)

</div>