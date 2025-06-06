//! 多租户架构实现

use async_trait::async_trait;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::error::{EnterpriseError, Result};

/// 多租户架构
pub struct MultiTenantArchitecture {
    tenant_manager: TenantManager,
    isolation_engine: IsolationEngine,
    resource_allocator: ResourceAllocator,
    billing_manager: BillingManager,
    quota_manager: QuotaManager,
    auto_scaler: AutoScaler,
}

/// 租户管理器
pub struct TenantManager {
    tenants: HashMap<String, Tenant>,
    tenant_configs: HashMap<String, TenantConfiguration>,
}

/// 租户
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    /// 租户ID
    pub id: String,
    
    /// 租户名称
    pub name: String,
    
    /// 租户类型
    pub tenant_type: TenantType,
    
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// 状态
    pub status: TenantStatus,
    
    /// 联系信息
    pub contact_info: ContactInfo,
    
    /// 订阅计划
    pub subscription_plan: SubscriptionPlan,
    
    /// 配额
    pub quotas: ResourceQuotas,
    
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// 租户类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TenantType {
    /// 个人
    Individual,
    /// 小企业
    SmallBusiness,
    /// 企业
    Enterprise,
    /// 政府
    Government,
    /// 教育机构
    Educational,
}

/// 租户状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TenantStatus {
    /// 活跃
    Active,
    /// 暂停
    Suspended,
    /// 试用
    Trial,
    /// 已删除
    Deleted,
    /// 待激活
    PendingActivation,
}

/// 联系信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    /// 主要联系人
    pub primary_contact: String,
    
    /// 邮箱
    pub email: String,
    
    /// 电话
    pub phone: Option<String>,
    
    /// 地址
    pub address: Option<Address>,
    
    /// 公司信息
    pub company_info: Option<CompanyInfo>,
}

/// 地址
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    /// 街道
    pub street: String,
    
    /// 城市
    pub city: String,
    
    /// 州/省
    pub state: String,
    
    /// 邮编
    pub postal_code: String,
    
    /// 国家
    pub country: String,
}

/// 公司信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyInfo {
    /// 公司名称
    pub name: String,
    
    /// 行业
    pub industry: String,
    
    /// 员工数量
    pub employee_count: Option<u32>,
    
    /// 年收入
    pub annual_revenue: Option<u64>,
}

/// 订阅计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionPlan {
    /// 计划ID
    pub plan_id: String,
    
    /// 计划名称
    pub plan_name: String,
    
    /// 计划类型
    pub plan_type: PlanType,
    
    /// 价格
    pub price: PlanPricing,
    
    /// 功能
    pub features: Vec<String>,
    
    /// 开始时间
    pub start_date: DateTime<Utc>,
    
    /// 结束时间
    pub end_date: Option<DateTime<Utc>>,
    
    /// 自动续费
    pub auto_renewal: bool,
}

/// 计划类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanType {
    /// 免费
    Free,
    /// 基础
    Basic,
    /// 专业
    Professional,
    /// 企业
    Enterprise,
    /// 自定义
    Custom,
}

/// 计划定价
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanPricing {
    /// 基础价格
    pub base_price: f64,
    
    /// 计费周期
    pub billing_cycle: BillingCycle,
    
    /// 货币
    pub currency: String,
    
    /// 使用量定价
    pub usage_pricing: Vec<UsagePricing>,
}

/// 计费周期
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BillingCycle {
    /// 月付
    Monthly,
    /// 年付
    Yearly,
    /// 按使用量
    PayAsYouGo,
}

/// 使用量定价
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePricing {
    /// 资源类型
    pub resource_type: String,
    
    /// 单价
    pub unit_price: f64,
    
    /// 计费单位
    pub billing_unit: String,
    
    /// 免费额度
    pub free_tier: Option<u64>,
}

/// 资源配额
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuotas {
    /// CPU配额（核心数）
    pub cpu_cores: Option<u32>,
    
    /// 内存配额（GB）
    pub memory_gb: Option<u32>,
    
    /// 存储配额（GB）
    pub storage_gb: Option<u64>,
    
    /// 网络带宽（Mbps）
    pub bandwidth_mbps: Option<u32>,
    
    /// API调用次数（每月）
    pub api_calls_per_month: Option<u64>,
    
    /// 并发连接数
    pub concurrent_connections: Option<u32>,
    
    /// 用户数量
    pub max_users: Option<u32>,
    
    /// 自定义配额
    pub custom_quotas: HashMap<String, u64>,
}

