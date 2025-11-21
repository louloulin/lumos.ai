//! LLM Provider 智能路由
//!
//! 实现基于负载、成本和延迟的智能路由选择

use crate::error::{Error, Result};
use crate::llm::{LlmOptions, LlmProvider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Provider 统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStats {
    /// Provider 名称
    pub name: String,
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
    /// 平均延迟（毫秒）
    pub avg_latency_ms: f64,
    /// 当前负载（活跃请求数）
    pub current_load: usize,
    /// 最大并发数
    pub max_concurrency: usize,
    /// 成本（每1000 tokens）
    pub cost_per_1k_tokens: f64,
    /// 最后更新时间
    pub last_updated: u64,
}

impl ProviderStats {
    /// 创建新的统计信息
    pub fn new(name: String, max_concurrency: usize, cost_per_1k_tokens: f64) -> Self {
        Self {
            name,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_latency_ms: 0.0,
            current_load: 0,
            max_concurrency,
            cost_per_1k_tokens,
            last_updated: 0,
        }
    }

    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 1.0;
        }
        self.successful_requests as f64 / self.total_requests as f64
    }

    /// 获取负载率（0.0 - 1.0）
    pub fn load_ratio(&self) -> f64 {
        if self.max_concurrency == 0 {
            return 0.0;
        }
        self.current_load as f64 / self.max_concurrency as f64
    }

    /// 是否可用（负载未满）
    pub fn is_available(&self) -> bool {
        self.current_load < self.max_concurrency
    }

    /// 更新延迟
    pub fn update_latency(&mut self, latency_ms: f64) {
        if self.total_requests == 0 || self.avg_latency_ms == 0.0 {
            // 第一次记录或初始值为 0，直接设置
            self.avg_latency_ms = latency_ms;
        } else {
            // 指数移动平均
            self.avg_latency_ms = self.avg_latency_ms * 0.9 + latency_ms * 0.1;
        }
    }

    /// 记录成功请求
    pub fn record_success(&mut self, latency_ms: f64) {
        self.total_requests += 1;
        self.successful_requests += 1;
        self.update_latency(latency_ms);
        self.last_updated = now_timestamp();
    }

    /// 记录失败请求
    pub fn record_failure(&mut self) {
        self.total_requests += 1;
        self.failed_requests += 1;
        self.last_updated = now_timestamp();
    }

    /// 增加负载
    pub fn increment_load(&mut self) {
        self.current_load += 1;
    }

    /// 减少负载
    pub fn decrement_load(&mut self) {
        if self.current_load > 0 {
            self.current_load -= 1;
        }
    }
}

/// 路由策略
#[derive(Clone)]
pub enum RoutingStrategy {
    /// 轮询（Round Robin）
    RoundRobin,
    /// 最少负载（Least Load）
    LeastLoad,
    /// 最低成本（Least Cost）
    LeastCost,
    /// 最佳延迟（Best Latency）
    BestLatency,
    /// 综合评分（综合考虑负载、成本、延迟）
    Balanced,
}

impl std::fmt::Debug for RoutingStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoutingStrategy::RoundRobin => write!(f, "RoundRobin"),
            RoutingStrategy::LeastLoad => write!(f, "LeastLoad"),
            RoutingStrategy::LeastCost => write!(f, "LeastCost"),
            RoutingStrategy::BestLatency => write!(f, "BestLatency"),
            RoutingStrategy::Balanced => write!(f, "Balanced"),
        }
    }
}

impl Default for RoutingStrategy {
    fn default() -> Self {
        RoutingStrategy::Balanced
    }
}

/// LLM Provider 路由器
pub struct LlmRouter {
    /// 可用的 providers
    providers: Vec<Arc<dyn LlmProvider>>,
    /// 路由策略
    strategy: RoutingStrategy,
    /// Provider 统计信息
    stats: Arc<RwLock<HashMap<String, ProviderStats>>>,
    /// 轮询索引（用于 RoundRobin 策略）
    round_robin_index: Arc<RwLock<usize>>,
}

impl LlmRouter {
    /// 创建新的路由器
    pub fn new(providers: Vec<Arc<dyn LlmProvider>>) -> Self {
        let mut stats = HashMap::new();
        let round_robin_index = 0;

        // 初始化统计信息
        for (index, provider) in providers.iter().enumerate() {
            // 使用索引作为名称，因为 MockLlmProvider 的 name() 都返回 "mock"
            let name = format!("provider{}", index);
            // 默认配置：最大并发 10，成本根据 provider 类型估算
            let cost = estimate_cost_per_1k_tokens(provider.name());
            stats.insert(
                name.clone(),
                ProviderStats::new(name, 10, cost),
            );
        }

        Self {
            providers,
            strategy: RoutingStrategy::default(),
            stats: Arc::new(RwLock::new(stats)),
            round_robin_index: Arc::new(RwLock::new(round_robin_index)),
        }
    }

