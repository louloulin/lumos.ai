# 3. 技术栈

Lumosai 项目使用了多种现代技术和工具，以实现高性能、跨平台和可扩展的 AI Agent 框架。本章节详细介绍项目的技术栈选择及其原因。

## 3.1 核心技术

### 3.1.1 语言选择

| 组件 | 语言 | 原因 |
|------|------|------|
| 核心库 (lumosai_core) | Rust | 高性能、内存安全、无垃圾回收、并发支持、跨平台编译 |
| 客户端库 (@lumosai/client-js) | TypeScript | 类型安全、前端兼容性、广泛的生态系统 |
| UI 层 (lumosai_ui) | TypeScript + React | 组件化设计、响应式界面、广泛的社区支持 |

**Rust 的主要优势**：
- 零成本抽象，性能接近 C/C++
- 内存安全保证，无需垃圾回收器
- 强大的类型系统和模式匹配
- 优秀的并发和异步支持
- 跨平台编译能力
- 工具链完善（Cargo、Clippy、Rustfmt）

**TypeScript 的主要优势**：
- 静态类型检查，提高代码质量
- 与 JavaScript 生态系统兼容
- 良好的 IDE 支持和开发体验
- 广泛的社区和库支持

### 3.1.2 核心框架和库

#### Rust 核心库

| 类别 | 库 | 用途 |
|------|-----|------|
| 异步运行时 | Tokio | 异步任务调度、IO 操作 |
| 网络通信 | libp2p | P2P 网络实现 |
| 序列化 | serde | 数据序列化/反序列化 |
| Web 绑定 | wasm-bindgen | WebAssembly 接口生成 |
| FFI | cbindgen | C 语言接口生成 |
| 日志 | tracing | 结构化日志和追踪 |
| 命令行 | clap | 命令行参数解析 |
| 测试 | proptest, criterion | 属性测试、基准测试 |
| 错误处理 | thiserror, anyhow | 错误定义和处理 |

#### TypeScript/JavaScript 库

| 类别 | 库 | 用途 |
|------|-----|------|
| UI 框架 | React | 用户界面构建 |
| 状态管理 | Redux Toolkit | 应用状态管理 |
| UI 组件 | Chakra UI, TailwindCSS | 组件库和样式系统 |
| 图表可视化 | D3.js, React-Vis | 数据可视化 |
| API 客户端 | Axios, TanStack Query | 网络请求和缓存 |
| WebSocket | Socket.IO | 实时通信 |
| 测试 | Jest, Testing Library | 单元和集成测试 |
| 构建工具 | Vite | 开发环境和构建优化 |

### 3.1.3 存储技术

Lumosai 支持多种存储后端，以适应不同的部署环境和需求：

| 存储类型 | 技术选择 | 用途 |
|---------|----------|------|
| 向量存储 | Qdrant, Milvus, FAISS | 嵌入向量存储和相似性搜索 |
| 文档存储 | SQLite, PostgreSQL | 结构化数据存储 |
| KV 存储 | RocksDB, LMDB | 高性能键值存储 |
| 内存数据库 | Redis | 缓存和快速访问数据 |
| 分布式存储 | IPFS, CAS | 内容寻址和分布式存储 |
| 本地文件系统 | - | 简单部署的默认存储 |

### 3.1.4 模型集成

Lumosai 设计了灵活的模型适配层，支持多种 LLM 提供商和模型：

| 提供商 | 支持模型 | 集成方式 |
|--------|----------|----------|
| OpenAI | GPT-3.5/4 系列 | API 集成 |
| Anthropic | Claude 系列 | API 集成 |
| 本地模型 | LLaMA, Mistral, Qwen | 直接集成 |
| HuggingFace | 各类开源模型 | API/直接集成 |
| 自定义模型 | - | 适配器接口 |

## 3.2 架构模式

### 3.2.1 核心架构模式

Lumosai 采用了多种架构模式，确保系统的可扩展性和可维护性：

