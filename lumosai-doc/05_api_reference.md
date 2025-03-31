# 5. API 参考

本章节提供 Lumosai 框架的详细 API 参考，包括核心模块、客户端库和服务器 API。

## 5.1 lumosai_core API

### 5.1.1 Agent API

#### Agent 接口

Agent 是 Lumosai 的核心概念，提供智能代理的实现。

```rust
/// Agent trait 定义了智能代理的基本接口
pub trait Agent: Send + Sync {
    /// 获取代理唯一标识符
    fn id(&self) -> &str;
    
    /// 获取代理名称
    fn name(&self) -> &str;
    
    /// 获取代理描述
    fn description(&self) -> Option<&str>;
    
    /// 执行代理，处理输入并返回结果
    fn execute(&self, input: Value) -> BoxFuture<'static, Result<Value, AgentError>>;
    
    /// 流式执行代理，返回处理过程中的中间结果
    fn execute_stream(&self, input: Value) -> BoxStream<'static, Result<ExecutionChunk, AgentError>>;
    
    /// 获取代理可用的工具列表
    fn tools(&self) -> Vec<Arc<dyn Tool>>;
    
    /// 获取代理关联的内存管理器
    fn memory_manager(&self) -> Arc<dyn MemoryManager>;
}
```

#### SimpleAgent 实现

```rust
/// 基本 Agent 实现，支持大多数常见场景
pub struct SimpleAgent {
    id: String,
    name: String,
    description: Option<String>,
    llm_provider: Arc<dyn LlmProvider>,
    tools: Vec<Arc<dyn Tool>>,
    memory_manager: Arc<dyn MemoryManager>,
    executor: Arc<dyn Executor>,
}

impl SimpleAgent {
    /// 创建新的 SimpleAgent 实例
    pub fn new(
        name: impl Into<String>,
        description: impl Into<Option<String>>,
        llm_provider: Arc<dyn LlmProvider>,
    ) -> Self;
    
    /// 添加工具
    pub fn add_tool(&mut self, tool: impl Tool + 'static) -> &mut Self;
    
    /// 设置内存管理器
    pub fn with_memory(&mut self, memory_manager: Arc<dyn MemoryManager>) -> &mut Self;
    
    /// 设置执行器
    pub fn with_executor(&mut self, executor: Arc<dyn Executor>) -> &mut Self;
}
```

#### AgentBuilder 构建器

```rust
/// Agent 构建器，提供流畅的 API 创建 Agent
pub struct AgentBuilder {
    config: AgentConfig,
    llm_provider: Option<Arc<dyn LlmProvider>>,
    tools: Vec<Arc<dyn Tool>>,
    memory_manager: Option<Arc<dyn MemoryManager>>,
    executor: Option<Arc<dyn Executor>>,
}

impl AgentBuilder {
    /// 创建新的 AgentBuilder
    pub fn new(name: impl Into<String>) -> Self;
    
    /// 设置描述
    pub fn description(mut self, description: impl Into<String>) -> Self;
    
    /// 设置 LLM 提供商
    pub fn llm_provider(mut self, provider: Arc<dyn LlmProvider>) -> Self;
    
    /// 添加工具
    pub fn add_tool(mut self, tool: impl Tool + 'static) -> Self;
    
    /// 设置内存管理器
    pub fn memory_manager(mut self, manager: Arc<dyn MemoryManager>) -> Self;
    
    /// 设置执行器
    pub fn executor(mut self, executor: Arc<dyn Executor>) -> Self;
    
    /// 构建 Agent 实例
    pub fn build(self) -> Result<SimpleAgent, AgentError>;
}
```

### 5.1.2 工具 API

工具（Tool）是 Agent 的扩展能力，允许 Agent 执行特定任务。

#### Tool 接口

```rust
/// Tool trait 定义工具的基本接口
pub trait Tool: Send + Sync {
    /// 获取工具名称
    fn name(&self) -> &str;
    
    /// 获取工具描述
    fn description(&self) -> &str;
    
    /// 获取工具的 JSON Schema 定义
    fn schema(&self) -> Value;
    
    /// 执行工具，处理参数并返回结果
    fn execute(&self, params: Value) -> BoxFuture<'static, Result<Value, ToolError>>;
}
```

#### FunctionTool 实现

```rust
/// 基于函数的工具实现
pub struct FunctionTool<F> {
    name: String,
    description: String,
    schema: Value,
    func: F,
}

impl<F, Fut> FunctionTool<F>
where
    F: Fn(Value) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Value, ToolError>> + Send + 'static,
{
    /// 创建新的函数工具
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        func: F,
    ) -> Self;
    
    /// 设置参数 Schema
    pub fn with_schema(mut self, schema: Value) -> Self;
}
```

#### HttpTool 实现