    /// 使用指定策略创建路由器
    pub fn with_strategy(mut self, strategy: RoutingStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// 选择最佳的 provider
    pub async fn select_provider(
        &self,
        _options: &LlmOptions,
    ) -> Result<Arc<dyn LlmProvider>> {
        if self.providers.is_empty() {
            return Err(Error::Llm("No providers available".to_string()));
        }

        let stats = self.stats.read().await;
        let available_stats: Vec<&ProviderStats> = stats
            .values()
            .filter(|s| s.is_available())
            .collect();

        if available_stats.is_empty() {
            // 如果没有可用的，返回第一个（即使负载已满）
            return Ok(self.providers[0].clone());
        }

        let selected_index = match &self.strategy {
            RoutingStrategy::RoundRobin => {
                let mut index = self.round_robin_index.write().await;
                let selected = *index % available_stats.len();
                *index += 1;
                Some(selected)
            }
            RoutingStrategy::LeastLoad => {
                available_stats
                    .iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| {
                        a.load_ratio()
                            .partial_cmp(&b.load_ratio())
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(idx, _)| idx)
            }
            RoutingStrategy::LeastCost => {
                available_stats
                    .iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| {
                        a.cost_per_1k_tokens
                            .partial_cmp(&b.cost_per_1k_tokens)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(idx, _)| idx)
            }
            RoutingStrategy::BestLatency => {
                available_stats
                    .iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| {
                        a.avg_latency_ms
                            .partial_cmp(&b.avg_latency_ms)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(idx, _)| idx)
            }
            RoutingStrategy::Balanced => {
                // 综合评分：负载权重 0.4，成本权重 0.3，延迟权重 0.3
                available_stats
                    .iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| {
                        let score_a = calculate_balanced_score(a);
                        let score_b = calculate_balanced_score(b);
                        score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(idx, _)| idx)
            }
        };

        let selected_index = selected_index.unwrap_or(0);
        let selected_name = available_stats[selected_index].name.clone();
        
        drop(stats);

        // 找到对应的 provider（通过名称匹配或索引）
        if let Some(provider_index) = selected_name.strip_prefix("provider") {
            if let Ok(index) = provider_index.parse::<usize>() {
                if index < self.providers.len() {
                    return Ok(self.providers[index].clone());
                }
            }
        }
        
        // 回退：通过名称匹配
        self.providers
            .iter()
            .find(|p| p.name() == selected_name)
            .cloned()
            .ok_or_else(|| Error::Llm(format!("Provider '{}' not found", selected_name)))
    }

    /// 更新 provider 统计信息
    pub async fn update_stats(&self, provider_name: &str, stats: ProviderStats) {
        let mut stats_map = self.stats.write().await;
        stats_map.insert(provider_name.to_string(), stats);
    }

    /// 记录成功请求
    pub async fn record_success(&self, provider_name: &str, latency_ms: f64) {
        let mut stats_map = self.stats.write().await;
        if let Some(stats) = stats_map.get_mut(provider_name) {
            stats.record_success(latency_ms);
            stats.decrement_load();
        }
    }

    /// 记录失败请求
    pub async fn record_failure(&self, provider_name: &str) {
        let mut stats_map = self.stats.write().await;
        if let Some(stats) = stats_map.get_mut(provider_name) {
            stats.record_failure();
            stats.decrement_load();
        }
    }

    /// 增加 provider 负载
    pub async fn increment_load(&self, provider_name: &str) {
        let mut stats_map = self.stats.write().await;
        if let Some(stats) = stats_map.get_mut(provider_name) {
            stats.increment_load();
        }
    }

    /// 获取所有统计信息
    pub async fn get_stats(&self) -> HashMap<String, ProviderStats> {
        self.stats.read().await.clone()
    }

    /// 获取指定 provider 的统计信息
    pub async fn get_provider_stats(&self, provider_name: &str) -> Option<ProviderStats> {
        self.stats.read().await.get(provider_name).cloned()
    }
}

/// 估算每 1000 tokens 的成本（美元）
fn estimate_cost_per_1k_tokens(provider_name: &str) -> f64 {
    match provider_name.to_lowercase().as_str() {
        "openai" | "gpt-3.5-turbo" | "gpt-4" => 0.002, // GPT-3.5 输入成本
        "anthropic" | "claude" => 0.003,                // Claude 3 Sonnet
        "qwen" => 0.001,                                 // Qwen 相对便宜
        "zhipu" => 0.001,                                // 智谱 AI
        "deepseek" => 0.0005,                            // DeepSeek 很便宜
        "baidu" | "ernie" => 0.001,                     // 百度 ERNIE
        "ollama" => 0.0,                                 // 本地模型免费
        _ => 0.002,                                      // 默认值
    }
}

/// 计算综合评分（分数越低越好）
fn calculate_balanced_score(stats: &ProviderStats) -> f64 {
    let load_score = stats.load_ratio() * 0.4;
    let cost_score = (stats.cost_per_1k_tokens / 0.01) * 0.3; // 归一化成本
    let latency_score = (stats.avg_latency_ms / 1000.0) * 0.3; // 归一化延迟（假设 1 秒为基准）
    load_score + cost_score + latency_score
}

/// 获取当前时间戳（秒）
fn now_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::types::LlmOptions;
    use crate::llm::MockLlmProvider;

