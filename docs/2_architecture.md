# 2. 系统架构

Lumos-X采用多层架构设计，包括核心引擎层、中间通信层和前端应用层。这种设计既保证了系统的高性能，也提供了良好的跨平台兼容性和灵活性。

## 2.1 整体架构概览

Lumos-X的整体架构可以分为以下几个主要部分：

```
┌──────────────────────────────────────────────────────────────────┐
│                     Lumos-X Platform                             │
├──────────────┬───────────────────────────┬──────────────────────┤
│ lumosai_ui   │    @lomusai/client-js     │     lumos_server     │
│              │                           │                       │
│ - Agent UI   │ - API封装                 │ - Agent引擎           │
│ - Chat界面   │ - WASM/Native绑定        │ - 内存服务            │
│ - 控制面板   │ - P2P协议前端实现         │ - 工作流引擎          │
│ - P2P监控   │ - 类型定义与转换           │ - 工具注册            │
├──────────────┴───────────────────────────┴──────────────────────┤
│                     lumos_core (Rust Core)                       │
│                                                                  │
│ - Agent实现    - P2P网络     - 内存管理     - 工作流执行         │
│ - WASM绑定     - FFI接口     - 安全通信     - 分布式协作         │
├───────────────────────────────────────────────────────────────────┤
│                        Platform Layers                            │
├─────────────┬──────────────────────────┬─────────────────────────┤
│  Storage    │      Integration         │    Network Layer        │
├─────────────┼──────────────────────────┼─────────────────────────┤
│ - Vector DB │ - 外部模型集成           │ - libp2p协议            │
│ - 文档存储  │ - 工具连接器             │ - DHT分布式哈希表       │
│ - 元数据    │ - 知识库连接             │ - 对等发现              │
│ - 本地存储  │ - 区块链集成             │ - 资源共享              │
└─────────────┴──────────────────────────┴─────────────────────────┘
```

## 2.2 核心组件

### 2.2.1 lumos_core (Rust核心库)

lumos_core是整个系统的核心引擎，采用Rust语言实现，提供高性能、内存安全和并发处理能力。主要负责：

- **Agent执行引擎**：处理Agent的核心逻辑和执行流程
- **内存管理**：提供高效的内存管理和持久化机制
- **P2P网络**：实现基于libp2p的去中心化网络通信
- **外部接口（FFI）**：提供WebAssembly和Native绑定，与前端交互

核心库的设计采用模块化架构，每个功能组件都有清晰的边界和职责，便于单独测试和维护。

### 2.2.2 @lomusai/client-js (客户端库)

@lomusai/client-js是连接前端UI和Rust核心的桥梁，采用TypeScript实现，主要职责包括：

- **API封装**：为UI层提供友好的TypeScript API
- **绑定层**：通过WebAssembly或Native Addon与Rust核心交互
- **类型定义**：提供完整的TypeScript类型定义，增强开发体验
- **状态管理**：处理UI和核心之间的状态同步

客户端库支持多种运行模式，可以根据环境自动选择最适合的交互方式。

### 2.2.3 lumos_server (服务器)

lumos_server是部署在服务端的核心组件，基于Rust实现，提供更强大的计算能力和多用户支持：

- **API服务**：提供RESTful和gRPC接口，支持远程调用
- **多租户管理**：处理多用户/多组织的隔离和权限
- **分布式协调**：管理分布式执行和资源调度
- **服务发现**：支持服务注册和发现

服务器组件可选部署，适用于需要更强大计算能力或多用户协作的场景。

### 2.2.4 lumosai_ui (用户界面)

lumosai_ui是系统的视觉呈现层，基于React和TypeScript构建，提供直观友好的用户体验：

- **Agent设计器**：可视化Agent创建和配置工具
- **对话界面**：与Agent交互的聊天界面
- **仪表板**：系统状态和性能监控
- **P2P网络可视化**：展示和管理P2P网络拓扑

UI层采用响应式设计，支持桌面、移动和Web多种平台。

## 2.3 部署模式

Lumos-X支持三种主要部署模式，可以根据需求灵活选择：

### 2.3.1 桌面单机模式

