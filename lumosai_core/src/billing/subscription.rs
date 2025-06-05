//! 订阅管理系统
//! 
//! 提供完整的订阅生命周期管理、计划变更、试用期管理等功能

use super::*;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 订阅状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubscriptionStatus {
    /// 试用期
    Trial,
    /// 活跃
    Active,
    /// 已暂停
    Paused,
    /// 已取消
    Cancelled,
    /// 已过期
    Expired,
    /// 待付款
    PastDue,
    /// 不完整 (需要额外信息)
    Incomplete,
}

/// 订阅计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionPlan {
    /// 计划ID
    pub id: String,
    /// 计划名称
    pub name: String,
    /// 计划描述
    pub description: String,
    /// 定价模型
    pub pricing: PricingModel,
    /// 计费周期
    pub billing_cycle: BillingCycle,
    /// 资源限制
    pub resource_limits: HashMap<String, u64>,
    /// 功能特性
    pub features: Vec<String>,
    /// 是否可用
    pub is_active: bool,
    /// 试用期天数
    pub trial_days: Option<u32>,
    /// 设置费用
    pub setup_fee: Option<f64>,
    /// 创建时间
    pub created_at: SystemTime,
    /// 更新时间
    pub updated_at: SystemTime,
}

impl SubscriptionPlan {
    /// 创建新的订阅计划
    pub fn new(
        id: String,
        name: String,
        description: String,
        pricing: PricingModel,
        billing_cycle: BillingCycle,
    ) -> Self {
        let now = SystemTime::now();
        
        Self {
            id,
            name,
            description,
            pricing,
            billing_cycle,
            resource_limits: HashMap::new(),
            features: Vec::new(),
            is_active: true,
            trial_days: None,
            setup_fee: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// 添加资源限制
    pub fn with_resource_limit(mut self, resource_type: String, limit: u64) -> Self {
        self.resource_limits.insert(resource_type, limit);
        self
    }
    
    /// 添加功能特性
    pub fn with_feature(mut self, feature: String) -> Self {
        self.features.push(feature);
        self
    }
    
    /// 设置试用期
    pub fn with_trial(mut self, days: u32) -> Self {
        self.trial_days = Some(days);
        self
    }
    
    /// 设置设置费用
    pub fn with_setup_fee(mut self, fee: f64) -> Self {
        self.setup_fee = Some(fee);
        self
    }
    
    /// 获取基础价格
    pub fn get_base_price(&self) -> f64 {
        match &self.pricing {
            PricingModel::Fixed { amount, .. } => *amount,
            PricingModel::Tiered { tiers, .. } => {
                tiers.first().map(|t| t.fixed_fee.unwrap_or(0.0)).unwrap_or(0.0)
            },
            PricingModel::UsageBased { base_price, .. } => *base_price,
            PricingModel::Hybrid { base_price, .. } => *base_price,
        }
    }
}

/// 订阅
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    /// 订阅ID
    pub id: String,
    /// 租户ID
    pub tenant_id: Uuid,
    /// 订阅计划ID
    pub plan_id: String,
    /// 订阅状态
    pub status: SubscriptionStatus,
    /// 当前周期开始时间
    pub current_period_start: SystemTime,
    /// 当前周期结束时间
    pub current_period_end: SystemTime,
    /// 试用期结束时间
    pub trial_end: Option<SystemTime>,
    /// 取消时间
    pub cancelled_at: Option<SystemTime>,
    /// 取消生效时间
    pub cancel_at_period_end: bool,
    /// 创建时间
    pub created_at: SystemTime,
    /// 更新时间
    pub updated_at: SystemTime,
    /// 订阅元数据
    pub metadata: HashMap<String, String>,
    /// 折扣信息
    pub discount: Option<SubscriptionDiscount>,
    /// 附加组件
    pub addons: Vec<SubscriptionAddon>,
}

impl Subscription {
    /// 创建新订阅
    pub fn new(tenant_id: Uuid, plan_id: String, plan: &SubscriptionPlan) -> Self {
        let now = SystemTime::now();
        let period_end = plan.billing_cycle.next_billing_date(now);
        
        // 计算试用期结束时间
        let trial_end = plan.trial_days.map(|days| {
            now + std::time::Duration::from_secs(days as u64 * 24 * 60 * 60)
        });
        
        // 如果有试用期，状态为Trial，否则为Active
        let status = if trial_end.is_some() {
            SubscriptionStatus::Trial
        } else {
            SubscriptionStatus::Active
        };
        
        Self {
            id: Uuid::new_v4().to_string(),
            tenant_id,
            plan_id,
            status,
            current_period_start: now,
            current_period_end: period_end,
            trial_end,
            cancelled_at: None,
            cancel_at_period_end: false,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
            discount: None,
            addons: Vec::new(),
        }
    }
    
