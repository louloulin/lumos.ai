//! 核心通用类型

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use uuid::Uuid;

/// 通用ID生成函数
pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

/// 通用metadata类型
pub type Metadata = HashMap<String, Value>;

/// 支持序列化和反序列化的配置特性
pub trait Config: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync {}

/// 可以被序列化和反序列化的核心类型特性
pub trait CoreType: serde::Serialize + serde::de::DeserializeOwned + Clone + Send + Sync {}

// 为所有满足条件的类型实现CoreType特性
impl<T> CoreType for T where T: serde::Serialize + serde::de::DeserializeOwned + Clone + Send + Sync {}

/// 通用ID类型，使用UUID
pub type Id = String;

/// 通用时间戳类型
pub use chrono::{DateTime, Utc}; 