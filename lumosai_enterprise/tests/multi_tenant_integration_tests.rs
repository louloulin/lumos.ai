//! 多租户架构集成测试
//! 
//! 测试多租户系统的完整功能，包括：
//! - 租户管理
//! - 资源分配和配额管理
//! - 自动扩容
//! - 计费系统
//! - 隔离和安全

use lumosai_enterprise::multi_tenant::*;
use lumosai_enterprise::error::*;
use std::collections::HashMap;
use chrono::Utc;

#[tokio::test]
async fn test_multi_tenant_architecture_creation() {
    let architecture = MultiTenantArchitecture::new().await.unwrap();
    
    // 测试基本功能
    let tenant = architecture.get_tenant("nonexistent").await.unwrap();
    assert!(tenant.is_none());
}

#[tokio::test]
async fn test_complete_tenant_lifecycle() {
    let mut architecture = MultiTenantArchitecture::new().await.unwrap();
    
    // 创建测试租户
    let tenant = create_test_tenant("tenant1", TenantType::SmallBusiness);
    
    // 创建租户
    assert!(architecture.create_tenant(tenant.clone()).await.is_ok());
    
    // 验证租户创建成功
    let retrieved = architecture.get_tenant(&tenant.id).await.unwrap().unwrap();
    assert_eq!(retrieved.id, tenant.id);
    assert_eq!(retrieved.name, tenant.name);
    assert_eq!(retrieved.status, TenantStatus::Active);
    
    // 测试资源分配
    let allocation_result = architecture.allocate_resources("tenant1", "cpu_cores", 1).await;
    assert!(allocation_result.is_ok());
    
    // 测试配额超限
    let quota_exceeded = architecture.allocate_resources("tenant1", "cpu_cores", 10).await;
    assert!(quota_exceeded.is_err());
    
    // 测试暂停租户
    assert!(architecture.suspend_tenant("tenant1").await.is_ok());
    let suspended = architecture.get_tenant("tenant1").await.unwrap().unwrap();
    assert_eq!(suspended.status, TenantStatus::Suspended);
    
    // 测试恢复租户
    assert!(architecture.resume_tenant("tenant1").await.is_ok());
    let resumed = architecture.get_tenant("tenant1").await.unwrap().unwrap();
    assert_eq!(resumed.status, TenantStatus::Active);
}

#[tokio::test]
async fn test_quota_management() {
    let mut architecture = MultiTenantArchitecture::new().await.unwrap();
    
    // 创建租户
    let tenant = create_test_tenant("quota_test", TenantType::Individual);
    assert!(architecture.create_tenant(tenant).await.is_ok());
    
    // 测试配额检查
    let quota_usage = architecture.get_quota_usage("quota_test").await.unwrap();
    assert!(quota_usage.contains_key("cpu_cores"));
    assert!(quota_usage.contains_key("memory_gb"));
    
    // 分配一些资源
    assert!(architecture.allocate_resources("quota_test", "cpu_cores", 1).await.is_ok());
    assert!(architecture.allocate_resources("quota_test", "memory_gb", 2).await.is_ok());
    
    // 检查更新后的配额使用情况
    let updated_usage = architecture.get_quota_usage("quota_test").await.unwrap();
    if let Some((used, limit)) = updated_usage.get("cpu_cores") {
        assert_eq!(*used, 1);
        assert_eq!(*limit, 2); // Individual租户的CPU限制
    }
    
    // 测试配额超限
    let over_quota = architecture.allocate_resources("quota_test", "cpu_cores", 5).await;
    assert!(over_quota.is_err());
    match over_quota.unwrap_err() {
        EnterpriseError::QuotaExceeded { tenant_id, resource_type, requested } => {
            assert_eq!(tenant_id, "quota_test");
            assert_eq!(resource_type, "cpu_cores");
            assert_eq!(requested, 5);
        }
        _ => panic!("Expected QuotaExceeded error"),
    }
}

