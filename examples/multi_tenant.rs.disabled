//! 多租户功能演示
//! 
//! 展示如何实现企业级多租户架构，包括：
//! - 租户隔离和管理
//! - 资源配额控制
//! - 数据隔离策略
//! - 租户级配置

use lumosai_core::prelude::*;
use lumosai_core::agent::{AgentBuilder, BasicAgent};
use lumosai_core::tenant::{TenantManager, TenantConfig, ResourceQuota, IsolationLevel};
use lumosai_core::billing::{BillingManager, UsageTracker, PricingPlan};
use lumosai_core::llm::{MockLlmProvider, Message, Role};
use std::sync::Arc;
use std::collections::HashMap;
use serde_json::json;
use chrono::{DateTime, Utc};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏢 多租户功能演示");
    println!("==================");
    
    // 演示1: 租户管理
    demo_tenant_management().await?;
    
    // 演示2: 资源隔离
    demo_resource_isolation().await?;
    
    // 演示3: 配额管理
    demo_quota_management().await?;
    
    // 演示4: 计费系统
    demo_billing_system().await?;
    
    Ok(())
}

/// 演示租户管理
async fn demo_tenant_management() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示1: 租户管理 ===");
    
    // 创建租户管理器
    let tenant_manager = TenantManager::new(TenantManagerConfig {
        enable_auto_provisioning: true,
        enable_resource_isolation: true,
        enable_data_encryption: true,
        default_isolation_level: IsolationLevel::Namespace,
        max_tenants: 1000,
    })?;
    
    println!("租户管理器配置:");
    println!("  自动配置: 启用");
    println!("  资源隔离: 启用");
    println!("  数据加密: 启用");
    println!("  默认隔离级别: 命名空间");
    
    // 创建不同类型的租户
    let tenants = vec![
        TenantConfig {
            id: "tenant_enterprise_001".to_string(),
            name: "Acme Corporation".to_string(),
            tier: TenantTier::Enterprise,
            isolation_level: IsolationLevel::Database,
            resource_quota: ResourceQuota {
                max_agents: 100,
                max_requests_per_minute: 10000,
                max_storage_gb: 1000,
                max_concurrent_users: 500,
                max_api_calls_per_day: 1000000,
            },
            features: vec![
                "advanced_analytics".to_string(),
                "custom_models".to_string(),
                "priority_support".to_string(),
                "sla_guarantee".to_string(),
            ],
            created_at: Utc::now(),
            billing_plan: PricingPlan::Enterprise,
        },
        TenantConfig {
            id: "tenant_pro_001".to_string(),
            name: "StartupXYZ".to_string(),
            tier: TenantTier::Professional,
            isolation_level: IsolationLevel::Namespace,
            resource_quota: ResourceQuota {
                max_agents: 20,
                max_requests_per_minute: 1000,
                max_storage_gb: 100,
                max_concurrent_users: 50,
                max_api_calls_per_day: 100000,
            },
            features: vec![
                "basic_analytics".to_string(),
                "standard_support".to_string(),
            ],
            created_at: Utc::now(),
            billing_plan: PricingPlan::Professional,
        },
        TenantConfig {
            id: "tenant_basic_001".to_string(),
            name: "Individual Developer".to_string(),
            tier: TenantTier::Basic,
            isolation_level: IsolationLevel::Logical,
            resource_quota: ResourceQuota {
                max_agents: 5,
                max_requests_per_minute: 100,
                max_storage_gb: 10,
                max_concurrent_users: 5,
                max_api_calls_per_day: 10000,
            },
            features: vec![
                "community_support".to_string(),
            ],
            created_at: Utc::now(),
            billing_plan: PricingPlan::Basic,
        },
    ];
    
    // 注册租户
    println!("\n=== 租户注册 ===");
    for tenant in &tenants {
        tenant_manager.create_tenant(tenant.clone()).await?;
        
        let tier_icon = match tenant.tier {
            TenantTier::Enterprise => "💎",
            TenantTier::Professional => "⭐",
            TenantTier::Basic => "🔰",
        };
        
        println!("  {} 租户: {} ({})", tier_icon, tenant.name, tenant.id);
        println!("    层级: {:?}", tenant.tier);
        println!("    隔离级别: {:?}", tenant.isolation_level);
        println!("    最大Agent数: {}", tenant.resource_quota.max_agents);
        println!("    功能: {:?}", tenant.features);
        println!();
    }
    
    // 获取租户信息
    println!("=== 租户信息查询 ===");
    let tenant_list = tenant_manager.list_tenants().await?;
    println!("  总租户数: {}", tenant_list.len());
    
    for tenant in &tenant_list {
        let status = tenant_manager.get_tenant_status(&tenant.id).await?;
        println!("  租户 {}: {}", tenant.name, status.status);
        println!("    活跃用户: {}", status.active_users);
        println!("    资源使用率: {:.1}%", status.resource_utilization * 100.0);
    }
    
    Ok(())
}

