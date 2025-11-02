# LumosAI Crates 目录

> **版本**: v0.3.0  
> **状态**: 🚧 迁移中  
> **最后更新**: 2025-11-02

---

## 📋 目录结构

```
crates/
├── core/          # 核心层（4 个 crates）
│   ├── lumosai-types/     # 核心类型和 Traits
│   ├── lumosai-error/     # 错误处理
│   ├── lumosai-config/    # 配置管理
│   └── lumosai-logger/    # 日志系统
│
├── runtime/       # 运行时层（7 个 crates）
│   ├── lumosai-agent/     # Agent 系统
│   ├── lumosai-workflow/  # 工作流引擎
│   ├── lumosai-tool/      # 工具系统
│   ├── lumosai-memory/    # 内存管理
│   ├── lumosai-llm/       # LLM 抽象
│   ├── lumosai-rag/       # RAG 系统
│   └── lumosai-vector/    # 向量存储
│
├── services/      # 服务层（6 个 crates）
│   ├── lumosai-mcp/       # MCP 协议
│   ├── lumosai-network/   # 网络通信
│   ├── lumosai-auth/      # 认证授权
│   ├── lumosai-security/  # 安全模块
│   ├── lumosai-telemetry/ # 监控遥测
│   └── lumosai-enterprise/# 企业功能
│
├── extensions/    # 扩展层（4 个 crates）
│   ├── lumosai-multimodal/# 多模态
│   ├── lumosai-voice/     # 语音处理
│   ├── lumosai-cloud/     # 云服务
│   └── lumosai-evals/     # 评估框架
│
├── tools/         # 工具层（3 个 crates）
│   ├── lumosai-cli/       # CLI 工具
│   ├── lumosai-macro/     # 宏系统
│   └── lumosai-derive/    # 派生宏
│
└── bindings/      # 绑定层（1 个 crate）
    └── lumosai-bindings/  # 多语言绑定
```

---

## 🎯 分层架构

```
┌─────────────────────────────────────────┐
│         Applications Layer              │  CLI, UI, Examples
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│         Extensions Layer                │  Multimodal, Voice, Cloud, Evals
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│         Services Layer                  │  MCP, Network, Auth, Security
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│         Runtime Layer                   │  Agent, Workflow, Tool, Memory, LLM
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│         Core Layer                      │  Types, Error, Config, Logger
└─────────────────────────────────────────┘
```

**依赖规则**:
- ✅ 上层可以依赖下层
- ❌ 下层不能依赖上层
- ❌ 同层之间尽量避免依赖

---

## 📦 Crates 说明

### Core Layer (核心层)

#### lumosai-types
- **职责**: 核心类型定义和 Traits
- **包含**: AgentTrait, WorkflowTrait, ToolTrait, MemoryTrait, LlmProviderTrait
- **依赖**: 无（最底层）
- **状态**: ⏸️ 待迁移

#### lumosai-error
- **职责**: 错误处理和 Result 类型
- **包含**: Error enum, Result type, 友好错误消息
- **依赖**: 无
- **状态**: ⏸️ 待迁移

#### lumosai-config
- **职责**: 配置管理和验证
- **包含**: 配置加载器、验证器、配置类型
- **依赖**: lumosai-error
- **状态**: ⏸️ 待迁移

#### lumosai-logger
- **职责**: 日志系统
- **包含**: Logger trait, 控制台日志, 文件日志
- **依赖**: lumosai-error
- **状态**: ⏸️ 待迁移

---

### Runtime Layer (运行时层)

#### lumosai-agent
- **职责**: Agent 系统实现
- **包含**: AgentBuilder, BasicAgent, 协作功能, 流式响应
- **依赖**: lumosai-types, lumosai-error, lumosai-llm, lumosai-tool, lumosai-memory
- **状态**: ⏸️ 待迁移

#### lumosai-workflow
- **职责**: 工作流引擎
- **包含**: Workflow trait, Step trait, 执行引擎
- **依赖**: lumosai-types, lumosai-error
- **状态**: ⏸️ 待迁移

#### lumosai-tool
- **职责**: 工具系统
- **包含**: Tool trait, ToolRegistry, 内置工具
- **依赖**: lumosai-types, lumosai-error
- **状态**: ⏸️ 待迁移

