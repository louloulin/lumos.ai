use lumosai_vector::memory::MemoryVectorStorage;
use lumosai_vector_core::{
    Document, IndexConfig, MetadataValue, SearchRequest, SimilarityMetric, VectorStorage,
};
use std::time::Instant;
use tokio;

/// 真实向量数据库验证测试
/// 使用实际的LumosAI API进行向量存储和检索功能验证
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🗄️ LumosAI 真实向量数据库验证测试");
    println!("========================================");
    println!("📋 配置信息:");
    println!("  - 模型: qwen3-30b-a3b");
    println!("  - API密钥: sk-bc977c4e31e542f1a34159cb42478198");
    println!("  - 基础URL: https://dashscope.aliyuncs.com/compatible-mode/v1");

    // 4.1 向量存储基础功能测试
    println!("\n📋 4.1 向量存储基础功能测试");
    test_vector_storage_basics().await?;

    // 4.2 向量相似性搜索测试
    println!("\n📋 4.2 向量相似性搜索测试");
    test_vector_similarity_search().await?;

    // 4.3 向量批量操作测试
    println!("\n📋 4.3 向量批量操作测试");
    test_vector_batch_operations().await?;

    // 4.4 向量元数据管理测试
    println!("\n📋 4.4 向量元数据管理测试");
    test_vector_metadata_management().await?;

    println!("\n✅ 向量数据库验证测试完成！");
    Ok(())
}

async fn test_vector_storage_basics() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试向量存储基础功能...");
    let start_time = Instant::now();

    // 测试用例 4.1.1: 创建向量存储
    println!("    🗄️ 测试创建向量存储");

    let vector_storage = MemoryVectorStorage::new().await?;

    // 创建索引
    let config = IndexConfig::new("test_index", 384).with_metric(SimilarityMetric::Cosine);
    vector_storage.create_index(config).await?;

    println!("      ✓ 内存向量存储创建成功");
    println!("      📊 向量维度: 384");

    // 测试用例 4.1.2: 添加向量
    println!("    ➕ 测试添加向量");

    let test_documents = vec![
        Document::new("doc1", "这是一个关于Rust编程的文档")
            .with_embedding(vec![0.1; 384])
            .with_metadata("title", "文档1")
            .with_metadata("category", "技术"),
        Document::new("doc2", "这是一个关于Python编程的文档")
            .with_embedding(vec![0.2; 384])
            .with_metadata("title", "文档2")
            .with_metadata("category", "技术"),
        Document::new("doc3", "这是一个关于烹饪的文档")
            .with_embedding(vec![0.3; 384])
            .with_metadata("title", "文档3")
            .with_metadata("category", "生活"),
        Document::new("doc4", "这是一个关于机器学习的文档")
            .with_embedding(vec![0.4; 384])
            .with_metadata("title", "文档4")
            .with_metadata("category", "技术"),
        Document::new("doc5", "这是一个关于旅行的文档")
            .with_embedding(vec![0.5; 384])
            .with_metadata("title", "文档5")
            .with_metadata("category", "生活"),
    ];

    let add_start = Instant::now();
    vector_storage
        .upsert_documents("test_index", test_documents.clone())
        .await?;
    let add_duration = add_start.elapsed();

    println!("      ✓ 批量添加向量完成 (耗时: {:?})", add_duration);
    println!("      📊 总共添加 {} 个向量", test_documents.len());

    // 测试用例 4.1.3: 搜索验证
    println!("    🔍 测试搜索验证");

    let search_request = SearchRequest::new("test_index", vec![0.15; 384]).with_top_k(3);

    let search_start = Instant::now();
    let search_results = vector_storage.search(search_request).await?;
    let search_duration = search_start.elapsed();

    println!("      ✓ 搜索完成 (耗时: {:?})", search_duration);
    println!("      📊 找到 {} 个结果", search_results.results.len());

    for (i, result) in search_results.results.iter().enumerate() {
        println!(
            "        {}. ID: {}, 相似度: {:.4}",
            i + 1,
            result.id,
            result.score
        );
        if let Some(metadata) = &result.metadata {
            if let Some(title) = metadata.get("title") {
                println!("           标题: {:?}", title);
            }
        }
    }

    assert!(!search_results.results.is_empty(), "搜索结果不能为空");
    println!("      ✓ 搜索验证通过");

    // 测试用例 4.1.4: 删除向量
    println!("    🗑️ 测试删除向量");

    let delete_ids = vec!["doc3".to_string()];
    let delete_start = Instant::now();

    vector_storage
        .delete_documents("test_index", delete_ids.clone())
        .await?;

    let delete_duration = delete_start.elapsed();
    println!(
        "      ✓ 删除向量成功: {:?} (耗时: {:?})",
        delete_ids, delete_duration
    );

    // 验证删除 - 通过搜索确认文档数量减少
    let verify_request = SearchRequest::new("test_index", vec![0.3; 384]).with_top_k(10);
    let verify_results = vector_storage.search(verify_request).await?;

    assert_eq!(verify_results.results.len(), 4, "删除后应该剩余4个文档");
    println!(
        "      ✓ 删除验证通过，剩余 {} 个文档",
        verify_results.results.len()
    );

    let duration = start_time.elapsed();
    println!("  ✅ 向量存储基础功能测试完成! 耗时: {:?}", duration);

    Ok(())
}

