# RAG (Retrieval Augmented Generation) 架构分析

## 1. 现状分析

### 1.1 当前实现概览

#### Lumos.ai 当前RAG实现
- **位置**: `lumosai_core/src/rag.rs`, `lumosai_rag/`
- **架构**: 基础RAG管道实现
- **特点**: 
  - 简单的文档处理和嵌入生成
  - 基于内存的向量存储
  - 基础的查询和检索功能

#### Rig框架RAG实现
- **位置**: `rig/rig-core/examples/rag*.rs`
- **架构**: Agent + VectorStore + Embedding模式
- **特点**:
  - 强类型的Embed trait系统
  - 多种向量存储后端支持
  - 与Agent系统深度集成

### 1.2 架构对比分析

| 特性 | Lumos.ai | Rig | Mastra (参考) |
|------|----------|-----|---------------|
| **文档处理** | 基础文本分割 | Embed trait | 高级chunking策略 |
| **嵌入生成** | 简单哈希函数 | 多provider支持 | 多模型支持 |
| **向量存储** | 内存存储 | 多后端支持 | 统一接口 |
| **查询系统** | 基础相似度 | 高级检索 | 混合搜索 |
| **Agent集成** | 基础集成 | 深度集成 | 原生支持 |

## 2. Rig框架深度分析

### 2.1 核心设计模式

#### Embed Trait系统
```rust
// rig的核心设计 - 强类型嵌入
#[derive(Embed, Serialize, Clone, Debug)]
struct WordDefinition {
    id: String,
    word: String,
    #[embed]  // 标记需要嵌入的字段
    definitions: Vec<String>,
}
```

**优势**:
- 编译时类型安全
- 自动嵌入生成
- 灵活的字段选择

#### VectorStore抽象
```rust
// 统一的向量存储接口
pub trait VectorStoreIndex<M: EmbeddingModel> {
    async fn top_n<T>(&self, query: &str, n: usize) -> Result<Vec<(f64, String, T)>>;
    async fn top_n_ids(&self, query: &str, n: usize) -> Result<Vec<(f64, String)>>;
}
```

### 2.2 Agent集成模式
```rust
// Rig的RAG Agent模式
let rag_agent = client.agent("gpt-4")
    .preamble("You are a dictionary assistant...")
    .dynamic_context(1, vector_index)  // 动态上下文注入
    .build();
```

**特点**:
- 动态上下文注入
- 自动检索和上下文构建
- 无缝的LLM集成

## 3. Mastra RAG原语分析

### 3.1 设计理念
基于代码分析，Mastra的RAG设计理念包括：

1. **声明式配置**: 通过配置而非代码定义RAG管道
2. **模块化组件**: 独立的chunk、embed、store组件
3. **管道抽象**: 统一的处理和查询管道
4. **多后端支持**: 支持多种存储和嵌入后端

### 3.2 核心组件

#### 文档处理管道
```rust
// Mastra风格的管道配置
pipeline: {
    chunk: {
        chunk_size: 1000,
        chunk_overlap: 200,
        separator: "\n",
        strategy: "recursive"
    },
    embed: {
        model: "text-embedding-3-small",
        dimensions: 1536,
        max_retries: 3
    },
    store: {
        db: "pgvector",
        collection: "embeddings",
        connection_string: env!("DATABASE_URL")
    }
}
```

#### 查询管道
```rust
query_pipeline: {
    rerank: true,
    top_k: 5,
    filter: r#"{ "type": { "$in": ["article", "faq"] } }"#,
    hybrid_search: {
        enabled: true,
        weight: {
            semantic: 0.7,
            keyword: 0.3
        }
    }
}
```

## 4. 问题识别

### 4.1 当前Lumos.ai RAG的问题

1. **架构分散**: RAG功能分散在多个模块中
2. **功能重复**: vector存储在多处重复实现
3. **集成困难**: 与Agent系统集成不够深入
4. **扩展性差**: 难以添加新的存储后端或嵌入模型
5. **配置复杂**: 缺乏统一的配置接口

### 4.2 Vector存储重复问题

#### 重复的实现
- `lumosai_core::vector::VectorStorage`
- `lumosai_stores::vector::VectorStore`
- `lumos_vector::storage::VectorStorage`
- `lumosai_rag::retriever::VectorStore`

#### 接口不一致
- 方法签名差异
- 错误处理不统一
- 配置方式不同

## 5. 改造计划

### 5.1 总体架构设计

#### 5.1.1 分层架构
```
┌─────────────────────────────────────────┐
│           Application Layer             │
│  (RAG Pipelines, Agent Integration)     │
├─────────────────────────────────────────┤
│           Abstraction Layer             │
│     (Unified Traits & Interfaces)      │
├─────────────────────────────────────────┤
│           Implementation Layer          │
│  (Storage Backends, Embedding Models)   │
├─────────────────────────────────────────┤
│           Foundation Layer              │
│    (Core Types, Error Handling)        │
└─────────────────────────────────────────┘
```

