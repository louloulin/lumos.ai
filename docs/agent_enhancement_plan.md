# Lomusai Agent系统增强计划

基于对Mastra Agent系统的分析，本文档提出了Lomusai Agent系统的增强计划，旨在让其具备与Mastra同等的功能，同时保持Rust的优势。

## 1. 核心功能增强

### 1.1 Agent配置增强

**当前状态**：
- 基本的Agent配置，包含name、instructions和memory_config
- 有限的Stream和Generate选项

**目标增强**：
- ✅ 增加模型配置选项，支持多LLM模型
- ✅ 增加语音配置选项
- ✅ 增加遥测配置选项
- ✅ 增加结构化输出支持
- ✅ 增加中断信号支持

```rust
// 增强的AgentConfig结构
pub struct AgentConfig {
    pub name: String,
    pub instructions: String,
    pub memory_config: Option<MemoryConfig>,
    pub model_id: Option<String>,
    pub voice_config: Option<VoiceConfig>,
    pub telemetry: Option<TelemetrySettings>,
}

// 语音配置
pub struct VoiceConfig {
    pub provider: Option<String>,
    pub voice_id: Option<String>,
    pub settings: Option<serde_json::Value>,
}

// 遥测设置
pub struct TelemetrySettings {
    pub is_enabled: bool,
    pub record_inputs: bool,
    pub record_outputs: bool,
    pub function_id: Option<String>,
    pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
}
```

### 1.2 Agent接口增强

**当前状态**：
- 基本的Agent trait，提供generate和stream方法
- 简单的工具支持和内存集成

**目标增强**：
- ✅ 增加结构化输出支持
- ✅ 增加事件回调机制(onStepFinish, onFinish)
- ✅ 增加更细粒度的工具选择控制
- ✅ 增加对语音功能的支持
- ✅ 增强内存管理，添加工作内存(Working Memory)支持

```rust
#[async_trait]
pub trait Agent: Base + Send + Sync {
    // 现有方法...
    
    // 新增方法
    async fn generate_structured<T: DeserializeOwned + Send + 'static>(
        &self, 
        messages: &[Message], 
        options: &AgentGenerateOptions
    ) -> Result<T>;
    
    async fn stream_with_callbacks<'a>(
        &'a self, 
        messages: &'a [Message], 
        options: &'a AgentStreamOptions,
        on_step_finish: Option<Box<dyn FnMut(AgentStep) + Send + 'a>>,
        on_finish: Option<Box<dyn FnOnce(AgentGenerateResult) + Send + 'a>>
    ) -> Result<BoxStream<'a, Result<String>>>;
    
    // 语音支持
    async fn speak(&self, text: &str, options: &VoiceOptions) -> Result<BoxStream<'_, Result<Vec<u8>>>>;
    async fn listen(&self, audio: impl AsyncRead + Send + 'static, options: &ListenOptions) -> Result<String>;
}
```

### 1.3 Agent实现改进

**当前状态**：
- 基本的BasicAgent实现
- 简单的工具执行流程
- 有限的流式处理支持

**目标增强**：
- ✅ 实现结构化输出解析和验证
- ✅ 改进流处理支持实时结果
- ⏳ 添加工作内存支持
- ✅ 增强工具执行流程
- ✅ 添加语音集成
- ✅ 改进错误处理和异常恢复

## 2. 工具系统增强

### 2.1 工具API改进

**当前状态**：
- 基本的Tool trait
- 功能有限的工具执行环境

**目标增强**：
- ⏳ 增加类型安全的工具接口
- ⏳ 支持Zod schema或JSON Schema的参数验证
- ✅ 添加工具执行上下文，包含线程ID和资源ID
- ⏳ 支持多种工具格式(Mastra格式、Vercel AI SDK格式)
- ✅ 添加中断信号支持
- ✅ 改进错误处理和输出验证

```rust
// 增强的Tool trait
#[async_trait]
pub trait Tool: Base + Send + Sync {
    fn id(&self) -> &str;
    fn description(&self) -> &str;
    fn schema(&self) -> ToolSchema;
    fn output_schema(&self) -> Option<serde_json::Value>;
    
    async fn execute(
        &self, 
        params: serde_json::Value, 
        context: ToolExecutionContext, 
        options: &ToolExecutionOptions
    ) -> Result<serde_json::Value>;
}

// 工具执行上下文
pub struct ToolExecutionContext {
    pub thread_id: Option<String>,
    pub resource_id: Option<String>,
    pub run_id: Option<String>,
    pub tool_call_id: Option<String>,
    pub messages: Option<Vec<Message>>,
    pub abort_signal: Option<tokio::sync::watch::Receiver<bool>>,
}
```

### 2.2 工具注册和管理

**当前状态**：
- 简单的工具注册机制
- 缺少工具集和第三方工具支持

**目标增强**：
- ⏳ 实现工具集(Toolsets)支持
- ⏳ 支持MCP工具集成
- ⏳ 动态工具注册和发现
- ⏳ 细粒度的工具权限控制

## 3. 内存系统增强

### 3.1 工作内存支持

**当前状态**：
- 基本的对话历史存储
- 有限的内存查询能力