/// 租户配置
#[derive(Debug, Clone)]
pub struct TenantConfiguration {
    /// 数据库配置
    pub database_config: DatabaseConfig,
    
    /// 缓存配置
    pub cache_config: CacheConfig,
    
    /// 安全配置
    pub security_config: TenantSecurityConfig,
    
    /// 功能开关
    pub feature_flags: HashMap<String, bool>,
    
    /// 自定义设置
    pub custom_settings: HashMap<String, String>,
}

/// 数据库配置
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// 数据库类型
    pub db_type: DatabaseType,
    
    /// 连接字符串
    pub connection_string: String,
    
    /// 连接池大小
    pub pool_size: u32,
    
    /// 是否启用读写分离
    pub read_write_split: bool,
}

/// 数据库类型
#[derive(Debug, Clone)]
pub enum DatabaseType {
    /// 共享数据库，独立Schema
    SharedDatabaseSeparateSchema,
    /// 共享数据库，共享Schema
    SharedDatabaseSharedSchema,
    /// 独立数据库
    SeparateDatabase,
}

/// 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 缓存类型
    pub cache_type: CacheType,
    
    /// 缓存大小（MB）
    pub cache_size_mb: u32,
    
    /// TTL（秒）
    pub default_ttl_seconds: u32,
    
    /// 是否启用分布式缓存
    pub distributed: bool,
}

/// 缓存类型
#[derive(Debug, Clone)]
pub enum CacheType {
    /// 内存缓存
    InMemory,
    /// Redis
    Redis,
    /// 混合缓存
    Hybrid,
}

/// 租户安全配置
#[derive(Debug, Clone)]
pub struct TenantSecurityConfig {
    /// 是否启用加密
    pub encryption_enabled: bool,
    
    /// 加密密钥
    pub encryption_key: Option<String>,
    
    /// 是否启用审计
    pub audit_enabled: bool,
    
    /// IP白名单
    pub ip_whitelist: Vec<String>,
    
    /// 安全策略
    pub security_policies: Vec<String>,
}

/// 租户上下文
#[derive(Debug, Clone)]
pub struct TenantContext {
    /// 租户ID
    pub tenant_id: String,
    
    /// 用户ID
    pub user_id: Option<String>,
    
    /// 请求ID
    pub request_id: String,
    
    /// 会话信息
    pub session_info: Option<SessionInfo>,
    
    /// 权限
    pub permissions: Vec<String>,
    
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// 会话信息
#[derive(Debug, Clone)]
pub struct SessionInfo {
    /// 会话ID
    pub session_id: String,
    
    /// IP地址
    pub ip_address: String,
    
    /// 用户代理
    pub user_agent: String,
    
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// 最后访问时间
    pub last_accessed: DateTime<Utc>,
}

/// 隔离引擎
pub struct IsolationEngine {
    isolation_strategy: IsolationStrategy,
    data_partitioner: DataPartitioner,
    network_isolator: NetworkIsolator,
}

/// 隔离策略
#[derive(Debug, Clone)]
pub enum IsolationStrategy {
    /// 物理隔离
    Physical,
    /// 逻辑隔离
    Logical,
    /// 混合隔离
    Hybrid,
}

/// 数据分区器
pub struct DataPartitioner {
    partition_strategy: PartitionStrategy,
    partition_mappings: HashMap<String, String>,
}

/// 分区策略
#[derive(Debug, Clone)]
pub enum PartitionStrategy {
    /// 按租户ID分区
    ByTenantId,
    /// 按地理位置分区
    ByGeography,
    /// 按数据类型分区
    ByDataType,
    /// 自定义分区
    Custom,
}

/// 网络隔离器
pub struct NetworkIsolator {
    virtual_networks: HashMap<String, VirtualNetwork>,
    firewall_rules: Vec<FirewallRule>,
}

/// 虚拟网络
#[derive(Debug, Clone)]
pub struct VirtualNetwork {
    /// 网络ID
    pub network_id: String,
    
    /// 租户ID
    pub tenant_id: String,
    
    /// CIDR块
    pub cidr_block: String,
    
    /// 子网
    pub subnets: Vec<Subnet>,
}

/// 子网
#[derive(Debug, Clone)]
pub struct Subnet {
    /// 子网ID
    pub subnet_id: String,
    
    /// CIDR块
    pub cidr_block: String,
    
