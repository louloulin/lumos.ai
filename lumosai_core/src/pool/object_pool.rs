//! Object Pool Implementation
//!
//! 对象池实现，用于复用昂贵的对象（如 Agent、Tool 实例）

use super::{PoolConfig, PoolStats};
use crate::Result;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, Semaphore};
use tokio::time::timeout;

/// 对象池配置
pub type ObjectPoolConfig = PoolConfig;

/// 可池化对象特征
pub trait Poolable: Send + Sync + 'static {
    /// 重置对象状态
    fn reset(&mut self);

    /// 检查对象是否可用
    fn is_valid(&self) -> bool;

    /// 获取对象创建时间
    fn created_at(&self) -> Instant;
}

/// 池化对象包装器
pub struct PooledObject<T: Poolable> {
    object: Option<T>,
    pool: Arc<ObjectPoolInner<T>>,
    acquired_at: Instant,
}

impl<T: Poolable> PooledObject<T> {
    /// 获取内部对象的引用
    pub fn as_ref(&self) -> &T {
        self.object.as_ref().unwrap()
    }
    
    /// 获取内部对象的可变引用
    pub fn as_mut(&mut self) -> &mut T {
        self.object.as_mut().unwrap()
    }
    
    /// 获取对象的所有权（不归还到池中）
    pub fn take(mut self) -> T {
        self.object.take().unwrap()
    }
}

impl<T: Poolable> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(obj) = self.object.take() {
            let pool = self.pool.clone();
            tokio::spawn(async move {
                pool.release(obj).await;
            });
        }
    }
}

impl<T: Poolable> std::ops::Deref for PooledObject<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T: Poolable> std::ops::DerefMut for PooledObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

/// 对象工厂
pub trait ObjectFactory<T: Poolable>: Send + Sync {
    /// 创建新对象
    fn create(&self) -> Result<T>;
}

/// 对象池内部实现
struct ObjectPoolInner<T: Poolable> {
    config: ObjectPoolConfig,
    idle_objects: Arc<Mutex<Vec<T>>>,
    semaphore: Arc<Semaphore>,
    stats: Arc<Mutex<PoolStats>>,
    factory: Arc<dyn ObjectFactory<T>>,
}

impl<T: Poolable + 'static> ObjectPoolInner<T> {
    /// 释放对象回池中
    async fn release(&self, mut obj: T) {
        // 检查对象是否有效
        if !obj.is_valid() {
            // 无效对象直接丢弃
            let mut stats = self.stats.lock().await;
            stats.total_destroys += 1;
            stats.total -= 1;
            stats.active -= 1;
            drop(stats);
            
            // 释放信号量
            self.semaphore.add_permits(1);
            return;
        }
        
        // 重置对象状态
        obj.reset();
        
        // 检查对象生命周期
        if let Some(max_lifetime) = self.config.max_lifetime {
            if obj.created_at().elapsed() > max_lifetime {
                // 对象超过最大生命周期，丢弃
                let mut stats = self.stats.lock().await;
                stats.total_destroys += 1;
                stats.total -= 1;
                stats.active -= 1;
                drop(stats);
                
                self.semaphore.add_permits(1);
                return;
            }
        }
        
        // 归还对象到空闲池
        let mut idle = self.idle_objects.lock().await;
        idle.push(obj);
        drop(idle);
        
        // 更新统计
        let mut stats = self.stats.lock().await;
        stats.total_releases += 1;
        stats.active -= 1;
        stats.idle += 1;
        stats.calculate_utilization();
    }
}

/// 对象池
pub struct ObjectPool<T: Poolable> {
    inner: Arc<ObjectPoolInner<T>>,
}

impl<T: Poolable + 'static> ObjectPool<T> {
    /// 创建新的对象池
    pub fn new(config: ObjectPoolConfig, factory: Arc<dyn ObjectFactory<T>>) -> Self {
        let inner = Arc::new(ObjectPoolInner {
            config: config.clone(),
            idle_objects: Arc::new(Mutex::new(Vec::new())),
            semaphore: Arc::new(Semaphore::new(config.max_size)),
            stats: Arc::new(Mutex::new(PoolStats::default())),
            factory,
        });
        
        Self { inner }
    }
    
    /// 获取对象
    pub async fn acquire(&self) -> Result<PooledObject<T>> {
        let start = Instant::now();
        
        // 尝试获取信号量许可
        let permit = match timeout(
            self.inner.config.acquire_timeout,
            self.inner.semaphore.acquire()
        ).await {
            Ok(Ok(permit)) => permit,
            Ok(Err(_)) => {
                let mut stats = self.inner.stats.lock().await;
                stats.acquire_timeouts += 1;
                return Err(crate::Error::Timeout("Failed to acquire object".to_string()));
            }
            Err(_) => {
                let mut stats = self.inner.stats.lock().await;
                stats.acquire_timeouts += 1;
                return Err(crate::Error::Timeout("Object acquire timeout".to_string()));
            }
        };
        
        // 忘记许可（我们手动管理）
        permit.forget();
        
        // 尝试从空闲池获取对象
        let mut idle = self.inner.idle_objects.lock().await;
        let obj = if let Some(obj) = idle.pop() {
            drop(idle);
            obj
        } else {
            drop(idle);
            // 创建新对象
            let obj = self.inner.factory.create()?;
            
            let mut stats = self.inner.stats.lock().await;
            stats.total_creates += 1;
            stats.total += 1;
            drop(stats);
            
            obj
        };
        
        // 更新统计
        let mut stats = self.inner.stats.lock().await;
        stats.total_acquires += 1;
        stats.active += 1;
        if stats.idle > 0 {
            stats.idle -= 1;
        }
        
        // 更新平均获取时间
        let acquire_time = start.elapsed().as_millis() as f64;
        if stats.total_acquires == 1 {
            stats.avg_acquire_time_ms = acquire_time;
        } else {
            stats.avg_acquire_time_ms = 
                (stats.avg_acquire_time_ms * (stats.total_acquires - 1) as f64 + acquire_time) 
                / stats.total_acquires as f64;
        }
        
        stats.calculate_utilization();
        drop(stats);
        
        Ok(PooledObject {
            object: Some(obj),
            pool: self.inner.clone(),
            acquired_at: Instant::now(),
        })
    }
    
    /// 获取池统计信息
    pub async fn stats(&self) -> PoolStats {
        self.inner.stats.lock().await.clone()
    }
    
    /// 预热对象池（创建最小数量的对象）
    pub fn warmup(&self) -> Result<()> {
        let min_size = self.inner.config.min_size;
        let mut objects = Vec::new();
        
        for _ in 0..min_size {
            let obj = self.inner.factory.create()?;
            objects.push(obj);
        }
        
        // 使用 tokio::spawn 异步初始化
        let idle = self.inner.idle_objects.clone();
        let stats = self.inner.stats.clone();
        
        tokio::spawn(async move {
            let mut idle = idle.lock().await;
            idle.extend(objects);
            
            let mut stats = stats.lock().await;
            stats.total = min_size;
            stats.idle = min_size;
            stats.total_creates = min_size as u64;
            stats.calculate_utilization();
        });
        
        Ok(())
    }
}

impl<T: Poolable> Clone for ObjectPool<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

