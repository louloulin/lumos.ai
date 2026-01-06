# 文档结构与统一入口（Content Map）

> 本文用于统一梳理当前文档结构，给出“规范入口”与“历史/扩展”文档的映射关系，便于用户和维护者快速找到权威内容，同时避免重复与信息分散。

## 规范入口（Canonical Entries）
- 快速开始：`docs/quick-start/README.md`
- 总览索引：`docs/index.md`
- 架构说明：`docs/ARCHITECTURE.md`（可视化与结构化示意）
- 系统架构设计：`docs/2_architecture.md`（技术设计与模块说明）
- API 参考：`docs/api-reference/README.md`
- 教程导航：`docs/tutorials/README.md`
- 深度指南：`docs/guides/README.md`
- 历史索引：`docs/history/INDEX.md`
- 概览索引：`docs/overview/INDEX.md`

## 重复与合并建议（不破坏现有链接）
- 架构文档
  - 现有：`docs/ARCHITECTURE.md`、`docs/2_architecture.md`
  - 建议：保留两者，`ARCHITECTURE.md` 侧重图示，`2_architecture.md` 侧重技术细节；在 `docs/index.md` 同时链接两者。
- API 参考
  - 现有：`docs/5_api_reference.md`、`docs/API_REFERENCE.md`、`docs/api-reference/README.md`
  - 建议：以 `docs/api-reference/README.md` 为主入口；保留另外两个作为补充/历史，后续章节逐步合并到主入口。
- 快速开始
  - 现有：`docs/QUICK_START.md`、`docs/quick-start/README.md`、`docs/getting_started.md`
  - 建议：统一入口为 `docs/quick-start/README.md`；另两个保留为补充说明，后续迁移/合并。
- 向量数据库
  - 现有：`docs/VECTOR_DATABASES.md`、`docs/tutorials/vector-databases.md`、`docs/vector_api_reference.md`、`docs/vector_database_optimization.md`
  - 建议：教学入口为 `docs/tutorials/vector-databases.md`；参考入口为 `docs/vector_api_reference.md`；优化指南作为进阶文档保留。
- 多 Agent
  - 现有：教程 `docs/tutorials/05-multi-agent.md`，研究/报告类 `docs/MULTI_AGENT_*`
  - 建议：教程入口以 `05-multi-agent.md` 为主；`MULTI_AGENT_*` 归档到历史与研究，保持索引可达。

## 链接策略
- 顶层索引 `docs/index.md`：统一链接到规范入口。
- 教程索引 `docs/tutorials/README.md`：链接到教程文档，并在适当位置指向 API 参考与指南。
- 深度指南 `docs/guides/README.md`：链接到部署、安全、监控、集成等专题。

## 维护约定
- 新增文档时，选择放入对应目录（tutorials/guides/api-reference/overview/history）。
- 若出现重复主题，优先扩展规范入口文档，并在历史文档顶部添加“迁移提示”。
- 避免立即删除/重命名，优先通过索引与提示实现平滑过渡；如需重命名，先在 PR 中声明影响范围与替代链接。

## 下一步可选动作（需确认后执行）
- 在重复文档顶部添加“统一入口提示”，指向规范入口（不改变路径）。
- 对 `5_api_reference.md` 与 `API_REFERENCE.md` 进行章节级迁移，合并到 `api-reference/README.md`。
- 将 `QUICK_START.md` 内容迁移至 `quick-start/README.md`，保留原文件加迁移提示。

---

此映射将持续更新，确保文档体系清晰、一致、易于导航。维护者可据此安排后续迁移计划。