```
┌─────────────────────────────────────────────────────────────┐
│                 Desktop Application Mode                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐     ┌───────────────┐     ┌────────────┐   │
│  │ lumosai_ui  │◄───►│ Rust Core     │◄───►│ Local DB   │   │
│  │ (Electron)  │     │ (WASM/Native) │     │            │   │
│  └─────────────┘     └───────────────┘     └────────────┘   │
│                             ▲                               │
│                             │                               │
│                             ▼                               │
│  ┌────────────┐      ┌───────────────┐     ┌────────────┐   │
│  │ 本地模型   │◄────►│ 外部API调用   │◄───►│ P2P节点    │   │
│  │ (可选)     │      │ (可选)        │     │ (可选)     │   │
│  └────────────┘      └───────────────┘     └────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

特点：
- 全部组件在用户设备本地运行
- 数据存储在本地，保障隐私
- 可选择性连接P2P网络或外部API
- 适合个人用户和注重隐私的场景

### 2.3.2 云服务模式

```
┌───────────────────────────────────────────────────────────────────────┐
│                           Cloud Deployment                            │
├───────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌─────────────┐     ┌───────────────────────────────────────────┐    │
│  │ Web UI      │◄───►│              API Gateway                  │    │
│  │ /移动应用   │     └───────────────────────────────────────────┘    │
│  └─────────────┘                         │                             │
│                                         ▼                             │
│  ┌─────────────────────────────────────────────────────────────┐     │
│  │                 Rust服务集群 (lumos_server)                 │     │
│  │                                                             │     │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐     │     │
│  │  │ Agent    │  │ 内存     │  │ 工作流   │  │ 工具     │     │     │
│  │  │ 服务     │  │ 服务     │  │ 服务     │  │ 服务     │     │     │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘     │     │
│  │                                                             │     │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐     │     │
│  │  │ 认证     │  │ 分析     │  │ 监控     │  │ P2P      │     │     │
│  │  │ 服务     │  │ 服务     │  │ 服务     │  │ 网关     │     │     │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘     │     │
│  └─────────────────────────────────────────────────────────────┘     │
│                                         │                             │
│                                         ▼                             │
│  ┌─────────────────────────────────────────────────────────────┐     │
│  │                     数据层                                  │     │
│  │                                                             │     │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐     │     │
│  │  │ SQL数据库│  │ 向量数据库│  │ 文档存储 │  │ 缓存     │     │     │
│  │  │          │  │          │  │          │  │          │     │     │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘     │     │
│  └─────────────────────────────────────────────────────────────┘     │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘
```

特点：
- 服务部署在云端，支持多用户访问
- 提供更强大的计算和存储能力
- 支持横向扩展和高可用性配置
- 适合团队和企业用户

### 2.3.3 去中心化P2P网络模式

```
┌───────────────────────────────────────────────────────────────────────┐
│                       Decentralized P2P Network                       │
├───────────────────────────────────────────────────────────────────────┤
│                                                                       │
│                        ┌───────────────┐                              │
│                        │ DHT & 节点    │                              │
│                        │ 发现服务      │                              │
│                        └───────────────┘                              │
│                               │                                       │
│                               ▼                                       │
│  ┌─────────────┐      ┌───────────────┐      ┌─────────────┐         │
│  │ 节点 A      │◄────►│ 节点 B        │◄────►│ 节点 C      │         │
│  │ (用户设备)  │      │ (中继节点)    │      │ (用户设备)  │         │
│  └─────────────┘      └───────────────┘      └─────────────┘         │
│        │                     │                      │                 │
│        ▼                     ▼                      ▼                 │
│  ┌─────────────┐      ┌───────────────┐      ┌─────────────┐         │
│  │ 本地Agent   │      │ 共享资源      │      │ 分布式存储  │         │
│  │ 执行引擎    │      │ (模型/数据)   │      │ (IPFS/CAS)  │         │
│  └─────────────┘      └───────────────┘      └─────────────┘         │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────┐     │
│  │                 内容寻址存储                               │     │
│  │   (Agent定义, 知识库, 共享资源)                            │     │
│  └─────────────────────────────────────────────────────────────┘     │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘
```

特点：
- 完全去中心化架构，不依赖中心服务器
- 数据和计算分布在网络节点中
- 支持资源共享和协作计算
- 提供更高的隐私保护和韧性
- 适合去中心化社区和隐私敏感应用

## 2.4 数据流与通信模型

Lumos-X采用多层数据流模型，根据部署模式自动适配最优通信路径：

### 2.4.1 本地通信流（桌面应用内）

```
UI组件 → Electron IPC → @lomusai/client-js → Rust FFI → lumos_core → 本地存储/模型
```

这种通信流特点是低延迟，无网络依赖，适合处理私有数据。在桌面应用模式下，所有处理都在本地完成，保障数据安全性。

### 2.4.2 云服务通信流（客户端-服务器）

```
UI → HTTP/gRPC → API网关 → 微服务集群 → 数据层
```

标准的客户端-服务器通信模式，使用HTTP、WebSocket或gRPC协议，适合需要大量计算的任务，可以利用云端的计算能力。

### 2.4.3 P2P通信流（去中心化网络）

```
节点A → libp2p协议 → (可选中继) → 节点B
```

点对点直接通信，适合分布式协作和数据共享。通信基于libp2p协议栈，支持内容寻址和节点发现。

### 2.4.4 混合通信流

Lumos-X的一大特点是支持混合通信模式，系统可以智能选择最佳通信路径：

- 敏感数据处理：优先选择本地通信
- 计算密集型任务：在可用时使用云服务
- 共享资源访问：通过P2P网络获取
- 网络中断情况：自动切换到本地模式

## 2.5 安全架构

Lumos-X的安全架构贯穿所有层级，主要包括：

### 2.5.1 认证与授权

- **分布式身份**：支持自主身份(DID)和去中心化验证
- **多因素认证**：支持多种认证方式，提高安全性
- **细粒度权限**：基于角色和属性的访问控制

### 2.5.2 数据安全

- **端到端加密**：所有通信都经过加密保护
- **零知识证明**：可验证计算而不泄露数据内容
- **隐私计算**：支持安全多方计算和差分隐私

### 2.5.3 存储安全

- **加密存储**：敏感数据存储前加密
- **内容寻址**：基于内容哈希的不可变存储
- **安全备份**：自动加密备份和恢复机制

### 2.5.4 网络安全

- **P2P安全通信**：基于libp2p的安全通信栈
- **节点验证**：基于密码学证明的节点身份验证
- **DoS防护**：分布式拒绝服务攻击防护机制 