    /// 可用区
    pub availability_zone: String,
}

/// 防火墙规则
#[derive(Debug, Clone)]
pub struct FirewallRule {
    /// 规则ID
    pub rule_id: String,
    
    /// 租户ID
    pub tenant_id: String,
    
    /// 源地址
    pub source: String,
    
    /// 目标地址
    pub destination: String,
    
    /// 端口
    pub port: u16,
    
    /// 协议
    pub protocol: String,
    
    /// 动作
    pub action: FirewallAction,
}

/// 防火墙动作
#[derive(Debug, Clone)]
pub enum FirewallAction {
    /// 允许
    Allow,
    /// 拒绝
    Deny,
    /// 记录
    Log,
}

/// 资源分配器
pub struct ResourceAllocator {
    allocation_policies: Vec<AllocationPolicy>,
    resource_pools: HashMap<String, ResourcePool>,
}

/// 分配策略
#[derive(Debug, Clone)]
pub struct AllocationPolicy {
    /// 策略ID
    pub policy_id: String,
    
    /// 资源类型
    pub resource_type: String,
    
    /// 分配算法
    pub allocation_algorithm: AllocationAlgorithm,
    
    /// 优先级
    pub priority: u32,
}

/// 分配算法
#[derive(Debug, Clone)]
pub enum AllocationAlgorithm {
    /// 先到先得
    FirstComeFirstServed,
    /// 优先级调度
    PriorityBased,
    /// 负载均衡
    LoadBalanced,
    /// 资源预留
    ResourceReservation,
}

/// 资源池
#[derive(Debug, Clone)]
pub struct ResourcePool {
    /// 池ID
    pub pool_id: String,
    
    /// 资源类型
    pub resource_type: String,
    
    /// 总容量
    pub total_capacity: u64,
    
    /// 已分配容量
    pub allocated_capacity: u64,
    
    /// 可用容量
    pub available_capacity: u64,
    
    /// 分配记录
    pub allocations: HashMap<String, ResourceAllocation>,
}

/// 资源分配
#[derive(Debug, Clone)]
pub struct ResourceAllocation {
    /// 分配ID
    pub allocation_id: String,
    
    /// 租户ID
    pub tenant_id: String,
    
    /// 资源数量
    pub amount: u64,
    
    /// 分配时间
    pub allocated_at: DateTime<Utc>,
    
    /// 过期时间
    pub expires_at: Option<DateTime<Utc>>,
}

/// 计费管理器
pub struct BillingManager {
    billing_engine: BillingEngine,
    invoice_generator: InvoiceGenerator,
    payment_processor: PaymentProcessor,
}

/// 计费引擎
pub struct BillingEngine {
    billing_rules: Vec<BillingRule>,
    usage_meters: HashMap<String, UsageMeter>,
}

/// 计费规则
#[derive(Debug, Clone)]
pub struct BillingRule {
    /// 规则ID
    pub rule_id: String,
    
    /// 资源类型
    pub resource_type: String,
    
    /// 计费模式
    pub billing_model: BillingModel,
    
    /// 费率
    pub rate: f64,
    
    /// 计费单位
    pub unit: String,
}

/// 计费模式
#[derive(Debug, Clone)]
pub enum BillingModel {
    /// 固定费用
    Fixed,
    /// 按使用量
    UsageBased,
    /// 分层定价
    Tiered,
    /// 阶梯定价
    Volume,
}

/// 使用量计量器
#[derive(Debug, Clone)]
pub struct UsageMeter {
    /// 计量器ID
    pub meter_id: String,
    
    /// 租户ID
    pub tenant_id: String,
    
    /// 资源类型
    pub resource_type: String,
    
    /// 当前使用量
    pub current_usage: u64,
    
    /// 累计使用量
    pub total_usage: u64,
    
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
}

/// 发票生成器
pub struct InvoiceGenerator {
    invoice_templates: HashMap<String, InvoiceTemplate>,
}

/// 发票模板
#[derive(Debug, Clone)]
pub struct InvoiceTemplate {
    /// 模板ID
    pub template_id: String,
    
    /// 模板名称
    pub template_name: String,
    
    /// 模板内容
    pub template_content: String,
    
    /// 支持的格式
    pub supported_formats: Vec<InvoiceFormat>,
}

/// 发票格式
#[derive(Debug, Clone)]
pub enum InvoiceFormat {
    /// PDF
    PDF,
    /// HTML
    HTML,
    /// JSON
    JSON,
    /// XML
    XML,
}

/// 支付处理器
pub struct PaymentProcessor {
    payment_gateways: Vec<PaymentGateway>,
    payment_methods: HashMap<String, PaymentMethod>,
}

/// 支付网关
#[derive(Debug, Clone)]
pub struct PaymentGateway {
    /// 网关ID
    pub gateway_id: String,
    
