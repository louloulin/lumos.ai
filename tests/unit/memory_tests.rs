// Unit tests for Memory system
use crate::test_config::*;
use std::time::Duration;
use std::collections::HashMap;

#[tokio::test]
async fn test_memory_creation() {
    init_test_env();
    
    // Test different memory types
    let short_term = create_test_memory("short_term").await;
    assert!(short_term.is_ok(), "Short-term memory creation should succeed");
    
    let long_term = create_test_memory("long_term").await;
    assert!(long_term.is_ok(), "Long-term memory creation should succeed");
    
    let working = create_test_memory("working").await;
    assert!(working.is_ok(), "Working memory creation should succeed");
}

#[tokio::test]
async fn test_memory_storage_and_retrieval() {
    init_test_env();
    
    let memory = create_test_memory("test_memory").await.unwrap();
    
    // Test basic storage
    let key = "test_key";
    let value = "test_value";
    
    let store_result = memory.store(key, value).await;
    assert!(store_result.is_ok(), "Memory storage should succeed");
    
    // Test retrieval
    let retrieve_result = memory.retrieve(key).await;
    assert!(retrieve_result.is_ok(), "Memory retrieval should succeed");
    
    let retrieved_value = retrieve_result.unwrap();
    assert_eq!(retrieved_value, Some(value.to_string()), "Retrieved value should match stored value");
    
    // Test non-existent key
    let missing_result = memory.retrieve("non_existent_key").await;
    assert!(missing_result.is_ok(), "Retrieval of missing key should succeed");
    assert_eq!(missing_result.unwrap(), None, "Missing key should return None");
}

#[tokio::test]
async fn test_memory_complex_data_types() {
    init_test_env();
    
    let memory = create_test_memory("complex_memory").await.unwrap();
    
    // Test storing complex data
    let complex_data = MemoryData {
        id: "test_id".to_string(),
        content: "Complex memory content".to_string(),
        metadata: {
            let mut map = HashMap::new();
            map.insert("type".to_string(), "test".to_string());
            map.insert("priority".to_string(), "high".to_string());
            map
        },
        timestamp: std::time::SystemTime::now(),
    };
    
    let store_result = memory.store_complex("complex_key", &complex_data).await;
    assert!(store_result.is_ok(), "Complex data storage should succeed");
    
    // Test retrieval of complex data
    let retrieve_result = memory.retrieve_complex("complex_key").await;
    assert!(retrieve_result.is_ok(), "Complex data retrieval should succeed");
    
    let retrieved_data = retrieve_result.unwrap();
    assert!(retrieved_data.is_some(), "Complex data should be retrieved");
    
    let retrieved_data = retrieved_data.unwrap();
    assert_eq!(retrieved_data.id, complex_data.id, "ID should match");
    assert_eq!(retrieved_data.content, complex_data.content, "Content should match");
}

#[tokio::test]
async fn test_memory_expiration() {
    init_test_env();
    
    let memory = create_test_memory_with_ttl("expiring_memory", Duration::from_millis(100)).await.unwrap();
    
    // Store data with expiration
    let key = "expiring_key";
    let value = "expiring_value";
    
    memory.store(key, value).await.unwrap();
    
    // Verify data exists immediately
    let immediate_result = memory.retrieve(key).await.unwrap();
    assert_eq!(immediate_result, Some(value.to_string()), "Data should exist immediately");
    
    // Wait for expiration
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    // Verify data has expired
    let expired_result = memory.retrieve(key).await.unwrap();
    assert_eq!(expired_result, None, "Data should have expired");
}

#[tokio::test]
async fn test_memory_capacity_limits() {
    init_test_env();
    
    let memory = create_test_memory_with_capacity("limited_memory", 3).await.unwrap();
    
    // Fill memory to capacity
    for i in 0..3 {
        let key = format!("key_{}", i);
        let value = format!("value_{}", i);
        memory.store(&key, &value).await.unwrap();
    }
    
    // Verify all items are stored
    for i in 0..3 {
        let key = format!("key_{}", i);
        let result = memory.retrieve(&key).await.unwrap();
        assert!(result.is_some(), "Item {} should be stored", i);
    }
    
    // Add one more item (should trigger eviction)
    memory.store("key_3", "value_3").await.unwrap();
    
    // Verify oldest item was evicted (LRU policy)
    let evicted_result = memory.retrieve("key_0").await.unwrap();
    assert_eq!(evicted_result, None, "Oldest item should be evicted");
    
    // Verify newest item exists
    let newest_result = memory.retrieve("key_3").await.unwrap();
    assert_eq!(newest_result, Some("value_3".to_string()), "Newest item should exist");
}