/// 演示资源隔离
async fn demo_resource_isolation() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示2: 资源隔离 ===");
    
    // 为不同租户创建隔离的Agent
    let enterprise_responses = vec![
        "企业级Agent响应：我可以访问高级功能和自定义模型。".to_string(),
    ];
    let pro_responses = vec![
        "专业版Agent响应：我提供标准功能和支持。".to_string(),
    ];
    let basic_responses = vec![
        "基础版Agent响应：我提供基本功能。".to_string(),
    ];
    
    // 企业租户Agent
    let enterprise_llm = Arc::new(MockLlmProvider::new(enterprise_responses));
    let enterprise_agent = AgentBuilder::new()
        .name("enterprise_assistant")
        .instructions("你是企业级AI助手，拥有高级功能")
        .model(enterprise_llm)
        .tenant_id("tenant_enterprise_001")
        .isolation_level(IsolationLevel::Database)
        .build()?;
    
    // 专业租户Agent
    let pro_llm = Arc::new(MockLlmProvider::new(pro_responses));
    let pro_agent = AgentBuilder::new()
        .name("pro_assistant")
        .instructions("你是专业版AI助手")
        .model(pro_llm)
        .tenant_id("tenant_pro_001")
        .isolation_level(IsolationLevel::Namespace)
        .build()?;
    
    // 基础租户Agent
    let basic_llm = Arc::new(MockLlmProvider::new(basic_responses));
    let basic_agent = AgentBuilder::new()
        .name("basic_assistant")
        .instructions("你是基础版AI助手")
        .model(basic_llm)
        .tenant_id("tenant_basic_001")
        .isolation_level(IsolationLevel::Logical)
        .build()?;
    
    println!("不同租户的Agent已创建，具有不同的隔离级别");
    
    // 测试资源隔离
    println!("\n=== 资源隔离测试 ===");
    
    let test_query = "请介绍你的功能";
    
    println!("  企业租户Agent:");
    let enterprise_response = enterprise_agent.generate(test_query).await?;
    println!("    {}", enterprise_response.content);
    
    println!("  专业租户Agent:");
    let pro_response = pro_agent.generate(test_query).await?;
    println!("    {}", pro_response.content);
    
    println!("  基础租户Agent:");
    let basic_response = basic_agent.generate(test_query).await?;
    println!("    {}", basic_response.content);
    
    // 验证数据隔离
    println!("\n=== 数据隔离验证 ===");
    
    // 模拟数据访问测试
    let data_access_tests = vec![
        ("tenant_enterprise_001", "customer_data", true),
        ("tenant_pro_001", "customer_data", true),
        ("tenant_basic_001", "customer_data", true),
        ("tenant_enterprise_001", "tenant_pro_001_data", false),
        ("tenant_pro_001", "tenant_enterprise_001_data", false),
        ("tenant_basic_001", "tenant_enterprise_001_data", false),
    ];
    
    for (tenant_id, resource, should_access) in data_access_tests {
        let can_access = simulate_data_access(tenant_id, resource).await?;
        let result_icon = if can_access == should_access { "✅" } else { "❌" };
        
        println!("  {} 租户 {} 访问 {}: {}", 
            result_icon, tenant_id, resource, 
            if can_access { "允许" } else { "拒绝" });
    }
    
    Ok(())
}

