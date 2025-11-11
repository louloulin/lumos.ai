# LumosAI 文档索引

欢迎使用 LumosAI 文档！这里提供从入门到生产的完整资料，覆盖架构、开发、部署、RAG、宏与DSL、多代理工作流、测试与最佳实践等。所有链接均指向本仓库已存在的文档，确保可用。

## 快速入口

- `docs/QUICK_START.md` 快速开始
- `docs/quick-start/installation.md` 安装指南（Quick Start 安装）
- `docs/getting_started.md` 入门指南

## 概览与架构

- `docs/1_overview.md` 项目概览
- `docs/2_architecture.md` 系统架构与设计
- `docs/ARCHITECTURE.md` 架构图与结构说明（包含可视化示意）
- `docs/3_tech_stack.md` 技术栈与依赖
- `docs/4_core_components.md` 核心组件（Agent、Tool、Memory、LLM、RAG、Workflow 等）

## 开发与参考

- `docs/5_api_reference.md` API 参考
- `docs/API_REFERENCE.md` API 参考（补充版）
- `docs/USER_GUIDE.md` 用户使用指南
- `docs/TOOL_USER_GUIDE.md` 工具使用指南
- `docs/6_development_guide.md` 开发指南
- `docs/7_deployment_guide.md` 部署指南

## RAG 与向量检索

- `docs/VECTOR_DATABASES.md` 向量数据库集成
- `docs/LANCEDB_IMPLEMENTATION.md` LanceDB 实现
- `docs/FASTEMBED_IMPLEMENTATION.md` FastEmbed 向量嵌入实现
- `docs/vector_api_reference.md` 向量 API 参考
- `docs/vector_database_optimization.md` 向量数据库优化指南

## 宏与 DSL

- `docs/dsl_macros.md` DSL 宏与用法
- `docs/PARAMETER_MACRO_ANALYSIS.md` 参数宏分析

## 多代理与工作流

- `docs/MULTI_AGENT_COLLABORATION_PATTERNS.md` 多代理协作模式
- `docs/MULTI_AGENT_QUICK_REFERENCE.md` 多代理快速参考
- `docs/MULTI_AGENT_IMPLEMENTATION_COMPLETE.md` 多代理实现完成报告
- `docs/MULTI_AGENT_TESTING_ANALYSIS.md` 多代理测试分析
- `docs/examples/README.md` 示例索引（包含 Chatbot、RAG、Workflow 等）

## 测试与质量

- `docs/testing/README.md` 测试指南
- `docs/analysis/CARGO_TEST_ANALYSIS_SUMMARY.md` Cargo 测试分析摘要
- `docs/analysis/COMPLETE_TEST_ANALYSIS_SUMMARY.md` 完整测试分析摘要
- `docs/MULTI_AGENT_E2E_TESTS_COMPLETE.md` 多代理 E2E 测试完成

## 最佳实践与性能

- `docs/BEST_PRACTICES.md` 最佳实践总览
- `docs/CHAIN_OPERATIONS_BEST_PRACTICES.md` Chain 操作最佳实践
- `docs/PERFORMANCE_GUIDE.md` 性能指南

## 指南与教程

- `docs/guides/README.md` 深度指南导航（架构、开发、部署、安全、监控、集成）
- `docs/tutorials/README.md` 教程导航
- `docs/quick-start/README.md` Quick Start 文档页

## 更新、发布与历史

- `docs/updates/` 更新日志与周报
- `docs/releases/CHANGELOG.md` 版本变更记录
- `docs/RELEASE_GUIDE.md` 发布与版本管理指南
- `docs/releases/RELEASE_NOTES_v0.2.0.md` v0.2.0 发行说明
- `docs/history/INDEX.md` 历史文档索引（本页将创建）

## 概览索引

- `docs/overview/INDEX.md` 概览与路线图索引（本页将创建）

## 常见问题与贡献

- `docs/8_faq.md` 常见问题
- `docs/contributing/CONTRIBUTING.md` 贡献指南

## 其他参考

- `docs/FRAMEWORK_COMPARISON.md` 框架对比

---

### 架构图位置说明
- 架构图与结构化示意位于 `docs/ARCHITECTURE.md`。
- 系统架构的详细说明参见 `docs/2_architecture.md`，两者互相补充：前者用于可视化展示，后者用于技术设计与模块划分说明。

### 系统要求
- Rust 1.70+
- Cargo
- 支持的操作系统: macOS、Linux、Windows

### 仓库与文档站点
- GitHub 仓库: https://github.com/louloulin/lumos.ai
- Crates.io: https://crates.io/crates/lumosai_core
- API 文档: https://docs.rs/lumosai_core