# LumosAI v0.3.0 Crates 目录重构计划

> **版本**: v0.3.0
> **创建日期**: 2025-11-02
> **状态**: 🚧 Phase 1 完成，Phase 2 进行中
> **优先级**: P1（Week 7-12 任务）
> **最后更新**: 2025-11-02

---

## 📋 目录

1. [执行摘要](#1-执行摘要)
2. [当前架构分析](#2-当前架构分析)
3. [目标架构设计](#3-目标架构设计)
4. [迁移计划](#4-迁移计划)
5. [实施步骤](#5-实施步骤)
6. [风险评估](#6-风险评估)
7. [验收标准](#7-验收标准)

---

## 1. 执行摘要

### 1.1 改造目标

将 LumosAI 项目从**扁平化包结构**重构为**分层 crates 架构**，提升代码组织性、可维护性和模块化程度。

**核心目标**:
- ✅ 清晰的分层架构（Core → Services → Applications）
- ✅ 独立的 crate 版本管理
- ✅ 更好的依赖隔离
- ✅ 简化的构建和测试流程
- ✅ 对标 Mastra 的包组织方式

### 1.2 预期成果

**改造前** (当前状态):
```
lumosai/
├── lumosai_core/              # 核心框架（153 个 .rs 文件）
├── lumosai_vector/            # 向量存储（8 个子包）
├── lumosai_rag/               # RAG 系统
├── lumosai_cli/               # CLI 工具
├── lumosai_mcp/               # MCP 协议
├── lumosai_enterprise/        # 企业功能
├── lumosai_auth/              # 认证
├── lumosai_security/          # 安全
├── lumosai_telemetry/         # 监控
├── lumosai_multimodal/        # 多模态
├── lumosai_voice/             # 语音
├── lumosai_cloud/             # 云服务
├── lumosai_network/           # 网络
├── lumosai_evals/             # 评估
├── lumosai_examples/          # 示例
├── lumos_macro/               # 宏
├── lumosai_derive/            # 派生宏
├── lumosai_bindings/          # 多语言绑定
├── lumosai_ai_extensions/     # AI 扩展
├── lumosai_marketplace/       # 市场
└── lumosai_ui/                # UI（有问题）
```

**改造后** (目标状态):
```
lumosai/
├── crates/
│   ├── core/                  # 核心层（4 个 crates）
│   │   ├── lumosai-types/     # 核心类型和 Traits
│   │   ├── lumosai-error/     # 错误处理
│   │   ├── lumosai-config/    # 配置管理
│   │   └── lumosai-logger/    # 日志系统
│   │
│   ├── runtime/               # 运行时层（7 个 crates）
│   │   ├── lumosai-agent/     # Agent 系统
│   │   ├── lumosai-workflow/  # 工作流引擎
│   │   ├── lumosai-tool/      # 工具系统
│   │   ├── lumosai-memory/    # 内存管理
│   │   ├── lumosai-llm/       # LLM 抽象
│   │   ├── lumosai-rag/       # RAG 系统
│   │   └── lumosai-vector/    # 向量存储
│   │
│   ├── services/              # 服务层（6 个 crates）
│   │   ├── lumosai-mcp/       # MCP 协议
│   │   ├── lumosai-network/   # 网络通信
│   │   ├── lumosai-auth/      # 认证授权
│   │   ├── lumosai-security/  # 安全模块
│   │   ├── lumosai-telemetry/ # 监控遥测
│   │   └── lumosai-enterprise/# 企业功能
│   │
│   ├── extensions/            # 扩展层（4 个 crates）
│   │   ├── lumosai-multimodal/# 多模态
│   │   ├── lumosai-voice/     # 语音处理
│   │   ├── lumosai-cloud/     # 云服务
│   │   └── lumosai-evals/     # 评估框架
│   │
│   ├── integrations/          # 集成层（未来扩展）
│   │   └── README.md          # 第三方集成占位
│   │
│   ├── tools/                 # 工具层（3 个 crates）
│   │   ├── lumosai-cli/       # CLI 工具
│   │   ├── lumosai-macro/     # 宏系统
│   │   └── lumosai-derive/    # 派生宏
│   │
│   └── bindings/              # 绑定层（1 个 crate）
│       └── lumosai-bindings/  # 多语言绑定
│
├── examples/                  # 示例代码（保持不变）
├── docs/                      # 文档（保持不变）
├── scripts/                   # 脚本（保持不变）
└── Cargo.toml                 # Workspace 配置
```

### 1.3 关键指标

| 指标 | 当前值 | 目标值 | 改进 |
|------|--------|--------|------|
| **顶层包数量** | 21 个 | 6 个分类目录 | -71% |
| **依赖层次** | 扁平化 | 4 层清晰分层 | +300% |
| **构建时间** | ~120s | ~90s | -25% |
| **测试隔离度** | 低 | 高 | +200% |
| **文档清晰度** | 6/10 | 9/10 | +50% |

---

## 2. 当前架构分析

### 2.1 包清单和分类

**当前活跃包** (21 个):

#### 核心层 (1 个)
- `lumosai_core` - 核心框架（153 个 .rs 文件，包含 8 个核心模块）
  - agent, workflow, tool, memory, llm, config, error, prelude
  - **问题**: 单一包过于庞大，职责不清晰

#### 服务层 (13 个)
- `lumosai_vector` - 向量存储（8 个子包：core, memory, lancedb, qdrant, weaviate, milvus, fastembed, postgres）
- `lumosai_rag` - RAG 系统
- `lumosai_cli` - CLI 工具
- `lumosai_mcp` - MCP 协议
- `lumosai_network` - 网络通信
- `lumosai_auth` - 认证授权
- `lumosai_security` - 安全模块
- `lumosai_telemetry` - 监控遥测
- `lumosai_enterprise` - 企业功能
- `lumosai_multimodal` - 多模态支持
- `lumosai_voice` - 语音处理
- `lumosai_cloud` - 云服务集成
- `lumosai_evals` - 评估框架

#### 工具层 (3 个)
- `lumos_macro` - 宏系统
- `lumosai_derive` - 派生宏
- `lumosai_bindings` - 多语言绑定

#### 示例层 (1 个)
- `lumosai_examples` - 示例代码

#### 排除的包 (3 个)
- `lumosai_ui` - UI 界面（有 LabelRole 错误）
- `lumosai_marketplace` - 市场功能（复杂性高）
- `lumosai_ai_extensions` - AI 扩展（重构中）

### 2.2 核心问题分析

#### 问题 1: lumosai_core 过于庞大

**当前状态**:
```
lumosai_core/src/
├── agent/          # 33 个子模块，职责混杂
├── workflow/       # 工作流引擎
├── tool/           # 工具系统（包含 builtin 工具）
├── memory/         # 内存管理
├── llm/            # LLM 抽象
├── config/         # 配置管理
├── error/          # 错误处理
├── logger/         # 日志系统
├── telemetry/      # 遥测系统
├── prelude/        # 便捷导入
├── compat/         # 兼容层
├── app/            # 应用层（待移除）
├── base/           # 基础组件
├── lumosai/        # 旧 API（待移除）
├── rag/            # RAG（应该在 lumosai_rag）
├── types/          # 类型定义
├── unified_api/    # 统一 API
└── vector/         # 向量（应该在 lumosai_vector）
```

**问题**:
- ❌ 单一包包含 153 个 .rs 文件
- ❌ 职责不清晰（core 包含了 rag 和 vector）
- ❌ 编译时间长（修改任何文件都需要重新编译整个 core）
- ❌ 测试耦合度高（无法独立测试各个模块）
- ❌ 依赖关系混乱（循环依赖风险）

#### 问题 2: 扁平化包结构

**当前状态**:
```
lumosai/
├── lumosai_core/
├── lumosai_vector/
├── lumosai_rag/
├── lumosai_cli/
├── lumosai_mcp/
├── ... (16 more packages)
```

**问题**:
- ❌ 21 个顶层包，难以导航
- ❌ 没有清晰的分层架构
- ❌ 依赖关系不明确
- ❌ 新手难以理解项目结构

#### 问题 3: 依赖关系混乱

**当前依赖图** (简化):
```
lumosai_core
  ├─> lumosai_vector (❌ 应该反向依赖)
  └─> lumosai_rag (❌ 应该反向依赖)

lumosai_rag
  ├─> lumosai_core
  └─> lumosai_vector

lumosai_cli
  ├─> lumosai_core
  ├─> lumosai_vector
  └─> lumosai_rag
```

**问题**:
- ❌ lumosai_core 依赖 lumosai_vector 和 lumosai_rag（应该反向）
- ❌ 循环依赖风险
- ❌ 无法独立发布各个包

### 2.3 对标分析：Mastra 的包组织

**Mastra 的成功经验**:
```
mastra/
├── packages/
│   ├── core/          # 核心框架
│   ├── memory/        # 内存处理
│   ├── rag/          # RAG 系统
│   ├── cli/          # CLI 工具
│   ├── auth/         # 认证
│   ├── evals/        # 评估
│   ├── mcp/          # MCP 协议
│   ├── server/       # 服务器
│   ├── cloud/        # 云部署
│   └── deployer/     # 部署工具
├── stores/           # 16+ 向量存储
├── integrations/     # 外部集成
├── workflows/        # 工作流模板
└── voice/           # 语音处理
```

**关键特点**:
- ✅ 清晰的 `packages/` 目录分类
- ✅ 独立的 `stores/` 目录（向量存储）
- ✅ 独立的 `integrations/` 目录
- ✅ 每个包职责单一
- ✅ 依赖关系清晰

---

## 3. 目标架构设计

### 3.1 分层架构原则

```
┌─────────────────────────────────────────┐
│         Applications Layer              │  用户应用
│  (CLI, UI, Examples, Custom Apps)       │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│         Extensions Layer                │  可选扩展
│  (Multimodal, Voice, Cloud, Evals)      │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│         Services Layer                  │  企业服务
│  (MCP, Network, Auth, Security, etc.)   │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│         Runtime Layer                   │  核心运行时
│  (Agent, Workflow, Tool, Memory, etc.)  │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│         Core Layer                      │  基础设施
│  (Types, Error, Config, Logger)         │
└─────────────────────────────────────────┘
```

**依赖规则**:
- ✅ 上层可以依赖下层
- ❌ 下层不能依赖上层
- ❌ 同层之间尽量避免依赖

### 3.2 Crates 目录结构

详见第 1.2 节的目标状态。

### 3.3 核心 Crates 拆分方案

#### 3.3.1 Core Layer (4 个 crates)

**lumosai-types** (新建):
```rust
// crates/core/lumosai-types/src/lib.rs
pub mod agent;      // AgentTrait, AgentStatus
pub mod workflow;   // WorkflowTrait, StepTrait
pub mod tool;       // ToolTrait, ToolSchema
pub mod memory;     // MemoryTrait
pub mod llm;        // LlmProviderTrait, Message, Role
pub mod common;     // 通用类型
```

**lumosai-error** (从 lumosai_core 提取):
```rust
// crates/core/lumosai-error/src/lib.rs
pub mod error;      // Error enum
pub mod result;     // Result type
pub mod friendly;   // 友好错误消息
```

**lumosai-config** (从 lumosai_core 提取):
```rust
// crates/core/lumosai-config/src/lib.rs
pub mod loader;     // 配置加载
pub mod validator;  // 配置验证
pub mod types;      // 配置类型
```

**lumosai-logger** (从 lumosai_core 提取):
```rust
// crates/core/lumosai-logger/src/lib.rs
pub mod logger;     // Logger trait
pub mod console;    // 控制台日志
pub mod file;       // 文件日志
```

#### 3.3.2 Runtime Layer (7 个 crates)

**lumosai-agent** (从 lumosai_core 提取):
```rust
// crates/runtime/lumosai-agent/src/lib.rs
pub mod builder;        // AgentBuilder
pub mod executor;       // BasicAgent
pub mod collaboration;  // 多 Agent 协作
pub mod streaming;      // 流式响应
pub mod session;        // 会话管理
```

**lumosai-workflow** (从 lumosai_core 提取):
```rust
// crates/runtime/lumosai-workflow/src/lib.rs
pub mod workflow;       // Workflow trait
pub mod step;           // Step trait
pub mod engine;         // 执行引擎
pub mod builder;        // WorkflowBuilder
```

**lumosai-tool** (从 lumosai_core 提取):
```rust
// crates/runtime/lumosai-tool/src/lib.rs
pub mod tool;           // Tool trait
pub mod registry;       // ToolRegistry
pub mod builtin;        // 内置工具
pub mod enhanced;       // 增强工具
```

**lumosai-memory** (从 lumosai_core 提取):
```rust
// crates/runtime/lumosai-memory/src/lib.rs
pub mod working;        // 工作内存
pub mod semantic;       // 语义内存
pub mod session;        // 会话内存
pub mod thread;         // 线程内存
```

**lumosai-llm** (从 lumosai_core 提取):
```rust
// crates/runtime/lumosai-llm/src/lib.rs
pub mod provider;       // LlmProvider trait
pub mod openai;         // OpenAI
pub mod anthropic;      // Anthropic
pub mod qwen;           // Qwen
pub mod zhipu;          // Zhipu
```

**lumosai-rag** (保持独立):
```rust
// crates/runtime/lumosai-rag/src/lib.rs
pub mod document;       // 文档处理
pub mod chunking;       // 分块
pub mod embedding;      // 嵌入
pub mod retriever;      // 检索器
```

**lumosai-vector** (保持独立，但移动到 crates/runtime):
```rust
// crates/runtime/lumosai-vector/src/lib.rs
pub mod core;           // 核心抽象
pub mod memory;         // 内存存储
pub mod lancedb;        // LanceDB
pub mod qdrant;         // Qdrant
pub mod weaviate;       // Weaviate
pub mod milvus;         // Milvus
```

---

## 4. 迁移计划

### 4.1 迁移阶段

**Phase 1: 准备阶段** (Week 7, 5 天) - ✅ **已完成**
- [x] 创建 `crates/` 目录结构（33 个目录，25 个 crates）
- [x] 创建文档（crates/README.md, integrations/README.md）
- [x] 创建验证脚本（scripts/verify_crates_structure.sh）
- [x] 验证目录结构（35/35 检查通过）
- [ ] 设置新的 Workspace 配置（待 Phase 2 开始时完成）
- [ ] 建立测试基准（待 Phase 2 开始时完成）

**Phase 2: Core Layer 迁移** (Week 8, 5 天)
- [ ] 提取 lumosai-types
- [ ] 提取 lumosai-error
- [ ] 提取 lumosai-config
- [ ] 提取 lumosai-logger
- [ ] 更新所有依赖

**Phase 3: Runtime Layer 迁移** (Week 9-10, 10 天)
- [ ] 拆分 lumosai-agent
- [ ] 拆分 lumosai-workflow
- [ ] 拆分 lumosai-tool
- [ ] 拆分 lumosai-memory
- [ ] 拆分 lumosai-llm
- [ ] 移动 lumosai-rag
- [ ] 移动 lumosai-vector

**Phase 4: Services Layer 迁移** (Week 11, 5 天)
- [ ] 移动 lumosai-mcp
- [ ] 移动 lumosai-network
- [ ] 移动 lumosai-auth
- [ ] 移动 lumosai-security
- [ ] 移动 lumosai-telemetry
- [ ] 移动 lumosai-enterprise

**Phase 5: Extensions & Tools 迁移** (Week 12, 5 天)
- [ ] 移动 lumosai-multimodal
- [ ] 移动 lumosai-voice
- [ ] 移动 lumosai-cloud
- [ ] 移动 lumosai-evals
- [ ] 移动 lumosai-cli
- [ ] 移动 lumosai-macro
- [ ] 移动 lumosai-derive
- [ ] 移动 lumosai-bindings

**Phase 6: 清理和验证** (Week 12, 2 天)
- [ ] 删除旧的顶层包目录
- [ ] 更新所有文档
- [ ] 运行完整测试套件
- [ ] 性能基准测试
- [ ] 发布 v0.3.0

### 4.2 迁移优先级

**P0 (必须完成)**:
- Core Layer 迁移
- Runtime Layer 迁移
- 测试通过率 100%

**P1 (重要)**:
- Services Layer 迁移
- 文档更新

**P2 (可选)**:
- Extensions Layer 迁移
- 性能优化

---

## 5. 实施步骤

### 5.1 Phase 1: 准备阶段 (Week 7)

#### Day 1: 创建目录结构

```bash
# 创建 crates 目录结构
mkdir -p crates/{core,runtime,services,extensions,tools,bindings}
mkdir -p crates/core/{lumosai-types,lumosai-error,lumosai-config,lumosai-logger}
mkdir -p crates/runtime/{lumosai-agent,lumosai-workflow,lumosai-tool,lumosai-memory,lumosai-llm,lumosai-rag,lumosai-vector}
mkdir -p crates/services/{lumosai-mcp,lumosai-network,lumosai-auth,lumosai-security,lumosai-telemetry,lumosai-enterprise}
mkdir -p crates/extensions/{lumosai-multimodal,lumosai-voice,lumosai-cloud,lumosai-evals}
mkdir -p crates/tools/{lumosai-cli,lumosai-macro,lumosai-derive}
mkdir -p crates/bindings/lumosai-bindings
```

#### Day 2: 创建 Workspace 配置

```toml
# Cargo.toml (新的 workspace 配置)
[workspace]
members = [
    # Core Layer
    "crates/core/lumosai-types",
    "crates/core/lumosai-error",
    "crates/core/lumosai-config",
    "crates/core/lumosai-logger",
    
    # Runtime Layer
    "crates/runtime/lumosai-agent",
    "crates/runtime/lumosai-workflow",
    "crates/runtime/lumosai-tool",
    "crates/runtime/lumosai-memory",
    "crates/runtime/lumosai-llm",
    "crates/runtime/lumosai-rag",
    "crates/runtime/lumosai-vector",
    
    # Services Layer
    "crates/services/lumosai-mcp",
    "crates/services/lumosai-network",
    "crates/services/lumosai-auth",
    "crates/services/lumosai-security",
    "crates/services/lumosai-telemetry",
    "crates/services/lumosai-enterprise",
    
    # Extensions Layer
    "crates/extensions/lumosai-multimodal",
    "crates/extensions/lumosai-voice",
    "crates/extensions/lumosai-cloud",
    "crates/extensions/lumosai-evals",
    
    # Tools Layer
    "crates/tools/lumosai-cli",
    "crates/tools/lumosai-macro",
    "crates/tools/lumosai-derive",
    
    # Bindings Layer
    "crates/bindings/lumosai-bindings",
    
    # Examples
    "examples",
]

[workspace.dependencies]
# Core dependencies
lumosai-types = { path = "crates/core/lumosai-types" }
lumosai-error = { path = "crates/core/lumosai-error" }
lumosai-config = { path = "crates/core/lumosai-config" }
lumosai-logger = { path = "crates/core/lumosai-logger" }

# Runtime dependencies
lumosai-agent = { path = "crates/runtime/lumosai-agent" }
lumosai-workflow = { path = "crates/runtime/lumosai-workflow" }
lumosai-tool = { path = "crates/runtime/lumosai-tool" }
lumosai-memory = { path = "crates/runtime/lumosai-memory" }
lumosai-llm = { path = "crates/runtime/lumosai-llm" }
lumosai-rag = { path = "crates/runtime/lumosai-rag" }
lumosai-vector = { path = "crates/runtime/lumosai-vector" }

# ... (其他依赖)
```

#### Day 3-5: 创建迁移脚本

创建自动化迁移脚本：
- `scripts/migrate_core_layer.sh`
- `scripts/migrate_runtime_layer.sh`
- `scripts/update_imports.py`
- `scripts/verify_migration.sh`

### 5.2 Phase 2: Core Layer 迁移 (Week 8)

#### Step 1: 创建 lumosai-types (Day 1-2)

```bash
# 创建新 crate
cd crates/core/lumosai-types
cargo init --lib

# 从 lumosai_core 提取类型定义
cp ../../../lumosai_core/src/agent/trait_def.rs src/agent.rs
cp ../../../lumosai_core/src/workflow/mod.rs src/workflow.rs
cp ../../../lumosai_core/src/tool/tool.rs src/tool.rs
cp ../../../lumosai_core/src/memory/mod.rs src/memory.rs
cp ../../../lumosai_core/src/llm/provider.rs src/llm.rs
```

**Cargo.toml**:
```toml
[package]
name = "lumosai-types"
version = "0.3.0"
edition = "2021"

[dependencies]
async-trait = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
```

#### Step 2: 创建 lumosai-error (Day 2)

```bash
cd crates/core/lumosai-error
cargo init --lib

# 从 lumosai_core 提取错误处理
cp ../../../lumosai_core/src/error/mod.rs src/lib.rs
```

#### Step 3: 创建 lumosai-config (Day 3)

```bash
cd crates/core/lumosai-config
cargo init --lib

# 从 lumosai_core 提取配置管理
cp ../../../lumosai_core/src/config/ src/
```

#### Step 4: 创建 lumosai-logger (Day 3)

```bash
cd crates/core/lumosai-logger
cargo init --lib

# 从 lumosai_core 提取日志系统
cp ../../../lumosai_core/src/logger/ src/
```

#### Step 5: 更新依赖和测试 (Day 4-5)

```bash
# 运行测试
cargo test --workspace

# 更新所有导入
python scripts/update_imports.py \
  --from "lumosai_core::error" \
  --to "lumosai_error"

# 验证迁移
./scripts/verify_migration.sh core
```

### 5.3 Phase 3: Runtime Layer 迁移 (Week 9-10)

#### Step 1: 拆分 lumosai-agent (Day 1-2)

**目录结构**:
```
crates/runtime/lumosai-agent/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── builder.rs       # 从 lumosai_core/src/agent/builder.rs
│   ├── executor.rs      # 从 lumosai_core/src/agent/executor.rs
│   ├── collaboration.rs # 从 lumosai_core/src/agent/collaboration.rs
│   ├── streaming.rs     # 从 lumosai_core/src/agent/streaming.rs
│   ├── session.rs       # 从 lumosai_core/src/agent/session.rs
│   └── config.rs        # 从 lumosai_core/src/agent/config.rs
└── tests/
    └── integration_tests.rs
```

**Cargo.toml**:
```toml
[package]
name = "lumosai-agent"
version = "0.3.0"
edition = "2021"

[dependencies]
lumosai-types = { workspace = true }
lumosai-error = { workspace = true }
lumosai-config = { workspace = true }
lumosai-logger = { workspace = true }
lumosai-llm = { workspace = true }
lumosai-tool = { workspace = true }
lumosai-memory = { workspace = true }

tokio = { workspace = true }
async-trait = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
```

#### Step 2: 拆分 lumosai-workflow (Day 3)

类似 lumosai-agent 的步骤。

#### Step 3: 拆分 lumosai-tool (Day 4)

类似 lumosai-agent 的步骤。

#### Step 4: 拆分 lumosai-memory (Day 5)

类似 lumosai-agent 的步骤。

#### Step 5: 拆分 lumosai-llm (Day 6-7)

类似 lumosai-agent 的步骤。

#### Step 6: 移动 lumosai-rag 和 lumosai-vector (Day 8-10)

```bash
# 移动 lumosai_rag
mv lumosai_rag crates/runtime/lumosai-rag

# 移动 lumosai_vector
mv lumosai_vector crates/runtime/lumosai-vector

# 更新 Cargo.toml 路径
sed -i 's|path = "lumosai_rag"|path = "crates/runtime/lumosai-rag"|g' Cargo.toml
sed -i 's|path = "lumosai_vector"|path = "crates/runtime/lumosai-vector"|g' Cargo.toml
```

### 5.4 Phase 4-5: Services & Extensions 迁移 (Week 11-12)

#### 批量移动脚本

```bash
#!/bin/bash
# scripts/migrate_services.sh

SERVICES=(
  "lumosai_mcp"
  "lumosai_network"
  "lumosai_auth"
  "lumosai_security"
  "lumosai_telemetry"
  "lumosai_enterprise"
)

for service in "${SERVICES[@]}"; do
  echo "Migrating $service..."
  mv "$service" "crates/services/${service//_/-}"

  # 更新 Cargo.toml
  sed -i "s|path = \"$service\"|path = \"crates/services/${service//_/-}\"|g" Cargo.toml
done

echo "Services migration complete!"
```

### 5.5 Phase 6: 清理和验证 (Week 12)

#### 清理旧目录

```bash
#!/bin/bash
# scripts/cleanup_old_structure.sh

# 确认所有测试通过
cargo test --workspace || exit 1

# 删除旧的顶层包目录（已移动到 crates/）
OLD_DIRS=(
  "lumosai_core"
  "lumosai_vector"
  "lumosai_rag"
  # ... 其他已迁移的目录
)

for dir in "${OLD_DIRS[@]}"; do
  if [ -d "$dir" ]; then
    echo "Removing old directory: $dir"
    rm -rf "$dir"
  fi
done

echo "Cleanup complete!"
```

#### 验证脚本

```bash
#!/bin/bash
# scripts/verify_migration.sh

echo "🔍 Verifying migration..."

# 1. 检查所有 crates 可以编译
echo "1. Checking compilation..."
cargo check --workspace || exit 1

# 2. 运行所有测试
echo "2. Running tests..."
cargo test --workspace || exit 1

# 3. 检查依赖关系
echo "3. Checking dependencies..."
cargo tree --workspace --depth 1

# 4. 检查循环依赖
echo "4. Checking for circular dependencies..."
cargo depgraph --workspace-only | grep -i "cycle" && exit 1

# 5. 运行 clippy
echo "5. Running clippy..."
cargo clippy --workspace --all-targets -- -D warnings || exit 1

# 6. 检查格式
echo "6. Checking formatting..."
cargo fmt --all -- --check || exit 1

echo "✅ Migration verification complete!"
```

---

## 6. 风险评估

### 6.1 技术风险

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| **循环依赖** | 高 | 中 | 严格遵循分层架构，使用依赖检查工具 |
| **测试失败** | 高 | 高 | 每个阶段完成后运行完整测试套件 |
| **性能下降** | 中 | 低 | 性能基准测试，优化编译配置 |
| **API 破坏** | 高 | 中 | 保留兼容层，渐进式迁移 |

### 6.2 项目风险

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| **时间超期** | 中 | 中 | 分阶段实施，优先 P0 任务 |
| **资源不足** | 中 | 低 | 自动化迁移脚本，减少手工工作 |
| **文档滞后** | 低 | 高 | 每个阶段同步更新文档 |

---

## 7. 验收标准

### 7.1 功能验收

- [ ] 所有测试通过（406 个测试，100% 通过率）
- [ ] 所有示例代码正常运行
- [ ] API 兼容性保持（或提供迁移指南）
- [ ] 文档完整更新

### 7.2 性能验收

- [ ] 编译时间 ≤ 90 秒（当前 ~120 秒）
- [ ] 测试运行时间 ≤ 60 秒（当前 ~66 秒）
- [ ] 内存占用无明显增加

### 7.3 质量验收

- [ ] Clippy 检查通过（0 warnings）
- [ ] 代码覆盖率 ≥ 50%
- [ ] 文档覆盖率 ≥ 80%
- [ ] 依赖关系清晰（无循环依赖）

---

## 8. 后续计划

### 8.1 v0.3.1 (Week 13-14)

- 优化构建性能
- 添加更多集成测试
- 完善文档和示例

### 8.2 v0.4.0 (Week 15-18)

- 实现 `integrations/` 目录
- 添加 50+ 第三方集成
- 实现插件系统

---

## 附录

### A. 迁移工具脚本

#### A.1 自动更新导入脚本

```python
#!/usr/bin/env python3
# scripts/update_imports.py

import os
import re
import sys
from pathlib import Path

# 导入映射表
IMPORT_MAPPINGS = {
    # Core Layer
    "lumosai_core::error": "lumosai_error",
    "lumosai_core::config": "lumosai_config",
    "lumosai_core::logger": "lumosai_logger",

    # Runtime Layer
    "lumosai_core::agent": "lumosai_agent",
    "lumosai_core::workflow": "lumosai_workflow",
    "lumosai_core::tool": "lumosai_tool",
    "lumosai_core::memory": "lumosai_memory",
    "lumosai_core::llm": "lumosai_llm",

    # Types
    "lumosai_core::types": "lumosai_types",
}

def update_imports_in_file(file_path):
    """更新单个文件中的导入语句"""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()

    original_content = content

    # 替换所有导入
    for old_import, new_import in IMPORT_MAPPINGS.items():
        # 匹配 use 语句
        pattern = rf'use\s+{re.escape(old_import)}(::|\s|;)'
        replacement = f'use {new_import}\\1'
        content = re.sub(pattern, replacement, content)

    # 如果内容有变化，写回文件
    if content != original_content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        return True
    return False

def main():
    """主函数"""
    workspace_root = Path(__file__).parent.parent
    updated_files = []

    # 遍历所有 .rs 文件
    for rs_file in workspace_root.rglob('*.rs'):
        # 跳过 target 目录
        if 'target' in rs_file.parts:
            continue

        if update_imports_in_file(rs_file):
            updated_files.append(rs_file)
            print(f"✅ Updated: {rs_file.relative_to(workspace_root)}")

    print(f"\n📊 Total files updated: {len(updated_files)}")

if __name__ == '__main__':
    main()
```

#### A.2 依赖关系检查脚本

```bash
#!/bin/bash
# scripts/check_dependencies.sh

echo "🔍 Checking dependency graph..."

# 安装 cargo-depgraph（如果未安装）
if ! command -v cargo-depgraph &> /dev/null; then
    echo "Installing cargo-depgraph..."
    cargo install cargo-depgraph
fi

# 生成依赖图
cargo depgraph --workspace-only \
    --dedup-transitive-deps \
    | dot -Tpng > dependency_graph.png

echo "✅ Dependency graph saved to dependency_graph.png"

# 检查循环依赖
if cargo depgraph --workspace-only | grep -i "cycle"; then
    echo "❌ Circular dependencies detected!"
    exit 1
else
    echo "✅ No circular dependencies found"
fi

# 检查分层架构
echo ""
echo "📊 Checking layer dependencies..."

# Core Layer 不应该依赖其他层
CORE_DEPS=$(cargo tree -p lumosai-types -p lumosai-error -p lumosai-config -p lumosai-logger \
    | grep -E "lumosai-(agent|workflow|tool|memory|llm|rag|vector|mcp|network)" || true)

if [ -n "$CORE_DEPS" ]; then
    echo "❌ Core layer has invalid dependencies:"
    echo "$CORE_DEPS"
    exit 1
else
    echo "✅ Core layer dependencies are valid"
fi

# Runtime Layer 不应该依赖 Services/Extensions
RUNTIME_DEPS=$(cargo tree -p lumosai-agent -p lumosai-workflow -p lumosai-tool \
    | grep -E "lumosai-(mcp|network|auth|security|multimodal|voice)" || true)

if [ -n "$RUNTIME_DEPS" ]; then
    echo "❌ Runtime layer has invalid dependencies:"
    echo "$RUNTIME_DEPS"
    exit 1
else
    echo "✅ Runtime layer dependencies are valid"
fi

echo ""
echo "✅ All dependency checks passed!"
```

#### A.3 性能基准测试脚本

```bash
#!/bin/bash
# scripts/benchmark_migration.sh

echo "📊 Running performance benchmarks..."

# 记录开始时间
START_TIME=$(date +%s)

# 1. 编译时间测试
echo "1. Testing compilation time..."
cargo clean
time cargo build --workspace --release 2>&1 | tee build_time.log

# 2. 测试运行时间
echo "2. Testing test execution time..."
time cargo test --workspace 2>&1 | tee test_time.log

# 3. 增量编译时间
echo "3. Testing incremental compilation..."
touch crates/core/lumosai-types/src/lib.rs
time cargo build --workspace 2>&1 | tee incremental_build_time.log

# 4. 包大小统计
echo "4. Checking package sizes..."
du -sh target/release/deps/* | sort -h > package_sizes.log

# 记录结束时间
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo ""
echo "✅ Benchmarks complete! Duration: ${DURATION}s"
echo "📄 Results saved to:"
echo "  - build_time.log"
echo "  - test_time.log"
echo "  - incremental_build_time.log"
echo "  - package_sizes.log"
```

### B. 迁移检查清单

#### B.1 Phase 2 检查清单 (Core Layer)

- [ ] **lumosai-types**
  - [ ] 所有 trait 定义已提取
  - [ ] 公共类型已提取
  - [ ] 测试通过
  - [ ] 文档完整

- [ ] **lumosai-error**
  - [ ] Error enum 已提取
  - [ ] Result type 已提取
  - [ ] 友好错误消息已提取
  - [ ] 测试通过

- [ ] **lumosai-config**
  - [ ] 配置加载器已提取
  - [ ] 配置验证器已提取
  - [ ] 测试通过

- [ ] **lumosai-logger**
  - [ ] Logger trait 已提取
  - [ ] 控制台日志已提取
  - [ ] 文件日志已提取
  - [ ] 测试通过

#### B.2 Phase 3 检查清单 (Runtime Layer)

- [ ] **lumosai-agent**
  - [ ] AgentBuilder 已提取
  - [ ] BasicAgent 已提取
  - [ ] 协作功能已提取
  - [ ] 流式响应已提取
  - [ ] 会话管理已提取
  - [ ] 所有测试通过（32 个 Agent 测试）

- [ ] **lumosai-workflow**
  - [ ] Workflow trait 已提取
  - [ ] Step trait 已提取
  - [ ] 执行引擎已提取
  - [ ] 所有测试通过（6 个 Workflow 测试）

- [ ] **lumosai-tool**
  - [ ] Tool trait 已提取
  - [ ] ToolRegistry 已提取
  - [ ] 内置工具已提取
  - [ ] 所有测试通过（7 个 Tool 测试）

- [ ] **lumosai-memory**
  - [ ] 工作内存已提取
  - [ ] 语义内存已提取
  - [ ] 会话内存已提取
  - [ ] 所有测试通过（9 个 Memory 测试）

- [ ] **lumosai-llm**
  - [ ] LlmProvider trait 已提取
  - [ ] 所有 Provider 已提取（OpenAI, Anthropic, Qwen, Zhipu）
  - [ ] 所有测试通过（20 个 LLM 测试）

- [ ] **lumosai-rag**
  - [ ] 已移动到 crates/runtime/
  - [ ] 路径更新完成
  - [ ] 测试通过

- [ ] **lumosai-vector**
  - [ ] 已移动到 crates/runtime/
  - [ ] 所有子包路径更新
  - [ ] 测试通过

#### B.3 验证检查清单

- [ ] **编译检查**
  - [ ] `cargo check --workspace` 通过
  - [ ] `cargo build --workspace` 通过
  - [ ] `cargo build --workspace --release` 通过

- [ ] **测试检查**
  - [ ] `cargo test --workspace` 通过
  - [ ] 测试通过率 = 100% (406/406)
  - [ ] 无测试超时

- [ ] **代码质量检查**
  - [ ] `cargo clippy --workspace --all-targets` 无警告
  - [ ] `cargo fmt --all -- --check` 通过
  - [ ] 代码覆盖率 ≥ 50%

- [ ] **依赖检查**
  - [ ] 无循环依赖
  - [ ] 分层架构正确
  - [ ] 依赖版本一致

- [ ] **文档检查**
  - [ ] README.md 已更新
  - [ ] CLAUDE.md 已更新
  - [ ] API 文档已更新
  - [ ] 示例代码已更新

### C. 常见问题和解决方案

#### C.1 循环依赖问题

**问题**: 迁移后出现循环依赖

**解决方案**:
1. 使用 `cargo depgraph` 识别循环
2. 将共享类型提取到 `lumosai-types`
3. 使用依赖注入打破循环

#### C.2 导入路径错误

**问题**: 大量导入路径需要更新

**解决方案**:
1. 使用 `scripts/update_imports.py` 自动更新
2. 使用 IDE 的全局搜索替换功能
3. 运行 `cargo check` 找出遗漏的导入

#### C.3 测试失败

**问题**: 迁移后部分测试失败

**解决方案**:
1. 检查测试依赖是否正确
2. 确认测试辅助函数已迁移
3. 更新测试中的导入路径

#### C.4 编译时间增加

**问题**: 迁移后编译时间变长

**解决方案**:
1. 启用增量编译
2. 使用 `sccache` 缓存编译结果
3. 优化依赖关系，减少不必要的依赖

### D. 参考资料

#### D.1 外部资源

- [Mastra 项目结构](https://github.com/mastra-ai/mastra)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Cargo Workspace 文档](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [Cargo Book - Large Projects](https://doc.rust-lang.org/cargo/guide/project-layout.html)

#### D.2 内部文档

- `lumos4.2.md` - Week 1-6 改造计划
- `CLAUDE.md` - 开发规范
- `LUMOSAI_ARCHITECTURE.md` - 架构文档
- `WEEK2_PROGRESS_REPORT.md` - Week 2 进度报告

#### D.3 相关工具

- `cargo-depgraph` - 依赖图可视化
- `cargo-tree` - 依赖树查看
- `cargo-modules` - 模块结构可视化
- `sccache` - 编译缓存

### E. 时间线和里程碑

| Week | 阶段 | 任务 | 交付物 | 状态 |
|------|------|------|--------|------|
| **Week 7** | Phase 1 | 准备阶段 | 目录结构、脚本、基准 | ⏸️ 待开始 |
| **Week 8** | Phase 2 | Core Layer 迁移 | 4 个 core crates | ⏸️ 待开始 |
| **Week 9** | Phase 3.1 | Runtime Layer 迁移（前半） | agent, workflow, tool | ⏸️ 待开始 |
| **Week 10** | Phase 3.2 | Runtime Layer 迁移（后半） | memory, llm, rag, vector | ⏸️ 待开始 |
| **Week 11** | Phase 4 | Services Layer 迁移 | 6 个 service crates | ⏸️ 待开始 |
| **Week 12** | Phase 5-6 | Extensions & 清理 | 4 个 extension crates + 验证 | ⏸️ 待开始 |

**关键里程碑**:
- ✅ **M1**: Core Layer 迁移完成（Week 8 结束）
- ✅ **M2**: Runtime Layer 迁移完成（Week 10 结束）
- ✅ **M3**: 所有迁移完成（Week 12 结束）
- ✅ **M4**: v0.3.0 发布（Week 12 结束）

---

**文档版本**: v1.0
**最后更新**: 2025-11-02
**维护者**: LumosAI Team
**审阅者**: Claude (Augment Agent)

---

## 快速开始

### 立即执行

如果您准备开始迁移，请按照以下步骤操作：

```bash
# 1. 创建新分支
git checkout -b feature/crates-migration

# 2. 运行准备脚本
./scripts/prepare_migration.sh

# 3. 开始 Phase 1
./scripts/migrate_phase1.sh

# 4. 验证
./scripts/verify_migration.sh

# 5. 提交
git add -A
git commit -m "feat: Phase 1 - Prepare crates directory structure"
```

### 获取帮助

如果遇到问题，请：
1. 查看 [常见问题](#c-常见问题和解决方案)
2. 运行 `./scripts/verify_migration.sh` 诊断问题
3. 查看 GitHub Issues
4. 联系维护团队

---

## 📊 实施进度追踪

### Phase 1: 准备阶段 - ✅ 完成 (2025-11-02)

**完成的任务**:
- ✅ 创建 crates/ 目录结构（33 个目录）
- ✅ 创建 25 个 crate 子目录
- ✅ 创建 crates/README.md 文档
- ✅ 创建 crates/integrations/README.md 文档
- ✅ 创建 scripts/verify_crates_structure.sh 验证脚本
- ✅ 验证目录结构（35/35 检查通过）

**提交记录**:
```
bb40f00 feat: Phase 1.1-1.3 - Create crates directory structure
```

**验证结果**:
```
Total directories: 33
Total checks: 35
Passed: 35
Failed: 0
✅ All checks passed!
```

**下一步**: Phase 2 - Core Layer 迁移

---

### Phase 2: Core Layer 迁移 - ⏸️ 待开始

**计划任务**:
- [ ] 提取 lumosai-types
- [ ] 提取 lumosai-error
- [ ] 提取 lumosai-config
- [ ] 提取 lumosai-logger
- [ ] 更新所有依赖

**预计时间**: 5 天

---

### Phase 3: Runtime Layer 迁移 - ⏸️ 待开始

**计划任务**:
- [ ] 拆分 lumosai-agent
- [ ] 拆分 lumosai-workflow
- [ ] 拆分 lumosai-tool
- [ ] 拆分 lumosai-memory
- [ ] 拆分 lumosai-llm
- [ ] 移动 lumosai-rag
- [ ] 移动 lumosai-vector

**预计时间**: 10 天

---

### Phase 4-6: 后续阶段 - ⏸️ 待开始

详见上文迁移计划。

---

**祝迁移顺利！** 🚀

