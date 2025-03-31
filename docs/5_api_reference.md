# 5. API参考

本章节详细说明Lumos-X提供的主要API接口、数据结构和使用示例，帮助开发者更好地理解和使用Lumos-X平台。

## 5.1 客户端API (@lomusai/client-js)

### 5.1.1 核心客户端

#### LumosClient

主客户端类，提供所有功能的入口点。

```typescript
import { LumosClient } from '@lomusai/client-js';

// 创建客户端实例
const client = new LumosClient({
  mode: 'local',  // 'local', 'cloud', 或 'hybrid'
  p2p: {
    enabled: true,
    bootstrapNodes: ['/ip4/127.0.0.1/tcp/9090/ws/p2p/QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N']
  }
});

// 初始化客户端
await client.initialize();
```

**方法**:

| 方法 | 描述 | 返回值 |
|------|------|--------|
| `constructor(options: ClientOptions)` | 创建客户端实例 | `LumosClient` |
| `initialize(): Promise<void>` | 初始化客户端 | `Promise<void>` |
| `createAgent(config: AgentConfig): Promise<AgentInstance>` | 创建Agent | `Promise<AgentInstance>` |
| `getAgent(id: string): Promise<AgentInstance\|null>` | 获取Agent实例 | `Promise<AgentInstance\|null>` |
| `listAgents(): Promise<AgentSummary[]>` | 列出所有Agent | `Promise<AgentSummary[]>` |
| `memory(): Promise<MemoryManager>` | 获取内存管理器 | `Promise<MemoryManager>` |
| `p2p(): Promise<P2PManager>` | 获取P2P网络管理器 | `Promise<P2PManager>` |

#### ClientOptions

客户端配置选项。

```typescript
export interface ClientOptions {
  // API密钥(云模式下必须)
  apiKey?: string;
  
  // 运行模式: 'local'(本地), 'cloud'(云), 或 'hybrid'(混合)
  mode?: 'local' | 'cloud' | 'hybrid';
  
  // 云API端点(仅cloud或hybrid模式)
  endpoint?: string;
  
  // P2P网络配置
  p2p?: {
    // 是否启用P2P
    enabled: boolean;
    
    // 引导节点列表(多地址格式)
    bootstrapNodes?: string[];
    
    // 监听地址列表
    listenAddresses?: string[];
    
    // 本地节点ID(可选)
    peerId?: string;
  };
  
  // 本地Rust WASM路径(可选)
  rustWasmPath?: string;
}
```

### 5.1.2 Agent API

#### AgentConfig

Agent配置接口。

```typescript
export interface AgentConfig {
  // Agent ID (可选，如未提供则自动生成)
  id?: string;
  
  // Agent名称
  name: string;
  
  // Agent描述
  description?: string;
  
  // Agent能力列表
  capabilities: Capability[];
  
  // 内存配置
  memory: MemoryConfig;
  
  // 工作流配置(可选)
  workflow?: WorkflowConfig;
  
  // 元数据
  metadata?: Record<string, any>;
}
```

#### AgentInstance

Agent实例接口，提供与Agent交互的方法。

```typescript
export interface AgentInstance {
  // Agent ID
  id: string;
  
  // Agent配置
  config: AgentConfig;
  
  // 处理输入并返回结果
  process(input: any): Promise<any>;
  
  // 流式处理输入
  processStream(input: any): AsyncIterable<ExecutionChunk>;
  
  // 保存Agent配置
  save(): Promise<void>;
  
  // 更新Agent配置
  update(updates: Partial<AgentConfig>): Promise<void>;
  
  // 删除Agent
  delete(): Promise<void>;
}
```

**使用示例**:

