//! 使用量跟踪系统
//! 
//! 提供实时使用量监控、聚合统计、限制检查和预警功能

use super::*;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 使用量记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    /// 记录ID
    pub id: String,
    /// 租户ID
    pub tenant_id: Uuid,
    /// 资源类型
    pub resource_type: String,
    /// 使用量
    pub quantity: u64,
    /// 单位
    pub unit: String,
    /// 记录时间
    pub timestamp: SystemTime,
    /// 元数据
    pub metadata: HashMap<String, String>,
    /// 成本 (如果已计算)
    pub cost: Option<f64>,
}

impl UsageRecord {
    /// 创建新的使用量记录
    pub fn new(
        tenant_id: Uuid,
        resource_type: String,
        quantity: u64,
        unit: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            tenant_id,
            resource_type,
            quantity,
            unit,
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
            cost: None,
        }
    }
    
    /// 添加元数据
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// 设置成本
    pub fn with_cost(mut self, cost: f64) -> Self {
        self.cost = Some(cost);
        self
    }
}

/// 使用量统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    /// 租户ID
    pub tenant_id: Uuid,
    /// 统计周期
    pub period: (SystemTime, SystemTime),
    /// 按资源类型分组的使用量
    pub usage_by_resource: HashMap<String, ResourceUsageStats>,
    /// 总成本
    pub total_cost: f64,
    /// 货币
    pub currency: String,
    /// 生成时间
    pub generated_at: SystemTime,
}

/// 资源使用量统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageStats {
    /// 资源类型
    pub resource_type: String,
    /// 总使用量
    pub total_usage: u64,
    /// 平均使用量
    pub average_usage: f64,
    /// 峰值使用量
    pub peak_usage: u64,
    /// 使用次数
    pub usage_count: u64,
    /// 总成本
    pub total_cost: f64,
    /// 平均成本
    pub average_cost: f64,
    /// 单位
    pub unit: String,
    /// 每日使用量分布
    pub daily_usage: Vec<DailyUsage>,
}

/// 每日使用量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsage {
    /// 日期
    pub date: SystemTime,
    /// 使用量
    pub usage: u64,
    /// 成本
    pub cost: f64,
}

/// 使用量限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageLimit {
    /// 限制ID
    pub id: String,
    /// 租户ID
    pub tenant_id: Uuid,
    /// 资源类型
    pub resource_type: String,
    /// 限制类型
    pub limit_type: UsageLimitType,
    /// 限制值
    pub limit_value: u64,
    /// 当前使用量
    pub current_usage: u64,
    /// 重置周期
    pub reset_period: Duration,
    /// 上次重置时间
    pub last_reset: SystemTime,
    /// 是否启用
    pub enabled: bool,
    /// 警告阈值 (百分比)
    pub warning_threshold: f64,
    /// 创建时间
    pub created_at: SystemTime,
}

/// 使用量限制类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsageLimitType {
    /// 硬限制 (超过后拒绝服务)
    Hard,
    /// 软限制 (超过后发送警告)
    Soft,
    /// 计费限制 (超过后额外收费)
    Billing,
}

impl UsageLimit {
    /// 创建新的使用量限制
    pub fn new(
        tenant_id: Uuid,
        resource_type: String,
        limit_type: UsageLimitType,
        limit_value: u64,
        reset_period: Duration,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            tenant_id,
            resource_type,
            limit_type,
            limit_value,
            current_usage: 0,
            reset_period,
            last_reset: SystemTime::now(),
            enabled: true,
            warning_threshold: 0.8, // 80%
            created_at: SystemTime::now(),
        }
    }
    
    /// 检查是否需要重置
    pub fn should_reset(&self) -> bool {
        SystemTime::now().duration_since(self.last_reset).unwrap_or(Duration::ZERO) >= self.reset_period
    }
    
    /// 重置使用量
    pub fn reset(&mut self) {
        self.current_usage = 0;
        self.last_reset = SystemTime::now();
    }
    
    /// 检查是否超过限制
    pub fn is_exceeded(&self) -> bool {
        self.current_usage >= self.limit_value
    }
    
    /// 检查是否达到警告阈值
    pub fn is_warning_threshold_reached(&self) -> bool {
        self.current_usage as f64 >= self.limit_value as f64 * self.warning_threshold
    }
    
    /// 获取使用率
    pub fn usage_percentage(&self) -> f64 {
        if self.limit_value == 0 {
            0.0
        } else {
            (self.current_usage as f64 / self.limit_value as f64) * 100.0
        }
    }
}