```rust
/// 基于 HTTP 的工具实现，支持远程 API 调用
pub struct HttpTool {
    name: String,
    description: String,
    base_url: String,
    client: reqwest::Client,
    headers: HeaderMap,
}

impl HttpTool {
    /// 创建新的 HTTP 工具
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Self;
    
    /// 添加请求头
    pub fn add_header(&mut self, key: impl Into<String>, value: impl Into<String>) -> Result<&mut Self, ToolError>;
    
    /// 设置认证
    pub fn with_auth(&mut self, auth_type: AuthType, credentials: impl Into<String>) -> &mut Self;
}
```

### 5.1.3 内存管理 API

内存管理（Memory）提供 Agent 的记忆能力，支持短期和长期记忆。

#### MemoryManager 接口

```rust
/// 内存管理接口，定义内存操作
pub trait MemoryManager: Send + Sync {
    /// 存储内存项
    async fn store(&self, item: MemoryItem) -> Result<String, MemoryError>;
    
    /// 检索内存项
    async fn retrieve(&self, id: &str) -> Result<Option<MemoryItem>, MemoryError>;
    
    /// 查询内存
    async fn query(&self, query: MemoryQuery) -> Result<Vec<MemoryItem>, MemoryError>;
    
    /// 更新内存项
    async fn update(&self, id: &str, updates: MemoryUpdate) -> Result<MemoryItem, MemoryError>;
    
    /// 删除内存项
    async fn delete(&self, id: &str) -> Result<bool, MemoryError>;
}
```

#### MemoryItem 结构

```rust
/// 内存项，表示一条记忆
pub struct MemoryItem {
    /// 唯一标识符
    pub id: Option<String>,
    
    /// 内容
    pub content: Value,
    
    /// 元数据
    pub metadata: HashMap<String, Value>,
    
    /// 嵌入向量（可选）
    pub embedding: Option<Vec<f32>>,
    
    /// 标签
    pub tags: Option<Vec<String>>,
    
    /// 类型
    pub type_: String,
}

impl MemoryItem {
    /// 创建新的内存项
    pub fn new(content: Value, type_: impl Into<String>) -> Self;
    
    /// 添加元数据
    pub fn add_metadata(&mut self, key: impl Into<String>, value: Value) -> &mut Self;
    
    /// 设置嵌入向量
    pub fn with_embedding(&mut self, embedding: Vec<f32>) -> &mut Self;
    
    /// 添加标签
    pub fn add_tag(&mut self, tag: impl Into<String>) -> &mut Self;
}
```

#### LocalMemory 实现

```rust
/// 本地内存实现，使用内存和可选的本地存储
pub struct LocalMemory {
    items: RwLock<HashMap<String, MemoryItem>>,
    storage: Option<Arc<dyn Store>>,
    embedder: Option<Arc<dyn EmbeddingModel>>,
}

impl LocalMemory {
    /// 创建新的本地内存
    pub fn new() -> Self;
    
    /// 设置存储后端
    pub fn with_storage(&mut self, storage: Arc<dyn Store>) -> &mut Self;
    
    /// 设置嵌入模型
    pub fn with_embedder(&mut self, embedder: Arc<dyn EmbeddingModel>) -> &mut Self;
}
```

### 5.1.4 LLM 模型 API

LLM 模型 API 提供与各种大型语言模型的统一接口。

#### LlmProvider 接口

```rust
/// LLM 提供商接口，定义与语言模型的交互
pub trait LlmProvider: Send + Sync {
    /// 获取提供商名称
    fn name(&self) -> &str;
    
    /// 获取可用模型列表
    fn models(&self) -> Vec<String>;
    
    /// 完成请求，生成响应
    fn complete(&self, request: LlmRequest) -> BoxFuture<'static, Result<LlmResponse, LlmError>>;
    
    /// 流式完成请求，生成分块响应
    fn complete_stream(&self, request: LlmRequest) -> BoxStream<'static, Result<LlmResponseChunk, LlmError>>;
}
```

#### LlmRequest 结构

```rust
/// LLM 请求结构
pub struct LlmRequest {
    /// 模型名称
    pub model: String,
    
    /// 消息列表
    pub messages: Vec<Message>,
    
    /// 温度参数
    pub temperature: Option<f32>,
    
    /// 最大令牌数
    pub max_tokens: Option<u32>,
    
    /// 可用工具
    pub tools: Option<Vec<ToolDefinition>>,
    
    /// 是否强制调用工具
    pub tool_choice: Option<ToolChoice>,
}

/// 消息结构
pub struct Message {
    /// 消息角色
    pub role: MessageRole,
    
    /// 消息内容
    pub content: String,
    
    /// 工具调用（如果有）
    pub tool_calls: Option<Vec<ToolCall>>,
    
    /// 工具调用结果（如果有）
    pub tool_call_result: Option<ToolCallResult>,
}
```

#### OpenAiAdapter 实现

```rust
/// OpenAI 模型适配器
pub struct OpenAiAdapter {
    api_key: String,
    model: String,
    client: reqwest::Client,
    base_url: String,
}

impl OpenAiAdapter {
    /// 创建新的 OpenAI 适配器
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self;
    
    /// 设置自定义基础 URL
    pub fn with_base_url(&mut self, base_url: impl Into<String>) -> &mut Self;
}
```