/// 演示配额管理
async fn demo_quota_management() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示3: 配额管理 ===");
    
    // 创建配额管理器
    let quota_manager = QuotaManager::new(QuotaConfig {
        enable_real_time_monitoring: true,
        enable_auto_scaling: true,
        enable_quota_alerts: true,
        grace_period_minutes: 5,
    })?;
    
    println!("配额管理器配置:");
    println!("  实时监控: 启用");
    println!("  自动扩容: 启用");
    println!("  配额告警: 启用");
    
    // 模拟不同租户的资源使用
    println!("\n=== 资源使用模拟 ===");
    
    let usage_scenarios = vec![
        ("tenant_enterprise_001", ResourceType::Agents, 45, 100),
        ("tenant_enterprise_001", ResourceType::RequestsPerMinute, 8500, 10000),
        ("tenant_pro_001", ResourceType::Agents, 18, 20),
        ("tenant_pro_001", ResourceType::RequestsPerMinute, 950, 1000),
        ("tenant_basic_001", ResourceType::Agents, 5, 5), // 达到限制
        ("tenant_basic_001", ResourceType::RequestsPerMinute, 105, 100), // 超过限制
    ];
    
    for (tenant_id, resource_type, current_usage, quota_limit) in usage_scenarios {
        let usage_percentage = (current_usage as f64 / quota_limit as f64) * 100.0;
        
        // 记录资源使用
        quota_manager.record_usage(tenant_id, resource_type.clone(), current_usage).await?;
        
        // 检查配额状态
        let quota_status = quota_manager.check_quota(tenant_id, &resource_type).await?;
        
        let status_icon = match quota_status.status {
            QuotaStatus::Normal => "🟢",
            QuotaStatus::Warning => "🟡",
            QuotaStatus::Critical => "🟠",
            QuotaStatus::Exceeded => "🔴",
        };
        
        println!("  {} 租户 {} - {:?}: {}/{} ({:.1}%)", 
            status_icon, tenant_id, resource_type, current_usage, quota_limit, usage_percentage);
        
        if quota_status.status == QuotaStatus::Exceeded {
            println!("    ⚠️  配额超限，建议升级套餐");
        } else if quota_status.status == QuotaStatus::Critical {
            println!("    ⚠️  接近配额限制");
        }
    }
    
    // 配额使用统计
    println!("\n=== 配额使用统计 ===");
    
    for tenant_id in ["tenant_enterprise_001", "tenant_pro_001", "tenant_basic_001"] {
        let usage_stats = quota_manager.get_usage_statistics(tenant_id).await?;
        
        println!("  租户 {}:", tenant_id);
        println!("    总体使用率: {:.1}%", usage_stats.overall_utilization * 100.0);
        println!("    最高使用资源: {:?}", usage_stats.highest_usage_resource);
        println!("    预计耗尽时间: {:?}", usage_stats.estimated_depletion_time);
        
        if usage_stats.recommendations.len() > 0 {
            println!("    建议:");
            for recommendation in &usage_stats.recommendations {
                println!("      - {}", recommendation);
            }
        }
        println!();
    }
    
    Ok(())
}