    /// 网关名称
    pub gateway_name: String,
    
    /// 支持的支付方式
    pub supported_methods: Vec<PaymentMethodType>,
    
    /// 手续费率
    pub fee_rate: f64,
}

/// 支付方式类型
#[derive(Debug, Clone)]
pub enum PaymentMethodType {
    /// 信用卡
    CreditCard,
    /// 借记卡
    DebitCard,
    /// 银行转账
    BankTransfer,
    /// 数字钱包
    DigitalWallet,
    /// 加密货币
    Cryptocurrency,
}

/// 支付方式
#[derive(Debug, Clone)]
pub struct PaymentMethod {
    /// 方式ID
    pub method_id: String,
    
    /// 租户ID
    pub tenant_id: String,
    
    /// 方式类型
    pub method_type: PaymentMethodType,
    
    /// 是否为默认方式
    pub is_default: bool,
    
    /// 详细信息
    pub details: HashMap<String, String>,
}

impl MultiTenantArchitecture {
    /// 创建新的多租户架构
    pub async fn new() -> Result<Self> {
        Ok(Self {
            tenant_manager: TenantManager::new().await?,
            isolation_engine: IsolationEngine::new().await?,
            resource_allocator: ResourceAllocator::new().await?,
            billing_manager: BillingManager::new().await?,
            quota_manager: QuotaManager::new().await?,
            auto_scaler: AutoScaler::new().await?,
        })
    }

    /// 创建租户
    pub async fn create_tenant(&mut self, tenant: Tenant) -> Result<()> {
        // 创建租户
        self.tenant_manager.create_tenant(tenant.clone()).await?;

        // 设置配额
        self.quota_manager.set_quota(&tenant.id, tenant.quotas.clone()).await?;

        // 设置默认扩容策略
        let scaling_policy = ScalingPolicy {
            tenant_id: tenant.id.clone(),
            min_instances: 1,
            max_instances: match tenant.tenant_type {
                TenantType::Individual => 3,
                TenantType::SmallBusiness => 10,
                TenantType::Enterprise => 50,
                TenantType::Government => 100,
                TenantType::Educational => 20,
            },
            cpu_threshold: 0.8,
            memory_threshold: 0.8,
            scale_up_cooldown_minutes: 5,
            scale_down_cooldown_minutes: 10,
        };
        self.auto_scaler.set_scaling_policy(scaling_policy).await?;

        Ok(())
    }

    /// 获取租户
    pub async fn get_tenant(&self, tenant_id: &str) -> Result<Option<Tenant>> {
        self.tenant_manager.get_tenant(tenant_id).await
    }

    /// 分配资源
    pub async fn allocate_resources(&mut self, tenant_id: &str, resource_type: &str, amount: u64) -> Result<String> {
        // 检查配额
        if !self.quota_manager.check_quota(tenant_id, resource_type, amount).await? {
            return Err(EnterpriseError::QuotaExceeded {
                tenant_id: tenant_id.to_string(),
                resource_type: resource_type.to_string(),
                requested: amount,
            });
        }

        // 分配资源
        let allocation_id = self.resource_allocator.allocate(tenant_id, resource_type, amount).await?;

        // 更新配额使用量
        self.quota_manager.update_usage(tenant_id, resource_type, amount).await?;

        // 记录计费
        self.billing_manager.record_usage(tenant_id, resource_type, amount).await?;

        Ok(allocation_id)
    }

    /// 检查自动扩容
    pub async fn check_auto_scaling(&mut self, tenant_id: &str, cpu_usage: f64, memory_usage: f64) -> Result<Option<u32>> {
        self.auto_scaler.check_scaling(tenant_id, cpu_usage, memory_usage).await
    }

    /// 获取租户配额使用情况
    pub async fn get_quota_usage(&self, tenant_id: &str) -> Result<HashMap<String, (u64, u64)>> {
        self.quota_manager.get_quota_usage(tenant_id).await
    }

    /// 获取租户账单
    pub async fn get_tenant_bill(&self, tenant_id: &str) -> Result<f64> {
        self.billing_manager.calculate_bill(tenant_id).await
    }