### 5.1.5 工作流 API

工作流 API 支持创建多步骤、多 Agent 协作的流程。

#### Workflow 结构

```rust
/// 工作流，表示多步骤任务流程
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<Step>,
    pub inputs: Value,
}

impl Workflow {
    /// 创建新的工作流
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self;
    
    /// 添加步骤
    pub fn add_step(&mut self, step: Step) -> &mut Self;
    
    /// 设置描述
    pub fn with_description(&mut self, description: impl Into<String>) -> &mut Self;
    
    /// 设置输入
    pub fn with_inputs(&mut self, inputs: Value) -> &mut Self;
    
    /// 执行工作流
    pub async fn execute(&self, context: Context) -> Result<Value, WorkflowError>;
}
```

#### Step 结构

```rust
/// 工作流步骤
pub struct Step {
    pub id: String,
    pub name: String,
    pub agent_id: String,
    pub instructions: Option<String>,
    pub condition: Option<Condition>,
    pub outputs: Option<Vec<OutputMapping>>,
}

impl Step {
    /// 创建新的步骤
    pub fn new(id: impl Into<String>, name: impl Into<String>, agent_id: impl Into<String>) -> Self;
    
    /// 设置执行条件
    pub fn with_condition(&mut self, condition: Condition) -> &mut Self;
    
    /// 设置指令
    pub fn with_instructions(&mut self, instructions: impl Into<String>) -> &mut Self;
    
    /// 添加输出映射
    pub fn add_output_mapping(&mut self, mapping: OutputMapping) -> &mut Self;
}
```

#### Condition 结构

```rust
/// 条件表达式，决定步骤是否执行
pub enum Condition {
    /// 总是执行
    Always,
    
    /// 当前面的步骤完成时执行
    StepCompleted(String),
    
    /// 当值满足条件时执行
    ValueEquals { path: String, value: Value },
    
    /// 当值匹配正则表达式时执行
    ValueMatches { path: String, pattern: String },
    
    /// 组合条件 - 与
    And(Vec<Condition>),
    
    /// 组合条件 - 或
    Or(Vec<Condition>),
    
    /// 组合条件 - 非
    Not(Box<Condition>),
}
```

## 5.2 @lumosai/client-js API

JavaScript 客户端库提供从 JavaScript/TypeScript 访问 Lumosai 功能的接口。

### 5.2.1 客户端 API

#### LumosClient 类

```typescript
/**
 * Lumosai 客户端主类
 */
export class LumosClient {
  /**
   * 创建新的客户端实例
   */
  constructor(config?: Partial<LumosConfig>);
  
  /**
   * 初始化客户端
   */
  async initialize(): Promise<void>;
  
  /**
   * 创建 Agent 实例
   */
  createAgent(config: AgentConfig): Agent;
  
  /**
   * 创建工作流实例
   */
  createWorkflow(config: WorkflowConfig): Workflow;
  
  /**
   * 获取内存管理器
   */
  memory(): Memory;
  
  /**
   * 获取存储管理器
   */
  storage(): Storage;
  
  /**
   * 设置 LLM 提供商
   */
  setLlmProvider(provider: LlmProviderConfig): void;
}
```

#### Agent 类

```typescript
/**
 * Agent 客户端类
 */
export class Agent {
  /**
   * 获取 Agent ID
   */
  id(): string;
  
  /**
   * 执行 Agent
   */
  async execute(input: any): Promise<any>;
  
  /**
   * 流式执行 Agent
   */
  async* executeStream(input: any): AsyncGenerator<ExecutionChunk, void, unknown>;
  
  /**
   * 添加工具
   */
  addTool(tool: Tool): this;
  
  /**
   * 获取工具列表
   */
  tools(): Tool[];
  
  /**
   * 更新 Agent 配置
   */
  updateConfig(config: Partial<AgentConfig>): this;
}
```

### 5.2.2 工具 API

```typescript
/**
 * 工具接口
 */
export interface Tool {
  /** 工具名称 */
  name: string;
  
  /** 工具描述 */
  description: string;
  
  /** 参数模式 */
  schema: Record<string, any>;
  
  /** 执行工具 */
  execute(params: any): Promise<any>;
}

/**
 * 函数工具
 */
export class FunctionTool implements Tool {
  /**
   * 创建函数工具
   */
  constructor(
    name: string,
    description: string,
    fn: (params: any) => Promise<any> | any,
    schema?: Record<string, any>
  );
}

/**
 * HTTP 工具
 */
export class HttpTool implements Tool {
  /**
   * 创建 HTTP 工具
   */
  constructor(
    name: string,
    description: string,
    baseUrl: string,
    options?: HttpToolOptions
  );
  
  /**
   * 添加请求头
   */
  addHeader(key: string, value: string): this;
  
  /**
   * 设置认证
   */
  setAuth(type: 'basic' | 'bearer' | 'api-key', credentials: string): this;
}
``` 