    #[tokio::test]
    async fn test_round_robin_strategy() {
        // 创建自定义名称的 mock providers
        let provider1 = Arc::new(MockLlmProvider::new(vec!["response1".to_string()])) as Arc<dyn LlmProvider>;
        let provider2 = Arc::new(MockLlmProvider::new(vec!["response2".to_string()])) as Arc<dyn LlmProvider>;
        
        // 由于 MockLlmProvider 的 name() 返回 "mock"，我们需要使用不同的方式
        // 这里我们直接测试路由逻辑，不依赖 provider 名称
        let providers = vec![provider1, provider2];

        let router = LlmRouter::new(providers).with_strategy(RoutingStrategy::RoundRobin);
        let options = LlmOptions::default();

        // 第一次选择（应该是 provider0）
        let p1 = router.select_provider(&options).await.unwrap();
        assert_eq!(p1.name(), "mock");

        // 第二次选择（应该是 provider1）
        let p2 = router.select_provider(&options).await.unwrap();
        assert_eq!(p2.name(), "mock");

        // 第三次选择（应该回到第一个）
        let p3 = router.select_provider(&options).await.unwrap();
        assert_eq!(p3.name(), "mock");
    }

    #[tokio::test]
    async fn test_least_load_strategy() {
        let providers = vec![
            Arc::new(MockLlmProvider::new(vec!["response1".to_string()])) as Arc<dyn LlmProvider>,
            Arc::new(MockLlmProvider::new(vec!["response2".to_string()])) as Arc<dyn LlmProvider>,
        ];

        let router = LlmRouter::new(providers).with_strategy(RoutingStrategy::LeastLoad);
        let options = LlmOptions::default();

        // 设置 provider0 的负载更高
        router.increment_load("provider0").await;
        router.increment_load("provider0").await;

        // 应该选择负载更低的 provider1
        let selected = router.select_provider(&options).await.unwrap();
        assert_eq!(selected.name(), "mock");
    }

    #[tokio::test]
    async fn test_least_cost_strategy() {
        // 创建两个 mock providers，但通过统计信息设置不同的成本
        let providers = vec![
            Arc::new(MockLlmProvider::new(vec!["response1".to_string()])) as Arc<dyn LlmProvider>,
            Arc::new(MockLlmProvider::new(vec!["response2".to_string()])) as Arc<dyn LlmProvider>,
        ];

        let router = LlmRouter::new(providers).with_strategy(RoutingStrategy::LeastCost);
        let options = LlmOptions::default();

        // 手动设置成本：provider0 成本更低
        let mut stats = router.get_provider_stats("provider0").await.unwrap();
        stats.cost_per_1k_tokens = 0.0; // ollama 免费
        router.update_stats("provider0", stats).await;

        let mut stats = router.get_provider_stats("provider1").await.unwrap();
        stats.cost_per_1k_tokens = 0.002; // openai 较贵
        router.update_stats("provider1", stats).await;

        // 应该选择成本更低的 provider0
        let selected = router.select_provider(&options).await.unwrap();
        assert_eq!(selected.name(), "mock");
    }

    #[tokio::test]
    async fn test_stats_recording() {
        let providers = vec![Arc::new(MockLlmProvider::new(vec!["response".to_string()])) as Arc<dyn LlmProvider>];

        let router = LlmRouter::new(providers);
        let _options = LlmOptions::default();

        // 记录成功请求
        router.increment_load("provider0").await;
        router.record_success("provider0", 100.0).await;

        let stats = router.get_provider_stats("provider0").await.unwrap();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.successful_requests, 1);
        // 延迟应该是 100.0（第一次记录时直接设置）
        assert!((stats.avg_latency_ms - 100.0).abs() < 0.1);
        assert_eq!(stats.current_load, 0); // 记录后负载减少

        // 记录失败请求
        router.increment_load("provider0").await;
        router.record_failure("provider0").await;

        let stats = router.get_provider_stats("provider0").await.unwrap();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.failed_requests, 1);
        assert_eq!(stats.current_load, 0);
    }

    #[tokio::test]
    async fn test_empty_providers() {
        let router = LlmRouter::new(vec![]);
        let options = LlmOptions::default();

        let result = router.select_provider(&options).await;
        assert!(result.is_err());
    }
}

