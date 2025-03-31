//! 服务发现功能

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use tokio::time::interval;
use std::sync::atomic::{AtomicUsize, Ordering};
use futures::future::{ready, FutureExt};

use crate::error::{Error, Result};
use crate::types::{AgentId, AgentCapability, AgentType, AgentLocation};

/// 服务注册信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistration {
    /// Agent ID
    pub id: AgentId,
    /// Agent 类型
    pub agent_type: AgentType,
    /// Agent 能力
    pub capabilities: Vec<AgentCapability>,
    /// Agent 位置
    pub location: Option<AgentLocation>,
    /// 注册时间
    pub registered_at: SystemTime,
    /// 最后心跳时间
    pub last_heartbeat: SystemTime,
    /// TTL (生存时间，秒)
    pub ttl: u64,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl ServiceRegistration {
    /// 创建新的服务注册
    pub fn new(id: AgentId, agent_type: AgentType) -> Self {
        let now = SystemTime::now();
        Self {
            id,
            agent_type,
            capabilities: Vec::new(),
            location: None,
            registered_at: now,
            last_heartbeat: now,
            ttl: 60, // 默认60秒
            metadata: HashMap::new(),
        }
    }
    
    /// 添加能力
    pub fn with_capability(mut self, capability: AgentCapability) -> Self {
        self.capabilities.push(capability);
        self
    }
    
    /// 设置位置
    pub fn with_location(mut self, location: AgentLocation) -> Self {
        self.location = Some(location);
        self
    }
    
    /// 设置TTL
    pub fn with_ttl(mut self, ttl: u64) -> Self {
        self.ttl = ttl;
        self
    }
    
    /// 添加元数据
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// 更新心跳
    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = SystemTime::now();
    }
    
    /// 检查是否过期
    pub fn is_expired(&self) -> bool {
        if let Ok(elapsed) = self.last_heartbeat.elapsed() {
            elapsed.as_secs() > self.ttl
        } else {
            false
        }
    }
}

/// 服务发现查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceQuery {
    /// Agent 类型 (可选)
    pub agent_type: Option<AgentType>,
    /// 需要的能力 (可选)
    pub required_capabilities: Vec<String>,
    /// 元数据过滤条件
    pub metadata_filters: HashMap<String, String>,
}

/// 服务变化事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceEvent {
    /// 服务注册
    Registered(ServiceRegistration),
    /// 服务注销
    Deregistered(AgentId),
    /// 服务更新
    Updated(ServiceRegistration),
    /// 服务过期
    Expired(AgentId),
}

/// 服务发现接口
#[async_trait]
pub trait ServiceDiscovery: Send + Sync {
    /// 注册服务
    async fn register(&self, registration: ServiceRegistration) -> Result<()>;
    
    /// 注销服务
    async fn deregister(&self, id: &AgentId) -> Result<()>;
    
    /// 发送心跳
    async fn heartbeat(&self, id: &AgentId) -> Result<()>;
    
    /// 发现服务 - 按条件查询
    async fn discover(&self, query: &ServiceQuery) -> Result<Vec<ServiceRegistration>>;
    
    /// 通过ID查找服务
    async fn get_by_id(&self, id: &AgentId) -> Result<ServiceRegistration>;
    
    /// 查询所有服务
    async fn get_all(&self) -> Result<Vec<ServiceRegistration>>;
    
    /// 设置服务变化的监听函数
    async fn set_listener(&self, listener: Box<dyn Fn(ServiceEvent) + Send + Sync>);
}

/// 内存中的服务发现实现
pub struct InMemoryServiceDiscovery {
    /// 注册服务映射
    registrations: DashMap<AgentId, ServiceRegistration>,
    /// 事件监听器
    listeners: RwLock<Vec<Box<dyn Fn(ServiceEvent) + Send + Sync>>>,
    /// 注册回调
    register_callbacks: RwLock<Vec<Box<dyn Fn(ServiceEvent) + Send + Sync>>>,
    /// 更新回调
    update_callbacks: RwLock<Vec<Box<dyn Fn(ServiceEvent) + Send + Sync>>>,
    /// 注销回调
    deregister_callbacks: RwLock<Vec<Box<dyn Fn(ServiceEvent) + Send + Sync>>>,
}

