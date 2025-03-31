//! Agent网络基本类型定义

use std::fmt;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Agent ID
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentId(String);

impl AgentId {
    /// 创建新的随机Agent ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
    
    /// 从字符串创建Agent ID
    pub fn from_str(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    /// 获取Agent ID字符串
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// 获取内部字符串值的克隆
    pub fn value(&self) -> String {
        self.0.clone()
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for AgentId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<&str> for AgentId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}

/// Agent类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    /// 领导者节点
    Leader,
    /// 协调者节点
    Coordinator,
    /// 工作节点
    Worker,
    /// 常规节点
    Regular,
    /// 自定义类型
    Custom(String),
}

impl fmt::Display for AgentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentType::Leader => write!(f, "Leader"),
            AgentType::Coordinator => write!(f, "Coordinator"),
            AgentType::Worker => write!(f, "Worker"),
            AgentType::Regular => write!(f, "Regular"),
            AgentType::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

/// Agent状态
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    /// 已初始化
    Initialized,
    /// 正在运行
    Running,
    /// 已暂停
    Paused,
    /// 已停止
    Stopped,
    /// 错误状态
    Error,
}

/// Agent能力
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCapability {
    /// 能力名称
    pub name: String,
    /// 能力描述
    pub description: String,
    /// 能力元数据
    pub metadata: Option<serde_json::Value>,
}

impl AgentCapability {
    /// 创建新的Agent能力
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            metadata: None,
        }
    }
    
    /// 添加能力元数据
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Agent地理位置信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentLocation {
    /// 经度
    pub longitude: f64,
    /// 纬度
    pub latitude: f64,
    /// 区域标识
    pub region: Option<String>,
    /// 位置描述
    pub description: Option<String>,
}

impl AgentLocation {
    /// 创建新的位置信息
    pub fn new(longitude: f64, latitude: f64) -> Self {
        Self {
            longitude,
            latitude,
            region: None,
            description: None,
        }
    }
    
    /// 设置区域
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }
    
    /// 设置描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
} 