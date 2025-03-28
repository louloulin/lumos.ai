# Lomusai

Lomusai是一个用Rust实现的AI应用开发框架，提供高性能、类型安全的AI应用开发工具。该项目是从TypeScript框架Mastra迁移到Rust的重写版本，旨在提供更高的性能和更强的类型安全保障。

## 主要功能

- **工作流引擎**：基于图的状态机，支持条件执行和并行处理
- **Agent系统**：支持工具调用和上下文管理的智能代理
- **向量存储**：高效的向量数据管理，支持多种相似度计算方法
- **内存系统**：用于存储和检索上下文信息
- **工具集成**：可扩展的工具接口和执行系统
- **LLM集成**：支持多种大型语言模型的统一接口
- **RAG系统**：文档处理、向量嵌入和相似度搜索
- **评估框架**：用于评估LLM输出的系统

## 向量存储功能

向量存储模块提供了高效的向量存储和检索系统，支持多种相似度度量方法，可用于实现语义搜索、推荐系统等功能。

主要特性：

- **多种实现**：内存存储和SQLite存储（可选）
- **相似度度量**：支持余弦相似度、欧氏距离和点积
- **向量索引**：支持创建、查询、更新和删除向量索引
- **元数据过滤**：支持基于向量元数据的复杂过滤条件
- **内置嵌入**：提供测试用随机嵌入生成器

## 快速开始

```rust
use lomusai_core::{
    vector::{
        VectorStorage, 
        create_memory_vector_storage, 
        SimilarityMetric,
    },
};

async fn vector_example() {
    // 创建向量存储
    let storage = create_memory_vector_storage();
    
    // 创建向量索引
    storage.create_index("test_index", 384, Some(SimilarityMetric::Cosine)).await.unwrap();
    
    // 存储向量
    let vectors = vec![vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 0.0]];
    let ids = storage.upsert("test_index", vectors, None, None).await.unwrap();
    
    // 进行向量搜索
    let results = storage.query(
        "test_index",
        vec![1.0, 0.0, 0.0],  // 查询向量
        2,                    // 返回前2个结果
        None,                 // 无过滤条件
        false,                // 不包含向量数据
    ).await.unwrap();
    
    // 处理搜索结果
    for result in results {
        println!("ID: {}, 相似度: {}", result.id, result.score);
    }
}
```

## 项目状态

该项目目前处于开发阶段，已实现的功能包括：

- [x] 核心错误处理系统
- [x] 工作流引擎
- [x] 内存系统
- [x] 向量存储系统
- [x] 评估框架

更多信息请参考[详细迁移计划](lomusai.md)。 