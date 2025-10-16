//! 自动扩容模块

use crate::{CloudError, Result};

pub struct AutoScalingManager {}

impl AutoScalingManager {
    pub async fn new() -> Result<Self> {
        Ok(Self {})
    }
}