```typescript
// 创建Agent配置
const agentConfig = {
  name: "文档助手",
  description: "帮助用户管理和检索文档",
  capabilities: [
    {
      id: "text-generation",
      type: "llm",
      model: "gpt-4"
    },
    {
      id: "doc-search",
      type: "retrieval",
      config: {
        dataSourceId: "my-documents"
      }
    }
  ],
  memory: {
    type: "simple",
    config: {
      maxItems: 10
    }
  }
};

// 创建Agent
const agent = await client.createAgent(agentConfig);

// 使用Agent处理请求
const result = await agent.process({
  message: "查找关于P2P网络的文档"
});

// 流式处理
for await (const chunk of agent.processStream({
  message: "详细解释内容寻址存储的工作原理"
})) {
  console.log(chunk.type, chunk.content);
}
```

### 5.1.3 内存管理API

#### MemoryManager

内存管理器接口，提供存储和检索内存项的方法。

```typescript
export interface MemoryManager {
  // 存储内存项
  store(item: MemoryItem): Promise<string>;
  
  // 检索内存项
  retrieve(id: string): Promise<MemoryItem | null>;
  
  // 查询内存
  query(params: MemoryQuery): Promise<MemoryItem[]>;
  
  // 更新内存项
  update(id: string, updates: Partial<MemoryItem>): Promise<MemoryItem>;
  
  // 删除内存项
  delete(id: string): Promise<boolean>;
}
```

#### MemoryItem

内存项接口，表示存储在内存中的数据项。

```typescript
export interface MemoryItem {
  // 内存项ID(可选，如未提供则自动生成)
  id?: string;
  
  // 内容(任意JSON可序列化对象)
  content: any;
  
  // 元数据
  metadata: Record<string, any>;
  
  // 嵌入向量(可选)
  embedding?: number[];
  
  // 标签列表(可选)
  tags?: string[];
  
  // 类型标识
  type: string;
  
  // 创建时间(可选)
  createdAt?: Date;
  
  // 最后更新时间(可选)
  updatedAt?: Date;
}
```

**使用示例**:

```typescript
const memoryManager = await client.memory();

// 存储内存项
const id = await memoryManager.store({
  content: {
    message: "这是一个重要的记忆",
    source: "user"
  },
  metadata: {
    importance: "high",
    category: "conversation"
  },
  tags: ["important", "user-input"],
  type: "message"
});

// 查询内存
const results = await memoryManager.query({
  filter: {
    tags: {
      $contains: "important"
    },
    "metadata.importance": "high"
  },
  limit: 10
});

// 更新内存项
await memoryManager.update(id, {
  metadata: {
    importance: "medium"
  }
});
```

### 5.1.4 P2P网络API

#### P2PManager

P2P网络管理器接口，提供P2P网络操作方法。

```typescript
export interface P2PManager {
  // 获取本地节点ID
  getNodeId(): Promise<string>;
  
  // 连接到远程节点
  connect(multiaddr: string): Promise<void>;
  
  // 发布内容到主题
  publish(topic: string, data: any): Promise<void>;
  
  // 订阅主题
  subscribe(topic: string, handler: (data: any, from: string) => void): Promise<() => void>;
  
  // 存储内容
  storeContent(data: any): Promise<string>;
  
  // 获取内容
  getContent(cid: string): Promise<any>;
  
  // 获取已连接的节点列表
  getPeers(): Promise<PeerInfo[]>;
}
```

**使用示例**:

```typescript
const p2pManager = await client.p2p();

// 获取本地节点ID
const nodeId = await p2pManager.getNodeId();
console.log(`本地节点ID: ${nodeId}`);

// 连接到另一个节点
await p2pManager.connect("/ip4/192.168.1.10/tcp/9000/ws/p2p/QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N");

// 订阅主题
const unsubscribe = await p2pManager.subscribe("notifications", (data, from) => {
  console.log(`收到来自 ${from} 的消息:`, data);
});

// 发布消息到主题
await p2pManager.publish("notifications", {
  type: "update",
  content: "重要通知内容"
});

// 存储内容
const cid = await p2pManager.storeContent({
  type: "document",
  title: "P2P网络指南",
  content: "详细介绍P2P网络的工作原理..."
});

// 获取内容
const content = await p2pManager.getContent(cid);

// 获取已连接的节点列表
const peers = await p2pManager.getPeers();
```

