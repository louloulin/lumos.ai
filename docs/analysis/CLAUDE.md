# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

**LumosAI** 是一个企业级AI应用开发框架，使用 Rust 作为核心，结合 TypeScript/JavaScript 构建前端界面。该项目支持 RAG 系统、多 Agent 协作、工作流编排等企业级 AI 应用功能。

当前版本：v0.2.0（开发中），v0.1.4（稳定发布版）

### 文档编写原则
- 避免使用营销性词汇（如 "powerful", "built-in", "complete", "out-of-the-box"）
- 避免过度热情的号召性用语（如 "Check out", "Learn more", "Explore"）
- 避免模糊的营销术语（如 "production-ready", "makes it easy", "automatically handles"）
- 为工程师编写，专注于技术细节和实现原理

## 开发环境设置

### 前置要求
- Rust 1.75+ （推荐使用最新稳定版）
- Node.js 18+
- pnpm 8+ 或 bun （项目使用 bun 作为包管理器）
- Git

### 环境初始化
```bash
# 安装 Rust 依赖
cargo build

# 安装前端依赖
pnpm install

# 运行所有测试确保环境正常
./scripts/run_tests.sh
```

## 核心开发命令

### Rust 核心开发
```bash
# 基础开发命令
cargo build                    # 构建所有 workspace
cargo build --release          # 生产构建
cargo test                     # 运行所有测试
cargo clippy --all-targets     # 代码质量检查
cargo fmt --all                # 代码格式化

# 测试命令
./scripts/run_tests.sh         # 综合测试脚本（包含所有测试套件）
./scripts/run_tests.sh unit    # 仅单元测试
./scripts/run_tests.sh integration # 仅集成测试
./scripts/run_tests.sh examples # 验证示例
./scripts/run_tests.sh performance # 性能测试
./scripts/run_tests.sh coverage # 代码覆盖率测试
./scripts/run_tests.sh quality # 代码质量检查（fmt + clippy）

# 性能测试
cargo bench                    # 基准测试
cargo nextest run              # 并行测试运行器
```

### 前端开发 (package.json)
```bash
# 使用 bun（推荐）
bun run dev                   # 启动开发服务器
bun run dev:ui               # UI 开发模式
bun run build:ui             # 构建 UI
bun run build:all            # 构建所有包

# 使用 pnpm
pnpm dev                     # 启动开发服务器
pnpm dev:ui                  # UI 开发模式
pnpm build:ui                # 构建 UI
pnpm build:all               # 构建所有包
pnpm test                    # 运行前端测试
```

## 项目架构

### Workspace 结构
项目采用 Cargo workspace 管理多个核心包（实际成员以根目录 Cargo.toml 为准）：

**当前活跃包 (22个)**：
```
lumosai/
├── lumosai_core/              # 核心框架 (Agent, Workflow, Tool, Memory, LLM)
├── lumosai_vector/            # 向量数据库抽象层
│   ├── core/                  # 向量存储核心接口
│   ├── memory/                # 内存向量存储
│   ├── lancedb/               # LanceDB 集成
│   ├── fastembed/             # FastEmbed 集成
│   ├── milvus/                # Milvus 集成
│   ├── qdrant/                # Qdrant 集成
│   ├── weaviate/              # Weaviate 集成
│   └── postgres/              # PostgreSQL 集成
├── lumosai_rag/               # RAG 系统实现
├── lumosai_cli/               # 命令行工具
├── lumosai_mcp/               # Model Context Protocol
├── lumosai_network/           # 网络通信层
├── lumosai_enterprise/        # 企业级功能
├── lumosai_evals/             # 评估框架
├── lumosai_examples/          # 示例代码
├── lumos_macro/              # 宏定义
├── lumosai_derive/           # 派生宏
├── lumosai_multimodal/       # 多模态支持
├── lumosai_bindings/         # 多语言绑定
├── lumosai_auth/             # 认证模块
├── lumosai_security/         # 安全模块
├── lumosai_telemetry/        # 监控模块
├── lumosai_voice/            # 语音模块
├── lumosai_cloud/            # 云服务集成
└── mastra/                   # 独立的 TypeScript monorepo
```