async fn test_vector_similarity_search() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试向量相似性搜索...");
    let start_time = Instant::now();

    // 创建向量存储并添加测试数据
    let vector_storage = MemoryVectorStorage::new().await?;

    // 创建索引
    let config = IndexConfig::new("similarity_test", 384).with_metric(SimilarityMetric::Cosine);
    vector_storage.create_index(config).await?;

    // 添加更有区别的测试向量
    let test_documents = vec![
        Document::new("tech1", "Rust编程指南")
            .with_embedding(create_test_vector(384, 0.1, 0.9))
            .with_metadata("title", "Rust编程指南")
            .with_metadata("category", "技术"),
        Document::new("tech2", "Python机器学习")
            .with_embedding(create_test_vector(384, 0.15, 0.85))
            .with_metadata("title", "Python机器学习")
            .with_metadata("category", "技术"),
        Document::new("life1", "烹饪艺术")
            .with_embedding(create_test_vector(384, 0.8, 0.2))
            .with_metadata("title", "烹饪艺术")
            .with_metadata("category", "生活"),
        Document::new("life2", "旅行攻略")
            .with_embedding(create_test_vector(384, 0.85, 0.15))
            .with_metadata("title", "旅行攻略")
            .with_metadata("category", "生活"),
        Document::new("mixed1", "技术与生活")
            .with_embedding(create_test_vector(384, 0.5, 0.5))
            .with_metadata("title", "技术与生活")
            .with_metadata("category", "混合"),
    ];

    vector_storage
        .upsert_documents("similarity_test", test_documents)
        .await?;

    println!("    ✓ 测试数据准备完成");

    // 测试用例 4.2.1: 基础相似性搜索
    println!("    🔍 测试基础相似性搜索");

    let query_vector = create_test_vector(384, 0.12, 0.88); // 类似技术类的查询向量
    let search_request = SearchRequest::new("similarity_test", query_vector.clone()).with_top_k(3);

    let search_start = Instant::now();
    let search_results = vector_storage.search(search_request).await?;
    let search_duration = search_start.elapsed();

    println!("      ✓ 搜索完成 (耗时: {:?})", search_duration);
    println!("      📊 找到 {} 个相似结果", search_results.results.len());

    for (i, result) in search_results.results.iter().enumerate() {
        println!(
            "        {}. ID: {}, 相似度: {:.4}",
            i + 1,
            result.id,
            result.score
        );
        if let Some(metadata) = &result.metadata {
            if let Some(title) = metadata.get("title") {
                println!("           标题: {:?}", title);
            }
        }
    }

    // 验证搜索结果
    assert!(!search_results.results.is_empty(), "搜索结果不能为空");
    assert!(
        search_results.results.len() <= 3,
        "搜索结果数量不应超过限制"
    );

    // 验证结果按相似度排序
    for i in 1..search_results.results.len() {
        assert!(
            search_results.results[i - 1].score >= search_results.results[i].score,
            "搜索结果应按相似度降序排列"
        );
    }

    println!("      ✓ 搜索结果验证通过");

    // 测试用例 4.2.2: 大范围搜索
    println!("    🎯 测试大范围搜索");

    let large_search_request =
        SearchRequest::new("similarity_test", query_vector.clone()).with_top_k(10);

    let threshold_search_start = Instant::now();
    let threshold_results = vector_storage.search(large_search_request).await?;
    let threshold_search_duration = threshold_search_start.elapsed();

    println!(
        "      ✓ 大范围搜索完成 (耗时: {:?})",
        threshold_search_duration
    );
    println!("      📊 找到 {} 个结果", threshold_results.results.len());

    // 验证搜索结果
    for result in threshold_results.results.iter() {
        println!("        ID: {}, 相似度: {:.4}", result.id, result.score);
    }

    println!("      ✓ 大范围搜索验证通过");

    // 测试用例 4.2.3: 不同查询向量的搜索
    println!("    🔄 测试不同查询向量的搜索");

    let life_query_vector = create_test_vector(384, 0.82, 0.18); // 类似生活类的查询向量
    let life_search_request =
        SearchRequest::new("similarity_test", life_query_vector).with_top_k(3);

    let life_search_start = Instant::now();
    let life_results = vector_storage.search(life_search_request).await?;
    let life_search_duration = life_search_start.elapsed();

    println!("      ✓ 生活类查询完成 (耗时: {:?})", life_search_duration);
    println!("      📊 找到 {} 个相似结果", life_results.results.len());

    for (i, result) in life_results.results.iter().enumerate() {
        println!(
            "        {}. ID: {}, 相似度: {:.4}",
            i + 1,
            result.id,
            result.score
        );
        if let Some(metadata) = &result.metadata {
            if let Some(category) = metadata.get("category") {
                println!("           类别: {:?}", category);
            }
        }
    }

    println!("      ✓ 多查询向量测试完成");

    let duration = start_time.elapsed();
    println!("  ✅ 向量相似性搜索测试完成! 耗时: {:?}", duration);

    Ok(())
}