    /// 获取扩容历史
    pub async fn get_scaling_history(&self, tenant_id: &str) -> Result<Vec<ScalingEvent>> {
        self.auto_scaler.get_scaling_history(tenant_id).await
    }

    /// 暂停租户
    pub async fn suspend_tenant(&mut self, tenant_id: &str) -> Result<()> {
        if let Some(mut tenant) = self.tenant_manager.get_tenant(tenant_id).await? {
            tenant.status = TenantStatus::Suspended;
            self.tenant_manager.update_tenant(tenant).await?;
        }
        Ok(())
    }

    /// 恢复租户
    pub async fn resume_tenant(&mut self, tenant_id: &str) -> Result<()> {
        if let Some(mut tenant) = self.tenant_manager.get_tenant(tenant_id).await? {
            tenant.status = TenantStatus::Active;
            self.tenant_manager.update_tenant(tenant).await?;
        }
        Ok(())
    }
}

// 实现各个组件...
impl TenantManager {
    async fn new() -> Result<Self> {
        Ok(Self {
            tenants: HashMap::new(),
            tenant_configs: HashMap::new(),
        })
    }

    async fn create_tenant(&mut self, tenant: Tenant) -> Result<()> {
        self.tenants.insert(tenant.id.clone(), tenant);
        Ok(())
    }

    async fn get_tenant(&self, tenant_id: &str) -> Result<Option<Tenant>> {
        Ok(self.tenants.get(tenant_id).cloned())
    }

    async fn update_tenant(&mut self, tenant: Tenant) -> Result<()> {
        self.tenants.insert(tenant.id.clone(), tenant);
        Ok(())
    }

    /// 列出所有租户
    pub async fn list_tenants(&self) -> Result<Vec<Tenant>> {
        Ok(self.tenants.values().cloned().collect())
    }

    /// 删除租户
    pub async fn delete_tenant(&mut self, tenant_id: &str) -> Result<()> {
        self.tenants.remove(tenant_id);
        self.tenant_configs.remove(tenant_id);
        Ok(())
    }

    /// 设置租户配置
    pub async fn set_tenant_config(&mut self, tenant_id: &str, config: TenantConfiguration) -> Result<()> {
        self.tenant_configs.insert(tenant_id.to_string(), config);
        Ok(())
    }

    /// 获取租户配置
    pub async fn get_tenant_config(&self, tenant_id: &str) -> Result<Option<TenantConfiguration>> {
        Ok(self.tenant_configs.get(tenant_id).cloned())
    }
}

impl IsolationEngine {
    async fn new() -> Result<Self> {
        Ok(Self {
            isolation_strategy: IsolationStrategy::Logical,
            data_partitioner: DataPartitioner {
                partition_strategy: PartitionStrategy::ByTenantId,
                partition_mappings: HashMap::new(),
            },
            network_isolator: NetworkIsolator {
                virtual_networks: HashMap::new(),
                firewall_rules: Vec::new(),
            },
        })
    }
}

impl ResourceAllocator {
    async fn new() -> Result<Self> {
        Ok(Self {
            allocation_policies: Vec::new(),
            resource_pools: HashMap::new(),
        })
    }
    
    async fn allocate(&mut self, _tenant_id: &str, _resource_type: &str, _amount: u64) -> Result<String> {
        // 简化实现
        Ok(Uuid::new_v4().to_string())
    }
}

impl BillingManager {
    async fn new() -> Result<Self> {
        Ok(Self {
            billing_engine: BillingEngine {
                billing_rules: Vec::new(),
                usage_meters: HashMap::new(),
            },
            invoice_generator: InvoiceGenerator {
                invoice_templates: HashMap::new(),
            },
            payment_processor: PaymentProcessor {
                payment_gateways: Vec::new(),
                payment_methods: HashMap::new(),
            },
        })
    }

    /// 记录使用量
    pub async fn record_usage(&mut self, tenant_id: &str, resource_type: &str, amount: u64) -> Result<()> {
        let meter_id = format!("{}_{}", tenant_id, resource_type);

        if let Some(meter) = self.billing_engine.usage_meters.get_mut(&meter_id) {
            meter.current_usage += amount;
            meter.total_usage += amount;
            meter.last_updated = Utc::now();
        } else {
            let meter = UsageMeter {
                meter_id: meter_id.clone(),
                tenant_id: tenant_id.to_string(),
                resource_type: resource_type.to_string(),
                current_usage: amount,
                total_usage: amount,
                last_updated: Utc::now(),
            };
            self.billing_engine.usage_meters.insert(meter_id, meter);
        }

        Ok(())
    }