**暂时排除的包**：
- `lumosai_ui/` 和 `lumosai_ui/web-server/` (UI 相关包有 LabelRole 错误)
- `lumosai_vector/postgres/` (依赖问题)
- `lumosai_marketplace/` (复杂性)
- `lumosai_ai_extensions/` (重构中)

### 分层架构
```
应用层: Web UI + CLI + 自定义应用
API层: REST API + GraphQL + WebSocket API  
服务层: Agents + Workflows + Memory + Tools + RAG + Security
核心层: Traits + Types + Config + Utilities
基础设施层: Databases + Storage + Queues + External APIs
```

### 核心模块职责

**lumosai_core**: 框架核心，经过 v2.0 重构后包含 8 个核心模块：
- `agent`: Agent 核心功能（构建器、配置、执行器、协作）
- `workflow`: 工作流引擎（步骤、执行引擎、构建器）  
- `tool`: 工具系统（注册表、上下文、内置工具集）
- `memory`: 内存管理（工作内存、语义内存、会话管理）
- `llm`: LLM 抽象（OpenAI、Anthropic、Qwen、Zhipu 等提供商）
- `config`: 配置管理（YAML 配置、验证器）
- `error`: 错误处理（友好错误、分类错误）
- `prelude`: 便捷导入

**lumosai_vector**: 向量数据库抽象层，支持多种后端：
- 内存存储（用于开发和测试）
- LanceDB（高性能向量数据库）
- Qdrant、Weaviate、Milvus、PostgreSQL

**lumosai_rag**: RAG 系统实现，包含：
- 文档处理（PDF、Markdown、网页）
- 智能分块（递归、语义分块）
- 嵌入生成（OpenAI、Zhipu 提供商）
- 混合检索（语义 + 关键词匹配）

**lumosai_mcp**: Model Context Protocol 实现

**lumosai_enterprise**: 企业级功能（认证、授权、多租户、监控）

**lumosai_cli**: 命令行工具，支持 `dev`、`build`、`run`、`ui` 等子命令

## 开发规范

### Rust 代码规范
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 进行代码质量检查
- 遵循 Rust API 指南 (RFC 2841)
- 所有 public API 必须有文档注释
- 使用 `thiserror` 处理错误，`anyhow` 用于应用层错误传播

### TypeScript 代码规范
- 遵循 TypeScript 严格模式
- 使用 ESLint 和 Prettier 进行代码检查和格式化
- 前端组件使用 React hooks 模式

### 测试规范
- 所有核心功能必须有单元测试
- 集成测试覆盖跨模块交互
- 性能测试使用 criterion 基准测试框架
- 代码覆盖率目标：核心模块 > 90%，整体 > 80%
- 使用 `./scripts/run_tests.sh` 运行完整测试套件
- 示例代码必须能够正常编译和运行

## 构建和部署

### 本地开发
```bash
# 启动完整开发环境
cargo run --bin lumosai-ui     # 启动 Web UI
cargo run --bin lumosai-cli    # 使用 CLI 工具

# 开发模式监控
cargo watch -x build           # 监听文件变化并构建
```

### 生产构建
```bash
# 构建优化版本
cargo build --release

# 构建所有平台目标
./scripts/build-all.sh

# 运行测试确保质量
./scripts/run_tests.sh && cargo clippy
```

## 安全考虑

### 依赖管理
- 使用 `cargo-deny` 进行依赖安全审计
- 定期更新依赖：`cargo update`
- 使用 `cargo-audit` 检查安全漏洞

### 代码安全
- 所有外部输入必须进行验证和清理
- 敏感数据使用环境变量或安全存储
- 认证和授权使用企业级安全标准

## 调试和故障排除

### 常见问题
1. **编译错误**: 检查 Rust 版本兼容性，运行 `cargo update`
2. **测试失败**: 使用 `cargo test -- --nocapture` 查看详细输出
3. **依赖冲突**: 检查 workspace 中的依赖版本一致性

### 调试工具
```bash
# 启用调试日志
RUST_LOG=debug cargo run

# 使用 lldb 进行 Rust 调试
rust-lldb target/debug/lumosai-ui

# 性能分析
cargo flamegraph
```

## 文档和示例

### 文档生成
```bash
# 生成 API 文档
cargo doc --no-deps --open

# 构建 Markdown 文档
pnpm docs:build
```

