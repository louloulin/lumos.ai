# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

**LumosAI** 是一个企业级AI应用开发框架，使用 Rust 作为核心，结合 TypeScript/JavaScript 构建前端界面。该项目支持 RAG 系统、多 Agent 协作、工作流编排等企业级 AI 应用功能。

### 文档编写原则
- 避免使用营销性词汇（如 "powerful", "built-in", "complete", "out-of-the-box"）
- 避免过度热情的号召性用语（如 "Check out", "Learn more", "Explore"）
- 避免模糊的营销术语（如 "production-ready", "makes it easy", "automatically handles"）
- 为工程师编写，专注于技术细节和实现原理

## 开发环境设置

### 前置要求
- Rust 1.75+
- Node.js 18+
- pnpm 8+

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
pnpm dev                       # 启动开发服务器
pnpm dev:ui                   # UI 开发模式
pnpm build:ui                 # 构建 UI
pnpm build:all                # 构建所有包
pnpm test                     # 运行前端测试
```

## 项目架构

### Workspace 结构
项目采用 Cargo workspace 管理 20 个核心包（实际成员以 Cargo.toml 为准）：

```
lumosai/
├── lumosai_core/              # 核心框架 (Agent, RAG, 工作流)
├── lumosai_cli/               # 命令行工具
├── lumosai_ui/                # Web UI (Dioxus + Axum)
├── lumosai_ui/web-server/     # Web UI 服务器
├── lumosai_vector/            # 向量数据库抽象层
├── lumosai_rag/               # RAG 系统实现
├── lumosai_network/           # 网络通信层
├── lumosai_mcp/               # Model Context Protocol
├── lumosai_bindings/          # 多语言绑定 (Python, Node.js, WASM)
├── lumosai_enterprise/        # 企业级功能
├── lumosai_evals/             # 评估框架
├── lumosai_marketplace/       # 市场功能
├── lumosai_examples/          # 示例代码
├── lumos_macro/              # 宏定义
├── lumosai_derive/           # 派生宏
└── mastra/                   # 独立的 TypeScript monorepo
```

### 分层架构
```
应用层: Web UI + CLI + 自定义应用
API层: REST API + GraphQL + WebSocket API  
服务层: Agents + Workflows + Memory + Tools + RAG + Security
核心层: Traits + Types + Config + Utilities
基础设施层: Databases + Storage + Queues + External APIs
```

### 核心模块职责

**lumosai_core**: 框架核心，定义 Agent、Workflow、Tool 等基础 trait 和类型
**lumosai_vector**: 向量数据库抽象层，支持 Qdrant、Weaviate、PostgreSQL 等多种后端
**lumosai_rag**: RAG 系统实现，包含文档处理、嵌入生成、检索算法
**lumosai_mcp**: Model Context Protocol 实现，支持模型上下文协议
**lumosai_enterprise**: 企业级功能，包含认证、授权、多租户、监控等
**lumosai_ui**: 基于 Dioxus 的 Web UI 框架

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
示例代码位于 `examples/` 目录，包含 12 个核心示例：
- basic_agent - 基本 Agent 使用
- rag_system - RAG 系统配置
- tool_integration - 工具集成
- memory_system - 内存系统
- vector_storage - 向量存储
- streaming_response - 流式响应
- multi_agent_workflow - 多 Agent 工作流
- enhanced_features_demo - 增强功能演示
- performance_benchmark - 性能基准测试
- auth_demo - 认证演示
- monitoring_demo_simple - 监控演示
- simplified_api_complete_demo - 完整 API 演示

运行示例：`cargo run --example <example_name>`

## 贡献指南

### 代码审查
- 所有 PR 必须通过 CI/CD 检查
- 代码必须通过所有测试和代码质量检查
- 大型功能更改需要先创建 issue 讨论

### 发布流程
- 使用 `cargo-release` 进行版本管理
- 遵循语义化版本控制
- 发布前必须运行完整测试套件
- 版本号定义在根目录 Cargo.toml 中（当前版本：0.1.4）
- 使用 release.toml 配置发布参数

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