    /// 是否在试用期
    pub fn is_in_trial(&self) -> bool {
        if let Some(trial_end) = self.trial_end {
            SystemTime::now() < trial_end && self.status == SubscriptionStatus::Trial
        } else {
            false
        }
    }
    
    /// 是否已过期
    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.current_period_end && 
        !matches!(self.status, SubscriptionStatus::Active | SubscriptionStatus::Trial)
    }
    
    /// 取消订阅
    pub fn cancel(&mut self, at_period_end: bool) {
        let now = SystemTime::now();
        
        if at_period_end {
            self.cancel_at_period_end = true;
            self.cancelled_at = Some(now);
        } else {
            self.status = SubscriptionStatus::Cancelled;
            self.cancelled_at = Some(now);
            self.current_period_end = now;
        }
        
        self.updated_at = now;
    }
    
    /// 续费订阅
    pub fn renew(&mut self, billing_cycle: &BillingCycle) {
        let now = SystemTime::now();
        
        self.current_period_start = self.current_period_end;
        self.current_period_end = billing_cycle.next_billing_date(self.current_period_start);
        self.status = SubscriptionStatus::Active;
        self.updated_at = now;
        
        // 如果试用期已结束，清除试用期信息
        if let Some(trial_end) = self.trial_end {
            if now > trial_end {
                self.trial_end = None;
            }
        }
    }
    
    /// 暂停订阅
    pub fn pause(&mut self) {
        self.status = SubscriptionStatus::Paused;
        self.updated_at = SystemTime::now();
    }
    
    /// 恢复订阅
    pub fn resume(&mut self) {
        self.status = SubscriptionStatus::Active;
        self.updated_at = SystemTime::now();
    }
    
    /// 应用折扣
    pub fn apply_discount(&mut self, discount: SubscriptionDiscount) {
        self.discount = Some(discount);
        self.updated_at = SystemTime::now();
    }
    
    /// 添加附加组件
    pub fn add_addon(&mut self, addon: SubscriptionAddon) {
        self.addons.push(addon);
        self.updated_at = SystemTime::now();
    }
    
    /// 移除附加组件
    pub fn remove_addon(&mut self, addon_id: &str) {
        self.addons.retain(|addon| addon.id != addon_id);
        self.updated_at = SystemTime::now();
    }
}

/// 订阅折扣
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionDiscount {
    /// 折扣ID
    pub id: String,
    /// 折扣类型
    pub discount_type: DiscountType,
    /// 折扣值
    pub value: f64,
    /// 开始时间
    pub start_date: SystemTime,
    /// 结束时间
    pub end_date: Option<SystemTime>,
    /// 是否只适用于首次计费
    pub first_time_only: bool,
}

/// 折扣类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscountType {
    /// 百分比折扣
    Percentage,
    /// 固定金额折扣
    FixedAmount,
    /// 免费试用延期
    TrialExtension { days: u32 },
}

/// 订阅附加组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionAddon {
    /// 附加组件ID
    pub id: String,
    /// 附加组件名称
    pub name: String,
    /// 附加组件描述
    pub description: String,
    /// 定价
    pub pricing: PricingModel,
    /// 数量
    pub quantity: u32,
    /// 添加时间
    pub added_at: SystemTime,
}

/// 订阅管理器
#[derive(Debug)]
pub struct SubscriptionManager {
    /// 订阅计划
    plans: HashMap<String, SubscriptionPlan>,
    /// 活跃订阅
    subscriptions: HashMap<String, Subscription>,
    /// 租户订阅映射
    tenant_subscriptions: HashMap<Uuid, String>,
}

