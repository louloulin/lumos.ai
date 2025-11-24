//! LLM Provider 成本监控
//!
//! 提供成本跟踪、查询和报告功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// 成本记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRecord {
    /// Provider 名称
    pub provider_name: String,
    /// 输入 tokens
    pub input_tokens: u32,
    /// 输出 tokens
    pub output_tokens: u32,
    /// 总 tokens
    pub total_tokens: u32,
    /// 成本（美元）
    pub cost: f64,
    /// 时间戳（秒）
    pub timestamp: u64,
    /// 请求 ID（可选，用于追踪）
    pub request_id: Option<String>,
}

impl CostRecord {
    /// 创建新的成本记录
    pub fn new(
        provider_name: String,
        input_tokens: u32,
        output_tokens: u32,
        cost_per_1k_input: f64,
        cost_per_1k_output: f64,
    ) -> Self {
        let total_tokens = input_tokens + output_tokens;
        let cost = (input_tokens as f64 / 1000.0) * cost_per_1k_input
            + (output_tokens as f64 / 1000.0) * cost_per_1k_output;

        Self {
            provider_name,
            input_tokens,
            output_tokens,
            total_tokens,
            cost,
            timestamp: now_timestamp(),
            request_id: None,
        }
    }

    /// 使用总 tokens 创建成本记录（简化版本）
    pub fn from_total_tokens(
        provider_name: String,
        total_tokens: u32,
        cost_per_1k_tokens: f64,
    ) -> Self {
        let cost = (total_tokens as f64 / 1000.0) * cost_per_1k_tokens;

        Self {
            provider_name,
            input_tokens: 0,
            output_tokens: total_tokens,
            total_tokens,
            cost,
            timestamp: now_timestamp(),
            request_id: None,
        }
    }
}

/// 成本统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostStats {
    /// 总成本（美元）
    pub total_cost: f64,
    /// 总输入 tokens
    pub total_input_tokens: u64,
    /// 总输出 tokens
    pub total_output_tokens: u64,
    /// 总 tokens
    pub total_tokens: u64,
    /// 请求数量
    pub request_count: u64,
    /// 按 provider 的成本统计
    pub by_provider: HashMap<String, ProviderCostStats>,
}

/// Provider 成本统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCostStats {
    /// Provider 名称
    pub provider_name: String,
    /// 总成本（美元）
    pub total_cost: f64,
    /// 总输入 tokens
    pub total_input_tokens: u64,
    /// 总输出 tokens
    pub total_output_tokens: u64,
    /// 总 tokens
    pub total_tokens: u64,
    /// 请求数量
    pub request_count: u64,
}

/// 成本监控器
pub struct CostMonitor {
    /// 成本记录
    records: Arc<RwLock<Vec<CostRecord>>>,
    /// Provider 成本配置（每 1000 tokens 的成本）
    provider_costs: Arc<RwLock<HashMap<String, (f64, f64)>>>,
}

