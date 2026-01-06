# 教程 04：RAG 检索增强生成系统

> 本教程覆盖简化 RAG API 与生产级 RAG 管道；解释差异与迁移路径，并给出函数级注释的代码示例。

## 🧩 简化 RAG API（教学占位）
- 位置：`src/rag.rs`，类型与函数：`RagSystem`、`rag::simple()`、`rag::builder()`。
- 说明：`SimpleRagImpl` 采用内存与关键词匹配，返回示例性结果，适合教程演示，不适用于生产。

```rust
use lumosai::prelude::*;

#[tokio::main]
/// 使用简化 RAG 管道：添加文档并进行问答
async fn main() -> Result<()> {
    let rag = lumosai::rag::simple().await?;

    /// 添加文档到简化知识库
    rag.add_document("doc_1", "LumosAI 是一个模块化 AI 框架，支持 Agent、RAG 与工具集成").await?;

    /// 发起问题，获得增强回答（教学实现）
    let ans = rag.answer("LumosAI 的核心模块是什么？").await?;
    println!("RAG 回答: {}", ans);
    Ok(())
}
```

### 构建器用法（简化）
```rust
use lumosai::prelude::*;

#[tokio::main]
/// 通过 builder 自定义分块与检索参数（简化实现）
async fn main() -> Result<()> {
    let storage = lumosai::vector::memory().await?;

    let rag = lumosai::rag::builder()
        .storage(storage)
        .embedding_provider("openai")
        .chunking_strategy("semantic")
        .chunk_size(800)
        .chunk_overlap(100)
        .retrieval_strategy("hybrid")
        .top_k(10)
        .build()
        .await?;

    Ok(())
}
```

## 🏭 生产级 RAG（`lumosai_core` 与 `lumosai_rag`）
- 推荐入口：`lumosai_core::rag::{RagPipeline, RagPipelineBuilder, BasicRagPipeline}`。
- 向量存储：结合 `lumosai_vector`（Memory/Qdrant/Postgres/LanceDB 等）。

```rust
use lumosai_core::prelude::*;
use lumosai_core::rag::{RagPipelineBuilder, DocumentSource};

#[tokio::main]
/// 使用生产级 RagPipelineBuilder 构建管道并查询
async fn main() -> Result<()> {
    // 1) 构建管道并添加数据源
    let pipeline = RagPipelineBuilder::new("kb")
        .add_source(DocumentSource::from_text("LumosAI 支持 Agent、工具与 RAG 集成。"))
        .build()
        .await?;

    // 2) 执行查询
    let result = pipeline.query("介绍 LumosAI 的核心能力", 5).await?;
    println!("相关文档数量: {}", result.documents.len());
    Ok(())
}
```

### 与向量存储结合
```rust
use lumosai_vector::memory::MemoryVectorStorage;
use lumosai_vector_core::{VectorStorage, IndexConfig, Document, SearchQuery};
use std::sync::Arc;

#[tokio::main]
/// 在生产中使用内存向量存储（或替换为 Qdrant/Postgres）
async fn main() -> anyhow::Result<()> {
    let store = Arc::new(MemoryVectorStorage::new());
    store.create_index("kb", IndexConfig::default()).await?;

    // 插入文档供检索
    store.insert("kb", Document { id: "doc1".into(), content: "LumosAI 是模块化框架".into(), metadata: None }).await?;

    // 执行检索
    let results = store.search("kb", SearchQuery::Text { text: "模块化" }, 5).await?;
    println!("检索结果: {}", results.len());
    Ok(())
}
```

### 将 RAG 接入 Agent
```rust
use lumosai_core::prelude::*;
use std::sync::Arc;

#[tokio::main]
/// 构建带 RAG 能力的 Agent（示例接口）
async fn main() -> Result<()> {
    // 1) 构建 RAG 管道（简化演示）
    let pipeline = lumosai_core::rag::create_basic_rag_pipeline("kb", |text| {
        // 示例嵌入函数：返回伪向量（生产中替换为真实嵌入提供商）
        Ok(vec![text.len() as f32])
    });

    // 2) 配置 Agent 并绑定 RAG（不同版本可能使用 builder.with_rag(...)）
    let agent = lumosai_core::agent::AgentBuilder::new()
        .name("RAG 助手")
        .instructions("回答问题时结合知识库检索")
        .model_name("gpt-4o")
        // .with_rag(rag_config) // 若版本提供该方法
        .build()?;

    let resp = agent.generate("LumosAI 的模块组成有哪些？").await?;
    println!("回复: {}", resp);
    Ok(())
}
```

## 🔎 差异与迁移建议
- 教学实现（`src/rag.rs`）仅用于演示，默认简化，不进行真实嵌入与语义检索。
- 生产建议使用 `lumosai_core` + `lumosai_rag` + `lumosai_vector`，根据场景启用重排序与高级分块策略。
- 在 Agent 层与 RAG 管道解耦，通过 Builder 注入 RAG 能力或运行时组合。

## ✅ 最佳实践
- 文档分块策略根据文本类型选择（递归字符/语义分块/自适应分块）。
- 向量存储层面合理设置 `IndexConfig` 与集合名称，保证检索性能。
- 对外部嵌入/向量服务进行健康检查与重试策略配置。
- 在评估集上进行检索质量与生成质量评估（`lumosai_evals`）。

---

如果你有特定的知识库结构或外部向量服务（如 pgvector/Qdrant），告诉我具体需求，我会给出端到端的落地示例。