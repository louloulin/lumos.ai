//! Resource Pool Tests
//!
//! 资源池功能测试

use lumosai_core::pool::{
    ConnectionPool, ConnectionPoolConfig, ObjectPool, ObjectPoolConfig,
    ResourceMonitor, ResourceStats,
};
use lumosai_core::pool::resource_monitor::ResourceMonitorConfig;
use lumosai_core::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

// ============================================================================
// Mock Implementations
// ============================================================================

/// Mock 连接
#[derive(Debug, Clone)]
struct MockConnection {
    id: usize,
    created_at: Instant,
    healthy: bool,
}

#[async_trait::async_trait]
impl lumosai_core::pool::connection_pool::Connection for MockConnection {
    async fn is_healthy(&self) -> bool {
        self.healthy
    }
    
    async fn reset(&mut self) -> Result<()> {
        Ok(())
    }
    
    fn created_at(&self) -> Instant {
        self.created_at
    }
}

/// Mock 连接工厂
struct MockConnectionFactory {
    counter: Arc<tokio::sync::Mutex<usize>>,
}

impl MockConnectionFactory {
    fn new() -> Self {
        Self {
            counter: Arc::new(tokio::sync::Mutex::new(0)),
        }
    }
}

#[async_trait::async_trait]
impl lumosai_core::pool::connection_pool::ConnectionFactory<MockConnection> for MockConnectionFactory {
    async fn create(&self) -> Result<MockConnection> {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        Ok(MockConnection {
            id: *counter,
            created_at: Instant::now(),
            healthy: true,
        })
    }
}

/// Mock 对象
#[derive(Debug, Clone)]
struct MockObject {
    id: usize,
    created_at: Instant,
    valid: bool,
}

impl lumosai_core::pool::object_pool::Poolable for MockObject {
    fn reset(&mut self) {
        // 重置对象状态
    }
    
    fn is_valid(&self) -> bool {
        self.valid
    }
    
    fn created_at(&self) -> Instant {
        self.created_at
    }
}

/// Mock 对象工厂
struct MockObjectFactory {
    counter: Arc<std::sync::Mutex<usize>>,
}

impl MockObjectFactory {
    fn new() -> Self {
        Self {
            counter: Arc::new(std::sync::Mutex::new(0)),
        }
    }
}

impl lumosai_core::pool::object_pool::ObjectFactory<MockObject> for MockObjectFactory {
    fn create(&self) -> Result<MockObject> {
        let mut counter = self.counter.lock().unwrap();
        *counter += 1;
        Ok(MockObject {
            id: *counter,
            created_at: Instant::now(),
            valid: true,
        })
    }
}

// ============================================================================
// Connection Pool Tests
// ============================================================================

#[tokio::test]
async fn test_connection_pool_creation() {
    println!("🧪 测试连接池创建");
    
    let config = ConnectionPoolConfig {
        min_size: 5,
        max_size: 20,
        ..Default::default()
    };
    
    let factory = Arc::new(MockConnectionFactory::new());
    let pool = ConnectionPool::new(config, factory);
    
    let stats = pool.stats().await;
    assert_eq!(stats.total, 0);
    assert_eq!(stats.active, 0);
    assert_eq!(stats.idle, 0);
    
    println!("✅ 连接池创建测试通过");
}

#[tokio::test]
async fn test_connection_pool_acquire_release() {
    println!("🧪 测试连接池获取和释放");
    
    let config = ConnectionPoolConfig {
        min_size: 2,
        max_size: 10,
        ..Default::default()
    };
    
    let factory = Arc::new(MockConnectionFactory::new());
    let pool = ConnectionPool::new(config, factory);
    
    // 获取连接
    let conn1 = pool.acquire().await.unwrap();
    let stats = pool.stats().await;
    assert_eq!(stats.active, 1);
    assert_eq!(stats.total_acquires, 1);
    
    // 获取第二个连接
    let conn2 = pool.acquire().await.unwrap();
    let stats = pool.stats().await;
    assert_eq!(stats.active, 2);
    
    // 释放连接
    drop(conn1);
    sleep(Duration::from_millis(10)).await;
    
    let stats = pool.stats().await;
    assert_eq!(stats.active, 1);
    assert_eq!(stats.idle, 1);
    
    drop(conn2);
    sleep(Duration::from_millis(10)).await;
    
    let stats = pool.stats().await;
    assert_eq!(stats.active, 0);
    assert_eq!(stats.idle, 2);
    
    println!("✅ 连接池获取和释放测试通过");
}

