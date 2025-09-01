use std::collections::HashMap;
use std::time::Duration;
use tokio;
use lumosai_core::cache::{AdvancedCache, Cache, CacheConfig, CacheEvictionPolicy};
use lumosai_core::data_processing::{
    AdvancedDataProcessor, ProcessingPipeline, ProcessingRule, DataOperation
};
use lumosai_core::error::Result;

/// 测试高级缓存系统基础功能
#[tokio::test]
async fn test_advanced_cache_basic_operations() -> Result<()> {
    let config = CacheConfig {
        max_size: 100,
        default_ttl: Some(Duration::from_secs(60)),
        eviction_policy: CacheEvictionPolicy::LRU,
        cleanup_interval: Duration::from_secs(10),
        enable_metrics: true,
    };
    
    let cache = AdvancedCache::<String>::new(config);
    
    // 测试设置和获取
    cache.set("test_key", "test_value".to_string(), None).await?;
    let value = cache.get("test_key").await;
    assert_eq!(value, Some("test_value".to_string()));
    
    // 测试不存在的键
    let missing = cache.get("missing_key").await;
    assert_eq!(missing, None);
    
    // 测试删除
    assert!(cache.remove("test_key").await);
    assert!(!cache.remove("missing_key").await);
    
    // 测试清空
    cache.set("key1", "value1".to_string(), None).await?;
    cache.set("key2", "value2".to_string(), None).await?;
    cache.clear().await?;
    assert_eq!(cache.size().await, 0);
    
    println!("✅ 高级缓存基础操作测试通过");
    Ok(())
}

/// 测试缓存TTL功能
#[tokio::test]
async fn test_cache_ttl_functionality() -> Result<()> {
    let config = CacheConfig {
        max_size: 10,
        default_ttl: Some(Duration::from_millis(100)),
        eviction_policy: CacheEvictionPolicy::TTL,
        cleanup_interval: Duration::from_millis(50),
        enable_metrics: true,
    };
    
    let cache = AdvancedCache::<String>::new(config);
    
    // 设置短期TTL的项
    cache.set("short_lived", "temporary".to_string(), Some(Duration::from_millis(50))).await?;
    
    // 立即检查应该存在
    assert!(cache.contains("short_lived").await);
    
    // 等待过期
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // 现在应该过期了
    let expired = cache.get("short_lived").await;
    assert_eq!(expired, None);
    
    println!("✅ 缓存TTL功能测试通过");
    Ok(())
}

/// 测试缓存指标收集
#[tokio::test]
async fn test_cache_metrics() -> Result<()> {
    let cache = AdvancedCache::<i32>::new(CacheConfig::default());
    
    // 执行一些操作
    cache.set("key1", 1, None).await?;
    cache.set("key2", 2, None).await?;
    
    // 命中
    let _ = cache.get("key1").await;
    let _ = cache.get("key2").await;
    
    // 未命中
    let _ = cache.get("missing1").await;
    let _ = cache.get("missing2").await;
    
    // 检查指标
    let metrics = cache.metrics().await;
    assert_eq!(metrics.hits, 2);
    assert_eq!(metrics.misses, 2);
    assert_eq!(metrics.size, 2);
    
    println!("✅ 缓存指标收集测试通过");
    Ok(())
}

/// 测试数据处理系统基础功能
#[tokio::test]
async fn test_data_processing_basic() -> Result<()> {
    let processor = AdvancedDataProcessor::new();
    
    // 创建简单的处理规则
    let clean_rule = ProcessingRule {
        id: "clean_text".to_string(),
        name: "Clean Text".to_string(),
        operation: DataOperation::Clean,
        config: HashMap::new(),
        conditions: Vec::new(),
        priority: 1,
        enabled: true,
    };
    
    // 创建处理管道
    let pipeline = ProcessingPipeline {
        id: "test_pipeline".to_string(),
        name: "Test Pipeline".to_string(),
        description: "Test processing pipeline".to_string(),
        rules: vec![clean_rule],
        input_schema: None,
        output_schema: None,
        metadata: HashMap::new(),
    };
    
    // 注册管道
    processor.register_pipeline(pipeline)?;
    
    // 处理数据
    let input = serde_json::json!("  hello world  ");
    let result = processor.process_data("test_pipeline", input).await?;
    
    assert!(result.success);
    assert_eq!(result.processed_data, serde_json::json!("hello world"));
    assert!(result.errors.is_empty());
    
    println!("✅ 数据处理基础功能测试通过");
    Ok(())
}

