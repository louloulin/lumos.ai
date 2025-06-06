//! 云原生监控模块

use crate::{Result, CloudError};

pub struct CloudMonitoring {}

impl CloudMonitoring {
    pub async fn new() -> Result<Self> {
        Ok(Self {})
    }
}