impl CostMonitor {
    /// 创建新的成本监控器
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(Vec::new())),
            provider_costs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 设置 provider 的成本配置
    ///
    /// # 参数
    ///
    /// * `provider_name` - Provider 名称
    /// * `cost_per_1k_input` - 每 1000 输入 tokens 的成本（美元）
    /// * `cost_per_1k_output` - 每 1000 输出 tokens 的成本（美元）
    pub async fn set_provider_cost(
        &self,
        provider_name: String,
        cost_per_1k_input: f64,
        cost_per_1k_output: f64,
    ) {
        let mut costs = self.provider_costs.write().await;
        costs.insert(provider_name, (cost_per_1k_input, cost_per_1k_output));
    }

    /// 记录成本
    pub async fn record_cost(
        &self,
        provider_name: String,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<CostRecord, String> {
        let costs = self.provider_costs.read().await;
        let (cost_per_1k_input, cost_per_1k_output) = costs
            .get(&provider_name)
            .copied()
            .unwrap_or((0.002, 0.002)); // 默认成本

        let record = CostRecord::new(
            provider_name,
            input_tokens,
            output_tokens,
            cost_per_1k_input,
            cost_per_1k_output,
        );

        let mut records = self.records.write().await;
        records.push(record.clone());

        Ok(record)
    }

    /// 使用总 tokens 记录成本（简化版本）
    pub async fn record_cost_simple(
        &self,
        provider_name: String,
        total_tokens: u32,
    ) -> std::result::Result<CostRecord, String> {
        let costs = self.provider_costs.read().await;
        let (cost_per_1k_input, _) = costs
            .get(&provider_name)
            .copied()
            .unwrap_or((0.002, 0.002)); // 默认成本

        let record = CostRecord::from_total_tokens(provider_name, total_tokens, cost_per_1k_input);

        let mut records = self.records.write().await;
        records.push(record.clone());

        Ok(record)
    }

    /// 获取总成本统计
    pub async fn get_total_stats(&self) -> CostStats {
        let records = self.records.read().await;
        let mut total_cost = 0.0;
        let mut total_input_tokens = 0u64;
        let mut total_output_tokens = 0u64;
        let mut total_tokens = 0u64;
        let mut by_provider: HashMap<String, ProviderCostStats> = HashMap::new();

        for record in records.iter() {
            total_cost += record.cost;
            total_input_tokens += record.input_tokens as u64;
            total_output_tokens += record.output_tokens as u64;
            total_tokens += record.total_tokens as u64;

            let provider_stats = by_provider
                .entry(record.provider_name.clone())
                .or_insert_with(|| ProviderCostStats {
                    provider_name: record.provider_name.clone(),
                    total_cost: 0.0,
                    total_input_tokens: 0,
                    total_output_tokens: 0,
                    total_tokens: 0,
                    request_count: 0,
                });

            provider_stats.total_cost += record.cost;
            provider_stats.total_input_tokens += record.input_tokens as u64;
            provider_stats.total_output_tokens += record.output_tokens as u64;
            provider_stats.total_tokens += record.total_tokens as u64;
            provider_stats.request_count += 1;
        }

        CostStats {
            total_cost,
            total_input_tokens,
            total_output_tokens,
            total_tokens,
            request_count: records.len() as u64,
            by_provider,
        }
    }

    /// 获取指定时间范围内的成本统计
    pub async fn get_stats_in_range(&self, start_time: u64, end_time: u64) -> CostStats {
        let records = self.records.read().await;
        let filtered_records: Vec<&CostRecord> = records
            .iter()
            .filter(|r| r.timestamp >= start_time && r.timestamp <= end_time)
            .collect();

        let mut total_cost = 0.0;
        let mut total_input_tokens = 0u64;
        let mut total_output_tokens = 0u64;
        let mut total_tokens = 0u64;
        let mut by_provider: HashMap<String, ProviderCostStats> = HashMap::new();

        for record in filtered_records.iter() {
            total_cost += record.cost;
            total_input_tokens += record.input_tokens as u64;
            total_output_tokens += record.output_tokens as u64;
            total_tokens += record.total_tokens as u64;

            let provider_stats = by_provider
                .entry(record.provider_name.clone())
                .or_insert_with(|| ProviderCostStats {
                    provider_name: record.provider_name.clone(),
                    total_cost: 0.0,
                    total_input_tokens: 0,
                    total_output_tokens: 0,
                    total_tokens: 0,
                    request_count: 0,
                });

            provider_stats.total_cost += record.cost;
            provider_stats.total_input_tokens += record.input_tokens as u64;
            provider_stats.total_output_tokens += record.output_tokens as u64;
            provider_stats.total_tokens += record.total_tokens as u64;
            provider_stats.request_count += 1;
        }

        CostStats {
            total_cost,
            total_input_tokens,
            total_output_tokens,
            total_tokens,
            request_count: filtered_records.len() as u64,
            by_provider,
        }
    }

    /// 获取指定 provider 的成本统计
    pub async fn get_provider_stats(&self, provider_name: &str) -> Option<ProviderCostStats> {
        let records = self.records.read().await;
        let mut total_cost = 0.0;
        let mut total_input_tokens = 0u64;
        let mut total_output_tokens = 0u64;
        let mut total_tokens = 0u64;
        let mut request_count = 0u64;

        for record in records.iter() {
            if record.provider_name == provider_name {
                total_cost += record.cost;
                total_input_tokens += record.input_tokens as u64;
                total_output_tokens += record.output_tokens as u64;
                total_tokens += record.total_tokens as u64;
                request_count += 1;
            }
        }

        if request_count == 0 {
            return None;
        }

        Some(ProviderCostStats {
            provider_name: provider_name.to_string(),
            total_cost,
            total_input_tokens,
            total_output_tokens,
            total_tokens,
            request_count,
        })
    }

    /// 清除所有记录
    pub async fn clear(&self) {
        let mut records = self.records.write().await;
        records.clear();
    }

    /// 获取所有记录
    pub async fn get_all_records(&self) -> Vec<CostRecord> {
        let records = self.records.read().await;
        records.clone()
    }
}

