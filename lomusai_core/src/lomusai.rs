//! Lomusai主类实现，作为整个框架的入口点和资源管理器

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::{Serialize, Deserialize};

use crate::agent::{Agent, AgentConfig};
use crate::base::{Base, ComponentConfig};
use crate::error::{Error, Result};
use crate::logger::{Component, Logger, LogLevel, create_logger, create_noop_logger};
use crate::memory::Memory;
use crate::storage::Storage;
use crate::telemetry::{Event, TelemetrySink};
use crate::tool::Tool;
use crate::vector::VectorStorage;
use crate::workflow::Workflow;

/// Lomusai配置
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LomusaiConfig {
    /// 名称
    pub name: Option<String>,
    /// 日志级别
    pub log_level: Option<LogLevel>,
    /// 是否禁用日志
    pub disable_logger: bool,
}

/// Lomusai主类
pub struct Lomusai {
    /// 配置
    config: LomusaiConfig,
    /// 日志器
    logger: Arc<dyn Logger>,
    /// 遥测
    telemetry: Option<Arc<dyn TelemetrySink>>,
    /// Agents
    agents: Mutex<HashMap<String, Arc<dyn Agent>>>,
    /// 向量存储
    vectors: Mutex<HashMap<String, Arc<dyn VectorStorage>>>,
    /// 工作流
    workflows: Mutex<HashMap<String, Arc<dyn Workflow>>>,
    /// 存储
    storage: Option<Arc<dyn Storage>>,
    /// 内存
    memory: Option<Arc<dyn Memory>>,
}

impl Lomusai {
    /// 创建新的Lomusai实例
    pub fn new(config: LomusaiConfig) -> Self {
        let logger = if config.disable_logger {
            create_noop_logger()
        } else {
            create_logger(
                config.name.clone().unwrap_or_else(|| "Lomusai".to_string()),
                Component::Llm,
                config.log_level.unwrap_or(LogLevel::Info),
            )
        };

        Self {
            config,
            logger,
            telemetry: None,
            agents: Mutex::new(HashMap::new()),
            vectors: Mutex::new(HashMap::new()),
            workflows: Mutex::new(HashMap::new()),
            storage: None,
            memory: None,
        }
    }

    /// 设置日志器
    pub fn set_logger(&mut self, logger: Arc<dyn Logger>) {
        self.logger = logger;
    }

    /// 设置遥测
    pub fn set_telemetry(&mut self, telemetry: Arc<dyn TelemetrySink>) {
        self.telemetry = Some(telemetry.clone());
    }

    /// 获取日志器
    pub fn get_logger(&self) -> Arc<dyn Logger> {
        self.logger.clone()
    }

    /// 获取遥测
    pub fn get_telemetry(&self) -> Option<Arc<dyn TelemetrySink>> {
        self.telemetry.clone()
    }

    /// 注册Agent
    pub fn register_agent(&self, name: impl Into<String>, agent: Arc<dyn Agent>) -> Result<()> {
        let name = name.into();
        let mut agents = self.agents.lock().map_err(|_| Error::Lock("无法锁定agents".to_string()))?;
        
        if agents.contains_key(&name) {
            return Err(Error::AlreadyExists(format!("Agent '{}'已存在", name)));
        }
        
        agents.insert(name, agent);
        Ok(())
    }

    /// 获取Agent
    pub fn get_agent(&self, name: &str) -> Result<Arc<dyn Agent>> {
        let agents = self.agents.lock().map_err(|_| Error::Lock("无法锁定agents".to_string()))?;
        
        agents.get(name)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Agent '{}'不存在", name)))
    }

    /// 获取所有Agent
    pub fn get_agents(&self) -> Result<Vec<(String, Arc<dyn Agent>)>> {
        let agents = self.agents.lock().map_err(|_| Error::Lock("无法锁定agents".to_string()))?;
        
        Ok(agents.iter()
            .map(|(name, agent)| (name.clone(), agent.clone()))
            .collect())
    }

    /// 注册向量存储
    pub fn register_vector(&self, name: impl Into<String>, vector: Arc<dyn VectorStorage>) -> Result<()> {
        let name = name.into();
        let mut vectors = self.vectors.lock().map_err(|_| Error::Lock("无法锁定vectors".to_string()))?;
        
        if vectors.contains_key(&name) {
            return Err(Error::AlreadyExists(format!("Vector '{}'已存在", name)));
        }
        
        vectors.insert(name, vector);
        Ok(())
    }

    /// 获取向量存储
    pub fn get_vector(&self, name: &str) -> Result<Arc<dyn VectorStorage>> {
        let vectors = self.vectors.lock().map_err(|_| Error::Lock("无法锁定vectors".to_string()))?;
        
        vectors.get(name)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Vector '{}'不存在", name)))
    }

    /// 注册工作流
    pub fn register_workflow(&self, name: impl Into<String>, workflow: Arc<dyn Workflow>) -> Result<()> {
        let name = name.into();
        let mut workflows = self.workflows.lock().map_err(|_| Error::Lock("无法锁定workflows".to_string()))?;
        
        if workflows.contains_key(&name) {
            return Err(Error::AlreadyExists(format!("Workflow '{}'已存在", name)));
        }
        
        workflows.insert(name, workflow);
        Ok(())
    }

    /// 获取工作流
    pub fn get_workflow(&self, name: &str) -> Result<Arc<dyn Workflow>> {
        let workflows = self.workflows.lock().map_err(|_| Error::Lock("无法锁定workflows".to_string()))?;
        
        workflows.get(name)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Workflow '{}'不存在", name)))
    }

    /// 设置存储
    pub fn set_storage(&mut self, storage: Arc<dyn Storage>) {
        self.storage = Some(storage);
    }

    /// 获取存储
    pub fn get_storage(&self) -> Option<Arc<dyn Storage>> {
        self.storage.clone()
    }

    /// 设置内存
    pub fn set_memory(&mut self, memory: Arc<dyn Memory>) {
        self.memory = Some(memory);
    }

    /// 获取内存
    pub fn get_memory(&self) -> Option<Arc<dyn Memory>> {
        self.memory.clone()
    }
}

impl Base for Lomusai {
    fn name(&self) -> Option<&str> {
        self.config.name.as_deref()
    }
    
    fn component(&self) -> Component {
        Component::Llm
    }
    
    fn logger(&self) -> Arc<dyn Logger> {
        self.logger.clone()
    }
    
    fn set_logger(&mut self, logger: Arc<dyn Logger>) {
        self.logger = logger;
    }
    
    fn telemetry(&self) -> Option<Arc<dyn TelemetrySink>> {
        self.telemetry.clone()
    }
    
    fn set_telemetry(&mut self, telemetry: Arc<dyn TelemetrySink>) {
        self.telemetry = Some(telemetry);
    }
} 