#### 5.1.2 Crate组织结构
```
lumos-vector/           # 核心向量存储抽象
├── lumos-vector-core/  # 核心trait和类型
├── lumos-vector-memory/# 内存存储实现
├── lumos-vector-sqlite/# SQLite存储实现
├── lumos-vector-qdrant/# Qdrant存储实现
└── lumos-vector-mongo/ # MongoDB存储实现

lumos-embedding/        # 嵌入生成抽象
├── lumos-embedding-core/    # 核心trait
├── lumos-embedding-openai/  # OpenAI实现
├── lumos-embedding-ollama/  # Ollama实现
└── lumos-embedding-local/   # 本地模型实现

lumos-rag/             # RAG系统
├── lumos-rag-core/    # 核心RAG抽象
├── lumos-rag-pipeline/# 管道实现
└── lumos-rag-agent/   # Agent集成
```

### 5.2 核心接口设计

#### 5.2.1 统一的Vector Storage接口
```rust
// lumos-vector-core/src/lib.rs
#[async_trait]
pub trait VectorStorage: Send + Sync {
    type Config: Send + Sync;
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn create_index(&self, config: IndexConfig) -> Result<(), Self::Error>;
    async fn upsert(&self, request: UpsertRequest) -> Result<UpsertResponse, Self::Error>;
    async fn search(&self, request: SearchRequest) -> Result<SearchResponse, Self::Error>;
    async fn delete(&self, request: DeleteRequest) -> Result<(), Self::Error>;
    async fn describe(&self, index_name: &str) -> Result<IndexInfo, Self::Error>;
}
```

#### 5.2.2 统一的Embedding接口
```rust
// lumos-embedding-core/src/lib.rs
#[async_trait]
pub trait EmbeddingModel: Send + Sync {
    type Config: Send + Sync;
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>, Self::Error>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, Self::Error>;
    fn dimensions(&self) -> usize;
    fn model_name(&self) -> &str;
}
```

#### 5.2.3 RAG Pipeline接口
```rust
// lumos-rag-core/src/lib.rs
#[async_trait]
pub trait RagPipeline: Send + Sync {
    type Config: Send + Sync;
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn process_documents(&mut self, docs: Vec<Document>) -> Result<ProcessResult, Self::Error>;
    async fn search(&self, query: &str, options: SearchOptions) -> Result<SearchResult, Self::Error>;
    async fn update_document(&mut self, doc: Document) -> Result<(), Self::Error>;
    async fn delete_document(&mut self, doc_id: &str) -> Result<(), Self::Error>;
}
```

### 5.3 实现策略

#### 5.3.1 阶段1: 核心抽象层 (Week 1-2)
- [ ] 创建 `lumos-vector-core` crate
- [ ] 定义统一的 VectorStorage trait
- [ ] 创建 `lumos-embedding-core` crate
- [ ] 定义统一的 EmbeddingModel trait
- [ ] 设计错误处理和类型系统

#### 5.3.2 阶段2: 基础实现 (Week 3-4)
- [ ] 实现 `lumos-vector-memory`
- [ ] 实现 `lumos-vector-sqlite`
- [ ] 实现 `lumos-embedding-openai`
- [ ] 创建集成测试套件

#### 5.3.3 阶段3: RAG系统 (Week 5-6)
- [ ] 创建 `lumos-rag-core`
- [ ] 实现文档处理管道
- [ ] 实现查询和检索系统
- [ ] 集成向量存储和嵌入模型

#### 5.3.4 阶段4: Agent集成 (Week 7-8)
- [ ] 创建 `lumos-rag-agent`
- [ ] 实现动态上下文注入
- [ ] 集成到现有Agent系统
- [ ] 创建高级RAG功能

### 5.4 迁移策略

#### 5.4.1 向后兼容
- 保留现有API作为deprecated
- 提供迁移指南和工具
- 逐步迁移现有代码

#### 5.4.2 渐进式替换
1. 新功能使用新架构
2. 现有功能逐步迁移
3. 最终移除旧实现

## 6. 技术决策

### 6.1 借鉴Rig的优秀设计
- **Embed trait系统**: 强类型嵌入生成
- **VectorStoreIndex模式**: 统一的检索接口
- **Agent集成模式**: 动态上下文注入

### 6.2 借鉴Mastra的配置理念
- **声明式配置**: 通过配置定义RAG管道
- **模块化设计**: 独立的组件可组合
- **管道抽象**: 统一的处理流程

### 6.3 Lumos.ai特色
- **宏系统**: 简化RAG管道创建
- **多语言绑定**: 支持Python/Node.js
- **企业特性**: 监控、安全、合规

## 7. 成功指标

### 7.1 技术指标
- [ ] 统一的接口覆盖率 > 95%
- [ ] 性能提升 > 20%
- [ ] 代码重复率 < 5%
- [ ] 测试覆盖率 > 90%

### 7.2 开发体验指标
- [ ] API一致性评分 > 9/10
- [ ] 文档完整性 > 95%
- [ ] 示例代码覆盖率 > 80%
- [ ] 社区反馈评分 > 8/10

## 8. 风险评估

### 8.1 技术风险
- **依赖冲突**: 多个crate可能导致依赖版本冲突
- **性能回归**: 抽象层可能影响性能
- **兼容性问题**: 新旧API兼容性维护

### 8.2 缓解策略
- 严格的依赖管理
- 性能基准测试
- 全面的兼容性测试
- 渐进式迁移计划