/// 演示计费系统
async fn demo_billing_system() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示4: 计费系统 ===");
    
    // 创建计费管理器
    let billing_manager = BillingManager::new(BillingConfig {
        enable_usage_tracking: true,
        enable_auto_billing: true,
        billing_cycle: BillingCycle::Monthly,
        currency: "USD".to_string(),
        tax_rate: 0.08, // 8% 税率
    })?;
    
    // 创建使用量追踪器
    let usage_tracker = UsageTracker::new();
    
    println!("计费系统配置:");
    println!("  使用量追踪: 启用");
    println!("  自动计费: 启用");
    println!("  计费周期: 月度");
    println!("  货币: USD");
    println!("  税率: 8%");
    
    // 模拟使用量数据
    println!("\n=== 使用量追踪 ===");
    
    let usage_data = vec![
        ("tenant_enterprise_001", UsageMetric::AgentRequests, 850000),
        ("tenant_enterprise_001", UsageMetric::StorageGB, 750),
        ("tenant_enterprise_001", UsageMetric::ComputeHours, 2400),
        ("tenant_pro_001", UsageMetric::AgentRequests, 75000),
        ("tenant_pro_001", UsageMetric::StorageGB, 85),
        ("tenant_pro_001", UsageMetric::ComputeHours, 180),
        ("tenant_basic_001", UsageMetric::AgentRequests, 8500),
        ("tenant_basic_001", UsageMetric::StorageGB, 8),
        ("tenant_basic_001", UsageMetric::ComputeHours, 25),
    ];
    
    for (tenant_id, metric, amount) in &usage_data {
        usage_tracker.record_usage(tenant_id, metric.clone(), *amount).await?;
        println!("  记录 {} - {:?}: {}", tenant_id, metric, amount);
    }
    
    // 生成账单
    println!("\n=== 账单生成 ===");
    
    for tenant_id in ["tenant_enterprise_001", "tenant_pro_001", "tenant_basic_001"] {
        let bill = billing_manager.generate_bill(
            tenant_id,
            Utc::now() - chrono::Duration::days(30),
            Utc::now(),
        ).await?;
        
        println!("  租户 {} 月度账单:", tenant_id);
        println!("    账单ID: {}", bill.bill_id);
        println!("    计费周期: {} 到 {}", 
            bill.billing_period_start.format("%Y-%m-%d"),
            bill.billing_period_end.format("%Y-%m-%d"));
        
        println!("    费用明细:");
        for item in &bill.line_items {
            println!("      - {}: ${:.2}", item.description, item.amount);
        }
        
        println!("    小计: ${:.2}", bill.subtotal);
        println!("    税费: ${:.2}", bill.tax_amount);
        println!("    总计: ${:.2}", bill.total_amount);
        println!("    状态: {:?}", bill.status);
        println!();
    }
    
    // 使用量分析
    println!("=== 使用量分析 ===");
    
    let usage_analysis = billing_manager.analyze_usage_trends(
        chrono::Duration::days(90)
    ).await?;
    
    println!("  过去90天使用量趋势:");
    println!("    总收入: ${:.2}", usage_analysis.total_revenue);
    println!("    平均每租户收入: ${:.2}", usage_analysis.average_revenue_per_tenant);
    println!("    增长率: {:.1}%", usage_analysis.growth_rate * 100.0);
    
    println!("    热门功能:");
    for (feature, usage) in &usage_analysis.top_features {
        println!("      - {}: {} 次使用", feature, usage);
    }
    
    println!("    成本优化建议:");
    for recommendation in &usage_analysis.cost_optimization_recommendations {
        println!("      - {}", recommendation);
    }
    
    Ok(())
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 模拟数据访问检查
async fn simulate_data_access(tenant_id: &str, resource: &str) -> Result<bool, Box<dyn std::error::Error>> {
    // 简单的访问控制逻辑
    if resource == "customer_data" {
        // 所有租户都可以访问自己的客户数据
        Ok(true)
    } else if resource.starts_with(tenant_id) {
        // 只能访问自己租户的数据
        Ok(true)
    } else {
        // 不能访问其他租户的数据
        Ok(false)
    }
}

// ============================================================================
// 数据结构定义
// ============================================================================

#[derive(Debug, Clone)]
struct TenantManagerConfig {
    enable_auto_provisioning: bool,
    enable_resource_isolation: bool,
    enable_data_encryption: bool,
    default_isolation_level: IsolationLevel,
    max_tenants: u32,
}