## 5.2 Rust核心API

### 5.2.1 Agent API

Rust核心库提供的Agent API接口。

```rust
// 创建Agent
pub fn create_agent(config: &str) -> Result<String, AgentError>;

// 执行Agent
pub fn execute_agent(agent_id: &str, input: &str) -> Result<String, AgentError>;

// 流式执行Agent
pub fn execute_agent_stream(agent_id: &str, input: &str, callback: AgentCallback) -> Result<(), AgentError>;

// Agent回调函数类型
pub type AgentCallback = extern "C" fn(chunk_type: i32, content: *const c_char, done: bool, error: *const c_char) -> ();
```

### 5.2.2 FFI接口

对外提供的FFI接口，用于其他语言集成。

```rust
// C API导出函数

#[no_mangle]
pub extern "C" fn lumos_create_agent(config: *const c_char) -> *mut c_char;

#[no_mangle]
pub extern "C" fn lumos_execute_agent(agent_id: *const c_char, input: *const c_char) -> *mut c_char;

#[no_mangle]
pub extern "C" fn lumos_execute_agent_stream(
    agent_id: *const c_char,
    input: *const c_char,
    callback: AgentCallback
) -> i32;

#[no_mangle]
pub extern "C" fn lumos_free_string(ptr: *mut c_char);
```

### 5.2.3 WebAssembly接口

WebAssembly绑定接口。

```rust
#[wasm_bindgen]
impl LumosFFI {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self;
    
    #[wasm_bindgen]
    pub fn create_agent(&mut self, config_json: &str) -> Result<String, JsValue>;
    
    #[wasm_bindgen]
    pub fn execute_agent(&self, agent_id: &str, input_json: &str) -> Result<String, JsValue>;
    
    #[wasm_bindgen]
    pub fn execute_agent_stream(&self, agent_id: &str, input_json: &str) -> Result<Stream, JsValue>;
    
    #[wasm_bindgen]
    pub fn memory_store(&self, item_json: &str) -> Result<String, JsValue>;
    
    #[wasm_bindgen]
    pub fn memory_retrieve(&self, id: &str) -> Result<String, JsValue>;
    
    #[wasm_bindgen]
    pub fn memory_query(&self, query_json: &str) -> Result<String, JsValue>;
}
```

## 5.3 服务器API

### 5.3.1 REST API

#### Agent API

```
POST /api/agents
描述: 创建新Agent
请求体: AgentConfig JSON
响应: Agent对象

GET /api/agents
描述: 获取所有Agent列表
查询参数:
  - limit: 限制返回数量
  - offset: 分页偏移量
响应: Agent对象数组

GET /api/agents/:id
描述: 获取单个Agent
路径参数:
  - id: Agent ID
响应: Agent对象

PUT /api/agents/:id
描述: 更新Agent
路径参数:
  - id: Agent ID
请求体: AgentConfig JSON
响应: 更新后的Agent对象

DELETE /api/agents/:id
描述: 删除Agent
路径参数:
  - id: Agent ID
响应: 删除状态

POST /api/agents/:id/execute
描述: 执行Agent
路径参数:
  - id: Agent ID
请求体: 执行输入JSON
响应: 执行结果

GET /api/agents/:id/stream
描述: 流式执行Agent(SSE)
路径参数:
  - id: Agent ID
查询参数:
  - input: 执行输入(URL编码JSON)
响应: 事件流(text/event-stream)
```

#### 内存API

```
POST /api/memory
描述: 存储内存项
请求体: MemoryItem JSON
响应: 包含ID的内存项

GET /api/memory/:id
描述: 获取内存项
路径参数:
  - id: 内存项ID
响应: MemoryItem对象

POST /api/memory/query
描述: 查询内存
请求体: MemoryQuery JSON
响应: MemoryItem对象数组

PUT /api/memory/:id
描述: 更新内存项
路径参数:
  - id: 内存项ID
请求体: 部分MemoryItem JSON
响应: 更新后的MemoryItem对象

DELETE /api/memory/:id
描述: 删除内存项
路径参数:
  - id: 内存项ID
响应: 删除状态
```