/// 测试数据处理管道管理
#[tokio::test]
async fn test_pipeline_management() -> Result<()> {
    let processor = AdvancedDataProcessor::new();
    
    // 创建测试管道
    let pipeline1 = ProcessingPipeline {
        id: "pipeline1".to_string(),
        name: "Pipeline 1".to_string(),
        description: "First pipeline".to_string(),
        rules: Vec::new(),
        input_schema: None,
        output_schema: None,
        metadata: HashMap::new(),
    };
    
    let pipeline2 = ProcessingPipeline {
        id: "pipeline2".to_string(),
        name: "Pipeline 2".to_string(),
        description: "Second pipeline".to_string(),
        rules: Vec::new(),
        input_schema: None,
        output_schema: None,
        metadata: HashMap::new(),
    };
    
    // 注册管道
    processor.register_pipeline(pipeline1)?;
    processor.register_pipeline(pipeline2)?;
    
    // 列出管道
    let pipelines = processor.list_pipelines()?;
    assert_eq!(pipelines.len(), 2);
    
    // 获取特定管道
    let retrieved = processor.get_pipeline("pipeline1")?;
    assert_eq!(retrieved.name, "Pipeline 1");
    
    // 删除管道
    processor.remove_pipeline("pipeline1")?;
    let remaining = processor.list_pipelines()?;
    assert_eq!(remaining.len(), 1);
    
    println!("✅ 管道管理功能测试通过");
    Ok(())
}

/// 测试数组数据处理
#[tokio::test]
async fn test_array_data_processing() -> Result<()> {
    let processor = AdvancedDataProcessor::new();
    
    // 创建数组处理规则
    let filter_rule = ProcessingRule {
        id: "filter_nulls".to_string(),
        name: "Filter Nulls".to_string(),
        operation: DataOperation::Filter,
        config: HashMap::new(),
        conditions: Vec::new(),
        priority: 1,
        enabled: true,
    };
    
    let sort_rule = ProcessingRule {
        id: "sort_array".to_string(),
        name: "Sort Array".to_string(),
        operation: DataOperation::Sort,
        config: HashMap::new(),
        conditions: Vec::new(),
        priority: 2,
        enabled: true,
    };
    
    let pipeline = ProcessingPipeline {
        id: "array_pipeline".to_string(),
        name: "Array Pipeline".to_string(),
        description: "Array processing pipeline".to_string(),
        rules: vec![filter_rule, sort_rule],
        input_schema: None,
        output_schema: None,
        metadata: HashMap::new(),
    };
    
    processor.register_pipeline(pipeline)?;
    
    // 测试数组处理
    let input = serde_json::json!(["c", "a", null, "b"]);
    let result = processor.process_data("array_pipeline", input).await?;
    
    assert!(result.success);
    // 应该过滤掉null并排序
    let expected = serde_json::json!(["a", "b", "c"]);
    assert_eq!(result.processed_data, expected);
    
    println!("✅ 数组数据处理测试通过");
    Ok(())
}

/// 测试数字聚合处理
#[tokio::test]
async fn test_number_aggregation() -> Result<()> {
    let processor = AdvancedDataProcessor::new();
    
    let aggregate_rule = ProcessingRule {
        id: "aggregate_numbers".to_string(),
        name: "Aggregate Numbers".to_string(),
        operation: DataOperation::Aggregate,
        config: HashMap::new(),
        conditions: Vec::new(),
        priority: 1,
        enabled: true,
    };
    
    let pipeline = ProcessingPipeline {
        id: "number_pipeline".to_string(),
        name: "Number Pipeline".to_string(),
        description: "Number processing pipeline".to_string(),
        rules: vec![aggregate_rule],
        input_schema: None,
        output_schema: None,
        metadata: HashMap::new(),
    };
    
    processor.register_pipeline(pipeline)?;
    
    // 测试数字聚合
    let input = serde_json::json!([1.0, 2.0, 3.0, 4.0, 5.0]);
    let result = processor.process_data("number_pipeline", input).await?;
    
    assert!(result.success);
    
    // 验证聚合结果
    let output = result.processed_data.as_object().unwrap();
    assert_eq!(output.get("sum").unwrap().as_f64().unwrap(), 15.0);
    assert_eq!(output.get("count").unwrap().as_u64().unwrap(), 5);
    
    println!("✅ 数字聚合处理测试通过");
    Ok(())
}

