//! 网络管理模块

use crate::{Result, CloudError};

pub struct NetworkManager {}

impl NetworkManager {
    pub async fn new() -> Result<Self> {
        Ok(Self {})
    }
}
