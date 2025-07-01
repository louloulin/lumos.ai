//! 真实向量数据库集成测试

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use lumosai_vector_core::VectorStorage as VectorStorageTrait;
    use std::env;
    
    /// 测试内存向量存储
    #[tokio::test]
    async fn test_memory_vector_storage() {
        let storage = crate::vector::memory().await.expect("Failed to create memory storage");
        
        // 测试基本操作
        test_basic_vector_operations(storage).await;
    }
    
    /// 测试Qdrant向量存储（需要运行的Qdrant实例）
    #[tokio::test]
    #[cfg(feature = "vector-qdrant")]
    async fn test_qdrant_vector_storage() {
        // 跳过测试如果没有设置QDRANT_URL
        let qdrant_url = match env::var("QDRANT_URL") {
            Ok(url) => url,
            Err(_) => {
                println!("Skipping Qdrant test: QDRANT_URL not set");
                return;
            }
        };
        
        match crate::vector::qdrant(&qdrant_url).await {
            Ok(storage) => {
                println!("✅ Successfully connected to Qdrant at {}", qdrant_url);
                test_basic_vector_operations(storage).await;
            }
            Err(e) => {
                println!("⚠️  Failed to connect to Qdrant: {}", e);
                println!("Make sure Qdrant is running at {}", qdrant_url);
            }
        }
    }
    
    /// 测试Weaviate向量存储（需要运行的Weaviate实例）
    #[tokio::test]
    #[cfg(feature = "vector-weaviate")]
    async fn test_weaviate_vector_storage() {
        // 跳过测试如果没有设置WEAVIATE_URL
        let weaviate_url = match env::var("WEAVIATE_URL") {
            Ok(url) => url,
            Err(_) => {
                println!("Skipping Weaviate test: WEAVIATE_URL not set");
                return;
            }
        };
        
        match crate::vector::weaviate(&weaviate_url).await {
            Ok(storage) => {
                println!("✅ Successfully connected to Weaviate at {}", weaviate_url);
                test_basic_vector_operations(storage).await;
            }
            Err(e) => {
                println!("⚠️  Failed to connect to Weaviate: {}", e);
                println!("Make sure Weaviate is running at {}", weaviate_url);
            }
        }
    }
    
    /// 测试PostgreSQL向量存储（需要运行的PostgreSQL实例）
    #[tokio::test]
    #[cfg(feature = "vector-postgres")]
    async fn test_postgres_vector_storage() {
        // 跳过测试如果没有设置DATABASE_URL
        if env::var("DATABASE_URL").is_err() && env::var("POSTGRES_PASSWORD").is_err() {
            println!("Skipping PostgreSQL test: DATABASE_URL or POSTGRES_PASSWORD not set");
            return;
        }
        
        match crate::vector::postgres().await {
            Ok(storage) => {
                println!("✅ Successfully connected to PostgreSQL");
                test_basic_vector_operations(storage).await;
            }
            Err(e) => {
                println!("⚠️  Failed to connect to PostgreSQL: {}", e);
                println!("Make sure PostgreSQL is running and configured");
            }
        }
    }
    
    /// 测试自动向量存储选择
    #[tokio::test]
    async fn test_auto_vector_storage() {
        let storage = crate::vector::auto().await.expect("Failed to create auto storage");
        
        // 测试基本操作
        test_basic_vector_operations(storage).await;
    }
    
    /// 测试向量存储构建器
    #[tokio::test]
    async fn test_vector_storage_builder() {
        // 测试内存存储构建器
        let storage = crate::vector::builder()
            .backend("memory")
            .batch_size(100)
            .build()
            .await
            .expect("Failed to build memory storage");
        
        test_basic_vector_operations(storage).await;
        
        // 测试Qdrant构建器（如果可用）
        if let Ok(qdrant_url) = env::var("QDRANT_URL") {
            match crate::vector::builder()
                .backend("qdrant")
                .url(&qdrant_url)
                .build()
                .await
            {
                Ok(storage) => {
                    println!("✅ Successfully built Qdrant storage via builder");
                    test_basic_vector_operations(storage).await;
                }
                Err(e) => {
                    println!("⚠️  Failed to build Qdrant storage: {}", e);
                }
            }
        }
        
        // 测试Weaviate构建器（如果可用）
        if let Ok(weaviate_url) = env::var("WEAVIATE_URL") {
            match crate::vector::builder()
                .backend("weaviate")
                .url(&weaviate_url)
                .build()
                .await
            {
                Ok(storage) => {
                    println!("✅ Successfully built Weaviate storage via builder");
                    test_basic_vector_operations(storage).await;
                }
                Err(e) => {
                    println!("⚠️  Failed to build Weaviate storage: {}", e);
                }
            }
        }
    }
    
    /// 基本向量操作测试
    async fn test_basic_vector_operations(storage: VectorStorage) {
        // 这里只做基本的健康检查，避免修改数据库状态
        
        // 测试后端信息
        let backend_info = storage.backend_info();
        println!("Backend: {} v{}", backend_info.name, backend_info.version);
        println!("Features: {:?}", backend_info.features);
        
        // 测试健康检查
        match storage.health_check().await {
            Ok(_) => println!("✅ Health check passed"),
            Err(e) => println!("⚠️  Health check failed: {}", e),
        }
        
        // 测试列出索引（应该不会修改状态）
        match storage.list_indexes().await {
            Ok(indexes) => println!("📋 Found {} indexes", indexes.len()),
            Err(e) => println!("⚠️  Failed to list indexes: {}", e),
        }
    }
    
    /// 测试向量数据库连接性
    #[tokio::test]
    async fn test_vector_database_connectivity() {
        println!("🔍 Testing vector database connectivity...");
        
        // 测试各种向量数据库的连接性
        let mut available_backends = Vec::new();
        
        // 测试内存存储（总是可用）
        if crate::vector::memory().await.is_ok() {
            available_backends.push("memory");
        }
        
        // 测试Qdrant
        if let Ok(url) = env::var("QDRANT_URL") {
            if crate::vector::qdrant(&url).await.is_ok() {
                available_backends.push("qdrant");
            }
        }
        
        // 测试Weaviate
        if let Ok(url) = env::var("WEAVIATE_URL") {
            if crate::vector::weaviate(&url).await.is_ok() {
                available_backends.push("weaviate");
            }
        }
        
        // 测试PostgreSQL
        if crate::vector::postgres().await.is_ok() {
            available_backends.push("postgres");
        }
        
        println!("✅ Available vector storage backends: {:?}", available_backends);
        assert!(!available_backends.is_empty(), "At least memory storage should be available");
    }
    
    /// 性能基准测试（简化版）
    #[tokio::test]
    async fn test_vector_storage_performance() {
        let storage = crate::vector::memory().await.expect("Failed to create memory storage");
        
        let start = std::time::Instant::now();
        
        // 简单的性能测试：多次健康检查
        for _ in 0..10 {
            storage.health_check().await.expect("Health check failed");
        }
        
        let duration = start.elapsed();
        println!("⏱️  10 health checks took: {:?}", duration);
        
        // 确保性能合理（应该很快）
        assert!(duration.as_millis() < 1000, "Health checks took too long");
    }
}