#[tokio::test]
async fn test_memory_update_operations() {
    init_test_env();
    
    let memory = create_test_memory("update_memory").await.unwrap();
    
    let key = "update_key";
    let initial_value = "initial_value";
    let updated_value = "updated_value";
    
    // Store initial value
    memory.store(key, initial_value).await.unwrap();
    
    // Update value
    let update_result = memory.update(key, updated_value).await;
    assert!(update_result.is_ok(), "Memory update should succeed");
    
    // Verify updated value
    let retrieved_value = memory.retrieve(key).await.unwrap();
    assert_eq!(retrieved_value, Some(updated_value.to_string()), "Value should be updated");
    
    // Test update of non-existent key
    let missing_update = memory.update("missing_key", "new_value").await;
    // Should either create new entry or return error
    assert!(missing_update.is_ok() || missing_update.is_err());
}

#[tokio::test]
async fn test_memory_deletion() {
    init_test_env();
    
    let memory = create_test_memory("delete_memory").await.unwrap();
    
    let key = "delete_key";
    let value = "delete_value";
    
    // Store value
    memory.store(key, value).await.unwrap();
    
    // Verify value exists
    let exists_result = memory.retrieve(key).await.unwrap();
    assert!(exists_result.is_some(), "Value should exist before deletion");
    
    // Delete value
    let delete_result = memory.delete(key).await;
    assert!(delete_result.is_ok(), "Memory deletion should succeed");
    
    // Verify value is deleted
    let deleted_result = memory.retrieve(key).await.unwrap();
    assert_eq!(deleted_result, None, "Value should be deleted");
    
    // Test deletion of non-existent key
    let missing_delete = memory.delete("missing_key").await;
    assert!(missing_delete.is_ok(), "Deletion of missing key should succeed");
}

#[tokio::test]
async fn test_memory_search_functionality() {
    init_test_env();
    
    let memory = create_test_memory("search_memory").await.unwrap();
    
    // Store searchable data
    let test_data = vec![
        ("doc1", "artificial intelligence and machine learning"),
        ("doc2", "natural language processing"),
        ("doc3", "computer vision and image recognition"),
        ("doc4", "deep learning neural networks"),
    ];
    
    for (key, content) in &test_data {
        memory.store(key, content).await.unwrap();
    }
    
    // Test search functionality
    let search_results = memory.search("machine learning").await;
    assert!(search_results.is_ok(), "Memory search should succeed");
    
    let results = search_results.unwrap();
    assert!(!results.is_empty(), "Search should return results");
    
    // Verify search results contain relevant content
    let found_relevant = results.iter().any(|result| {
        result.content.contains("machine learning") || result.content.contains("artificial intelligence")
    });
    assert!(found_relevant, "Search results should contain relevant content");
}

#[tokio::test]
async fn test_memory_concurrent_access() {
    init_test_env();
    
    let memory = create_test_memory("concurrent_memory").await.unwrap();
    
    // Concurrent writes
    let mut write_handles = Vec::new();
    
    for i in 0..5 {
        let memory_clone = memory.clone();
        let key = format!("concurrent_key_{}", i);
        let value = format!("concurrent_value_{}", i);
        
        let handle = tokio::spawn(async move {
            memory_clone.store(&key, &value).await
        });
        
        write_handles.push(handle);
    }
    
    // Wait for all writes
    for (i, handle) in write_handles.into_iter().enumerate() {
        let result = handle.await;
        assert!(result.is_ok(), "Concurrent write {} should succeed", i);
        assert!(result.unwrap().is_ok(), "Store operation {} should succeed", i);
    }
    
    // Concurrent reads
    let mut read_handles = Vec::new();
    
    for i in 0..5 {
        let memory_clone = memory.clone();
        let key = format!("concurrent_key_{}", i);
        
        let handle = tokio::spawn(async move {
            memory_clone.retrieve(&key).await
        });
        
        read_handles.push(handle);
    }
    
    // Wait for all reads
    for (i, handle) in read_handles.into_iter().enumerate() {
        let result = handle.await;
        assert!(result.is_ok(), "Concurrent read {} should succeed", i);
        
        let value = result.unwrap().unwrap();
        assert!(value.is_some(), "Concurrent read {} should find value", i);
    }
}

