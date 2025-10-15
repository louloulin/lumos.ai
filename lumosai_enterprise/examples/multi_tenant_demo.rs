//! 多租户架构演示
//!
//! 展示企业级多租户功能的完整使用示例

use chrono::Utc;
use lumosai_enterprise::error::*;
use lumosai_enterprise::multi_tenant::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🏢 Lumos.ai 企业级多租户架构演示");
    println!("=".repeat(50));

    // 创建多租户架构
    let mut architecture = MultiTenantArchitecture::new().await?;
    println!("✅ 多租户架构初始化完成");

    // 演示不同类型的租户
    demo_tenant_types(&mut architecture).await?;

    // 演示资源分配和配额管理
    demo_resource_management(&mut architecture).await?;

    // 演示自动扩容
    demo_auto_scaling(&mut architecture).await?;

    // 演示计费系统
    demo_billing_system(&mut architecture).await?;

    // 演示租户管理
    demo_tenant_management(&mut architecture).await?;

    println!("\n🎉 多租户架构演示完成！");
    Ok(())
}

/// 演示不同类型的租户
async fn demo_tenant_types(architecture: &mut MultiTenantArchitecture) -> Result<()> {
    println!("\n📋 演示：不同类型租户创建");
    println!("-".repeat(30));

    let tenant_types = vec![
        ("startup_inc", TenantType::SmallBusiness, "初创公司"),
        ("enterprise_corp", TenantType::Enterprise, "大型企业"),
        ("university", TenantType::Educational, "教育机构"),
        ("government_dept", TenantType::Government, "政府部门"),
        ("individual_dev", TenantType::Individual, "个人开发者"),
    ];

    for (id, tenant_type, description) in tenant_types {
        let tenant = create_demo_tenant(id, tenant_type.clone(), description);

        match architecture.create_tenant(tenant.clone()).await {
            Ok(_) => {
                println!("✅ 创建租户: {} ({})", tenant.name, description);

                // 显示租户配额信息
                if let Some(cpu_cores) = tenant.quotas.cpu_cores {
                    println!("   📊 CPU配额: {} 核心", cpu_cores);
                }
                if let Some(memory_gb) = tenant.quotas.memory_gb {
                    println!("   💾 内存配额: {} GB", memory_gb);
                }
                if let Some(api_calls) = tenant.quotas.api_calls_per_month {
                    println!("   🔄 API调用配额: {} 次/月", api_calls);
                }
            }
            Err(e) => println!("❌ 创建租户失败: {}", e),
        }
    }

    Ok(())
}

/// 演示资源分配和配额管理
async fn demo_resource_management(architecture: &mut MultiTenantArchitecture) -> Result<()> {
    println!("\n🔧 演示：资源分配和配额管理");
    println!("-".repeat(30));

    let tenant_id = "startup_inc";

    // 正常资源分配
    println!("📦 为租户 {} 分配资源:", tenant_id);

    let resources = vec![("cpu_cores", 2), ("memory_gb", 4), ("storage_gb", 100)];

    for (resource_type, amount) in resources {
        match architecture
            .allocate_resources(tenant_id, resource_type, amount)
            .await
        {
            Ok(allocation_id) => {
                println!(
                    "  ✅ 分配 {} {}: 分配ID {}",
                    amount, resource_type, allocation_id
                );
            }
            Err(e) => {
                println!("  ❌ 分配失败: {}", e);
            }
        }
    }

    // 尝试超配额分配
    println!("\n🚫 尝试超配额分配:");
    match architecture
        .allocate_resources(tenant_id, "cpu_cores", 10)
        .await
    {
        Ok(_) => println!("  ⚠️  意外成功（不应该发生）"),
        Err(e) => println!("  ✅ 正确拒绝: {}", e),
    }

    // 显示配额使用情况
    match architecture.get_quota_usage(tenant_id).await {
        Ok(usage) => {
            println!("\n📊 配额使用情况:");
            for (resource, (used, limit)) in usage {
                let percentage = if limit > 0 {
                    (used as f64 / limit as f64) * 100.0
                } else {
                    0.0
                };
                println!("  {} {}/{} ({:.1}%)", resource, used, limit, percentage);
            }
        }
        Err(e) => println!("❌ 获取配额使用情况失败: {}", e),
    }

    Ok(())
}

/// 演示自动扩容
async fn demo_auto_scaling(architecture: &mut MultiTenantArchitecture) -> Result<()> {
    println!("\n📈 演示：自动扩容");
    println!("-".repeat(30));

    let tenant_id = "enterprise_corp";

    // 模拟不同负载情况
    let load_scenarios = vec![
        (0.5, 0.6, "正常负载"),
        (0.9, 0.85, "高负载（需要扩容）"),
        (0.2, 0.3, "低负载（可以缩容）"),
        (0.95, 0.9, "极高负载（紧急扩容）"),
    ];

    for (cpu_usage, memory_usage, description) in load_scenarios {
        println!(
            "\n🔍 检查扩容: {} (CPU: {:.1}%, Memory: {:.1}%)",
            description,
            cpu_usage * 100.0,
            memory_usage * 100.0
        );

        match architecture
            .check_auto_scaling(tenant_id, cpu_usage, memory_usage)
            .await
        {
            Ok(Some(new_instances)) => {
                println!("  📊 扩容决策: 调整到 {} 个实例", new_instances);
            }
            Ok(None) => {
                println!("  ⚖️  无需调整实例数");
            }
            Err(e) => {
                println!("  ❌ 扩容检查失败: {}", e);
            }
        }
    }

    // 显示扩容历史
    match architecture.get_scaling_history(tenant_id).await {
        Ok(history) => {
            if !history.is_empty() {
                println!("\n📜 扩容历史:");
                for event in history.iter().take(5) {
                    // 只显示最近5次
                    println!(
                        "  {} {} -> {} 实例 ({})",
                        event.timestamp.format("%H:%M:%S"),
                        event.from_instances,
                        event.to_instances,
                        event.reason
                    );
                }
            }
        }
        Err(e) => println!("❌ 获取扩容历史失败: {}", e),
    }

    Ok(())
}