impl SubscriptionManager {
    /// 创建新的订阅管理器
    pub fn new() -> Self {
        Self {
            plans: HashMap::new(),
            subscriptions: HashMap::new(),
            tenant_subscriptions: HashMap::new(),
        }
    }
    
    /// 添加订阅计划
    pub fn add_plan(&mut self, plan: SubscriptionPlan) {
        self.plans.insert(plan.id.clone(), plan);
    }
    
    /// 获取订阅计划
    pub fn get_plan(&self, plan_id: &str) -> Option<&SubscriptionPlan> {
        self.plans.get(plan_id)
    }
    
    /// 列出所有可用计划
    pub fn list_active_plans(&self) -> Vec<&SubscriptionPlan> {
        self.plans.values().filter(|plan| plan.is_active).collect()
    }
    
    /// 创建订阅
    pub fn create_subscription(&mut self, tenant_id: Uuid, plan_id: String) -> BillingResult<Subscription> {
        let plan = self.plans.get(&plan_id)
            .ok_or_else(|| BillingError::InvalidConfiguration(format!("Plan not found: {}", plan_id)))?;
        
        if !plan.is_active {
            return Err(BillingError::InvalidConfiguration("Plan is not active".to_string()));
        }
        
        let subscription = Subscription::new(tenant_id, plan_id, plan);
        let subscription_id = subscription.id.clone();
        
        // 存储订阅
        self.subscriptions.insert(subscription_id.clone(), subscription.clone());
        self.tenant_subscriptions.insert(tenant_id, subscription_id);
        
        Ok(subscription)
    }
    
    /// 获取租户的订阅
    pub fn get_tenant_subscription(&self, tenant_id: &Uuid) -> Option<&Subscription> {
        if let Some(subscription_id) = self.tenant_subscriptions.get(tenant_id) {
            self.subscriptions.get(subscription_id)
        } else {
            None
        }
    }
    
    /// 更新订阅
    pub fn update_subscription(&mut self, subscription_id: &str, subscription: Subscription) -> BillingResult<()> {
        if self.subscriptions.contains_key(subscription_id) {
            self.subscriptions.insert(subscription_id.to_string(), subscription);
            Ok(())
        } else {
            Err(BillingError::SubscriptionNotFound(subscription_id.to_string()))
        }
    }
    
    /// 取消订阅
    pub fn cancel_subscription(&mut self, subscription_id: &str, at_period_end: bool) -> BillingResult<()> {
        if let Some(subscription) = self.subscriptions.get_mut(subscription_id) {
            subscription.cancel(at_period_end);
            Ok(())
        } else {
            Err(BillingError::SubscriptionNotFound(subscription_id.to_string()))
        }
    }
    
    /// 续费订阅
    pub fn renew_subscription(&mut self, subscription_id: &str) -> BillingResult<()> {
        if let Some(subscription) = self.subscriptions.get_mut(subscription_id) {
            let plan = self.plans.get(&subscription.plan_id)
                .ok_or_else(|| BillingError::InvalidConfiguration("Plan not found".to_string()))?;
            
            subscription.renew(&plan.billing_cycle);
            Ok(())
        } else {
            Err(BillingError::SubscriptionNotFound(subscription_id.to_string()))
        }
    }
    
    /// 检查需要续费的订阅
    pub fn get_subscriptions_due_for_renewal(&self) -> Vec<&Subscription> {
        let now = SystemTime::now();
        
        self.subscriptions.values()
            .filter(|sub| {
                matches!(sub.status, SubscriptionStatus::Active | SubscriptionStatus::Trial) &&
                now >= sub.current_period_end
            })
            .collect()
    }
    
    /// 检查过期的试用期
    pub fn get_expired_trials(&self) -> Vec<&Subscription> {
        let now = SystemTime::now();
        
        self.subscriptions.values()
            .filter(|sub| {
                sub.status == SubscriptionStatus::Trial &&
                sub.trial_end.map_or(false, |end| now > end)
            })
            .collect()
    }
}

impl Default for SubscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}