| 架构模式 | 应用场景 | 实现方式 |
|----------|----------|----------|
| 层次化架构 | 整体系统结构 | 核心层、业务层、UI层清晰分离 |
| 组件化架构 | 模块设计 | 基于接口的松耦合组件 |
| 插件架构 | 扩展机制 | 动态加载的插件系统 |
| 事件驱动 | 通信模型 | 基于事件和消息的组件通信 |
| 命令模式 | 工具执行 | 将工具操作封装为命令对象 |
| 适配器模式 | 外部集成 | 统一的适配器接口连接不同服务 |
| 工厂模式 | 对象创建 | Agent和工具的创建工厂 |
| 观察者模式 | 状态变化通知 | UI和核心状态同步 |

### 3.2.2 并发和异步模式

| 并发模式 | 应用场景 | 实现技术 |
|----------|----------|----------|
| 异步任务 | IO密集型操作 | Tokio的异步运行时 |
| 线程池 | CPU密集型计算 | Rayon并行库 |
| Actor模型 | 隔离状态的并发 | 基于Tokio的actor实现 |
| 异步流 | 连续数据处理 | Stream trait和async_stream |
| 并行执行 | 独立任务并行 | Future::join和任务分配 |

### 3.2.3 分布式模式

| 分布式模式 | 应用场景 | 实现技术 |
|------------|----------|----------|
| 内容寻址 | 分布式数据共享 | CID和哈希索引 |
| DHT | 资源发现 | Kademlia DHT |
| 发布-订阅 | 多节点通信 | libp2p的GossipSub |
| 状态同步 | 分布式状态管理 | CRDT数据结构 |
| 容错机制 | 节点故障恢复 | 重试、备份和降级策略 |

## 3.3 开发工具链

### 3.3.1 构建和包管理