#[derive(Debug, Clone)]
struct TenantConfig {
    id: String,
    name: String,
    tier: TenantTier,
    isolation_level: IsolationLevel,
    resource_quota: ResourceQuota,
    features: Vec<String>,
    created_at: DateTime<Utc>,
    billing_plan: PricingPlan,
}

#[derive(Debug, Clone, PartialEq)]
enum TenantTier {
    Basic,
    Professional,
    Enterprise,
}

#[derive(Debug, Clone, PartialEq)]
enum IsolationLevel {
    Logical,     // 逻辑隔离（共享数据库）
    Namespace,   // 命名空间隔离
    Database,    // 数据库隔离
    Physical,    // 物理隔离
}

#[derive(Debug, Clone)]
struct ResourceQuota {
    max_agents: u32,
    max_requests_per_minute: u32,
    max_storage_gb: u32,
    max_concurrent_users: u32,
    max_api_calls_per_day: u64,
}

#[derive(Debug, Clone, PartialEq)]
enum PricingPlan {
    Basic,
    Professional,
    Enterprise,
    Custom,
}

#[derive(Debug, Clone)]
struct TenantStatus {
    status: String,
    active_users: u32,
    resource_utilization: f64,
    last_activity: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct QuotaConfig {
    enable_real_time_monitoring: bool,
    enable_auto_scaling: bool,
    enable_quota_alerts: bool,
    grace_period_minutes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ResourceType {
    Agents,
    RequestsPerMinute,
    StorageGB,
    ConcurrentUsers,
    ApiCallsPerDay,
}

#[derive(Debug, Clone)]
struct QuotaStatusInfo {
    status: QuotaStatus,
    current_usage: u64,
    quota_limit: u64,
    utilization_percentage: f64,
}

#[derive(Debug, Clone, PartialEq)]
enum QuotaStatus {
    Normal,    // < 70%
    Warning,   // 70-85%
    Critical,  // 85-100%
    Exceeded,  // > 100%
}

#[derive(Debug, Clone)]
struct UsageStatistics {
    overall_utilization: f64,
    highest_usage_resource: ResourceType,
    estimated_depletion_time: Option<DateTime<Utc>>,
    recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
struct BillingConfig {
    enable_usage_tracking: bool,
    enable_auto_billing: bool,
    billing_cycle: BillingCycle,
    currency: String,
    tax_rate: f64,
}

#[derive(Debug, Clone)]
enum BillingCycle {
    Monthly,
    Quarterly,
    Annually,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum UsageMetric {
    AgentRequests,
    StorageGB,
    ComputeHours,
    DataTransferGB,
    ApiCalls,
}

#[derive(Debug, Clone)]
struct Bill {
    bill_id: String,
    tenant_id: String,
    billing_period_start: DateTime<Utc>,
    billing_period_end: DateTime<Utc>,
    line_items: Vec<BillLineItem>,
    subtotal: f64,
    tax_amount: f64,
    total_amount: f64,
    status: BillStatus,
    generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct BillLineItem {
    description: String,
    quantity: f64,
    unit_price: f64,
    amount: f64,
}

#[derive(Debug, Clone)]
enum BillStatus {
    Draft,
    Pending,
    Paid,
    Overdue,
    Cancelled,
}

#[derive(Debug, Clone)]
struct UsageAnalysis {
    total_revenue: f64,
    average_revenue_per_tenant: f64,
    growth_rate: f64,
    top_features: Vec<(String, u64)>,
    cost_optimization_recommendations: Vec<String>,
}

// ============================================================================
// 模拟实现（实际项目中应该有真实的实现）
// ============================================================================

struct TenantManager {
    tenants: Arc<tokio::sync::Mutex<HashMap<String, TenantConfig>>>,
}

impl TenantManager {
    fn new(_config: TenantManagerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            tenants: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        })
    }

    async fn create_tenant(&self, tenant: TenantConfig) -> Result<(), Box<dyn std::error::Error>> {
        let mut tenants = self.tenants.lock().await;
        tenants.insert(tenant.id.clone(), tenant);
        Ok(())
    }

    async fn list_tenants(&self) -> Result<Vec<TenantConfig>, Box<dyn std::error::Error>> {
        let tenants = self.tenants.lock().await;
        Ok(tenants.values().cloned().collect())
    }

    async fn get_tenant_status(&self, tenant_id: &str) -> Result<TenantStatus, Box<dyn std::error::Error>> {
        let tenants = self.tenants.lock().await;

        if tenants.contains_key(tenant_id) {
            Ok(TenantStatus {
                status: "active".to_string(),
                active_users: match tenant_id {
                    "tenant_enterprise_001" => 245,
                    "tenant_pro_001" => 32,
                    "tenant_basic_001" => 3,
                    _ => 0,
                },
                resource_utilization: match tenant_id {
                    "tenant_enterprise_001" => 0.75,
                    "tenant_pro_001" => 0.85,
                    "tenant_basic_001" => 0.95,
                    _ => 0.0,
                },
                last_activity: Utc::now(),
            })
        } else {
            Err("Tenant not found".into())
        }
    }
}

struct QuotaManager {
    usage_data: Arc<tokio::sync::Mutex<HashMap<String, HashMap<ResourceType, u64>>>>,
}

impl QuotaManager {
    fn new(_config: QuotaConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            usage_data: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        })
    }

    async fn record_usage(&self, tenant_id: &str, resource_type: ResourceType, usage: u64) -> Result<(), Box<dyn std::error::Error>> {
        let mut usage_data = self.usage_data.lock().await;
        let tenant_usage = usage_data.entry(tenant_id.to_string()).or_insert_with(HashMap::new);
        tenant_usage.insert(resource_type, usage);
        Ok(())
    }

    async fn check_quota(&self, tenant_id: &str, resource_type: &ResourceType) -> Result<QuotaStatusInfo, Box<dyn std::error::Error>> {
        let usage_data = self.usage_data.lock().await;

        let current_usage = usage_data
            .get(tenant_id)
            .and_then(|tenant_usage| tenant_usage.get(resource_type))
            .map(|&value| value)
            .unwrap_or(0);

        let quota_limit = match (tenant_id, resource_type) {
            ("tenant_enterprise_001", ResourceType::Agents) => 100,
            ("tenant_enterprise_001", ResourceType::RequestsPerMinute) => 10000,
            ("tenant_pro_001", ResourceType::Agents) => 20,
            ("tenant_pro_001", ResourceType::RequestsPerMinute) => 1000,
            ("tenant_basic_001", ResourceType::Agents) => 5,
            ("tenant_basic_001", ResourceType::RequestsPerMinute) => 100,
            _ => 0,
        };

        let utilization_percentage = (current_usage as f64 / quota_limit as f64) * 100.0;

        let status = if utilization_percentage > 100.0 {
            QuotaStatus::Exceeded
        } else if utilization_percentage > 85.0 {
            QuotaStatus::Critical
        } else if utilization_percentage > 70.0 {
            QuotaStatus::Warning
        } else {
            QuotaStatus::Normal
        };

        Ok(QuotaStatusInfo {
            status,
            current_usage,
            quota_limit,
            utilization_percentage,
        })
    }

    async fn get_usage_statistics(&self, tenant_id: &str) -> Result<UsageStatistics, Box<dyn std::error::Error>> {
        let usage_data = self.usage_data.lock().await;

        if let Some(tenant_usage) = usage_data.get(tenant_id) {
            let overall_utilization = match tenant_id {
                "tenant_enterprise_001" => 0.65,
                "tenant_pro_001" => 0.82,
                "tenant_basic_001" => 0.95,
                _ => 0.0,
            };

            let highest_usage_resource = ResourceType::RequestsPerMinute;

            let recommendations = if overall_utilization > 0.9 {
                vec![
                    "考虑升级到更高级别的套餐".to_string(),
                    "优化Agent使用频率".to_string(),
                ]
            } else if overall_utilization > 0.8 {
                vec![
                    "监控资源使用趋势".to_string(),
                ]
            } else {
                vec![]
            };

            Ok(UsageStatistics {
                overall_utilization,
                highest_usage_resource,
                estimated_depletion_time: if overall_utilization > 0.9 {
                    Some(Utc::now() + chrono::Duration::days(7))
                } else {
                    None
                },
                recommendations,
            })
        } else {
            Err("Tenant not found".into())
        }
    }
}

struct BillingManager;

impl BillingManager {
    fn new(_config: BillingConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self)
    }

