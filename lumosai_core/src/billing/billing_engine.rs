//! 计费引擎
//! 
//! 提供智能计费计算、发票生成、定价策略和成本优化功能

use super::*;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 计费引擎
#[derive(Debug)]
pub struct BillingEngine {
    /// 定价规则
    pricing_rules: HashMap<String, PricingRule>,
    /// 折扣规则
    discount_rules: HashMap<String, DiscountRule>,
    /// 税率配置
    tax_config: TaxConfiguration,
    /// 计费历史
    billing_history: HashMap<Uuid, Vec<Invoice>>,
}

impl BillingEngine {
    /// 创建新的计费引擎
    pub fn new() -> Self {
        Self {
            pricing_rules: HashMap::new(),
            discount_rules: HashMap::new(),
            tax_config: TaxConfiguration::default(),
            billing_history: HashMap::new(),
        }
    }
    
    /// 添加定价规则
    pub fn add_pricing_rule(&mut self, rule: PricingRule) {
        self.pricing_rules.insert(rule.resource_type.clone(), rule);
    }
    
    /// 添加折扣规则
    pub fn add_discount_rule(&mut self, rule: DiscountRule) {
        self.discount_rules.insert(rule.id.clone(), rule);
    }
    
    /// 设置税率配置
    pub fn set_tax_config(&mut self, config: TaxConfiguration) {
        self.tax_config = config;
    }
    
    /// 计算使用量成本
    pub fn calculate_usage_cost(
        &self,
        tenant_id: &Uuid,
        usage_stats: &UsageStats,
        subscription: &Subscription,
    ) -> BillingResult<Vec<BillingItem>> {
        let mut billing_items = Vec::new();
        
        for (resource_type, resource_stats) in &usage_stats.usage_by_resource {
            // 获取定价规则
            let pricing_rule = self.pricing_rules.get(resource_type)
                .ok_or_else(|| BillingError::InvalidConfiguration(
                    format!("No pricing rule found for resource: {}", resource_type)
                ))?;
            
            // 计算基础成本
            let base_cost = self.calculate_resource_cost(resource_stats, pricing_rule)?;
            
            // 应用折扣
            let discounted_cost = self.apply_discounts(tenant_id, resource_type, base_cost, subscription)?;
            
            // 创建计费项目
            let billing_item = BillingItem {
                id: Uuid::new_v4().to_string(),
                name: format!("{} Usage", resource_type),
                description: format!("Usage of {} for the billing period", resource_type),
                resource_type: resource_type.clone(),
                quantity: resource_stats.total_usage,
                unit_price: if resource_stats.total_usage > 0 {
                    discounted_cost / resource_stats.total_usage as f64
                } else {
                    0.0
                },
                total_amount: discounted_cost,
                billing_period: usage_stats.period,
                currency: usage_stats.currency.clone(),
                metadata: HashMap::new(),
            };
            
            billing_items.push(billing_item);
        }
        
        Ok(billing_items)
    }
    
    /// 计算资源成本
    fn calculate_resource_cost(
        &self,
        resource_stats: &ResourceUsageStats,
        pricing_rule: &PricingRule,
    ) -> BillingResult<f64> {
        match &pricing_rule.pricing_model {
            PricingModel::Fixed { amount, .. } => Ok(*amount),
            
            PricingModel::Tiered { tiers, .. } => {
                let mut total_cost = 0.0;
                let mut remaining_usage = resource_stats.total_usage;
                
                for tier in tiers {
                    if remaining_usage == 0 {
                        break;
                    }
                    
                    let tier_usage = if let Some(max_usage) = tier.max_usage {
                        remaining_usage.min(max_usage - tier.min_usage)
                    } else {
                        remaining_usage
                    };
                    
                    total_cost += tier_usage as f64 * tier.rate;
                    
                    if let Some(fixed_fee) = tier.fixed_fee {
                        total_cost += fixed_fee;
                    }
                    
                    remaining_usage = remaining_usage.saturating_sub(tier_usage);
                }
                
                Ok(total_cost)
            },
            
            PricingModel::UsageBased { base_price, usage_rates, .. } => {
                let mut total_cost = *base_price;
                
                if let Some(rate) = usage_rates.get(&resource_stats.resource_type) {
                    total_cost += resource_stats.total_usage as f64 * rate;
                }
                
                Ok(total_cost)
            },
            
            PricingModel::Hybrid { base_price, usage_rates, included_usage, .. } => {
                let mut total_cost = *base_price;
                
                if let Some(rate) = usage_rates.get(&resource_stats.resource_type) {
                    let included = included_usage.get(&resource_stats.resource_type).unwrap_or(&0);
                    let billable_usage = resource_stats.total_usage.saturating_sub(*included);
                    total_cost += billable_usage as f64 * rate;
                }
                
                Ok(total_cost)
            },
        }
    }
    
