//! 存储管理模块

use crate::{CloudError, Result};

pub struct StorageManager {}

impl StorageManager {
    pub async fn new() -> Result<Self> {
        Ok(Self {})
    }
}