/// 使用量跟踪器
#[derive(Debug)]
pub struct UsageTracker {
    /// 使用量记录
    usage_records: VecDeque<UsageRecord>,
    /// 使用量限制
    usage_limits: HashMap<String, UsageLimit>, // limit_id -> limit
    /// 租户限制映射
    tenant_limits: HashMap<Uuid, Vec<String>>, // tenant_id -> limit_ids
    /// 最大记录数
    max_records: usize,
    /// 聚合缓存
    stats_cache: HashMap<String, (UsageStats, SystemTime)>, // cache_key -> (stats, cached_at)
    /// 缓存过期时间
    cache_ttl: Duration,
}

impl UsageTracker {
    /// 创建新的使用量跟踪器
    pub fn new() -> Self {
        Self {
            usage_records: VecDeque::new(),
            usage_limits: HashMap::new(),
            tenant_limits: HashMap::new(),
            max_records: 100000, // 最多保存10万条记录
            stats_cache: HashMap::new(),
            cache_ttl: Duration::from_secs(300), // 5分钟缓存
        }
    }
    
    /// 记录使用量
    pub fn record_usage(&mut self, record: UsageRecord) -> BillingResult<()> {
        let tenant_id = record.tenant_id;
        let resource_type = record.resource_type.clone();
        let quantity = record.quantity;
        
        // 添加记录
        self.usage_records.push_back(record);
        
        // 限制记录数量
        while self.usage_records.len() > self.max_records {
            self.usage_records.pop_front();
        }
        
        // 更新使用量限制
        self.update_usage_limits(tenant_id, &resource_type, quantity)?;
        
        // 清除相关缓存
        self.invalidate_cache(&tenant_id);
        
        Ok(())
    }
    