/// 测试批量数据处理
#[tokio::test]
async fn test_batch_processing() -> Result<()> {
    let processor = AdvancedDataProcessor::new();
    
    let clean_rule = ProcessingRule {
        id: "batch_clean".to_string(),
        name: "Batch Clean".to_string(),
        operation: DataOperation::Clean,
        config: HashMap::new(),
        conditions: Vec::new(),
        priority: 1,
        enabled: true,
    };
    
    let pipeline = ProcessingPipeline {
        id: "batch_pipeline".to_string(),
        name: "Batch Pipeline".to_string(),
        description: "Batch processing pipeline".to_string(),
        rules: vec![clean_rule],
        input_schema: None,
        output_schema: None,
        metadata: HashMap::new(),
    };
    
    processor.register_pipeline(pipeline)?;
    
    // 准备批量数据
    let batch = vec![
        serde_json::json!("  item1  "),
        serde_json::json!("  item2  "),
        serde_json::json!("  item3  "),
    ];
    
    // 批量处理
    let results = processor.batch_process("batch_pipeline", batch).await?;
    
    assert_eq!(results.len(), 3);
    for (i, result) in results.iter().enumerate() {
        assert!(result.success);
        assert_eq!(result.processed_data, serde_json::json!(format!("item{}", i + 1)));
    }
    
    println!("✅ 批量数据处理测试通过");
    Ok(())
}

/// 测试处理指标
#[tokio::test]
async fn test_processing_metrics() -> Result<()> {
    let processor = AdvancedDataProcessor::new();
    
    let rule = ProcessingRule {
        id: "metrics_test".to_string(),
        name: "Metrics Test".to_string(),
        operation: DataOperation::Clean,
        config: HashMap::new(),
        conditions: Vec::new(),
        priority: 1,
        enabled: true,
    };
    
    let pipeline = ProcessingPipeline {
        id: "metrics_pipeline".to_string(),
        name: "Metrics Pipeline".to_string(),
        description: "Metrics test pipeline".to_string(),
        rules: vec![rule],
        input_schema: None,
        output_schema: None,
        metadata: HashMap::new(),
    };
    
    processor.register_pipeline(pipeline)?;
    
    // 处理一些数据
    for i in 0..3 {
        let data = serde_json::json!(format!("  test{}  ", i));
        let _ = processor.process_data("metrics_pipeline", data).await?;
    }
    
    // 检查指标
    let metrics = processor.get_metrics()?;
    assert_eq!(metrics.total_processed, 3);
    assert_eq!(metrics.successful_processed, 3);
    assert_eq!(metrics.failed_processed, 0);
    assert!(metrics.average_processing_time_ms >= 0.0);
    
    println!("✅ 处理指标测试通过");
    Ok(())
}

/// 综合高级功能测试
#[tokio::test]
async fn test_comprehensive_advanced_features() -> Result<()> {
    // 创建缓存和处理器
    let cache = AdvancedCache::<serde_json::Value>::new(CacheConfig::default());
    let processor = AdvancedDataProcessor::new();
    
    // 创建处理管道
    let rule = ProcessingRule {
        id: "comprehensive_rule".to_string(),
        name: "Comprehensive Rule".to_string(),
        operation: DataOperation::Transform,
        config: HashMap::new(),
        conditions: Vec::new(),
        priority: 1,
        enabled: true,
    };
    
    let pipeline = ProcessingPipeline {
        id: "comprehensive_pipeline".to_string(),
        name: "Comprehensive Pipeline".to_string(),
        description: "Comprehensive test pipeline".to_string(),
        rules: vec![rule],
        input_schema: None,
        output_schema: None,
        metadata: HashMap::new(),
    };
    
    processor.register_pipeline(pipeline)?;
    
    // 处理数据
    let input = serde_json::json!("TEST DATA");
    let result = processor.process_data("comprehensive_pipeline", input).await?;
    
    assert!(result.success);
    
    // 缓存结果
    cache.set("processed_result", result.processed_data.clone(), None).await?;
    
    // 验证缓存
    let cached = cache.get("processed_result").await;
    assert_eq!(cached, Some(result.processed_data));
    
    // 验证指标
    let cache_metrics = cache.metrics().await;
    assert!(cache_metrics.hits > 0);
    
    let processing_metrics = processor.get_metrics()?;
    assert!(processing_metrics.total_processed > 0);
    
    println!("✅ 综合高级功能测试通过");
    Ok(())
}