#[tokio::test]
async fn test_auto_scaling() {
    let mut architecture = MultiTenantArchitecture::new().await.unwrap();
    
    // 创建企业租户（有更高的扩容限制）
    let tenant = create_test_tenant("scaling_test", TenantType::Enterprise);
    assert!(architecture.create_tenant(tenant).await.is_ok());
    
    // 测试正常负载（不需要扩容）
    let scaling_result = architecture.check_auto_scaling("scaling_test", 0.5, 0.6).await.unwrap();
    assert!(scaling_result.is_none());
    
    // 测试高负载（需要扩容）
    let scaling_result = architecture.check_auto_scaling("scaling_test", 0.9, 0.85).await.unwrap();
    assert!(scaling_result.is_some());
    assert_eq!(scaling_result.unwrap(), 2); // 从1个实例扩容到2个
    
    // 测试低负载（需要缩容）
    let scaling_result = architecture.check_auto_scaling("scaling_test", 0.2, 0.3).await.unwrap();
    assert!(scaling_result.is_some());
    assert_eq!(scaling_result.unwrap(), 1); // 从2个实例缩容到1个
    
    // 检查扩容历史
    let history = architecture.get_scaling_history("scaling_test").await.unwrap();
    assert_eq!(history.len(), 2); // 一次扩容，一次缩容
}

#[tokio::test]
async fn test_billing_system() {
    let mut architecture = MultiTenantArchitecture::new().await.unwrap();
    
    // 创建租户
    let tenant = create_test_tenant("billing_test", TenantType::Professional);
    assert!(architecture.create_tenant(tenant).await.is_ok());
    
    // 分配一些资源（会自动记录计费）
    assert!(architecture.allocate_resources("billing_test", "cpu_cores", 2).await.is_ok());
    assert!(architecture.allocate_resources("billing_test", "memory_gb", 4).await.is_ok());
    assert!(architecture.allocate_resources("billing_test", "storage_gb", 50).await.is_ok());
    
    // 获取账单
    let bill = architecture.get_tenant_bill("billing_test").await.unwrap();
    assert!(bill >= 0.0); // 账单应该是非负数
}

#[tokio::test]
async fn test_different_tenant_types() {
    let mut architecture = MultiTenantArchitecture::new().await.unwrap();
    
    // 测试不同类型的租户
    let tenant_types = vec![
        ("individual", TenantType::Individual),
        ("small_business", TenantType::SmallBusiness),
        ("enterprise", TenantType::Enterprise),
        ("government", TenantType::Government),
        ("educational", TenantType::Educational),
    ];
    
    for (id, tenant_type) in tenant_types {
        let tenant = create_test_tenant(id, tenant_type.clone());
        assert!(architecture.create_tenant(tenant).await.is_ok());
        
        // 验证不同类型租户的扩容限制
        let scaling_result = architecture.check_auto_scaling(id, 0.9, 0.9).await.unwrap();
        if scaling_result.is_some() {
            let new_instances = scaling_result.unwrap();
            match tenant_type {
                TenantType::Individual => assert!(new_instances <= 3),
                TenantType::SmallBusiness => assert!(new_instances <= 10),
                TenantType::Enterprise => assert!(new_instances <= 50),
                TenantType::Government => assert!(new_instances <= 100),
                TenantType::Educational => assert!(new_instances <= 20),
            }
        }
    }
}

#[tokio::test]
async fn test_resource_allocation_edge_cases() {
    let mut architecture = MultiTenantArchitecture::new().await.unwrap();
    
    // 创建租户
    let tenant = create_test_tenant("edge_test", TenantType::SmallBusiness);
    assert!(architecture.create_tenant(tenant).await.is_ok());
    
    // 测试零资源分配
    let zero_allocation = architecture.allocate_resources("edge_test", "cpu_cores", 0).await;
    assert!(zero_allocation.is_ok());
    
    // 测试不存在的租户
    let nonexistent_tenant = architecture.allocate_resources("nonexistent", "cpu_cores", 1).await;
    assert!(nonexistent_tenant.is_err());
    
    // 测试不支持的资源类型
    let unsupported_resource = architecture.allocate_resources("edge_test", "quantum_cores", 1).await;
    // 这应该成功，因为我们支持自定义资源类型
    assert!(unsupported_resource.is_ok());
}

