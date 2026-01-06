# 🌟 LumosAI 文档中心

<div align="center">

**企业级AI应用开发框架 - 基于Rust构建的高性能、类型安全、可扩展的AI框架**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Documentation](https://img.shields.io/badge/docs-latest-brightgreen.svg)](./index.md)
[![Crates.io](https://img.shields.io/crates/v/lumosai.svg)](https://crates.io/crates/lumosai)

[开始使用](./learn/getting-started/README.md) • [API文档](./reference/api/README.md) • [示例](./learn/examples/README.md) • [社区](./community/README.md)

</div>

---

## 🚀 快速导航

### 🎯 我是新用户
**5分钟快速体验 LumosAI**

1. **[安装指南](./learn/getting-started/installation.md)** - 5分钟完成环境搭建
2. **[快速开始](./learn/getting-started/quick-start.md)** - 创建你的第一个 AI Agent
3. **[基础教程](./learn/tutorials/README.md)** - 掌握核心概念和用法

### 📚 我是开发者
**深入理解框架架构和API**

- **[核心概念](./learn/guides/concepts.md)** - 理解 LumosAI 的设计理念
- **[API参考](./reference/api/README.md)** - 完整的接口文档
- **[架构指南](./reference/architecture/overview.md)** - 系统架构和设计原理

### 🔧 我想查看示例
**实际可运行的代码示例**

- **[基础示例](./learn/examples/README.md)** - Agent、RAG、工具集成
- **[高级示例](./learn/examples/advanced.md)** - 多Agent协作、企业级功能
- **[最佳实践](./learn/guides/best-practices.md)** - 生产环境使用建议

---

## 📖 文档架构

### 🧭 学习路径 (Learn)
循序渐进的学习资源，从入门到精通

| 节 | 内容 | 时间 | 描述 |
|-----|------|------|------|
| [Getting Started](./learn/getting-started/README.md) | 环境搭建 + 快速体验 | 15分钟 | 安装配置并创建第一个Agent |
| [基础教程](./learn/tutorials/basics/README.md) | 核心概念 + 基础功能 | 2小时 | Agent、RAG、内存、工具 |
| [进阶教程](./learn/tutorials/advanced/README.md) | 多Agent协作 + 工作流 | 4小时 | 复杂场景的解决方案 |
| [专题指南](./learn/guides/README.md) | 生产实践 + 最佳实践 | 6小时 | 企业级应用指南 |

### 📋 参考文档 (Reference)
完整的技术参考和API文档

- **[API文档](./reference/api/README.md)** - 所有API的详细说明
- **[CLI工具](./reference/cli/README.md)** - 命令行工具使用指南
- **[配置参考](./reference/configuration/README.md)** - 配置选项和说明
- **[架构文档](./reference/architecture/README.md)** - 系统架构和技术原理

### 🤝 贡献指南 (Contribute)
参与项目开发和文档贡献

- **[开发指南](./contribute/development.md)** - 搭建开发环境
- **[文档贡献](./contribute/documentation.md)** - 文档贡献指南
- **[行为准则](./contribute/code-of-conduct.md)** - 社区行为准则

### 👥 社区资源 (Community)
获取帮助和参与社区讨论

- **[常见问题](./community/faq.md)** - FAQ 解答
- **[GitHub Issues](https://github.com/louloulin/lumos.ai/issues)** - 问题报告和功能请求
- **[社区讨论](https://github.com/louloulin/lumos.ai/discussions)** - 技术讨论和经验分享

---

## ✨ 核心特性

### 🤖 智能Agent系统
- **多模型支持**: OpenAI GPT、Anthropic Claude、本地模型
- **专业Agent**: 研究、写作、分析、自定义角色
- **工具集成**: 可扩展的工具系统，内置丰富工具
- **对话内存**: 持久化上下文和对话历史

### 🧠 高级RAG引擎
- **文档处理**: PDF、文本、Markdown、网页内容
- **智能分块**: 递归、语义、自定义分块策略
- **向量存储**: 内存、PostgreSQL、Qdrant、Weaviate后端
- **混合检索**: 语义搜索 + 关键词匹配

### 🔄 工作流编排
- **多Agent协作**: 顺序、并行、条件工作流
- **任务管理**: 复杂任务分解和执行
- **事件驱动**: 实时事件处理和路由
- **错误处理**: 健壮的重试机制和故障转移

### 🛡️ 企业级安全
- **身份认证**: JWT、OAuth2、API密钥、多因子认证
- **权限控制**: 基于角色的访问控制(RBAC)
- **多租户**: 隔离的租户环境和自定义配置
- **审计日志**: 完整的安全和合规日志

### ⚡ 高性能设计
- **Rust性能**: 内存安全、零成本抽象
- **异步并发**: 非阻塞I/O，高并发支持
- **智能缓存**: 多层缓存机制
- **水平扩展**: 负载均衡和分布式部署

---

## 🎯 按场景导航

### 我想构建...
- **AI助手** → [Agent基础教程](./learn/tutorials/basics/agent-basics.md)
- **知识问答** → [RAG系统教程](./learn/tutorials/basics/rag-system.md)
- **多Agent协作** → [高级工作流](./learn/tutorials/advanced/workflows.md)
- **企业应用** → [生产部署指南](./learn/guides/deployment.md)

### 我遇到了问题...
- **安装问题** → [安装故障排除](./learn/getting-started/troubleshooting.md)
- **配置问题** → [配置参考](./reference/configuration/README.md)
- **性能问题** → [性能优化指南](./learn/guides/performance.md)
- **安全问题** → [安全配置指南](./learn/guides/security.md)

---

## 📦 快速体验

### 最小化安装
```bash
# 添加到 Cargo.toml
cargo add lumosai
```

### 创建第一个Agent
```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 🤖 创建简单Agent
    let agent = Agent::builder()
        .name("assistant")
        .model("gpt-4")
        .system_prompt("You are a helpful AI assistant")
        .build()
        .await?;

    // 💬 开始对话
    let response = agent.chat("Hello, how are you?").await?;
    println!("Agent: {}", response);

    Ok(())
}
```

### Docker快速启动
```bash
# 启动本地开发服务
./scripts/quick-start.sh

# 运行示例
cargo run --example basic_agent
```

---

## 📊 项目信息

**当前版本**: v0.2.0  
**Rust版本**: 1.70+  
**支持平台**: Linux, macOS, Windows  
**代码仓库**: [github.com/louloulin/lumos.ai](https://github.com/louloulin/lumos.ai)

---

## 🔗 资源链接

- **GitHub仓库**: https://github.com/louloulin/lumos.ai
- **Crates.io**: https://crates.io/crates/lumosai
- **API文档**: https://docs.rs/lumosai
- **更新日志**: [CHANGELOG.md](../releases/CHANGELOG.md)

---

## 🆘 获取帮助

遇到问题？我们提供多种支持方式：

- 📖 **查看文档**: 浏览本文档中心
- 🔍 **搜索功能**: 使用页面搜索
- ❓ **FAQ**: 查看[常见问题](./community/faq.md)
- 🐛 **报告问题**: 在GitHub提交Issue
- 💬 **社区讨论**: 参与技术讨论

---

<div align="center">

**⭐ 如果 LumosAI 对你有帮助，请给我们一个 Star！**

[GitHub](https://github.com/louloulin/lumos.ai) • [Crates.io](https://crates.io/crates/lumosai) • [API Docs](https://docs.rs/lumosai)

**Built with ❤️ by the LumosAI team**

</div>