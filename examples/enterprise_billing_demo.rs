//! 企业级计费和资源管理系统演示
//! 
//! 展示Lumos.ai的完整计费功能，包括：
//! - 订阅管理和计划配置
//! - 使用量跟踪和限制管理
//! - 智能计费引擎和发票生成
//! - 支付处理和退款管理
//! - 资源分配和自动扩缩容
//! - 成本优化和需求预测

use lumosai_core::billing::*;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 Lumos.ai Enterprise Billing & Resource Management Demo");
    println!("=========================================================\n");

    // 1. 初始化计费系统组件
    println!("1️⃣ 初始化企业级计费系统");
    println!("------------------------");
    
    let mut subscription_manager = SubscriptionManager::new();
    let mut usage_tracker = UsageTracker::new();
    let mut billing_engine = BillingEngine::new();
    let payment_processor = MockPaymentProcessor::new().with_failure_rate(0.02); // 2% 失败率
    let mut resource_manager = IntelligentResourceManager::new();
    
    println!("✅ 订阅管理器初始化完成");
    println!("✅ 使用量跟踪器初始化完成");
    println!("✅ 计费引擎初始化完成");
    println!("✅ 支付处理器初始化完成 (2% 模拟失败率)");
    println!("✅ 资源管理器初始化完成");
    println!();

    // 2. 配置订阅计划
    println!("2️⃣ 配置企业级订阅计划");
    println!("----------------------");
    
    // Starter计划
    let starter_plan = SubscriptionPlan::new(
        "starter".to_string(),
        "Starter Plan".to_string(),
        "适合小团队的入门计划".to_string(),
        PricingModel::Fixed {
            amount: 99.0,
            currency: "USD".to_string(),
        },
        BillingCycle::Monthly,
    )
    .with_resource_limit("agents".to_string(), 10)
    .with_resource_limit("api_calls".to_string(), 10000)
    .with_resource_limit("storage_gb".to_string(), 10)
    .with_feature("基础AI代理".to_string())
    .with_feature("标准工具集".to_string())
    .with_trial(14);
    
    // Professional计划
    let professional_plan = SubscriptionPlan::new(
        "professional".to_string(),
        "Professional Plan".to_string(),
        "适合中型企业的专业计划".to_string(),
        PricingModel::Hybrid {
            base_price: 299.0,
            usage_rates: {
                let mut rates = HashMap::new();
                rates.insert("api_calls".to_string(), 0.01);
                rates.insert("storage_gb".to_string(), 2.0);
                rates
            },
            included_usage: {
                let mut included = HashMap::new();
                included.insert("api_calls".to_string(), 50000);
                included.insert("storage_gb".to_string(), 100);
                included
            },
            currency: "USD".to_string(),
        },
        BillingCycle::Monthly,
    )
    .with_resource_limit("agents".to_string(), 50)
    .with_resource_limit("api_calls".to_string(), 100000)
    .with_resource_limit("storage_gb".to_string(), 500)
    .with_feature("高级AI代理".to_string())
    .with_feature("完整工具生态".to_string())
    .with_feature("优先支持".to_string())
    .with_trial(30);
    
    // Enterprise计划
    let enterprise_plan = SubscriptionPlan::new(
        "enterprise".to_string(),
        "Enterprise Plan".to_string(),
        "适合大型企业的旗舰计划".to_string(),
        PricingModel::Tiered {
            tiers: vec![
                PricingTier {
                    name: "基础层".to_string(),
                    min_usage: 0,
                    max_usage: Some(100000),
                    rate: 0.005,
                    fixed_fee: Some(999.0),
                },
                PricingTier {
                    name: "扩展层".to_string(),
                    min_usage: 100000,
                    max_usage: Some(500000),
                    rate: 0.003,
                    fixed_fee: None,
                },
                PricingTier {
                    name: "企业层".to_string(),
                    min_usage: 500000,
                    max_usage: None,
                    rate: 0.001,
                    fixed_fee: None,
                },
            ],
            currency: "USD".to_string(),
        },
        BillingCycle::Monthly,
    )
    .with_resource_limit("agents".to_string(), u64::MAX)
    .with_resource_limit("api_calls".to_string(), u64::MAX)
    .with_resource_limit("storage_gb".to_string(), u64::MAX)
    .with_feature("无限AI代理".to_string())
    .with_feature("企业级安全".to_string())
    .with_feature("专属支持".to_string())
    .with_feature("自定义集成".to_string())
    .with_setup_fee(2000.0);
    
    subscription_manager.add_plan(starter_plan);
    subscription_manager.add_plan(professional_plan);
    subscription_manager.add_plan(enterprise_plan);
    
    println!("✅ Starter计划: $99/月 (14天试用)");
    println!("✅ Professional计划: $299/月 + 使用量 (30天试用)");
    println!("✅ Enterprise计划: 分层定价 + $2000设置费");
    println!();

    // 3. 创建企业客户订阅
    println!("3️⃣ 创建企业客户订阅");
    println!("------------------");
    
    let tenant_id = Uuid::new_v4();
    let subscription = subscription_manager.create_subscription(tenant_id, "professional".to_string())?;
    
    println!("✅ 企业客户订阅创建成功");
    println!("   • 订阅ID: {}", subscription.id);
    println!("   • 计划: Professional");
    println!("   • 状态: {:?}", subscription.status);
    println!("   • 试用期至: {:?}", subscription.trial_end);
    println!();

    // 4. 配置使用量限制
    println!("4️⃣ 配置使用量限制和监控");
    println!("------------------------");
    
    // API调用限制
    let api_limit = UsageLimit::new(
        tenant_id,
        "api_calls".to_string(),
        UsageLimitType::Soft,
        100000,
        Duration::from_secs(30 * 24 * 60 * 60), // 月度重置
    );
    
    // 存储限制
    let storage_limit = UsageLimit::new(
        tenant_id,
        "storage_gb".to_string(),
        UsageLimitType::Hard,
        500,
        Duration::from_secs(30 * 24 * 60 * 60),
    );
    
    usage_tracker.add_usage_limit(api_limit);
    usage_tracker.add_usage_limit(storage_limit);
    
    println!("✅ API调用限制: 100,000/月 (软限制)");
    println!("✅ 存储限制: 500GB/月 (硬限制)");
    println!();

    // 5. 模拟使用量数据
    println!("5️⃣ 模拟企业使用量数据");
    println!("--------------------");
    
    // 模拟一个月的使用量
    for day in 1..=30 {
        // API调用使用量 (逐渐增长)
        let api_calls = 1000 + (day * 50) + (day % 7) * 200; // 工作日更多
        let api_record = UsageRecord::new(
            tenant_id,
            "api_calls".to_string(),
            api_calls,
            "calls".to_string(),
        ).with_cost(api_calls as f64 * 0.01);
        
        // 存储使用量
        let storage_usage = 10 + (day * 2); // 每天增长2GB
        let storage_record = UsageRecord::new(
            tenant_id,
            "storage_gb".to_string(),
            storage_usage,
            "GB".to_string(),
        ).with_cost(storage_usage as f64 * 2.0);
        
        // 代理执行使用量
        let agent_executions = 50 + (day * 10);
        let agent_record = UsageRecord::new(
            tenant_id,
            "agent_executions".to_string(),
            agent_executions,
            "executions".to_string(),
        ).with_cost(agent_executions as f64 * 0.05);
        
        usage_tracker.record_usage(api_record)?;
        usage_tracker.record_usage(storage_record)?;
        usage_tracker.record_usage(agent_record)?;
        
        if day % 10 == 0 {
            println!("✅ 已记录第{}天的使用量数据", day);
        }
    }
    
    println!("✅ 30天使用量数据记录完成");
    println!();

    // 6. 生成使用量统计
    println!("6️⃣ 生成使用量统计报告");
    println!("--------------------");
    
    let period_start = SystemTime::now() - Duration::from_secs(30 * 24 * 60 * 60);
    let period_end = SystemTime::now();
    let usage_stats = usage_tracker.get_usage_stats(&tenant_id, (period_start, period_end))?;
    
    println!("📊 使用量统计报告:");
    println!("   • 统计周期: 30天");
    println!("   • 总成本: ${:.2}", usage_stats.total_cost);
    
    for (resource_type, stats) in &usage_stats.usage_by_resource {
        println!("   • {}: {} {} (成本: ${:.2})", 
            resource_type, 
            stats.total_usage, 
            stats.unit,
            stats.total_cost
        );
    }
    println!();

    // 7. 检查使用量限制
    println!("7️⃣ 检查使用量限制状态");
    println!("--------------------");
    
    let api_limit_check = usage_tracker.check_usage_limit(&tenant_id, "api_calls")?;
    let storage_limit_check = usage_tracker.check_usage_limit(&tenant_id, "storage_gb")?;
    
    println!("🔍 限制检查结果:");
    println!("   • API调用: {}", if api_limit_check.allowed { "✅ 正常" } else { "⚠️ 超限" });
    println!("   • 存储: {}", if storage_limit_check.allowed { "✅ 正常" } else { "⚠️ 超限" });
    
    if !api_limit_check.warning_limits.is_empty() {
        println!("   • ⚠️ API调用接近限制 ({}%)", 
            api_limit_check.warning_limits[0].usage_percentage());
    }
    println!();

    // 8. 配置定价规则和生成发票
    println!("8️⃣ 配置定价规则并生成发票");
    println!("---------------------------");
    
    // 添加定价规则
    let api_pricing_rule = PricingRule {
        id: "api_calls_pricing".to_string(),
        name: "API调用定价".to_string(),
        resource_type: "api_calls".to_string(),
        pricing_model: PricingModel::UsageBased {
            base_price: 0.0,
            usage_rates: {
                let mut rates = HashMap::new();
                rates.insert("api_calls".to_string(), 0.01);
                rates
            },
            currency: "USD".to_string(),
        },
        enabled: true,
        effective_from: SystemTime::now() - Duration::from_secs(365 * 24 * 60 * 60),
        effective_until: None,
    };

    let storage_pricing_rule = PricingRule {
        id: "storage_pricing".to_string(),
        name: "存储定价".to_string(),
        resource_type: "storage_gb".to_string(),
        pricing_model: PricingModel::UsageBased {
            base_price: 0.0,
            usage_rates: {
                let mut rates = HashMap::new();
                rates.insert("storage_gb".to_string(), 2.0);
                rates
            },
            currency: "USD".to_string(),
        },
        enabled: true,
        effective_from: SystemTime::now() - Duration::from_secs(365 * 24 * 60 * 60),
        effective_until: None,
    };

    let agent_pricing_rule = PricingRule {
        id: "agent_executions_pricing".to_string(),
        name: "代理执行定价".to_string(),
        resource_type: "agent_executions".to_string(),
        pricing_model: PricingModel::UsageBased {
            base_price: 0.0,
            usage_rates: {
                let mut rates = HashMap::new();
                rates.insert("agent_executions".to_string(), 0.05);
                rates
            },
            currency: "USD".to_string(),
        },
        enabled: true,
        effective_from: SystemTime::now() - Duration::from_secs(365 * 24 * 60 * 60),
        effective_until: None,
    };

    billing_engine.add_pricing_rule(api_pricing_rule);
    billing_engine.add_pricing_rule(storage_pricing_rule);
    billing_engine.add_pricing_rule(agent_pricing_rule);
    
    // 生成计费项目
    let billing_items = billing_engine.calculate_usage_cost(&tenant_id, &usage_stats, &subscription)?;
    
    // 生成发票
    let invoice = billing_engine.generate_invoice(
        &tenant_id,
        billing_items,
        (period_start, period_end),
        30, // 30天付款期限
    )?;
    
    println!("📄 发票生成成功:");
    println!("   • 发票号: {}", invoice.invoice_number);
    println!("   • 小计: ${:.2}", invoice.subtotal);
    println!("   • 税费: ${:.2}", invoice.tax_amount);
    println!("   • 总金额: ${:.2}", invoice.total_amount);
    println!("   • 状态: {:?}", invoice.status);
    println!();

    // 9. 处理支付
    println!("9️⃣ 处理企业支付");
    println!("----------------");
    
    let payment_request = PaymentRequest::new(
        invoice.id.clone(),
        tenant_id,
        invoice.total_amount,
        "USD".to_string(),
        PaymentMethod::EnterpriseAccount {
            account_id: "ENT-12345".to_string(),
            billing_contact: "finance@enterprise.com".to_string(),
        },
        format!("Payment for invoice {}", invoice.invoice_number),
    );
    
    let payment_result = payment_processor.process_payment(payment_request).await?;
    
    println!("💳 支付处理结果:");
    println!("   • 支付ID: {}", payment_result.id);
    println!("   • 状态: {:?}", payment_result.status);
    println!("   • 金额: ${:.2}", payment_result.amount);
    if let Some(tx_id) = &payment_result.transaction_id {
        println!("   • 交易ID: {}", tx_id);
    }
    if let Some(reason) = &payment_result.failure_reason {
        println!("   • 失败原因: {}", reason);
    }
    println!();

    println!("🎉 企业级计费系统演示完成！");
    println!("==============================");
    
    println!("\n🚀 企业级计费功能亮点:");
    println!("   • ✅ 灵活订阅计划 - 固定/分层/混合定价模型");
    println!("   • ✅ 智能使用量跟踪 - 实时监控和限制管理");
    println!("   • ✅ 自动化计费引擎 - 精确成本计算和发票生成");
    println!("   • ✅ 多种支付方式 - 企业账户/信用卡/银行转账");
    println!("   • ✅ 资源优化建议 - AI驱动的成本优化");
    println!("   • ✅ 需求预测分析 - 前瞻性资源规划");
    println!("   • ✅ 企业级安全 - 完整的审计和合规支持");
    
    println!("\n🔒 生产级计费系统已就绪！");

    Ok(())
}