| 工具 | 用途 | 配置文件 |
|------|------|----------|
| Cargo | Rust包管理和构建 | Cargo.toml |
| npm/pnpm | JavaScript包管理 | package.json |
| Docker | 容器化部署 | Dockerfile |
| GitHub Actions | CI/CD流程 | .github/workflows/*.yml |

### 3.3.2 开发和调试工具

| 工具 | 用途 | 配置文件 |
|------|------|----------|
| Rust Analyzer | Rust代码智能分析 | rust-analyzer.toml |
| VS Code | 集成开发环境 | .vscode/* |
| Chrome DevTools | UI调试 | - |
| Rust WASM调试 | WebAssembly调试 | - |
| Tokio Console | 异步任务监控 | - |

### 3.3.3 测试和质量保证

| 工具 | 用途 | 配置文件 |
|------|------|----------|
| Cargo Test | 单元测试和集成测试 | - |
| Jest | JavaScript测试 | jest.config.js |
| Proptest | 属性测试 | - |
| Clippy | Rust静态分析 | clippy.toml |
| ESLint | TypeScript静态分析 | .eslintrc.js |
| Rustfmt | Rust代码格式化 | rustfmt.toml |
| Prettier | TypeScript代码格式化 | .prettierrc |
| Husky | Git钩子管理 | .husky/* |

## 3.4 部署和运维技术

### 3.4.1 桌面应用部署

| 技术 | 用途 | 配置 |
|------|------|------|
| Electron | 跨平台桌面应用 | electron.config.js |
| Tauri | 轻量级桌面应用 | tauri.conf.json |
| SQLite | 本地存储 | - |
| 自动更新 | 版本管理 | update.json |

### 3.4.2 服务器部署

| 技术 | 用途 | 配置 |
|------|------|------|
| Docker | 容器化 | Dockerfile |
| Kubernetes | 容器编排 | k8s/*.yaml |
| Helm | K8s包管理 | chart/* |
| Terraform | 基础设施即代码 | *.tf |
| Prometheus | 监控系统 | prometheus.yml |
| Grafana | 可视化监控 | dashboards/*.json |
| Vault | 密钥管理 | vault.hcl |

### 3.4.3 边缘和嵌入式部署

| 技术 | 用途 | 配置 |
|------|------|------|
| WebAssembly | 浏览器运行 | wasm-pack.toml |
| WASI | 服务器WebAssembly | - |
| 交叉编译 | 不同目标平台 | .cargo/config.toml |
| 静态链接 | 独立可执行文件 | build.rs |

## 3.5 数据流和通信技术

### 3.5.1 API和协议

| 技术 | 用途 | 格式 |
|------|------|------|
| RESTful API | HTTP服务接口 | OpenAPI规范 |
| GraphQL | 灵活数据查询 | Schema定义 |
| gRPC | 高性能RPC | Protocol Buffers |
| WebSocket | 双向实时通信 | JSON消息 |
| libp2p | P2P通信协议 | 自定义协议 |

### 3.5.2 数据格式

| 格式 | 用途 | 库 |
|------|------|-----|
| JSON | 通用数据交换 | serde_json, json |
| MessagePack | 二进制序列化 | rmp, msgpack |
| Protocol Buffers | 高效序列化 | prost, protobuf-js |
| CBOR | 二进制数据格式 | ciborium, cbor-js |
| CID | 内容寻址标识符 | libipld, multiformats |

## 3.6 AI和机器学习技术

### 3.6.1 核心AI技术

| 技术 | 用途 | 实现 |
|------|------|------|
| LLM集成 | 智能代理基础 | 多模型适配器 |
| 工具使用 | 代理能力扩展 | 函数调用接口 |
| 规划算法 | 任务分解 | 基于LLM的规划器 |
| 向量嵌入 | 相似性搜索 | 多模型支持 |
| 记忆管理 | 短期和长期记忆 | 自适应存储策略 |

### 3.6.2 RAG相关技术

| 技术 | 用途 | 实现 |
|------|------|------|
| 文档分块 | 内容切分 | 多策略分块器 |
| 嵌入计算 | 语义表示 | 多模型支持 |
| 向量检索 | 相关内容查找 | ANN算法 |
| 重排序 | 结果优化 | 多级筛选策略 |
| 混合检索 | 综合检索能力 | 关键词+语义混合 |

### 3.6.3 评估技术

| 技术 | 用途 | 实现 |
|------|------|------|
| 自动评估 | 性能测量 | 基于LLM的评估器 |
| 人类反馈 | 质量评价 | 反馈收集接口 |
| 指标计算 | 客观评估 | 多维度指标 |
| A/B测试 | 方案比较 | 实验框架 |
| 异常检测 | 质量监控 | 统计和启发式方法 |

## 3.7 安全技术

### 3.7.1 认证和授权

| 技术 | 用途 | 实现 |
|------|------|------|
| JWT | 令牌认证 | jsonwebtoken |
| OAuth2 | 授权协议 | oauth2 |
| PASETO | 安全令牌 | paseto |
| RBAC | 基于角色的访问控制 | 自定义实现 |
| DID | 去中心化身份 | did_resolver |

### 3.7.2 加密技术

| 技术 | 用途 | 实现 |
|------|------|------|
| AES | 对称加密 | ring, aes |
| RSA/ECDSA | 非对称加密 | ring, rsa |
| libsodium | 现代加密库 | sodiumoxide |
| TLS | 传输层安全 | rustls, openssl |
| 零知识证明 | 隐私保护验证 | zkp |

### 3.7.3 隔离和沙箱

| 技术 | 用途 | 实现 |
|------|------|------|
| WASM沙箱 | 安全代码执行 | wasmtime |
| 容器隔离 | 进程隔离 | Docker, containerd |
| 权限限制 | 最小权限原则 | 自定义实现 |
| 资源限制 | 防止滥用 | cgroups, rlimit |
| 内存隔离 | 跨Agent保护 | Rust所有权系统 | 