/// 演示计费系统
async fn demo_billing_system(architecture: &mut MultiTenantArchitecture) -> Result<()> {
    println!("\n💰 演示：计费系统");
    println!("-".repeat(30));

    let tenant_id = "startup_inc";

    // 获取当前账单
    match architecture.get_tenant_bill(tenant_id).await {
        Ok(bill) => {
            println!("📄 租户 {} 当前账单: ${:.2}", tenant_id, bill);

            if bill > 0.0 {
                println!("  💡 账单包含之前分配的资源使用费用");
            } else {
                println!("  💡 当前无费用产生");
            }
        }
        Err(e) => println!("❌ 获取账单失败: {}", e),
    }

    Ok(())
}

/// 演示租户管理
async fn demo_tenant_management(architecture: &mut MultiTenantArchitecture) -> Result<()> {
    println!("\n👥 演示：租户管理操作");
    println!("-".repeat(30));

    let tenant_id = "individual_dev";

    // 暂停租户
    println!("⏸️  暂停租户: {}", tenant_id);
    match architecture.suspend_tenant(tenant_id).await {
        Ok(_) => {
            println!("  ✅ 租户已暂停");

            // 检查租户状态
            if let Ok(Some(tenant)) = architecture.get_tenant(tenant_id).await {
                println!("  📊 当前状态: {:?}", tenant.status);
            }
        }
        Err(e) => println!("  ❌ 暂停失败: {}", e),
    }

    // 恢复租户
    println!("\n▶️  恢复租户: {}", tenant_id);
    match architecture.resume_tenant(tenant_id).await {
        Ok(_) => {
            println!("  ✅ 租户已恢复");

            // 检查租户状态
            if let Ok(Some(tenant)) = architecture.get_tenant(tenant_id).await {
                println!("  📊 当前状态: {:?}", tenant.status);
            }
        }
        Err(e) => println!("  ❌ 恢复失败: {}", e),
    }

    Ok(())
}

/// 创建演示租户的辅助函数
fn create_demo_tenant(id: &str, tenant_type: TenantType, description: &str) -> Tenant {
    Tenant {
        id: id.to_string(),
        name: format!("{} ({})", description, id),
        tenant_type: tenant_type.clone(),
        created_at: Utc::now(),
        status: TenantStatus::Active,
        contact_info: ContactInfo {
            primary_contact: format!("Contact for {}", description),
            email: format!("{}@example.com", id),
            phone: None,
            address: None,
            company_info: Some(CompanyInfo {
                name: description.to_string(),
                industry: "Technology".to_string(),
                employee_count: Some(match tenant_type {
                    TenantType::Individual => 1,
                    TenantType::SmallBusiness => 25,
                    TenantType::Enterprise => 1000,
                    TenantType::Government => 500,
                    TenantType::Educational => 200,
                }),
                annual_revenue: None,
            }),
        },
        subscription_plan: SubscriptionPlan {
            plan_id: format!("{}_plan", id),
            plan_name: match tenant_type {
                TenantType::Individual => "个人版",
                TenantType::SmallBusiness => "商业版",
                TenantType::Enterprise => "企业版",
                TenantType::Government => "政府版",
                TenantType::Educational => "教育版",
            }
            .to_string(),
            plan_type: match tenant_type {
                TenantType::Individual => PlanType::Basic,
                TenantType::SmallBusiness => PlanType::Professional,
                TenantType::Enterprise => PlanType::Enterprise,
                TenantType::Government => PlanType::Enterprise,
                TenantType::Educational => PlanType::Professional,
            },
            price: PlanPricing {
                base_price: match tenant_type {
                    TenantType::Individual => 9.99,
                    TenantType::SmallBusiness => 49.99,
                    TenantType::Enterprise => 199.99,
                    TenantType::Government => 299.99,
                    TenantType::Educational => 29.99,
                },
                billing_cycle: BillingCycle::Monthly,
                currency: "USD".to_string(),
                usage_pricing: Vec::new(),
            },
            features: vec![
                "基础功能".to_string(),
                match tenant_type {
                    TenantType::Individual => "个人支持",
                    TenantType::SmallBusiness => "商业支持",
                    TenantType::Enterprise => "企业级支持",
                    TenantType::Government => "政府级安全",
                    TenantType::Educational => "教育折扣",
                }
                .to_string(),
            ],
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
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("demo".to_string(), "true".to_string());
            metadata.insert("created_by".to_string(), "multi_tenant_demo".to_string());
            metadata
        },
    }
}
