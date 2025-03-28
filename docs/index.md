# Lomusai 文档

欢迎使用Lomusai文档！Lomusai是一个用Rust实现的AI Agent框架，专注于性能、安全性和可扩展性。

## 目录

### 入门指南

- [快速开始](./quickstart.md)
- [安装指南](./installation.md)
- [基本概念](./concepts.md)

### 核心功能

- [代理 (Agents)](./agents.md)
- [工具 (Tools)](./tools.md)
- [LLM适配器](./llm_adapters.md)
- [内存系统](./memory.md)
- [RAG (检索增强生成)](./rag.md)
- [工作流引擎](./workflows.md)
- [评估框架](./eval.md)
- [MCP集成](./mcp.md)

### 宏与DSL

- [过程宏概述](./macros.md)
- [DSL宏详解](./dsl_macros.md)
- [自定义宏扩展](./custom_macros.md)

### 示例与教程

- [构建问答代理](./tutorials/qa_agent.md)
- [创建RAG应用](./tutorials/rag_app.md)
- [实现多代理工作流](./tutorials/multi_agent_workflow.md)
- [评估代理性能](./tutorials/agent_eval.md)

### API参考

- [lomusai_core API](./api/core.md)
- [lumos_macro API](./api/macros.md)

### 高级主题

- [性能优化](./advanced/performance.md)
- [安全最佳实践](./advanced/security.md)
- [可扩展架构](./advanced/scalability.md)
- [与其他系统集成](./advanced/integration.md)

### 其他

- [常见问题解答](./faq.md)
- [贡献指南](./contributing.md)
- [版本历史](./changelog.md)

## Lomusai框架架构

Lomusai框架由以下主要组件组成：

```
lomusai_core/           # 核心库
├── agent/             # 代理抽象和实现
├── tool/              # 工具抽象和实现
├── llm/               # LLM适配器
├── memory/            # 内存和状态管理
├── rag/               # 检索增强生成
├── eval/              # 评估框架
├── workflow/          # 工作流引擎
└── mcp/               # MCP客户端

lumos_macro/            # 宏库
├── tool_macro.rs      # 工具宏实现
├── agent_macro.rs     # 代理宏实现
├── llm_adapter_macro.rs # LLM适配器宏实现
├── workflow.rs        # 工作流DSL
├── rag.rs             # RAG管道DSL
├── eval.rs            # 评估套件DSL
└── mcp.rs             # MCP客户端DSL
```

## 系统要求

- Rust 1.70+
- Cargo
- 支持的操作系统: Linux, macOS, Windows

## 相关资源

- [GitHub仓库](https://github.com/yourusername/lomusai)
- [Crates.io页面](https://crates.io/crates/lomusai_core)
- [API文档](https://docs.rs/lomusai_core) 