impl Default for CostMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// 获取当前时间戳（秒）
fn now_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cost_monitor_basic() {
        let monitor = CostMonitor::new();
        
        // 设置 provider 成本
        monitor.set_provider_cost("openai".to_string(), 0.002, 0.002).await;
        
        // 记录成本
        let record = monitor.record_cost("openai".to_string(), 1000, 500).await.unwrap();
        assert_eq!(record.input_tokens, 1000);
        assert_eq!(record.output_tokens, 500);
        assert_eq!(record.total_tokens, 1500);
        assert!((record.cost - 0.003).abs() < 0.0001); // 1000 * 0.002/1000 + 500 * 0.002/1000 = 0.003
        
        // 获取统计
        let stats = monitor.get_total_stats().await;
        assert_eq!(stats.total_cost, record.cost);
        assert_eq!(stats.total_tokens, 1500);
        assert_eq!(stats.request_count, 1);
    }

    #[tokio::test]
    async fn test_cost_monitor_multiple_providers() {
        let monitor = CostMonitor::new();
        
        monitor.set_provider_cost("openai".to_string(), 0.002, 0.002).await;
        monitor.set_provider_cost("anthropic".to_string(), 0.003, 0.003).await;
        
        monitor.record_cost("openai".to_string(), 1000, 500).await.unwrap();
        monitor.record_cost("anthropic".to_string(), 2000, 1000).await.unwrap();
        
        let stats = monitor.get_total_stats().await;
        assert_eq!(stats.request_count, 2);
        assert_eq!(stats.by_provider.len(), 2);
        
        let openai_stats = stats.by_provider.get("openai").unwrap();
        assert_eq!(openai_stats.total_tokens, 1500);
        
        let anthropic_stats = stats.by_provider.get("anthropic").unwrap();
        assert_eq!(anthropic_stats.total_tokens, 3000);
    }

    #[tokio::test]
    async fn test_cost_monitor_time_range() {
        let monitor = CostMonitor::new();
        
        monitor.set_provider_cost("openai".to_string(), 0.002, 0.002).await;
        
        // 记录一些成本
        monitor.record_cost("openai".to_string(), 1000, 500).await.unwrap();
        
        // 等待一秒
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        let current_time = now_timestamp();
        let future_time = current_time + 100;
        
        // 获取时间范围内的统计（应该包含之前的记录）
        let stats = monitor.get_stats_in_range(0, future_time).await;
        assert_eq!(stats.request_count, 1);
    }

    #[tokio::test]
    async fn test_cost_monitor_provider_stats() {
        let monitor = CostMonitor::new();
        
        monitor.set_provider_cost("openai".to_string(), 0.002, 0.002).await;
        monitor.record_cost("openai".to_string(), 1000, 500).await.unwrap();
        monitor.record_cost("openai".to_string(), 2000, 1000).await.unwrap();
        
        let stats = monitor.get_provider_stats("openai").await.unwrap();
        assert_eq!(stats.request_count, 2);
        assert_eq!(stats.total_tokens, 4500);
    }

    #[tokio::test]
    async fn test_cost_monitor_simple() {
        let monitor = CostMonitor::new();
        
        monitor.set_provider_cost("openai".to_string(), 0.002, 0.002).await;
        
        let record = monitor.record_cost_simple("openai".to_string(), 1000).await.unwrap();
        assert_eq!(record.total_tokens, 1000);
        assert!((record.cost - 0.002).abs() < 0.0001);
    }
}