impl InMemoryServiceDiscovery {
    /// 创建新的内存服务发现
    pub fn new() -> Arc<Self> {
        let discovery = Arc::new(Self {
            registrations: DashMap::new(),
            listeners: RwLock::new(Vec::new()),
            register_callbacks: RwLock::new(Vec::new()),
            update_callbacks: RwLock::new(Vec::new()),
            deregister_callbacks: RwLock::new(Vec::new()),
        });
        
        // 启动过期检查
        InMemoryServiceDiscovery::start_expiry_check(Arc::clone(&discovery));
        
        discovery
    }
    
    /// 启动过期检查任务
    fn start_expiry_check(discovery: Arc<Self>) {
        tokio::spawn(async move {
            let mut check_interval = interval(Duration::from_secs(5));
            
            loop {
                check_interval.tick().await;
                
                let expired_ids: Vec<AgentId> = discovery.registrations.iter()
                    .filter(|entry| entry.value().is_expired())
                    .map(|entry| entry.key().clone())
                    .collect();
                
                for id in expired_ids {
                    if let Some((_, registration)) = discovery.registrations.remove(&id) {
                        // 通知监听器
                        let listeners = discovery.listeners.read().await;
                        for listener in listeners.iter() {
                            listener(ServiceEvent::Expired(id.clone()));
                        }
                    }
                }
            }
        });
    }
    
    /// 通知所有监听器
    async fn notify_listeners(&self, event: ServiceEvent) {
        let listeners = self.listeners.read().await;
        for listener in listeners.iter() {
            listener(event.clone());
        }
    }
}

#[async_trait]
impl ServiceDiscovery for InMemoryServiceDiscovery {
    async fn register(&self, registration: ServiceRegistration) -> Result<()> {
        let id = registration.id.clone();
        let is_update = self.registrations.contains_key(&id);
        
        // 插入注册
        self.registrations.insert(id.clone(), registration.clone());
        
        // 通知监听器
        if is_update {
            self.notify_listeners(ServiceEvent::Updated(registration)).await;
        } else {
            self.notify_listeners(ServiceEvent::Registered(registration)).await;
        }
        
        Ok(())
    }
    
    async fn deregister(&self, id: &AgentId) -> Result<()> {
        if let Some((_, _)) = self.registrations.remove(id) {
            // 通知监听器
            self.notify_listeners(ServiceEvent::Deregistered(id.clone())).await;
            Ok(())
        } else {
            Err(Error::Discovery(format!("服务不存在: {}", id)))
        }
    }
    
    async fn heartbeat(&self, id: &AgentId) -> Result<()> {
        if let Some(mut entry) = self.registrations.get_mut(id) {
            entry.update_heartbeat();
            Ok(())
        } else {
            Err(Error::Discovery(format!("服务不存在: {}", id)))
        }
    }
    
    async fn discover(&self, query: &ServiceQuery) -> Result<Vec<ServiceRegistration>> {
        let mut results = Vec::new();
        
        for entry in self.registrations.iter() {
            let registration = entry.value();
            
            // 检查是否过期
            if registration.is_expired() {
                continue;
            }
            
            // 检查Agent类型
            if let Some(ref agent_type) = query.agent_type {
                if &registration.agent_type != agent_type {
                    continue;
                }
            }
            
            // 检查能力
            let has_capabilities = query.required_capabilities.iter().all(|required_cap| {
                registration.capabilities.iter().any(|cap| &cap.name == required_cap)
            });
            
            if !query.required_capabilities.is_empty() && !has_capabilities {
                continue;
            }
            
            // 检查元数据
            let metadata_match = query.metadata_filters.iter().all(|(key, value)| {
                registration.metadata.get(key).map_or(false, |v| v == value)
            });
            
            if !query.metadata_filters.is_empty() && !metadata_match {
                continue;
            }
            
            // 所有条件匹配，添加到结果
            results.push(registration.clone());
        }
        
        Ok(results)
    }
    