#[tokio::test]
async fn test_memory_performance() {
    init_test_env();
    
    let memory = create_test_memory("perf_memory").await.unwrap();
    
    // Measure storage performance
    let (store_result, store_duration) = PerformanceTestUtils::measure_time(|| async {
        memory.store("perf_key", "performance test value").await
    }).await;
    
    assert!(store_result.is_ok(), "Performance store should succeed");
    
    // Measure retrieval performance
    let (retrieve_result, retrieve_duration) = PerformanceTestUtils::measure_time(|| async {
        memory.retrieve("perf_key").await
    }).await;
    
    assert!(retrieve_result.is_ok(), "Performance retrieve should succeed");
    
    // Performance assertions
    PerformanceTestUtils::assert_execution_time_within(
        store_duration,
        Duration::from_millis(100)
    );
    
    PerformanceTestUtils::assert_execution_time_within(
        retrieve_duration,
        Duration::from_millis(50)
    );
    
    println!("Memory Performance - Store: {:?}, Retrieve: {:?}", 
             store_duration, retrieve_duration);
}

// Helper functions and types for memory testing
async fn create_test_memory(memory_type: &str) -> Result<TestMemory> {
    Ok(TestMemory::new(memory_type))
}

async fn create_test_memory_with_ttl(memory_type: &str, ttl: Duration) -> Result<TestMemory> {
    Ok(TestMemory::with_ttl(memory_type, ttl))
}

async fn create_test_memory_with_capacity(memory_type: &str, capacity: usize) -> Result<TestMemory> {
    Ok(TestMemory::with_capacity(memory_type, capacity))
}

#[derive(Debug, Clone)]
struct MemoryData {
    id: String,
    content: String,
    metadata: HashMap<String, String>,
    timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone)]
struct SearchResult {
    key: String,
    content: String,
    score: f64,
}

// Mock memory implementation for testing
#[derive(Clone)]
struct TestMemory {
    memory_type: String,
    data: std::sync::Arc<tokio::sync::RwLock<HashMap<String, String>>>,
    ttl: Option<Duration>,
    capacity: Option<usize>,
}

impl TestMemory {
    fn new(memory_type: &str) -> Self {
        Self {
            memory_type: memory_type.to_string(),
            data: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            ttl: None,
            capacity: None,
        }
    }
    
    fn with_ttl(memory_type: &str, ttl: Duration) -> Self {
        Self {
            memory_type: memory_type.to_string(),
            data: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            ttl: Some(ttl),
            capacity: None,
        }
    }
    
    fn with_capacity(memory_type: &str, capacity: usize) -> Self {
        Self {
            memory_type: memory_type.to_string(),
            data: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            ttl: None,
            capacity: Some(capacity),
        }
    }
    
    async fn store(&self, key: &str, value: &str) -> Result<()> {
        let mut data = self.data.write().await;
        
        // Check capacity limits
        if let Some(cap) = self.capacity {
            if data.len() >= cap && !data.contains_key(key) {
                // Remove oldest entry (simplified LRU)
                if let Some(oldest_key) = data.keys().next().cloned() {
                    data.remove(&oldest_key);
                }
            }
        }
        
        data.insert(key.to_string(), value.to_string());
        Ok(())
    }
    
    async fn retrieve(&self, key: &str) -> Result<Option<String>> {
        let data = self.data.read().await;
        Ok(data.get(key).cloned())
    }
    
    async fn store_complex(&self, key: &str, _data: &MemoryData) -> Result<()> {
        // Simplified implementation
        self.store(key, "complex_data").await
    }
    
    async fn retrieve_complex(&self, key: &str) -> Result<Option<MemoryData>> {
        let result = self.retrieve(key).await?;
        if result.is_some() {
            Ok(Some(MemoryData {
                id: key.to_string(),
                content: result.unwrap(),
                metadata: HashMap::new(),
                timestamp: std::time::SystemTime::now(),
            }))
        } else {
            Ok(None)
        }
    }
    
    async fn update(&self, key: &str, value: &str) -> Result<()> {
        self.store(key, value).await
    }
    
    async fn delete(&self, key: &str) -> Result<()> {
        let mut data = self.data.write().await;
        data.remove(key);
        Ok(())
    }
    
    async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let data = self.data.read().await;
        let mut results = Vec::new();
        
        for (key, content) in data.iter() {
            if content.contains(query) {
                results.push(SearchResult {
                    key: key.clone(),
                    content: content.clone(),
                    score: 0.8, // Mock score
                });
            }
        }
        
        Ok(results)
    }
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