#[tokio::test]
async fn test_connection_pool_warmup() {
    println!("🧪 测试连接池预热");
    
    let config = ConnectionPoolConfig {
        min_size: 5,
        max_size: 20,
        ..Default::default()
    };
    
    let factory = Arc::new(MockConnectionFactory::new());
    let pool = ConnectionPool::new(config, factory);
    
    // 预热连接池
    pool.warmup().await.unwrap();
    
    let stats = pool.stats().await;
    assert_eq!(stats.total, 5);
    assert_eq!(stats.idle, 5);
    assert_eq!(stats.total_creates, 5);
    
    println!("✅ 连接池预热测试通过");
}

#[tokio::test]
async fn test_connection_pool_concurrent_access() {
    println!("🧪 测试连接池并发访问");
    
    let config = ConnectionPoolConfig {
        min_size: 2,
        max_size: 10,
        ..Default::default()
    };
    
    let factory = Arc::new(MockConnectionFactory::new());
    let pool = ConnectionPool::new(config, factory);
    
    let start = Instant::now();
    
    // 并发获取连接
    let mut handles = vec![];
    for _ in 0..5 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            let _conn = pool_clone.acquire().await.unwrap();
            sleep(Duration::from_millis(10)).await;
        });
        handles.push(handle);
    }
    
    // 等待所有任务完成
    for handle in handles {
        handle.await.unwrap();
    }
    
    let duration = start.elapsed();
    
    let stats = pool.stats().await;
    assert_eq!(stats.total_acquires, 5);
    assert!(duration < Duration::from_millis(100));
    
    println!("✅ 连接池并发访问测试通过 (耗时: {:?})", duration);
}

// ============================================================================
// Object Pool Tests
// ============================================================================

#[tokio::test]
async fn test_object_pool_creation() {
    println!("🧪 测试对象池创建");
    
    let config = ObjectPoolConfig {
        min_size: 5,
        max_size: 20,
        ..Default::default()
    };
    
    let factory = Arc::new(MockObjectFactory::new());
    let pool = ObjectPool::new(config, factory);
    
    let stats = pool.stats().await;
    assert_eq!(stats.total, 0);
    
    println!("✅ 对象池创建测试通过");
}

#[tokio::test]
async fn test_object_pool_acquire_release() {
    println!("🧪 测试对象池获取和释放");
    
    let config = ObjectPoolConfig {
        min_size: 2,
        max_size: 10,
        ..Default::default()
    };
    
    let factory = Arc::new(MockObjectFactory::new());
    let pool = ObjectPool::new(config, factory);
    
    // 获取对象
    let obj1 = pool.acquire().await.unwrap();
    let stats = pool.stats().await;
    assert_eq!(stats.active, 1);
    
    // 释放对象
    drop(obj1);
    sleep(Duration::from_millis(10)).await;
    
    let stats = pool.stats().await;
    assert_eq!(stats.active, 0);
    assert_eq!(stats.idle, 1);
    
    println!("✅ 对象池获取和释放测试通过");
}

#[tokio::test]
async fn test_object_pool_warmup() {
    println!("🧪 测试对象池预热");

    let config = ObjectPoolConfig {
        min_size: 5,
        max_size: 20,
        ..Default::default()
    };

    let factory = Arc::new(MockObjectFactory::new());
    let pool = ObjectPool::new(config, factory);

    // 预热对象池
    pool.warmup().unwrap();
    sleep(Duration::from_millis(50)).await;

    let stats = pool.stats().await;
    assert_eq!(stats.total, 5);
    assert_eq!(stats.idle, 5);

    println!("✅ 对象池预热测试通过");
}

#[tokio::test]
async fn test_object_pool_concurrent_access() {
    println!("🧪 测试对象池并发访问");

    let config = ObjectPoolConfig {
        min_size: 2,
        max_size: 10,
        ..Default::default()
    };

    let factory = Arc::new(MockObjectFactory::new());
    let pool = ObjectPool::new(config, factory);

    let start = Instant::now();

    // 并发获取对象
    let mut handles = vec![];
    for _ in 0..5 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            let _obj = pool_clone.acquire().await.unwrap();
            sleep(Duration::from_millis(10)).await;
        });
        handles.push(handle);
    }

    // 等待所有任务完成
    for handle in handles {
        handle.await.unwrap();
    }

    let duration = start.elapsed();

    let stats = pool.stats().await;
    assert_eq!(stats.total_acquires, 5);
    assert!(duration < Duration::from_millis(100));

    println!("✅ 对象池并发访问测试通过 (耗时: {:?})", duration);
}

// ============================================================================
// Resource Monitor Tests
// ============================================================================

#[tokio::test]
async fn test_resource_monitor_creation() {
    println!("🧪 测试资源监控器创建");

    let config = ResourceMonitorConfig::default();
    let monitor = ResourceMonitor::new(config);

    let stats = monitor.get_stats().await;
    assert_eq!(stats.active_connections, 0);
    assert_eq!(stats.pool_utilization, 0.0);

    println!("✅ 资源监控器创建测试通过");
}