    /// 获取租户使用量
    pub async fn get_tenant_usage(&self, tenant_id: &str) -> Result<HashMap<String, u64>> {
        let mut usage = HashMap::new();

        for meter in self.billing_engine.usage_meters.values() {
            if meter.tenant_id == tenant_id {
                usage.insert(meter.resource_type.clone(), meter.current_usage);
            }
        }

        Ok(usage)
    }

    /// 计算账单
    pub async fn calculate_bill(&self, tenant_id: &str) -> Result<f64> {
        let mut total_cost = 0.0;

        for meter in self.billing_engine.usage_meters.values() {
            if meter.tenant_id == tenant_id {
                // 查找对应的计费规则
                for rule in &self.billing_engine.billing_rules {
                    if rule.resource_type == meter.resource_type {
                        total_cost += meter.current_usage as f64 * rule.rate;
                        break;
                    }
                }
            }
        }

        Ok(total_cost)
    }
}

/// 配额管理器实现
pub struct QuotaManager {
    /// 租户配额
    tenant_quotas: HashMap<String, ResourceQuotas>,

    /// 配额使用情况
    quota_usage: HashMap<String, HashMap<String, u64>>,

    /// 配额告警阈值
    alert_thresholds: HashMap<String, f64>,
}

impl QuotaManager {
    /// 创建新的配额管理器
    pub async fn new() -> Result<Self> {
        Ok(Self {
            tenant_quotas: HashMap::new(),
            quota_usage: HashMap::new(),
            alert_thresholds: HashMap::new(),
        })
    }

    /// 设置租户配额
    pub async fn set_quota(&mut self, tenant_id: &str, quotas: ResourceQuotas) -> Result<()> {
        self.tenant_quotas.insert(tenant_id.to_string(), quotas);
        self.quota_usage.entry(tenant_id.to_string()).or_insert_with(HashMap::new);
        Ok(())
    }

    /// 检查配额是否足够
    pub async fn check_quota(&self, tenant_id: &str, resource_type: &str, requested_amount: u64) -> Result<bool> {
        if let Some(quotas) = self.tenant_quotas.get(tenant_id) {
            let current_usage = self.quota_usage
                .get(tenant_id)
                .and_then(|usage| usage.get(resource_type))
                .unwrap_or(&0);

            let quota_limit = match resource_type {
                "cpu_cores" => quotas.cpu_cores.unwrap_or(0) as u64,
                "memory_gb" => quotas.memory_gb.unwrap_or(0) as u64,
                "storage_gb" => quotas.storage_gb.unwrap_or(0),
                "api_calls" => quotas.api_calls_per_month.unwrap_or(0),
                _ => quotas.custom_quotas.get(resource_type).unwrap_or(&0).clone(),
            };

            Ok(current_usage + requested_amount <= quota_limit)
        } else {
            Ok(false) // 没有配额定义，拒绝请求
        }
    }

    /// 更新配额使用量
    pub async fn update_usage(&mut self, tenant_id: &str, resource_type: &str, amount: u64) -> Result<()> {
        let usage = self.quota_usage.entry(tenant_id.to_string()).or_insert_with(HashMap::new);
        *usage.entry(resource_type.to_string()).or_insert(0) += amount;

        // 检查是否需要告警
        self.check_quota_alerts(tenant_id, resource_type).await?;

        Ok(())
    }