### 5.3.2 gRPC API

主要gRPC服务定义。

```protobuf
syntax = "proto3";

package lumosai;

// Agent服务
service AgentService {
  // 创建或更新Agent
  rpc CreateOrUpdateAgent(AgentConfig) returns (AgentResponse);
  
  // 获取Agent
  rpc GetAgent(GetAgentRequest) returns (AgentResponse);
  
  // 列出Agent
  rpc ListAgents(ListAgentsRequest) returns (ListAgentsResponse);
  
  // 执行Agent
  rpc ExecuteAgent(ExecuteRequest) returns (ExecuteResponse);
  
  // 流式执行Agent
  rpc ExecuteAgentStream(ExecuteRequest) returns (stream ExecuteChunk);
}

// 内存服务
service MemoryService {
  // 存储内存项
  rpc Store(StoreRequest) returns (StoreResponse);
  
  // 检索内存项
  rpc Retrieve(RetrieveRequest) returns (RetrieveResponse);
  
  // 查询内存
  rpc Query(QueryRequest) returns (QueryResponse);
  
  // 更新内存项
  rpc Update(UpdateRequest) returns (UpdateResponse);
  
  // 删除内存项
  rpc Delete(DeleteRequest) returns (DeleteResponse);
}

// P2P网络服务
service P2PService {
  // 获取节点信息
  rpc GetNodeInfo(GetNodeInfoRequest) returns (NodeInfo);
  
  // 连接到节点
  rpc Connect(ConnectRequest) returns (ConnectResponse);
  
  // 发布内容
  rpc Publish(PublishRequest) returns (PublishResponse);
  
  // 订阅主题
  rpc Subscribe(SubscribeRequest) returns (stream SubscriptionMessage);
}
```

## 5.4 数据结构

### 5.4.1 Agent相关结构

#### AgentConfig

```typescript
export interface AgentConfig {
  id?: string;
  name: string;
  description?: string;
  capabilities: Capability[];
  memory: MemoryConfig;
  workflow?: WorkflowConfig;
  metadata?: Record<string, any>;
}
```

#### Capability

```typescript
export interface Capability {
  id: string;
  type: 'llm' | 'tool' | 'retrieval' | 'custom';
  apiProvider?: string;
  model?: string;
  config?: Record<string, any>;
}
```

#### ExecutionChunk

```typescript
export interface ExecutionChunk {
  type: 'thinking' | 'result' | 'tool_call' | 'tool_result' | 'error';
  content: any;
  timestamp: number;
}
```

### 5.4.2 内存相关结构

#### MemoryConfig

```typescript
export interface MemoryConfig {
  type: 'simple' | 'vector' | 'distributed';
  config?: Record<string, any>;
}
```

#### MemoryQuery

```typescript
export interface MemoryQuery {
  filter?: Record<string, any>;
  vector?: number[];
  similarity?: number;
  limit?: number;
  offset?: number;
  sort?: {
    field: string;
    direction: 'asc' | 'desc';
  }[];
}
```

### 5.4.3 P2P网络相关结构

#### PeerInfo

```typescript
export interface PeerInfo {
  id: string;
  addresses: string[];
  metadata?: Record<string, any>;
  lastSeen?: number;
}
```

#### NetworkStats

```typescript
export interface NetworkStats {
  peersCount: number;
  knownAddresses: number;
  bytesReceived: number;
  bytesSent: number;
  connectedSince: number;
}
```

## 5.5 错误处理

### 5.5.1 错误码

| 错误码 | 描述 |
|--------|------|
| `AGENT_NOT_FOUND` | 指定的Agent不存在 |
| `AGENT_CREATION_FAILED` | Agent创建失败 |
| `AGENT_EXECUTION_FAILED` | Agent执行失败 |
| `MEMORY_ITEM_NOT_FOUND` | 内存项不存在 |
| `MEMORY_OPERATION_FAILED` | 内存操作失败 |
| `P2P_CONNECTION_FAILED` | P2P连接失败 |
| `P2P_NOT_ENABLED` | P2P功能未启用 |
| `INVALID_CONFIGURATION` | 配置无效 |
| `UNAUTHORIZED` | 未授权访问 |
| `INTERNAL_ERROR` | 内部错误 |