### 示例项目
示例代码位于 `examples/` 目录，包含 50+ 个示例文件，涵盖：
- **基础示例**: basic_agent, rag_system, tool_integration, memory_system
- **高级示例**: multi_agent_workflow, enhanced_features_demo, performance_benchmark
- **验证示例**: *_validation.rs (各种功能的验证测试)
- **演示示例**: auth_demo, monitoring_demo_simple, simplified_api_complete_demo
- **工具示例**: tool_macro_demo, unified_memory_demo, friendly_errors_demo

运行示例：`cargo run --example <example_name>`

**重要示例**：
- `simplified_api_complete_demo.rs`: 完整 API 演示（⭐⭐⭐⭐⭐）
- `enhanced_features_demo.rs`: 增强功能演示（⭐⭐⭐）
- `performance_benchmark.rs`: 性能基准测试（⭐⭐⭐）

## 贡献指南

### 代码审查
- 所有 PR 必须通过 CI/CD 检查
- 代码必须通过所有测试和代码质量检查
- 大型功能更改需要先创建 issue 讨论

### 发布流程
- 使用 `cargo-release` 进行版本管理
- 遵循语义化版本控制
- 发布前必须运行完整测试套件
- 版本号定义：
  - 开发版本：v0.2.0（根目录 Cargo.toml）
  - 稳定版本：v0.1.4（README.md 中引用）
- 使用 `release.toml` 配置发布参数
- 发布脚本：`./scripts/release.sh`

## 测试架构

### 测试分类
- **单元测试**: `cargo test --lib` - 测试单个模块功能
- **集成测试**: `cargo test --tests integration` - 测试模块间交互
- **性能测试**: `cargo test --tests performance --release` - 性能基准测试
- **示例验证**: 自动验证所有示例代码能够正常运行

### 测试工具
- **cargo-tarpaulin**: 代码覆盖率分析（需要单独安装）
- **cargo-nextest**: 并行测试运行器，提高测试速度
- **criterion**: 性能基准测试框架
- **mockall**: Mock 框架，用于单元测试

### 测试配置
- 覆盖率阈值：80%（在 run_tests.sh 中配置）
- 性能基线：300秒（性能测试超时时间）
- 测试报告：生成在 `target/test-reports/` 目录
- 覆盖率报告：生成在 `target/coverage/` 目录

## 架构洞察和开发模式

### 项目状态
- **当前分支**: lumosai-simple（开发分支）
- **主分支**: main（用于 PR 合并）
- **最近提交**: 已完成 46 个新测试，P0-3 任务进行中

### 核心设计模式
1. **分层抽象**: 核心 trait 定义在 lumosai_core，具体实现在各个专门包中
2. **插件化架构**: 通过 trait 实现可插拔的组件（LLM 提供商、向量数据库等）
3. **宏驱动开发**: 使用过程宏简化工具创建和配置（`#[tool]` 宏）
4. **渐进式 API**: 提供从简单到复杂的三层 API 设计

### 依赖管理策略
- 使用 Arrow 生态系统（54.0.0 版本）进行数据处理
- LanceDB（0.18.0）用于向量存储
- 严格控制 zstd 版本以避免冲突
- Workspace 统一管理依赖版本

### 构建系统特点
- 支持特性门控（features）编译
- 排除问题包以保持构建稳定性
- 优化的 release 配置（lto=true, codegen-units=1）
- 支持调试信息的 release-with-debug profile

## 常见开发场景

### 添加新的 LLM 提供商
1. 在 `lumosai_core/src/llm/` 下创建新的提供商文件
2. 实现 `LlmProvider` trait
3. 在 `mod.rs` 中导出
4. 在 `providers.rs` 中注册

### 添加新的工具
1. 使用 `#[tool]` 宏创建工具函数
2. 或者在 `lumosai_core/src/tool/builtin/` 下添加新工具
3. 在 `mod.rs` 中注册工具

### 调试技巧
```bash
# 启用详细日志
RUST_LOG=debug cargo run --example basic_agent

# 检查特定包的编译
cargo check -p lumosai_core

# 运行特定测试
cargo test --test agent_tests -- --nocapture
```