#### lumosai-memory
- **职责**: 内存管理
- **包含**: 工作内存, 语义内存, 会话内存
- **依赖**: lumosai-types, lumosai-error
- **状态**: ⏸️ 待迁移

#### lumosai-llm
- **职责**: LLM 抽象层
- **包含**: LlmProvider trait, OpenAI, Anthropic, Qwen, Zhipu
- **依赖**: lumosai-types, lumosai-error
- **状态**: ⏸️ 待迁移

#### lumosai-rag
- **职责**: RAG 系统
- **包含**: 文档处理, 分块, 嵌入, 检索器
- **依赖**: lumosai-types, lumosai-error, lumosai-vector
- **状态**: ⏸️ 待迁移

#### lumosai-vector
- **职责**: 向量存储
- **包含**: 核心抽象, 内存存储, LanceDB, Qdrant, Weaviate, Milvus
- **依赖**: lumosai-types, lumosai-error
- **状态**: ⏸️ 待迁移

---

### Services Layer (服务层)

#### lumosai-mcp
- **职责**: Model Context Protocol
- **依赖**: lumosai-types, lumosai-error, lumosai-network
- **状态**: ⏸️ 待迁移

#### lumosai-network
- **职责**: 网络通信
- **依赖**: lumosai-types, lumosai-error
- **状态**: ⏸️ 待迁移

#### lumosai-auth
- **职责**: 认证授权
- **依赖**: lumosai-types, lumosai-error
- **状态**: ⏸️ 待迁移

#### lumosai-security
- **职责**: 安全模块
- **依赖**: lumosai-types, lumosai-error
- **状态**: ⏸️ 待迁移

#### lumosai-telemetry
- **职责**: 监控遥测
- **依赖**: lumosai-types, lumosai-error
- **状态**: ⏸️ 待迁移

#### lumosai-enterprise
- **职责**: 企业功能
- **依赖**: lumosai-types, lumosai-error, lumosai-auth, lumosai-security
- **状态**: ⏸️ 待迁移

---

### Extensions Layer (扩展层)

#### lumosai-multimodal
- **职责**: 多模态支持
- **依赖**: lumosai-types, lumosai-error, lumosai-llm
- **状态**: ⏸️ 待迁移

#### lumosai-voice
- **职责**: 语音处理
- **依赖**: lumosai-types, lumosai-error
- **状态**: ⏸️ 待迁移

#### lumosai-cloud
- **职责**: 云服务集成
- **依赖**: lumosai-types, lumosai-error
- **状态**: ⏸️ 待迁移

#### lumosai-evals
- **职责**: 评估框架
- **依赖**: lumosai-types, lumosai-error, lumosai-agent
- **状态**: ⏸️ 待迁移

---

### Tools Layer (工具层)

#### lumosai-cli
- **职责**: 命令行工具
- **依赖**: 所有 runtime 和 services crates
- **状态**: ⏸️ 待迁移

#### lumosai-macro
- **职责**: 宏系统
- **依赖**: 无
- **状态**: ⏸️ 待迁移

#### lumosai-derive
- **职责**: 派生宏
- **依赖**: 无
- **状态**: ⏸️ 待迁移

---

### Bindings Layer (绑定层)

#### lumosai-bindings
- **职责**: 多语言绑定
- **依赖**: 所有 runtime crates
- **状态**: ⏸️ 待迁移

---

## 📅 迁移进度

| 阶段 | 任务 | 状态 |
|------|------|------|
| **Phase 1** | 准备阶段 | 🚧 进行中 |
| **Phase 2** | Core Layer 迁移 | ⏸️ 待开始 |
| **Phase 3** | Runtime Layer 迁移 | ⏸️ 待开始 |
| **Phase 4** | Services Layer 迁移 | ⏸️ 待开始 |
| **Phase 5** | Extensions & Tools 迁移 | ⏸️ 待开始 |
| **Phase 6** | 清理和验证 | ⏸️ 待开始 |

---

## 📚 相关文档

- `../lumos4.4.md` - Crates 重构计划
- `../CODEBASE_ANALYSIS.md` - 代码库分析报告
- `../LUMOS4.4_SUMMARY.md` - 任务总结
- `../CLAUDE.md` - 开发规范

---

**最后更新**: 2025-11-02  
**维护者**: LumosAI Team