    async fn generate_bill(
        &self,
        tenant_id: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Bill, Box<dyn std::error::Error>> {
        let line_items = match tenant_id {
            "tenant_enterprise_001" => vec![
                BillLineItem {
                    description: "Enterprise Plan Base Fee".to_string(),
                    quantity: 1.0,
                    unit_price: 2000.0,
                    amount: 2000.0,
                },
                BillLineItem {
                    description: "Additional Agent Requests (850K)".to_string(),
                    quantity: 850.0,
                    unit_price: 0.002,
                    amount: 1700.0,
                },
                BillLineItem {
                    description: "Storage (750 GB)".to_string(),
                    quantity: 750.0,
                    unit_price: 0.5,
                    amount: 375.0,
                },
            ],
            "tenant_pro_001" => vec![
                BillLineItem {
                    description: "Professional Plan Base Fee".to_string(),
                    quantity: 1.0,
                    unit_price: 500.0,
                    amount: 500.0,
                },
                BillLineItem {
                    description: "Additional Agent Requests (75K)".to_string(),
                    quantity: 75.0,
                    unit_price: 0.005,
                    amount: 375.0,
                },
            ],
            "tenant_basic_001" => vec![
                BillLineItem {
                    description: "Basic Plan Base Fee".to_string(),
                    quantity: 1.0,
                    unit_price: 50.0,
                    amount: 50.0,
                },
            ],
            _ => vec![],
        };

        let subtotal = line_items.iter().map(|item| item.amount).sum::<f64>();
        let tax_amount = subtotal * 0.08; // 8% tax
        let total_amount = subtotal + tax_amount;

        Ok(Bill {
            bill_id: format!("BILL_{}", rand::random::<u32>()),
            tenant_id: tenant_id.to_string(),
            billing_period_start: start_date,
            billing_period_end: end_date,
            line_items,
            subtotal,
            tax_amount,
            total_amount,
            status: BillStatus::Pending,
            generated_at: Utc::now(),
        })
    }

    async fn analyze_usage_trends(
        &self,
        _duration: chrono::Duration,
    ) -> Result<UsageAnalysis, Box<dyn std::error::Error>> {
        Ok(UsageAnalysis {
            total_revenue: 15750.0,
            average_revenue_per_tenant: 5250.0,
            growth_rate: 0.15, // 15% growth
            top_features: vec![
                ("Agent Conversations".to_string(), 1250000),
                ("Data Storage".to_string(), 850),
                ("API Calls".to_string(), 2500000),
            ],
            cost_optimization_recommendations: vec![
                "实施智能缓存减少重复计算".to_string(),
                "优化存储使用，删除过期数据".to_string(),
                "考虑批量处理降低API调用成本".to_string(),
            ],
        })
    }
}

struct UsageTracker {
    usage_records: Arc<tokio::sync::Mutex<HashMap<String, HashMap<UsageMetric, u64>>>>,
}

impl UsageTracker {
    fn new() -> Self {
        Self {
            usage_records: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    async fn record_usage(&self, tenant_id: &str, metric: UsageMetric, amount: u64) -> Result<(), Box<dyn std::error::Error>> {
        let mut usage_records = self.usage_records.lock().await;
        let tenant_usage = usage_records.entry(tenant_id.to_string()).or_insert_with(HashMap::new);
        tenant_usage.insert(metric, amount);
        Ok(())
    }
}

// 简单的随机数生成器（用于演示）
mod rand {
    pub fn random<T>() -> T
    where
        T: From<u32>
    {
        T::from(12345) // 固定值用于演示
    }
}
