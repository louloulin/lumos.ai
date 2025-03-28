# Lomusai Core

Rust核心库，提供Lomusai框架的基础功能。

## 功能

- 工作流引擎：定义、执行和管理工作流
- 代理系统：创建可扩展的智能代理
- LLM提供者：支持不同的大型语言模型接口
- 存储系统：用于持久化数据的存储接口
- 向量存储：高效的向量存储和检索系统
- 内存模块：管理代理和工作流的状态

## 向量存储功能

向量存储模块提供了高效的向量存储和检索系统，支持多种相似度度量方法，可用于实现语义搜索、推荐系统等功能。

主要功能：

- 多种实现：内存存储和SQLite存储（可选）
- 相似度度量：支持余弦相似度、欧氏距离和点积
- 向量索引：支持创建、查询、更新和删除向量索引
- 元数据过滤：支持基于向量元数据的复杂过滤条件
- 内置嵌入：提供测试用随机嵌入生成器

### 使用示例

```rust
use lomusai_core::{
    vector::{
        VectorStorage, 
        create_memory_vector_storage, 
        SimilarityMetric,
        create_random_embedding,
    },
};

async fn vector_example() {
    // 创建向量存储
    let storage = create_memory_vector_storage();
    
    // 创建嵌入服务
    let embedder = create_random_embedding();
    
    // 创建向量索引
    storage.create_index("test_index", 384, Some(SimilarityMetric::Cosine)).await.unwrap();
    
    // 生成嵌入
    let texts = vec!["这是第一个文档".to_string(), "这是第二个文档".to_string()];
    let embeddings = embedder.embed_texts(&texts).await.unwrap();
    
    // 存储向量
    storage.upsert("test_index", embeddings, None, None).await.unwrap();
    
    // 进行向量搜索
    let query_text = "查询文档".to_string();
    let query_embedding = embedder.embed_texts(&[query_text]).await.unwrap().pop().unwrap();
    
    let results = storage.query(
        "test_index",
        query_embedding,
        2,  // 返回前2个结果
        None,  // 无过滤条件
        false,  // 不包含向量数据
    ).await.unwrap();
    
    // 处理搜索结果
    for result in results {
        println!("ID: {}, 相似度: {}", result.id, result.score);
    }
}
```

### 特性标志

- `vector_sqlite`: 启用SQLite向量存储支持
  
  ```toml
  [dependencies]
  lomusai_core = { version = "0.1.0", features = ["vector_sqlite"] }
  ``` 