    async fn get_by_id(&self, id: &AgentId) -> Result<ServiceRegistration> {
        if let Some(entry) = self.registrations.get(id) {
            let registration = entry.value().clone();
            if registration.is_expired() {
                Err(Error::Discovery(format!("服务已过期: {}", id)))
            } else {
                Ok(registration)
            }
        } else {
            Err(Error::Discovery(format!("服务不存在: {}", id)))
        }
    }
    
    async fn get_all(&self) -> Result<Vec<ServiceRegistration>> {
        let mut results = Vec::new();
        
        for entry in self.registrations.iter() {
            let registration = entry.value();
            if !registration.is_expired() {
                results.push(registration.clone());
            }
        }
        
        Ok(results)
    }
    
    async fn set_listener(&self, listener: Box<dyn Fn(ServiceEvent) + Send + Sync>) {
        let mut listeners = self.listeners.write().await;
        listeners.push(listener);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::time::sleep;
    
    #[tokio::test]
    async fn test_service_registration() {
        let discovery = InMemoryServiceDiscovery::new();
        
        let agent_id = AgentId::from_str("test-agent");
        let registration = ServiceRegistration::new(agent_id.clone(), AgentType::Regular)
            .with_capability(AgentCapability::new("search", "Search capability"))
            .with_metadata("region", "us-west");
        
        // 注册服务
        discovery.register(registration.clone()).await.unwrap();
        
        // 查询所有服务
        let services = discovery.get_all().await.unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].id, agent_id);
        