### 5.5.2 错误结构

```typescript
export interface LumosError {
  code: string;
  message: string;
  details?: any;
  stack?: string;
}
```

### 5.5.3 错误处理示例

```typescript
try {
  const agent = await client.createAgent(agentConfig);
  const result = await agent.process(input);
} catch (error) {
  if (error.code === 'AGENT_CREATION_FAILED') {
    console.error('Agent创建失败:', error.message);
    // 处理Agent创建失败
  } else if (error.code === 'AGENT_EXECUTION_FAILED') {
    console.error('Agent执行失败:', error.message);
    // 处理执行失败
  } else {
    console.error('未知错误:', error);
    // 处理其他错误
  }
}
```

## 5.6 使用示例

### 5.6.1 创建并使用Agent

```typescript
import { LumosClient } from '@lomusai/client-js';

async function main() {
  // 创建客户端实例
  const client = new LumosClient({
    mode: 'local',
    p2p: { enabled: true }
  });
  
  // 初始化客户端
  await client.initialize();
  
  // 创建Agent配置
  const agentConfig = {
    name: "研究助手",
    description: "帮助用户进行研究和信息收集",
    capabilities: [
      {
        id: "text-generation",
        type: "llm",
        model: "gpt-4"
      },
      {
        id: "web-search",
        type: "tool",
        config: {
          provider: "google"
        }
      },
      {
        id: "document-retrieval",
        type: "retrieval",
        config: {
          dataSourceId: "research-papers"
        }
      }
    ],
    memory: {
      type: "vector",
      config: {
        dimensions: 1536,
        maxItems: 1000
      }
    }
  };
  
  // 创建Agent
  const agent = await client.createAgent(agentConfig);
  console.log(`创建Agent: ${agent.id}`);
  
  // 使用Agent处理请求
  const result = await agent.process({
    message: "查找关于量子计算最新的研究进展"
  });
  
  console.log("处理结果:", result);
  
  // 流式处理示例
  console.log("流式处理开始:");
  for await (const chunk of agent.processStream({
    message: "详细解释量子纠缠的工作原理"
  })) {
    if (chunk.type === 'result') {
      process.stdout.write(chunk.content.text);
    } else if (chunk.type === 'tool_call') {
      console.log(`\n[调用工具: ${chunk.content.tool}]`);
    } else if (chunk.type === 'tool_result') {
      console.log(`\n[工具结果]`);
    }
  }
}

main().catch(console.error);
```

### 5.6.2 P2P网络示例

```typescript
import { LumosClient } from '@lomusai/client-js';

async function setupP2PNetwork() {
  // 创建客户端实例
  const client = new LumosClient({
    mode: 'local',
    p2p: {
      enabled: true,
      bootstrapNodes: [
        '/ip4/104.131.131.82/tcp/4001/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ'
      ]
    }
  });
  
  // 初始化客户端
  await client.initialize();
  
  // 获取P2P管理器
  const p2p = await client.p2p();
  
  // 获取节点ID
  const nodeId = await p2p.getNodeId();
  console.log(`本地节点ID: ${nodeId}`);
  
  // 订阅主题
  console.log("订阅 'research-updates' 主题...");
  const unsubscribe = await p2p.subscribe('research-updates', (data, from) => {
    console.log(`收到来自 ${from} 的更新:`);
    console.log(data);
  });
  
  // 发布内容到主题
  console.log("发布研究更新...");
  await p2p.publish('research-updates', {
    title: "新发现: 量子计算突破",
    author: nodeId,
    timestamp: Date.now(),
    summary: "研究人员发现了提高量子比特稳定性的新方法..."
  });
  
  // 存储内容
  console.log("存储研究报告...");
  const reportData = {
    title: "量子计算研究报告",
    content: "这是一份关于量子计算最新进展的研究报告...",
    author: "研究团队",
    date: new Date().toISOString()
  };
  
  const cid = await p2p.storeContent(reportData);
  console.log(`内容已存储，CID: ${cid}`);
  
  // 查找提供特定内容的节点
  console.log(`查找提供内容 ${cid} 的节点...`);
  const providers = await p2p.findProviders(cid);
  console.log(`找到 ${providers.length} 个提供者`);
  
  // 获取已连接的节点列表
  const peers = await p2p.getPeers();
  console.log(`已连接 ${peers.length} 个节点:`);
  peers.forEach(peer => {
    console.log(`- ${peer.id} (${peer.addresses.join(', ')})`);
  });
  
  return { client, p2p, cid, unsubscribe };
}

setupP2PNetwork().catch(console.error);
```

