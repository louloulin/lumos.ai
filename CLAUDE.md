# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

LumosAI 是一个用 Rust 构建的企业级 AI 框架，提供完整的 RAG 系统、Agent 框架、多 Agent 编排和事件驱动架构。项目采用模块化工作空间设计，包含 20+ 个独立包。

## 核心开发命令

### 构建和测试
```bash
# 构建整个工作空间
cargo build --workspace

# 运行所有测试
cargo test --workspace

# 运行特定包的测试
cargo test -p lumosai_core
cargo test -p lumosai_vector

# 运行示例
cargo run --example basic_agent
cargo run --example rag_system
cargo run --example multi_agent_workflow

# 运行完整测试套件（推荐）
./scripts/run_tests.sh
```

### 代码质量检查
```bash
# 代码格式化
cargo fmt

# Clippy 检查
cargo clippy --workspace -- -D warnings

# 文档生成
cargo doc --workspace --no-deps

# 测试覆盖率（需要安装 cargo-tarpaulin）
cargo install cargo-tarpaulin
cargo tarpaulin --workspace --out Html --output-dir target/coverage
```

### Docker 开发
```bash
# 启动开发环境服务
./scripts/quick-start.sh

# 构建 Docker 镜像
docker build -t lumosai .

# 使用 Docker Compose 启动向量数据库
docker-compose -f docker-compose.vector-dbs.yml up
```

## 架构概览

### 工作空间结构
- **lumosai_core** - 核心类型、特征、错误处理和配置管理
- **lumosai_vector** - 统一向量存储层（默认启用内存存储）
- **lumosai_rag** - 检索增强生成引擎和管道
- **lumosai_cli** - 命令行工具和示例运行器
- **lumosai_auth** - 身份验证和安全原语
- **lumosai_enterprise** - 企业级功能（RBAC、多租户、审计）
- **lumosai_telemetry** - 监控和可观测性
- **lumos_macro** - 过程宏和 DSL 支持
- **lumosai_derive** - 常见模式的派生宏

### 暂时排除的包
```toml
# 当前版本中排除的包（有编译问题或需要重构）
exclude = [
    "lumosai_vector/postgres",
    "lumosai_marketplace", 
    "lumosai_ui",
    "lumosai_bindings"
]
```

### 关键特性标志
- **default = ["integrations", "vector-memory"]**
- **integrations** - 启用 HTTP 客户端集成
- **vector-memory** - 启用内存向量存储（默认）

## 简化 API

项目提供简化的 API 便于快速开发：

```rust
use lumosai::prelude::*;

// 创建简单 Agent
let agent = lumosai::agent::simple("gpt-4", "You are helpful").await?;

// 创建向量存储
let storage = lumosai::vector::memory().await?;

// 创建 RAG 系统
let rag = lumosai::rag::simple(storage, "openai").await?;
```

## 测试策略

### 测试分类
- **单元测试**: 在各包的 `tests/` 目录中
- **集成测试**: `tests/` 目录下的跨包测试
- **示例验证**: 运行 `examples/` 中的示例程序
- **性能测试**: 使用 `--release` 模式的性能基准测试

### 测试运行方式
```bash
# 运行所有测试套件
./scripts/run_tests.sh

# 运行特定类型的测试
./scripts/run_tests.sh unit          # 仅单元测试
./scripts/run_tests.sh integration   # 仅集成测试
./scripts/run_tests.sh examples      # 仅示例验证
./scripts/run_tests.sh performance   # 仅性能测试
./scripts/run_tests.sh coverage      # 仅覆盖率测试
```

### API 密钥相关测试
某些测试需要外部 API 密钥，如果未设置会被跳过：
```bash
export DEEPSEEK_API_KEY=your_key_here
export OPENAI_API_KEY=your_key_here
```

## 常见开发模式

### 添加新示例
1. 在 `examples/` 目录创建新的 `.rs` 文件
2. 将示例添加到 `Cargo.toml` 的 `[[example]]` 部分
3. 运行 `cargo run --example your_example_name`

### 修复编译错误
1. 先运行 `cargo check --workspace` 查看所有错误
2. 按错误类型分组修复（API 不匹配、生命周期等）
3. 每修复一类错误后立即验证
4. 最后运行完整的测试套件

### 调试技巧
```bash
# 查看详细编译输出
RUST_LOG=debug cargo build

# 显示错误回溯
RUST_BACKTRACE=1 cargo test

# 运行单个测试进行调试
cargo test test_name -- --nocapture
```

## 项目状态

- **当前版本**: 0.2.0
- **Rust 版本**: 1.70+ (支持 Rust 2021 版本)
- **测试状态**: 247/271 测试通过 (91.1%)
- **编译状态**: ✅ 所有编译错误已修复
- **代码质量**: 通过 Clippy 检查，部分警告待处理

## 依赖管理

项目使用工作空间依赖管理，核心依赖在 `Cargo.toml` 的 `[workspace.dependencies]` 部分统一定义。

### 重要依赖版本
- **tokio**: 1.40 (完整特性)
- **serde**: 1.0 (派生特性)
- **arrow**: 54.0.0 (稳定版本)
- **lancedb**: 0.18.0
- **dioxus**: 0.6 (UI 框架，可选)

## 部署注意事项

### 生产构建
```bash
# 优化构建
cargo build --release --workspace

# 运行生产测试
cargo test --release --workspace

# 性能基准测试
cargo bench --workspace
```

### Docker 部署
- 使用 `Dockerfile` 进行容器化
- 使用 `docker-compose.yml` 进行服务编排
- 向量数据库需要单独的服务容器

## 故障排除

### 常见问题
1. **编译错误**: 检查 Rust 版本是否 >=1.70
2. **测试失败**: 检查是否设置了必需的 API 密钥
3. **依赖冲突**: 运行 `cargo update` 更新依赖
4. **内存不足**: 使用 `cargo build -j 1` 限制并行任务

### 获取帮助
- 查看 `docs/` 目录的详细文档
- 检查 GitHub Issues 中的已知问题
- 运行 `./scripts/run_tests.sh` 获取详细的错误报告