    /// 应用折扣
    fn apply_discounts(
        &self,
        tenant_id: &Uuid,
        resource_type: &str,
        base_cost: f64,
        subscription: &Subscription,
    ) -> BillingResult<f64> {
        let mut final_cost = base_cost;
        
        // 应用订阅折扣
        if let Some(subscription_discount) = &subscription.discount {
            final_cost = self.apply_subscription_discount(final_cost, subscription_discount)?;
        }
        
        // 应用全局折扣规则
        for discount_rule in self.discount_rules.values() {
            if discount_rule.applies_to_tenant(tenant_id) && 
               discount_rule.applies_to_resource(resource_type) &&
               discount_rule.is_active() {
                final_cost = discount_rule.apply_discount(final_cost)?;
            }
        }
        
        Ok(final_cost.max(0.0))
    }
    
    /// 应用订阅折扣
    fn apply_subscription_discount(
        &self,
        cost: f64,
        discount: &SubscriptionDiscount,
    ) -> BillingResult<f64> {
        let now = SystemTime::now();
        
        // 检查折扣是否有效
        if now < discount.start_date {
            return Ok(cost);
        }
        
        if let Some(end_date) = discount.end_date {
            if now > end_date {
                return Ok(cost);
            }
        }
        
        match discount.discount_type {
            DiscountType::Percentage => {
                Ok(cost * (1.0 - discount.value / 100.0))
            },
            DiscountType::FixedAmount => {
                Ok((cost - discount.value).max(0.0))
            },
            DiscountType::TrialExtension { .. } => {
                // 试用期延期不影响成本计算
                Ok(cost)
            },
        }
    }
    
    /// 生成发票
    pub fn generate_invoice(
        &mut self,
        tenant_id: &Uuid,
        billing_items: Vec<BillingItem>,
        billing_period: (SystemTime, SystemTime),
        due_days: u32,
    ) -> BillingResult<Invoice> {
        let mut invoice = Invoice::new(*tenant_id, billing_items, billing_period, due_days);
        
        // 计算税费
        let tax_amount = self.calculate_tax(&invoice)?;
        invoice.tax_amount = tax_amount;
        invoice.total_amount = invoice.subtotal + tax_amount - invoice.discount_amount;
        
        // 存储到历史记录
        self.billing_history.entry(*tenant_id).or_insert_with(Vec::new).push(invoice.clone());
        
        Ok(invoice)
    }
    
    /// 计算税费
    fn calculate_tax(&self, invoice: &Invoice) -> BillingResult<f64> {
        let tax_rate = self.tax_config.get_tax_rate(&invoice.tenant_id)?;
        Ok(invoice.subtotal * tax_rate)
    }
    
    /// 获取计费历史
    pub fn get_billing_history(&self, tenant_id: &Uuid) -> Vec<&Invoice> {
        self.billing_history.get(tenant_id).map(|invoices| invoices.iter().collect()).unwrap_or_default()
    }
    
    /// 计算预估成本
    pub fn estimate_cost(
        &self,
        tenant_id: &Uuid,
        resource_usage: &HashMap<String, u64>,
        subscription: &Subscription,
    ) -> BillingResult<CostEstimate> {
        let mut estimated_items = Vec::new();
        let mut total_cost = 0.0;
        
        for (resource_type, usage) in resource_usage {
            if let Some(pricing_rule) = self.pricing_rules.get(resource_type) {
                // 创建模拟的资源统计
                let resource_stats = ResourceUsageStats {
                    resource_type: resource_type.clone(),
                    total_usage: *usage,
                    average_usage: *usage as f64,
                    peak_usage: *usage,
                    usage_count: 1,
                    total_cost: 0.0,
                    average_cost: 0.0,
                    unit: "units".to_string(),
                    daily_usage: Vec::new(),
                };
                
                // 计算成本
                let base_cost = self.calculate_resource_cost(&resource_stats, pricing_rule)?;
                let final_cost = self.apply_discounts(tenant_id, resource_type, base_cost, subscription)?;
                
                estimated_items.push(CostEstimateItem {
                    resource_type: resource_type.clone(),
                    usage: *usage,
                    unit_price: if *usage > 0 { final_cost / *usage as f64 } else { 0.0 },
                    total_cost: final_cost,
                });
                
                total_cost += final_cost;
            }
        }
        
        // 计算税费
        let tax_amount = total_cost * self.tax_config.get_tax_rate(tenant_id)?;
        
        Ok(CostEstimate {
            tenant_id: *tenant_id,
            items: estimated_items,
            subtotal: total_cost,
            tax_amount,
            total_amount: total_cost + tax_amount,
            currency: "USD".to_string(),
            estimated_at: SystemTime::now(),
        })
    }
}