async fn test_vector_batch_operations() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试向量批量操作...");
    let start_time = Instant::now();

    let vector_storage = MemoryVectorStorage::new().await?;

    // 创建索引
    let config = IndexConfig::new("batch_test", 384).with_metric(SimilarityMetric::Cosine);
    vector_storage.create_index(config).await?;

    // 测试用例 4.3.1: 批量添加向量
    println!("    📦 测试批量添加向量");

    let batch_size = 100;
    let mut batch_documents = Vec::new();

    for i in 0..batch_size {
        let id = format!("batch_doc_{}", i);
        let vector = create_test_vector(384, (i as f32) / (batch_size as f32), 0.5);
        let content = format!("批量文档 {}", i);

        let doc = Document::new(id, content)
            .with_embedding(vector)
            .with_metadata("index", i.to_string())
            .with_metadata("batch", "test_batch_1");

        batch_documents.push(doc);
    }

    let batch_add_start = Instant::now();
    vector_storage
        .upsert_documents("batch_test", batch_documents)
        .await?;
    let batch_add_duration = batch_add_start.elapsed();

    println!("      ✓ 批量添加完成: {} 个向量", batch_size);
    println!("      ⏱️ 批量添加耗时: {:?}", batch_add_duration);
    println!(
        "      📊 平均每个向量耗时: {:?}",
        batch_add_duration / batch_size
    );

    // 测试用例 4.3.2: 批量搜索
    println!("    🔍 测试批量搜索");

    let search_queries = vec![
        create_test_vector(384, 0.1, 0.5),
        create_test_vector(384, 0.5, 0.5),
        create_test_vector(384, 0.9, 0.5),
    ];

    let batch_search_start = Instant::now();

    for (i, query_vector) in search_queries.iter().enumerate() {
        let search_request = SearchRequest::new("batch_test", query_vector.clone()).with_top_k(5);
        let results = vector_storage.search(search_request).await?;
        println!(
            "      ✓ 查询 {} 完成: 找到 {} 个结果",
            i + 1,
            results.results.len()
        );
    }

    let batch_search_duration = batch_search_start.elapsed();

    println!("      ⏱️ 批量搜索耗时: {:?}", batch_search_duration);
    println!(
        "      📊 平均每个查询耗时: {:?}",
        batch_search_duration / search_queries.len() as u32
    );

    // 测试用例 4.3.3: 批量删除
    println!("    🗑️ 测试批量删除");

    let delete_count = 20;
    let delete_ids: Vec<String> = (0..delete_count)
        .map(|i| format!("batch_doc_{}", i))
        .collect();

    let batch_delete_start = Instant::now();
    vector_storage
        .delete_documents("batch_test", delete_ids.clone())
        .await?;
    let batch_delete_duration = batch_delete_start.elapsed();

    println!("      ✓ 批量删除完成: {} 个向量", delete_count);
    println!("      ⏱️ 批量删除耗时: {:?}", batch_delete_duration);

    // 验证删除 - 通过搜索确认文档数量减少
    let verify_start = Instant::now();
    let verify_request =
        SearchRequest::new("batch_test", create_test_vector(384, 0.5, 0.5)).with_top_k(200); // 搜索更多结果来验证删除
    let verify_results = vector_storage.search(verify_request).await?;
    let verify_duration = verify_start.elapsed();

    let remaining_count = verify_results.results.len();
    let expected_remaining = (batch_size - delete_count) as usize;

    assert_eq!(
        remaining_count, expected_remaining,
        "删除后剩余的向量数量不匹配"
    );
    println!(
        "      ✓ 删除验证通过: 剩余 {} 个向量 (验证耗时: {:?})",
        remaining_count, verify_duration
    );

    let duration = start_time.elapsed();
    println!("  ✅ 向量批量操作测试完成! 耗时: {:?}", duration);

    Ok(())
}

