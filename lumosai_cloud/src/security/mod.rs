//! 安全管理模块

use crate::{CloudError, Result};

pub struct SecurityManager {}

impl SecurityManager {
    pub async fn new() -> Result<Self> {
        Ok(Self {})
    }
}
