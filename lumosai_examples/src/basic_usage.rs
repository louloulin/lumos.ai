use std::sync::Arc;
use std::collections::HashMap;

use lumosai_core::{
    Lumosai, LumosaiConfig, LogLevel,
    Base, create_memory_vector_storage, SimilarityMetric
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建Lumosai实例
    let config = LumosaiConfig {
        name: Some("ExampleApp".to_string()),
        log_level: Some(LogLevel::Debug),
        disable_logger: false,
    };
    
    let lumosai = Lumosai::new(config);
    
    // 创建一个向量存储
    let vector_storage = create_memory_vector_storage();
    
    // 注册向量存储
    lumosai.register_vector("main-vectors", Arc::new(vector_storage))?;
    
    // 获取向量存储并使用
    let retriever = lumosai.get_vector("main-vectors")?;
    
    // 创建一个测试索引，使用128维向量和余弦相似度
    retriever.create_index("test-index", 128, Some(SimilarityMetric::Cosine)).await?;
    println!("创建了索引 'test-index'");
    
    // 创建测试向量
    let test_vector = vec![0.1; 128]; // 一个简单的128维向量
    let mut metadata = HashMap::new();
    metadata.insert("description".to_string(), serde_json::json!("测试向量"));
    
    // 上传向量
    let ids = retriever.upsert(
        "test-index", 
        vec![test_vector], 
        None, 
        Some(vec![metadata])
    ).await?;
    
    println!("添加了向量, ID: {}", ids[0]);
    
    // 获取索引统计
    let stats = retriever.describe_index("test-index").await?;
    println!("索引统计: 向量数量 = {}, 维度 = {}", stats.count, stats.dimension);
    
    // 测试Base特性方法
    if let Some(name) = lumosai.name() {
        println!("Lumosai名称: {}", name);
    }
    
    println!("Lumosai组件类型: {:?}", lumosai.component());
    
    // 记录一些日志
    lumosai.logger().info("这是一条信息日志", None);
    lumosai.logger().debug("这是一条调试日志", None);
    
    println!("基本示例运行完成！");
    
    Ok(())
} 