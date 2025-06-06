//! 安全管理模块

use crate::{Result, CloudError};

pub struct SecurityManager {}

impl SecurityManager {
    pub async fn new() -> Result<Self> {
        Ok(Self {})
    }
}