/// 定价规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingRule {
    /// 规则ID
    pub id: String,
    /// 规则名称
    pub name: String,
    /// 适用的资源类型
    pub resource_type: String,
    /// 定价模型
    pub pricing_model: PricingModel,
    /// 是否启用
    pub enabled: bool,
    /// 生效时间
    pub effective_from: SystemTime,
    /// 失效时间
    pub effective_until: Option<SystemTime>,
}

/// 折扣规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscountRule {
    /// 规则ID
    pub id: String,
    /// 规则名称
    pub name: String,
    /// 折扣类型
    pub discount_type: DiscountType,
    /// 折扣值
    pub value: f64,
    /// 适用条件
    pub conditions: DiscountConditions,
    /// 生效时间
    pub effective_from: SystemTime,
    /// 失效时间
    pub effective_until: Option<SystemTime>,
    /// 是否启用
    pub enabled: bool,
}

impl DiscountRule {
    /// 检查是否适用于租户
    pub fn applies_to_tenant(&self, tenant_id: &Uuid) -> bool {
        self.conditions.tenant_ids.is_empty() || self.conditions.tenant_ids.contains(tenant_id)
    }
    
    /// 检查是否适用于资源
    pub fn applies_to_resource(&self, resource_type: &str) -> bool {
        self.conditions.resource_types.is_empty() || self.conditions.resource_types.contains(&resource_type.to_string())
    }
    
    /// 检查是否处于活跃状态
    pub fn is_active(&self) -> bool {
        if !self.enabled {
            return false;
        }
        
        let now = SystemTime::now();
        
        if now < self.effective_from {
            return false;
        }
        
        if let Some(effective_until) = self.effective_until {
            if now > effective_until {
                return false;
            }
        }
        
        true
    }
    
    /// 应用折扣
    pub fn apply_discount(&self, cost: f64) -> BillingResult<f64> {
        match self.discount_type {
            DiscountType::Percentage => {
                Ok(cost * (1.0 - self.value / 100.0))
            },
            DiscountType::FixedAmount => {
                Ok((cost - self.value).max(0.0))
            },
            DiscountType::TrialExtension { .. } => {
                // 试用期延期不影响成本
                Ok(cost)
            },
        }
    }
}

/// 折扣条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscountConditions {
    /// 适用的租户ID列表 (空表示所有租户)
    pub tenant_ids: Vec<Uuid>,
    /// 适用的资源类型列表 (空表示所有资源)
    pub resource_types: Vec<String>,
    /// 最小使用量要求
    pub min_usage: Option<u64>,
    /// 最大使用量限制
    pub max_usage: Option<u64>,
}

/// 税率配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxConfiguration {
    /// 默认税率
    pub default_rate: f64,
    /// 按地区的税率
    pub regional_rates: HashMap<String, f64>,
    /// 按租户的税率
    pub tenant_rates: HashMap<Uuid, f64>,
}

impl TaxConfiguration {
    /// 获取租户的税率
    pub fn get_tax_rate(&self, tenant_id: &Uuid) -> BillingResult<f64> {
        // 优先使用租户特定税率
        if let Some(rate) = self.tenant_rates.get(tenant_id) {
            return Ok(*rate);
        }
        
        // 使用默认税率
        Ok(self.default_rate)
    }
}

impl Default for TaxConfiguration {
    fn default() -> Self {
        Self {
            default_rate: 0.1, // 10%
            regional_rates: HashMap::new(),
            tenant_rates: HashMap::new(),
        }
    }
}

/// 成本估算
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    /// 租户ID
    pub tenant_id: Uuid,
    /// 估算项目
    pub items: Vec<CostEstimateItem>,
    /// 小计
    pub subtotal: f64,
    /// 税费
    pub tax_amount: f64,
    /// 总金额
    pub total_amount: f64,
    /// 货币
    pub currency: String,
    /// 估算时间
    pub estimated_at: SystemTime,
}

/// 成本估算项目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimateItem {
    /// 资源类型
    pub resource_type: String,
    /// 使用量
    pub usage: u64,
    /// 单价
    pub unit_price: f64,
    /// 总成本
    pub total_cost: f64,
}

impl Default for BillingEngine {
    fn default() -> Self {
        Self::new()
    }
}