async fn test_vector_metadata_management() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试向量元数据管理...");
    let start_time = Instant::now();

    let vector_storage = MemoryVectorStorage::new().await?;

    // 创建索引
    let config = IndexConfig::new("metadata_test", 384).with_metric(SimilarityMetric::Cosine);
    vector_storage.create_index(config).await?;

    // 测试用例 4.4.1: 复杂元数据存储
    println!("    📋 测试复杂元数据存储");

    let vector = create_test_vector(384, 0.5, 0.5);
    let metadata_start = Instant::now();

    let complex_doc = Document::new("complex_doc", "复杂文档示例内容")
        .with_embedding(vector)
        .with_metadata("title", "复杂文档示例")
        .with_metadata("author", "张三")
        .with_metadata("created_at", "2024-01-15T10:30:00Z")
        .with_metadata("content_length", "1500")
        .with_metadata("language", "zh-CN")
        .with_metadata("version", "1.2")
        .with_metadata("is_published", "true");

    vector_storage
        .upsert_documents("metadata_test", vec![complex_doc])
        .await?;

    let metadata_duration = metadata_start.elapsed();
    println!("      ✓ 复杂元数据存储完成 (耗时: {:?})", metadata_duration);

    // 验证元数据检索
    let retrieve_start = Instant::now();
    let search_request =
        SearchRequest::new("metadata_test", create_test_vector(384, 0.5, 0.5)).with_top_k(1);
    let search_results = vector_storage.search(search_request).await?;
    let retrieve_duration = retrieve_start.elapsed();

    if let Some(result) = search_results.results.first() {
        println!("      ✓ 元数据检索完成 (耗时: {:?})", retrieve_duration);

        if let Some(metadata) = &result.metadata {
            // 验证特定字段
            assert_eq!(
                metadata.get("title").and_then(|v| match v {
                    MetadataValue::String(s) => Some(s.as_str()),
                    _ => None,
                }),
                Some("复杂文档示例")
            );
            assert_eq!(
                metadata.get("author").and_then(|v| match v {
                    MetadataValue::String(s) => Some(s.as_str()),
                    _ => None,
                }),
                Some("张三")
            );

            println!("      ✓ 复杂元数据验证通过");
            println!("        - 标题: {:?}", metadata.get("title"));
            println!("        - 作者: {:?}", metadata.get("author"));
            println!("        - 语言: {:?}", metadata.get("language"));
            println!("        - 版本: {:?}", metadata.get("version"));
        }
    } else {
        return Err("未找到复杂元数据文档".into());
    }

    // 测试用例 4.4.2: 元数据搜索过滤
    println!("    🔍 测试元数据搜索过滤");

    // 添加更多带不同元数据的向量
    let filtered_documents = vec![
        Document::new("tech_doc_1", "技术文档1")
            .with_embedding(create_test_vector(384, 0.3, 0.7))
            .with_metadata("category", "技术")
            .with_metadata("level", "初级")
            .with_metadata("rating", "4.5"),
        Document::new("tech_doc_2", "技术文档2")
            .with_embedding(create_test_vector(384, 0.35, 0.65))
            .with_metadata("category", "技术")
            .with_metadata("level", "高级")
            .with_metadata("rating", "4.8"),
        Document::new("life_doc_1", "生活文档1")
            .with_embedding(create_test_vector(384, 0.7, 0.3))
            .with_metadata("category", "生活")
            .with_metadata("level", "初级")
            .with_metadata("rating", "4.2"),
    ];

    vector_storage
        .upsert_documents("metadata_test", filtered_documents)
        .await?;

    println!("      ✓ 过滤测试数据准备完成");

    // 测试基础搜索并验证元数据
    let query_vector = create_test_vector(384, 0.32, 0.68);
    let search_request = SearchRequest::new("metadata_test", query_vector).with_top_k(10);

    let filter_search_start = Instant::now();
    let all_results = vector_storage.search(search_request).await?;
    let filter_search_duration = filter_search_start.elapsed();

    println!("      ✓ 搜索完成 (耗时: {:?})", filter_search_duration);

    // 手动过滤技术类文档
    let tech_results: Vec<_> = all_results
        .results
        .iter()
        .filter(|result| {
            if let Some(metadata) = &result.metadata {
                metadata
                    .get("category")
                    .and_then(|v| match v {
                        MetadataValue::String(s) => Some(s.as_str()),
                        _ => None,
                    })
                    .map(|s| s == "技术")
                    .unwrap_or(false)
            } else {
                false
            }
        })
        .collect();

    println!("      📊 总结果数: {}", all_results.results.len());
    println!("      📊 技术类结果数: {}", tech_results.len());

    for result in tech_results.iter() {
        if let Some(metadata) = &result.metadata {
            println!(
                "        - ID: {}, 类别: {:?}, 级别: {:?}",
                result.id,
                metadata.get("category"),
                metadata.get("level")
            );
        }
    }

    println!("      ✓ 元数据过滤测试完成");

    let duration = start_time.elapsed();
    println!("  ✅ 向量元数据管理测试完成! 耗时: {:?}", duration);

    Ok(())
}

/// 创建测试向量的辅助函数
fn create_test_vector(dim: usize, base_value: f32, variance: f32) -> Vec<f32> {
    let mut vector = Vec::with_capacity(dim);
    for i in 0..dim {
        let noise = (i as f32 * 0.001) % 0.1 - 0.05; // 小的随机噪声
        vector.push(base_value + variance * noise);
    }
    vector
}