        // 通过ID查询
        let service = discovery.get_by_id(&agent_id).await.unwrap();
        assert_eq!(service.id, agent_id);
        assert_eq!(service.capabilities.len(), 1);
        assert_eq!(service.capabilities[0].name, "search");
        assert_eq!(service.metadata.get("region").unwrap(), "us-west");
    }
    
    #[tokio::test]
    async fn test_service_query() {
        let discovery = InMemoryServiceDiscovery::new();
        
        // 注册三个不同的服务
        let agent1 = AgentId::from_str("agent1");
        let reg1 = ServiceRegistration::new(agent1.clone(), AgentType::Regular)
            .with_capability(AgentCapability::new("search", "Search capability"))
            .with_metadata("region", "us-west");
        
        let agent2 = AgentId::from_str("agent2");
        let reg2 = ServiceRegistration::new(agent2.clone(), AgentType::Regular)
            .with_capability(AgentCapability::new("search", "Search capability"))
            .with_capability(AgentCapability::new("compute", "Compute capability"))
            .with_metadata("region", "us-west")
            .with_ttl(60);
        
        let agent3 = AgentId::from_str("agent3");
        let reg3 = ServiceRegistration::new(agent3.clone(), AgentType::Regular)
            .with_capability(AgentCapability::new("search", "Search capability"))
            .with_capability(AgentCapability::new("storage", "Storage capability"))
            .with_metadata("region", "us-west");
        
        discovery.register(reg1).await.unwrap();
        discovery.register(reg2).await.unwrap();
        discovery.register(reg3).await.unwrap();
        
        // 测试基于Agent类型的查询
        let type_query = ServiceQuery {
            agent_type: Some(AgentType::Regular),
            required_capabilities: Vec::new(),
            metadata_filters: HashMap::new(),
        };
        
        let results = discovery.discover(&type_query).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|r| r.id == agent1));
        assert!(results.iter().any(|r| r.id == agent3));
        
        // 测试基于能力的查询
        let capability_query = ServiceQuery {
            agent_type: None,
            required_capabilities: vec!["storage".to_string()],
            metadata_filters: HashMap::new(),
        };
        
        let results = discovery.discover(&capability_query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, agent3);
        
        // 测试基于元数据的查询
        let mut metadata_filters = HashMap::new();
        metadata_filters.insert("region".to_string(), "us-west".to_string());
        
        let metadata_query = ServiceQuery {
            agent_type: None,
            required_capabilities: Vec::new(),
            metadata_filters,
        };
        
        let results = discovery.discover(&metadata_query).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|r| r.id == agent1));
        assert!(results.iter().any(|r| r.id == agent3));
    }
    
    /// 测试服务事件
    #[tokio::test]
    async fn test_service_events() {
        let discovery = InMemoryServiceDiscovery::new();
        
        // 事件计数器
        let registered_count = Arc::new(AtomicUsize::new(0));
        let updated_count = Arc::new(AtomicUsize::new(0));
        let deregistered_count = Arc::new(AtomicUsize::new(0));
        
        // 设置监听器
        {
            let registered_count = Arc::clone(&registered_count);
            let updated_count = Arc::clone(&updated_count);
            let deregistered_count = Arc::clone(&deregistered_count);
            
            discovery.set_listener(Box::new(move |event| {
                match event {
                    ServiceEvent::Registered(_) => {
                        registered_count.fetch_add(1, Ordering::SeqCst);
                    },
                    ServiceEvent::Updated(_) => {
                        updated_count.fetch_add(1, Ordering::SeqCst);
                    },
                    ServiceEvent::Deregistered(_) => {
                        deregistered_count.fetch_add(1, Ordering::SeqCst);
                    },
                    _ => {}
                }
            })).await;
        }
        
        // 测试注册事件
        let agent_id = AgentId::from_str("test-agent");
        let registration = ServiceRegistration::new(agent_id.clone(), AgentType::Regular);
        
        discovery.register(registration.clone()).await.unwrap();
        assert_eq!(registered_count.load(Ordering::SeqCst), 1);
        
        // 测试更新事件
        let updated_registration = ServiceRegistration::new(agent_id.clone(), AgentType::Regular);
        discovery.register(updated_registration).await.unwrap();
        assert_eq!(updated_count.load(Ordering::SeqCst), 1);
        
        // 测试注销事件
        discovery.deregister(&agent_id).await.unwrap();
        assert_eq!(deregistered_count.load(Ordering::SeqCst), 1);
    }
    
    #[tokio::test]
    async fn test_service_expiry() {
        let discovery = InMemoryServiceDiscovery::new();
        
        // 注册带有非常短TTL的服务
        let agent_id = AgentId::from_str("short-lived");
        let registration = ServiceRegistration::new(agent_id.clone(), AgentType::Regular)
            .with_ttl(1); // 1秒TTL
        
        discovery.register(registration).await.unwrap();
        
        // 等待过期检查触发（至少5秒）
        sleep(Duration::from_secs(6)).await;
        
        // 服务应该已经过期
        let result = discovery.get_by_id(&agent_id).await;
        assert!(result.is_err());
        
        // 应该没有活跃服务
        let services = discovery.get_all().await.unwrap();
        assert_eq!(services.len(), 0);
    }
    
    #[tokio::test]
    async fn test_heartbeat() {
        let discovery = InMemoryServiceDiscovery::new();
        
        // 注册带有短TTL的服务
        let agent_id = AgentId::from_str("heartbeat-test");
        let registration = ServiceRegistration::new(agent_id.clone(), AgentType::Regular)
            .with_ttl(2); // 2秒TTL
        
        discovery.register(registration).await.unwrap();
        
        // 等待接近过期
        sleep(Duration::from_secs(1)).await;
        
        // 发送心跳
        discovery.heartbeat(&agent_id).await.unwrap();
        
        // 再等待1秒 - 如果没有心跳，服务应该过期，但因为有心跳，所以不会过期
        sleep(Duration::from_secs(1)).await;
        
        // 服务应该仍然有效
        let result = discovery.get_by_id(&agent_id).await;
        assert!(result.is_ok());
    }
} 