    /// 检查配额告警
    async fn check_quota_alerts(&self, tenant_id: &str, resource_type: &str) -> Result<()> {
        if let Some(quotas) = self.tenant_quotas.get(tenant_id) {
            if let Some(usage_map) = self.quota_usage.get(tenant_id) {
                if let Some(current_usage) = usage_map.get(resource_type) {
                    let quota_limit = match resource_type {
                        "cpu_cores" => quotas.cpu_cores.unwrap_or(0) as u64,
                        "memory_gb" => quotas.memory_gb.unwrap_or(0) as u64,
                        "storage_gb" => quotas.storage_gb.unwrap_or(0),
                        "api_calls" => quotas.api_calls_per_month.unwrap_or(0),
                        _ => quotas.custom_quotas.get(resource_type).unwrap_or(&0).clone(),
                    };

                    if quota_limit > 0 {
                        let usage_percentage = *current_usage as f64 / quota_limit as f64;
                        let threshold = self.alert_thresholds.get(resource_type).unwrap_or(&0.8);

                        if usage_percentage >= *threshold {
                            tracing::warn!(
                                "配额告警: 租户 {} 的 {} 使用率达到 {:.1}%",
                                tenant_id, resource_type, usage_percentage * 100.0
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// 获取配额使用情况
    pub async fn get_quota_usage(&self, tenant_id: &str) -> Result<HashMap<String, (u64, u64)>> {
        let mut result = HashMap::new();

        if let Some(quotas) = self.tenant_quotas.get(tenant_id) {
            if let Some(usage_map) = self.quota_usage.get(tenant_id) {
                for (resource_type, current_usage) in usage_map {
                    let quota_limit = match resource_type.as_str() {
                        "cpu_cores" => quotas.cpu_cores.unwrap_or(0) as u64,
                        "memory_gb" => quotas.memory_gb.unwrap_or(0) as u64,
                        "storage_gb" => quotas.storage_gb.unwrap_or(0),
                        "api_calls" => quotas.api_calls_per_month.unwrap_or(0),
                        _ => quotas.custom_quotas.get(resource_type).unwrap_or(&0).clone(),
                    };

                    result.insert(resource_type.clone(), (*current_usage, quota_limit));
                }
            }
        }

        Ok(result)
    }
}

/// 自动扩容器实现
pub struct AutoScaler {
    /// 扩容策略
    scaling_policies: HashMap<String, ScalingPolicy>,

    /// 扩容历史
    scaling_history: HashMap<String, Vec<ScalingEvent>>,

    /// 当前实例数
    current_instances: HashMap<String, u32>,

    /// 最后扩容时间
    last_scaling_time: HashMap<String, DateTime<Utc>>,
}

/// 扩容策略
#[derive(Debug, Clone)]
pub struct ScalingPolicy {
    /// 租户ID
    pub tenant_id: String,

    /// 最小实例数
    pub min_instances: u32,

    /// 最大实例数
    pub max_instances: u32,

    /// CPU阈值
    pub cpu_threshold: f64,

    /// 内存阈值
    pub memory_threshold: f64,

    /// 扩容冷却时间（分钟）
    pub scale_up_cooldown_minutes: i64,

    /// 缩容冷却时间（分钟）
    pub scale_down_cooldown_minutes: i64,
}

/// 扩容事件
#[derive(Debug, Clone)]
pub struct ScalingEvent {
    /// 事件ID
    pub event_id: String,

    /// 租户ID
    pub tenant_id: String,

    /// 事件类型
    pub event_type: ScalingEventType,

    /// 原实例数
    pub from_instances: u32,

    /// 目标实例数
    pub to_instances: u32,

    /// 触发原因
    pub reason: String,

    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// 扩容事件类型
#[derive(Debug, Clone)]
pub enum ScalingEventType {
    /// 扩容
    ScaleUp,
    /// 缩容
    ScaleDown,
}

impl AutoScaler {
    /// 创建新的自动扩容器
    pub async fn new() -> Result<Self> {
        Ok(Self {
            scaling_policies: HashMap::new(),
            scaling_history: HashMap::new(),
            current_instances: HashMap::new(),
            last_scaling_time: HashMap::new(),
        })
    }

    /// 设置扩容策略
    pub async fn set_scaling_policy(&mut self, policy: ScalingPolicy) -> Result<()> {
        let tenant_id = policy.tenant_id.clone();
        self.scaling_policies.insert(tenant_id.clone(), policy);
        self.current_instances.entry(tenant_id).or_insert(1);
        Ok(())
    }

    /// 检查是否需要扩容
    pub async fn check_scaling(&mut self, tenant_id: &str, cpu_usage: f64, memory_usage: f64) -> Result<Option<u32>> {
        if let Some(policy) = self.scaling_policies.get(tenant_id) {
            let current_instances = *self.current_instances.get(tenant_id).unwrap_or(&1);
            let now = Utc::now();

            // 检查冷却时间
            if let Some(last_time) = self.last_scaling_time.get(tenant_id) {
                let cooldown_duration = chrono::Duration::minutes(policy.scale_up_cooldown_minutes);
                if now - *last_time < cooldown_duration {
                    return Ok(None); // 还在冷却期
                }
            }

            // 检查是否需要扩容
            if (cpu_usage > policy.cpu_threshold || memory_usage > policy.memory_threshold)
                && current_instances < policy.max_instances {

                let new_instances = (current_instances + 1).min(policy.max_instances);
                self.execute_scaling(tenant_id, current_instances, new_instances, ScalingEventType::ScaleUp,
                    format!("CPU: {:.1}%, Memory: {:.1}%", cpu_usage * 100.0, memory_usage * 100.0)).await?;

                return Ok(Some(new_instances));
            }

            // 检查是否需要缩容
            if cpu_usage < policy.cpu_threshold * 0.5 && memory_usage < policy.memory_threshold * 0.5
                && current_instances > policy.min_instances {

                let new_instances = (current_instances - 1).max(policy.min_instances);
                self.execute_scaling(tenant_id, current_instances, new_instances, ScalingEventType::ScaleDown,
                    format!("CPU: {:.1}%, Memory: {:.1}%", cpu_usage * 100.0, memory_usage * 100.0)).await?;

                return Ok(Some(new_instances));
            }
        }

        Ok(None)
    }

    /// 执行扩容操作
    async fn execute_scaling(&mut self, tenant_id: &str, from_instances: u32, to_instances: u32,
                           event_type: ScalingEventType, reason: String) -> Result<()> {
        let event = ScalingEvent {
            event_id: Uuid::new_v4().to_string(),
            tenant_id: tenant_id.to_string(),
            event_type: event_type.clone(),
            from_instances,
            to_instances,
            reason,
            timestamp: Utc::now(),
        };

        // 记录扩容事件
        self.scaling_history.entry(tenant_id.to_string())
            .or_insert_with(Vec::new)
            .push(event);

        // 更新当前实例数
        self.current_instances.insert(tenant_id.to_string(), to_instances);
        self.last_scaling_time.insert(tenant_id.to_string(), Utc::now());

        tracing::info!(
            "执行扩容操作: 租户 {} 从 {} 个实例调整到 {} 个实例 ({:?})",
            tenant_id, from_instances, to_instances, event_type
        );

        Ok(())
    }

    /// 获取扩容历史
    pub async fn get_scaling_history(&self, tenant_id: &str) -> Result<Vec<ScalingEvent>> {
        Ok(self.scaling_history.get(tenant_id).cloned().unwrap_or_default())
    }

    /// 获取当前实例数
    pub async fn get_current_instances(&self, tenant_id: &str) -> Result<u32> {
        Ok(*self.current_instances.get(tenant_id).unwrap_or(&1))
    }
}
    use super::*;
    
    #[tokio::test]
    async fn test_multi_tenant_architecture_creation() {
        let architecture = MultiTenantArchitecture::new().await.unwrap();
        
        // 测试基本功能
        let tenant = architecture.get_tenant("nonexistent").await.unwrap();
        assert!(tenant.is_none());
    }
    
    #[tokio::test]
    async fn test_tenant_creation() {
        let mut architecture = MultiTenantArchitecture::new().await.unwrap();
        
        let tenant = Tenant {
            id: "tenant1".to_string(),
            name: "Test Tenant".to_string(),
            tenant_type: TenantType::SmallBusiness,
            created_at: Utc::now(),
            status: TenantStatus::Active,
            contact_info: ContactInfo {
                primary_contact: "John Doe".to_string(),
                email: "john@example.com".to_string(),
                phone: None,
                address: None,
                company_info: None,
            },
            subscription_plan: SubscriptionPlan {
                plan_id: "basic".to_string(),
                plan_name: "Basic Plan".to_string(),
                plan_type: PlanType::Basic,
                price: PlanPricing {
                    base_price: 29.99,
                    billing_cycle: BillingCycle::Monthly,
                    currency: "USD".to_string(),
                    usage_pricing: Vec::new(),
                },
                features: vec!["basic_features".to_string()],
                start_date: Utc::now(),
                end_date: None,
                auto_renewal: true,
            },
            quotas: ResourceQuotas {
                cpu_cores: Some(2),
                memory_gb: Some(4),
                storage_gb: Some(100),
                bandwidth_mbps: Some(10),
                api_calls_per_month: Some(10000),
                concurrent_connections: Some(100),
                max_users: Some(10),
                custom_quotas: HashMap::new(),
            },
            metadata: HashMap::new(),
        };
        
        assert!(architecture.create_tenant(tenant.clone()).await.is_ok());
        
        let retrieved = architecture.get_tenant(&tenant.id).await.unwrap().unwrap();
        assert_eq!(retrieved.id, tenant.id);
        assert_eq!(retrieved.name, tenant.name);
    }
}