### 5.6.3 内存管理示例

```typescript
import { LumosClient } from '@lomusai/client-js';

async function memoryManagementExample() {
  // 创建客户端实例
  const client = new LumosClient({ mode: 'local' });
  await client.initialize();
  
  // 获取内存管理器
  const memory = await client.memory();
  
  // 存储多个内存项
  const items = [
    {
      content: {
        type: "conversation",
        text: "量子计算基础概念讨论",
        participants: ["user", "assistant"]
      },
      metadata: {
        importance: "high",
        category: "quantum-computing",
        session: "s-001"
      },
      tags: ["quantum", "basics"],
      type: "conversation"
    },
    {
      content: {
        type: "research",
        title: "量子纠缠研究进展",
        summary: "最新的量子纠缠研究表明..."
      },
      metadata: {
        source: "research-paper",
        journal: "Quantum Science",
        year: 2023
      },
      tags: ["quantum", "entanglement", "research"],
      type: "document"
    },
    {
      content: {
        type: "code",
        language: "python",
        code: "from qiskit import QuantumCircuit\n# 量子电路示例\n..."
      },
      metadata: {
        purpose: "demonstration",
        complexity: "medium"
      },
      tags: ["quantum", "code", "qiskit"],
      type: "code-snippet"
    }
  ];
  
  // 存储所有项目
  const ids = await Promise.all(items.map(item => memory.store(item)));
  console.log("存储的内存项IDs:", ids);
  
  // 基于标签查询
  console.log("\n查询带有'quantum'和'research'标签的项目:");
  const researchItems = await memory.query({
    filter: {
      tags: {
        $contains: ["quantum", "research"]
      }
    }
  });
  console.log(`找到 ${researchItems.length} 个项目`);
  researchItems.forEach(item => console.log(` - ${item.content.title || item.content.text}`));
  
  // 基于元数据查询
  console.log("\n查询高重要性的项目:");
  const importantItems = await memory.query({
    filter: {
      "metadata.importance": "high"
    }
  });
  console.log(`找到 ${importantItems.length} 个项目`);
  importantItems.forEach(item => console.log(` - ${item.content.text}`));
  
  // 更新内存项
  const updateId = ids[0];
  console.log(`\n更新内存项 ${updateId}`);
  await memory.update(updateId, {
    metadata: {
      importance: "critical",
      updated: true
    },
    tags: [...items[0].tags, "important"]
  });
  
  // 检索更新后的项目
  const updatedItem = await memory.retrieve(updateId);
  console.log("更新后的项目:");
  console.log(JSON.stringify(updatedItem, null, 2));
  
  // 删除内存项
  console.log(`\n删除内存项 ${ids[2]}`);
  await memory.delete(ids[2]);
  
  // 确认删除
  const deletedItem = await memory.retrieve(ids[2]);
  console.log(`项目是否已删除: ${deletedItem === null}`);
  
  // 最终计数
  const finalCount = (await memory.query({ limit: 100 })).length;
  console.log(`\n内存中剩余 ${finalCount} 个项目`);
  
  return { memory, ids };
}

memoryManagementExample().catch(console.error);
``` 