**目标增强**：
- ✅ 添加工作内存支持
- ✅ 实现消息历史的语义搜索
- ✅ 支持内存上下文配置
- ⏳ 提供线程和资源管理

```rust
// 增强的内存配置
pub struct MemoryConfig {
    // 现有字段...
    
    // 新增字段
    pub last_messages: Option<usize>,
    pub semantic_recall: Option<SemanticRecallConfig>,
    pub working_memory: Option<WorkingMemoryConfig>,
    pub threads: Option<ThreadConfig>,
}

pub struct WorkingMemoryConfig {
    pub enabled: bool,
    pub template: Option<String>,
    pub content_type: Option<String>,
}

pub struct SemanticRecallConfig {
    pub top_k: usize,
    pub message_range: MessageRange,
}

pub struct MessageRange {
    pub before: usize,
    pub after: usize,
}
```

### 3.2 内存后端支持

**当前状态**：
- 简单的内存存储接口
- 基本的数据持久化

**目标增强**：
- ⏳ 支持多种存储后端(PostgreSQL, SQLite, KV存储等)
- ⏳ 向量存储集成
- ⏳ 支持内存快照和恢复
- ⏳ 提供线程管理API

### 内存系统增强

| 功能 | 当前状态 | 目标增强 |
|------|----------|----------|
| 工作内存 | ✅ **已实现** <br> 增加了WorkingMemory接口和BasicWorkingMemory实现<br>集成了Agent工作内存支持<br>添加了工作内存的配置和初始化 | 支持临时工作记忆，使Agent能够在会话中保持状态<br>允许Agent读取、写入和更新记忆<br>支持结构化和非结构化内容存储<br>API兼容Mastra工作内存 |
| 语义搜索 | ✅ **已实现** <br> 实现了SemanticMemory接口<br>基于向量数据库的语义检索<br>支持上下文窗口和消息范围<br>提供相似度检索API | 支持高级语义搜索<br>优化召回机制<br>支持文档和消息搜索 |
| 内存上下文 | ✅ **已实现** <br> 实现了记忆上下文窗口<br>支持消息前后关联检索<br>基于语义相关性 | 灵活配置上下文窗口大小<br>支持消息过滤和排序<br>上下文压缩和摘要机制 |

## 4. 语音集成

### 4.1 语音接口

**当前状态**：
- 缺少语音功能支持

**目标增强**：
- ✅ 添加文本到语音(TTS)支持
- ✅ 添加语音到文本(STT)支持
- ✅ 实现CompositeVoice模式，支持不同的提供商
- ✅ 支持实时语音交互

```rust
// 语音接口
#[async_trait]
pub trait VoiceProvider: Send + Sync {
    async fn speak(&self, text: &str, options: &VoiceOptions) -> Result<BoxStream<'_, Result<Vec<u8>>>>;
    async fn listen(&self, audio: impl AsyncRead + Send + 'static, options: &ListenOptions) -> Result<String>;
    
    // 实时语音支持
    async fn connect(&self) -> Result<()>;
    async fn send(&self, audio: impl AsyncRead + Send + 'static) -> Result<()>;
    async fn close(&self) -> Result<()>;
    
    fn on<E: VoiceEvent>(&self, callback: Box<dyn FnMut(E) + Send + 'static>) -> Result<()>;
}
```

### 4.2 语音提供商集成

**目标增强**：
- ✅ 支持OpenAI TTS/STT
- ⏳ 支持其他云提供商(Azure, Google等)
- ✅ 支持本地语音处理
- ✅ 实现语音流处理

## 5. 构建Mastra兼容的API

**目标增强**：
- ⏳ 构建类似Mastra的API接口
- ⏳ 支持前端框架集成
- ⏳ 提供TypeScript/JavaScript绑定
- ⏳ 支持WebSocket流式通信

## 实施计划

### 阶段1：基础增强（2-3周）
1. ✅ 更新Agent配置和接口
2. ✅ 改进工具系统
3. ⏳ 增强内存系统
4. ✅ 添加结构化输出支持

### 阶段2：高级功能（3-4周）
1. ⏳ 实现工作内存
2. ✅ 添加语音支持
3. ⏳ 实现MCP工具集成
4. ✅ 增强流处理和回调机制

### 阶段3：API和集成（2-3周）
1. ⏳ 构建Mastra兼容API
2. ⏳ 提供TypeScript/JavaScript绑定
3. ⏳ 实现WebSocket支持
4. ⏳ 添加示例和文档

## 总结

本增强计划将使Lomusai Agent系统达到与Mastra同等的功能水平，同时保持Rust的性能和安全优势。通过分阶段实施，我们可以逐步完善Agent系统，使其更强大、更灵活，满足复杂的人工智能应用需求。

## 实施进度

✅ = 已完成
⏳ = 进行中

**2024-03-30更新**：
1. 已完成Agent配置增强，包括添加模型配置、语音配置和遥测配置
2. 已实现基本的语音接口和提供商（OpenAI和Mock）
3. 已完成Agent接口增强，添加了结构化输出、回调机制和语音支持
4. 已完成部分工具系统的增强

**下一步计划**：
1. 解决VoiceProvider trait的对象安全问题
2. 实现工作内存支持
3. 完善工具系统，增加类型安全的接口和schema验证
4. 实现更多的语音提供商 