#[tokio::test]
async fn test_resource_monitor_update_stats() {
    println!("🧪 测试资源监控器更新统计");

    let config = ResourceMonitorConfig::default();
    let monitor = ResourceMonitor::new(config);

    let new_stats = ResourceStats {
        cpu_usage: 0.5,
        memory_usage: 1024 * 1024 * 100, // 100MB
        memory_usage_percent: 0.5,
        active_connections: 10,
        idle_connections: 5,
        total_connections: 15,
        pool_utilization: 0.67,
        queue_length: 2,
        avg_response_time_ms: 50.0,
        error_rate: 0.01,
        sampled_at: Instant::now(),
    };

    monitor.update_stats(new_stats.clone()).await;

    let stats = monitor.get_stats().await;
    assert_eq!(stats.active_connections, 10);
    assert_eq!(stats.pool_utilization, 0.67);

    println!("✅ 资源监控器更新统计测试通过");
}

#[tokio::test]
async fn test_resource_monitor_scale_up_detection() {
    println!("🧪 测试资源监控器扩容检测");

    let config = ResourceMonitorConfig {
        auto_scaling_enabled: true,
        scale_up_threshold: 0.8,
        min_instances: 5,
        max_instances: 50,
        ..Default::default()
    };

    let monitor = ResourceMonitor::new(config);

    // 模拟高负载
    let high_load_stats = ResourceStats {
        cpu_usage: 0.85,
        memory_usage: 1024 * 1024 * 800, // 800MB
        memory_usage_percent: 0.8,
        active_connections: 18,
        idle_connections: 2,
        total_connections: 20,
        pool_utilization: 0.9,
        queue_length: 15,
        avg_response_time_ms: 100.0,
        error_rate: 0.02,
        sampled_at: Instant::now(),
    };

    monitor.update_stats(high_load_stats).await;

    let event = monitor.check_scaling().await;
    assert!(event.is_some());

    if let Some(lumosai_core::pool::resource_monitor::ScalingEvent::ScaleUp { current, target, reason }) = event {
        println!("📈 扩容事件: {} -> {} (原因: {})", current, target, reason);
        assert!(target > current);
    } else {
        panic!("Expected ScaleUp event");
    }

    println!("✅ 资源监控器扩容检测测试通过");
}

#[tokio::test]
async fn test_resource_monitor_scale_down_detection() {
    println!("🧪 测试资源监控器缩容检测");

    let config = ResourceMonitorConfig {
        auto_scaling_enabled: true,
        scale_down_threshold: 0.3,
        min_instances: 5,
        max_instances: 50,
        ..Default::default()
    };

    let monitor = ResourceMonitor::new(config);

    // 模拟低负载
    let low_load_stats = ResourceStats {
        cpu_usage: 0.2,
        memory_usage: 1024 * 1024 * 200, // 200MB
        memory_usage_percent: 0.3,
        active_connections: 2,
        idle_connections: 18,
        total_connections: 20,
        pool_utilization: 0.1,
        queue_length: 0,
        avg_response_time_ms: 10.0,
        error_rate: 0.0,
        sampled_at: Instant::now(),
    };

    monitor.update_stats(low_load_stats).await;

    let event = monitor.check_scaling().await;
    assert!(event.is_some());

    if let Some(lumosai_core::pool::resource_monitor::ScalingEvent::ScaleDown { current, target, reason }) = event {
        println!("📉 缩容事件: {} -> {} (原因: {})", current, target, reason);
        assert!(target < current);
    } else {
        panic!("Expected ScaleDown event");
    }

    println!("✅ 资源监控器缩容检测测试通过");
}

#[tokio::test]
async fn test_resource_stats_utilization() {
    println!("🧪 测试资源统计利用率计算");

    // 测试扩容条件：利用率 > 0.85
    let high_util_stats = ResourceStats {
        active_connections: 18,
        idle_connections: 2,
        total_connections: 20,
        pool_utilization: 0.9,
        cpu_usage: 0.7,
        memory_usage: 1024 * 1024 * 500,
        memory_usage_percent: 0.5,
        queue_length: 5,
        ..Default::default()
    };

    assert!(high_util_stats.should_scale_up());
    assert!(!high_util_stats.should_scale_down());

    // 测试缩容条件：利用率 < 0.3 且队列为空
    let low_util_stats = ResourceStats {
        active_connections: 2,
        idle_connections: 18,
        total_connections: 20,
        pool_utilization: 0.1,
        cpu_usage: 0.2,
        memory_usage: 1024 * 1024 * 100,
        memory_usage_percent: 0.1,
        queue_length: 0,
        ..Default::default()
    };

    assert!(!low_util_stats.should_scale_up());
    assert!(low_util_stats.should_scale_down());

    println!("✅ 资源统计利用率计算测试通过");
}