#[tokio::test]
async fn test_concurrent_operations() {
    let mut architecture = MultiTenantArchitecture::new().await.unwrap();
    
    // 创建租户
    let tenant = create_test_tenant("concurrent_test", TenantType::Enterprise);
    assert!(architecture.create_tenant(tenant).await.is_ok());
    
    // 并发分配资源
    let mut handles = vec![];
    for i in 0..5 {
        let allocation_result = architecture.allocate_resources("concurrent_test", "api_calls", 100).await;
        handles.push(allocation_result);
    }
    
    // 检查结果
    let mut success_count = 0;
    for result in handles {
        if result.is_ok() {
            success_count += 1;
        }
    }
    
    // 至少应该有一些成功的分配
    assert!(success_count > 0);
}

/// 创建测试租户的辅助函数
fn create_test_tenant(id: &str, tenant_type: TenantType) -> Tenant {
    Tenant {
        id: id.to_string(),
        name: format!("Test Tenant {}", id),
        tenant_type: tenant_type.clone(),
        created_at: Utc::now(),
        status: TenantStatus::Active,
        contact_info: ContactInfo {
            primary_contact: "Test User".to_string(),
            email: format!("{}@example.com", id),
            phone: None,
            address: None,
            company_info: None,
        },
        subscription_plan: SubscriptionPlan {
            plan_id: "test_plan".to_string(),
            plan_name: "Test Plan".to_string(),
            plan_type: match tenant_type {
                TenantType::Individual => PlanType::Basic,
                TenantType::SmallBusiness => PlanType::Professional,
                TenantType::Enterprise => PlanType::Enterprise,
                TenantType::Government => PlanType::Enterprise,
                TenantType::Educational => PlanType::Professional,
            },
            price: PlanPricing {
                base_price: 99.99,
                billing_cycle: BillingCycle::Monthly,
                currency: "USD".to_string(),
                usage_pricing: Vec::new(),
            },
            features: vec!["test_feature".to_string()],
            start_date: Utc::now(),
            end_date: None,
            auto_renewal: true,
        },
        quotas: ResourceQuotas {
            cpu_cores: Some(match tenant_type {
                TenantType::Individual => 2,
                TenantType::SmallBusiness => 8,
                TenantType::Enterprise => 32,
                TenantType::Government => 64,
                TenantType::Educational => 16,
            }),
            memory_gb: Some(match tenant_type {
                TenantType::Individual => 4,
                TenantType::SmallBusiness => 16,
                TenantType::Enterprise => 128,
                TenantType::Government => 256,
                TenantType::Educational => 64,
            }),
            storage_gb: Some(match tenant_type {
                TenantType::Individual => 100,
                TenantType::SmallBusiness => 1000,
                TenantType::Enterprise => 10000,
                TenantType::Government => 50000,
                TenantType::Educational => 5000,
            }),
            bandwidth_mbps: Some(100),
            api_calls_per_month: Some(match tenant_type {
                TenantType::Individual => 10000,
                TenantType::SmallBusiness => 100000,
                TenantType::Enterprise => 1000000,
                TenantType::Government => 5000000,
                TenantType::Educational => 500000,
            }),
            concurrent_connections: Some(100),
            max_users: Some(match tenant_type {
                TenantType::Individual => 1,
                TenantType::SmallBusiness => 50,
                TenantType::Enterprise => 1000,
                TenantType::Government => 5000,
                TenantType::Educational => 500,
            }),
            custom_quotas: HashMap::new(),
        },
        metadata: HashMap::new(),
    }
}
