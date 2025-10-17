# LumosAI API 参考文档

欢迎来到 LumosAI API 参考文档！这里提供完整的 API 接口说明和使用示例。

## 📚 API 模块

### 🤖 核心模块

| 模块 | 描述 | 文档链接 |
|------|------|----------|
| [Agent](./agent.md) | AI Agent 核心接口 | 创建、配置和管理 Agent |
| [Tool](./tool.md) | 工具系统接口 | 工具定义、注册和执行 |
| [Memory](./memory.md) | 内存系统接口 | 内存管理和检索 |
| [Message](./message.md) | 消息处理接口 | 消息格式和处理 |

### 🧠 LLM 模块

| 模块 | 描述 | 文档链接 |
|------|------|----------|
| [LLM Provider](./llm-provider.md) | LLM 提供商接口 | 模型配置和调用 |
| [Model Config](./model-config.md) | 模型配置接口 | 参数设置和优化 |
| [Streaming](./streaming.md) | 流式响应接口 | 实时响应处理 |

### 🔧 工具模块

| 模块 | 描述 | 文档链接 |
|------|------|----------|
| [Tool Macro](./tool-macro.md) | 工具宏接口 | `#[tool]` 宏使用 |
| [Function Tool](./function-tool.md) | 函数工具接口 | 函数工具定义 |
| [Tool Execution](./tool-execution.md) | 工具执行接口 | 工具调用和结果处理 |

### 💾 存储模块

| 模块 | 描述 | 文档链接 |
|------|------|----------|
| [Vector Store](./vector-store.md) | 向量存储接口 | 向量数据库操作 |
| [Document](./document.md) | 文档处理接口 | 文档解析和分块 |
| [Embedding](./embedding.md) | 嵌入生成接口 | 文本向量化 |

### 🔍 RAG 模块

| 模块 | 描述 | 文档链接 |
|------|------|----------|
| [RAG System](./rag-system.md) | RAG 系统接口 | 检索增强生成 |
| [Retriever](./retriever.md) | 检索器接口 | 文档检索策略 |
| [Chunker](./chunker.md) | 分块器接口 | 文档分块策略 |

### 🌐 网络模块

| 模块 | 描述 | 文档链接 |
|------|------|----------|
| [Network](./network.md) | 网络通信接口 | Agent 间通信 |
| [Router](./router.md) | 消息路由接口 | 消息路由和分发 |
| [Discovery](./discovery.md) | 服务发现接口 | 服务注册和发现 |

### 🏢 企业模块

| 模块 | 描述 | 文档链接 |
|------|------|----------|
| [Auth](./auth.md) | 认证授权接口 | 用户认证和权限 |
| [Monitoring](./monitoring.md) | 监控接口 | 性能监控和日志 |
| [Billing](./billing.md) | 计费接口 | 使用量统计和计费 |

### 🎨 UI 模块

| 模块 | 描述 | 文档链接 |
|------|------|----------|
| [Web UI](./web-ui.md) | Web 界面接口 | Web 界面组件 |
| [Components](./components.md) | UI 组件接口 | 可复用 UI 组件 |

## 🚀 快速导航

### 按使用频率

#### 🔥 最常用 API
1. [Agent::builder()](./agent.md#builder) - 创建 Agent
2. [Agent::generate()](./agent.md#generate) - 生成回复
3. [#[tool] 宏](./tool-macro.md) - 定义工具
4. [Memory::basic()](./memory.md#basic) - 创建基础内存

#### ⭐ 常用 API
1. [Tool::execute()](./tool-execution.md#execute) - 执行工具
2. [SimpleRag::builder()](./rag-system.md#builder) - 创建 RAG 系统
3. [VectorStore::search()](./vector-store.md#search) - 向量搜索
4. [Agent::with_tools()](./agent.md#with_tools) - 添加工具

#### 📚 高级 API
1. [NetworkManager](./network.md) - 网络管理
2. [EnterpriseAuth](./auth.md) - 企业认证
3. [MonitoringSystem](./monitoring.md) - 监控系统

### 按开发阶段

#### 🎯 入门阶段
- [Agent API](./agent.md) - 基础 Agent 操作
- [Message API](./message.md) - 消息处理
- [Tool Macro](./tool-macro.md) - 简单工具定义

#### 🔧 开发阶段
- [Memory API](./memory.md) - 内存管理
- [RAG System API](./rag-system.md) - 知识检索
- [Vector Store API](./vector-store.md) - 向量存储

#### 🚀 生产阶段
- [Monitoring API](./monitoring.md) - 性能监控
- [Auth API](./auth.md) - 安全认证
- [Network API](./network.md) - 分布式部署

## 📖 API 文档约定

### 文档格式

每个 API 文档包含以下部分：

```markdown
# 模块名称

## 概述
模块的基本介绍和用途

## 核心类型
主要的结构体和枚举

## 方法列表
所有公开方法的列表

## 详细说明
每个方法的详细说明，包括：
- 函数签名
- 参数说明
- 返回值说明
- 使用示例
- 错误处理

## 示例代码
完整的使用示例

## 相关链接
相关模块和文档的链接
```

### 代码示例约定

- 所有示例都是完整可运行的代码
- 包含必要的导入语句
- 提供错误处理示例
- 使用中文注释说明

### 版本兼容性

- 🟢 **稳定 API**: 保证向后兼容
- 🟡 **实验性 API**: 可能在未来版本中变更
- 🔴 **已废弃 API**: 将在未来版本中移除

## 🔍 搜索和导航

### 快速搜索

使用浏览器的搜索功能（Ctrl+F / Cmd+F）快速查找：

- **按功能搜索**: 如 "generate", "execute", "search"
- **按类型搜索**: 如 "Agent", "Tool", "Memory"
- **按错误搜索**: 如 "Error", "Result"

### 交叉引用

文档中的链接类型：

- `[Agent]` - 链接到 Agent 模块
- `[Agent::generate()]` - 链接到具体方法
- `[示例: 基础用法]` - 链接到示例代码

## 🤝 贡献 API 文档

### 文档改进

我们欢迎以下类型的贡献：

1. **错误修正**: 修复文档中的错误
2. **示例改进**: 提供更好的代码示例
3. **说明完善**: 补充缺失的说明
4. **新 API 文档**: 为新功能编写文档

### 提交流程

1. Fork 项目仓库
2. 修改相关文档文件
3. 测试所有代码示例
4. 提交 Pull Request

### 文档标准

- **准确性**: 确保 API 签名和行为描述准确
- **完整性**: 包含所有公开 API
- **一致性**: 遵循统一的文档格式
- **可用性**: 提供实用的示例和说明

## 📞 获取帮助

### API 使用问题

- 📖 查看具体模块的文档
- 🔍 搜索相关示例代码
- 💬 在社区提问

### 文档问题

- 🐛 提交文档 Issue
- 📧 联系文档维护团队
- 🤝 参与文档改进

---

*完整、准确、实用的 API 参考，助力您的 LumosAI 开发之旅！* 📚