    /// 更新使用量限制
    fn update_usage_limits(&mut self, tenant_id: Uuid, resource_type: &str, quantity: u64) -> BillingResult<()> {
        if let Some(limit_ids) = self.tenant_limits.get(&tenant_id) {
            for limit_id in limit_ids {
                if let Some(limit) = self.usage_limits.get_mut(limit_id) {
                    if limit.resource_type == resource_type {
                        // 检查是否需要重置
                        if limit.should_reset() {
                            limit.reset();
                        }
                        
                        // 更新使用量
                        limit.current_usage += quantity;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// 添加使用量限制
    pub fn add_usage_limit(&mut self, limit: UsageLimit) {
        let tenant_id = limit.tenant_id;
        let limit_id = limit.id.clone();
        
        // 存储限制
        self.usage_limits.insert(limit_id.clone(), limit);
        
        // 更新租户限制映射
        self.tenant_limits.entry(tenant_id).or_insert_with(Vec::new).push(limit_id);
    }
    
    /// 检查使用量限制
    pub fn check_usage_limit(&mut self, tenant_id: &Uuid, resource_type: &str) -> BillingResult<UsageLimitCheck> {
        let mut result = UsageLimitCheck {
            allowed: true,
            exceeded_limits: Vec::new(),
            warning_limits: Vec::new(),
        };
        
        if let Some(limit_ids) = self.tenant_limits.get(tenant_id) {
            for limit_id in limit_ids {
                if let Some(limit) = self.usage_limits.get_mut(limit_id) {
                    if limit.resource_type == resource_type && limit.enabled {
                        // 检查是否需要重置
                        if limit.should_reset() {
                            limit.reset();
                        }
                        
                        // 检查限制
                        if limit.is_exceeded() {
                            result.exceeded_limits.push(limit.clone());
                            if matches!(limit.limit_type, UsageLimitType::Hard) {
                                result.allowed = false;
                            }
                        } else if limit.is_warning_threshold_reached() {
                            result.warning_limits.push(limit.clone());
                        }
                    }
                }
            }
        }
        
        Ok(result)
    }
    
    /// 获取使用量统计
    pub fn get_usage_stats(&mut self, tenant_id: &Uuid, period: (SystemTime, SystemTime)) -> BillingResult<UsageStats> {
        let cache_key = format!("{}-{}-{}", 
            tenant_id, 
            period.0.duration_since(UNIX_EPOCH).unwrap().as_secs(),
            period.1.duration_since(UNIX_EPOCH).unwrap().as_secs()
        );
        
        // 检查缓存
        if let Some((stats, cached_at)) = self.stats_cache.get(&cache_key) {
            if SystemTime::now().duration_since(*cached_at).unwrap_or(Duration::MAX) < self.cache_ttl {
                return Ok(stats.clone());
            }
        }
        
        // 计算统计
        let stats = self.calculate_usage_stats(tenant_id, period)?;
        
        // 缓存结果
        self.stats_cache.insert(cache_key, (stats.clone(), SystemTime::now()));
        
        Ok(stats)
    }
    
    /// 计算使用量统计
    fn calculate_usage_stats(&self, tenant_id: &Uuid, period: (SystemTime, SystemTime)) -> BillingResult<UsageStats> {
        let mut usage_by_resource: HashMap<String, Vec<&UsageRecord>> = HashMap::new();
        let mut total_cost = 0.0;
        
        // 过滤相关记录
        for record in &self.usage_records {
            if record.tenant_id == *tenant_id && 
               record.timestamp >= period.0 && 
               record.timestamp <= period.1 {
                
                usage_by_resource.entry(record.resource_type.clone())
                    .or_insert_with(Vec::new)
                    .push(record);
                
                if let Some(cost) = record.cost {
                    total_cost += cost;
                }
            }
        }
        
        // 计算每个资源的统计
        let mut resource_stats = HashMap::new();
        
        for (resource_type, records) in usage_by_resource {
            let total_usage: u64 = records.iter().map(|r| r.quantity).sum();
            let usage_count = records.len() as u64;
            let average_usage = if usage_count > 0 { total_usage as f64 / usage_count as f64 } else { 0.0 };
            let peak_usage = records.iter().map(|r| r.quantity).max().unwrap_or(0);
            
            let resource_cost: f64 = records.iter()
                .filter_map(|r| r.cost)
                .sum();
            let average_cost = if usage_count > 0 { resource_cost / usage_count as f64 } else { 0.0 };
            
            // 计算每日使用量
            let daily_usage = self.calculate_daily_usage(&records, period);
            
            let unit = records.first().map(|r| r.unit.clone()).unwrap_or_else(|| "units".to_string());
            
            resource_stats.insert(resource_type.clone(), ResourceUsageStats {
                resource_type,
                total_usage,
                average_usage,
                peak_usage,
                usage_count,
                total_cost: resource_cost,
                average_cost,
                unit,
                daily_usage,
            });
        }
        
        Ok(UsageStats {
            tenant_id: *tenant_id,
            period,
            usage_by_resource: resource_stats,
            total_cost,
            currency: "USD".to_string(),
            generated_at: SystemTime::now(),
        })
    }
    
    /// 计算每日使用量
    fn calculate_daily_usage(&self, records: &[&UsageRecord], period: (SystemTime, SystemTime)) -> Vec<DailyUsage> {
        let mut daily_map: HashMap<u64, (u64, f64)> = HashMap::new(); // day -> (usage, cost)
        
        for record in records {
            let days_since_epoch = record.timestamp.duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::ZERO).as_secs() / (24 * 60 * 60);
            
            let entry = daily_map.entry(days_since_epoch).or_insert((0, 0.0));
            entry.0 += record.quantity;
            entry.1 += record.cost.unwrap_or(0.0);
        }
        
        let mut daily_usage: Vec<DailyUsage> = daily_map.into_iter()
            .map(|(day, (usage, cost))| DailyUsage {
                date: UNIX_EPOCH + Duration::from_secs(day * 24 * 60 * 60),
                usage,
                cost,
            })
            .collect();
        
        daily_usage.sort_by_key(|d| d.date);
        daily_usage
    }
    
    /// 清除缓存
    fn invalidate_cache(&mut self, tenant_id: &Uuid) {
        let tenant_str = tenant_id.to_string();
        self.stats_cache.retain(|key, _| !key.starts_with(&tenant_str));
    }
    
    /// 获取租户的使用量限制
    pub fn get_tenant_limits(&self, tenant_id: &Uuid) -> Vec<&UsageLimit> {
        if let Some(limit_ids) = self.tenant_limits.get(tenant_id) {
            limit_ids.iter()
                .filter_map(|id| self.usage_limits.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// 清理过期记录
    pub fn cleanup_old_records(&mut self, retention_period: Duration) {
        let cutoff_time = SystemTime::now() - retention_period;
        
        while let Some(record) = self.usage_records.front() {
            if record.timestamp < cutoff_time {
                self.usage_records.pop_front();
            } else {
                break;
            }
        }
    }
}

/// 使用量限制检查结果
#[derive(Debug, Clone)]
pub struct UsageLimitCheck {
    /// 是否允许操作
    pub allowed: bool,
    /// 超过的限制
    pub exceeded_limits: Vec<UsageLimit>,
    /// 达到警告阈值的限制
    pub warning_limits: Vec<UsageLimit>,
}

impl Default for UsageTracker {
    fn default() -> Self {
        Self::new()
    }
}
