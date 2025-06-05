//! 企业级计费系统单元测试
//! 
//! 测试计费系统的核心功能，包括订阅管理、使用量跟踪、计费引擎和支付处理

use super::*;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

#[cfg(test)]
mod subscription_tests {
    use super::*;

    #[test]
    fn test_subscription_plan_creation() {
        let plan = SubscriptionPlan::new(
            "test_plan".to_string(),
            "Test Plan".to_string(),
            "A test subscription plan".to_string(),
            PricingModel::Fixed {
                amount: 99.0,
                currency: "USD".to_string(),
            },
            BillingCycle::Monthly,
        )
        .with_resource_limit("api_calls".to_string(), 10000)
        .with_feature("Basic AI Agents".to_string())
        .with_trial(14);

        assert_eq!(plan.id, "test_plan");
        assert_eq!(plan.name, "Test Plan");
        assert_eq!(plan.get_base_price(), 99.0);
        assert_eq!(plan.trial_days, Some(14));
        assert_eq!(plan.resource_limits.get("api_calls"), Some(&10000));
        assert!(plan.features.contains(&"Basic AI Agents".to_string()));
    }

    #[test]
    fn test_subscription_creation() {
        let plan = SubscriptionPlan::new(
            "test_plan".to_string(),
            "Test Plan".to_string(),
            "A test subscription plan".to_string(),
            PricingModel::Fixed {
                amount: 99.0,
                currency: "USD".to_string(),
            },
            BillingCycle::Monthly,
        ).with_trial(14);

        let tenant_id = Uuid::new_v4();
        let subscription = Subscription::new(tenant_id, "test_plan".to_string(), &plan);

        assert_eq!(subscription.tenant_id, tenant_id);
        assert_eq!(subscription.plan_id, "test_plan");
        assert_eq!(subscription.status, SubscriptionStatus::Trial);
        assert!(subscription.is_in_trial());
        assert!(!subscription.is_expired());
    }

    #[test]
    fn test_subscription_cancellation() {
        let plan = SubscriptionPlan::new(
            "test_plan".to_string(),
            "Test Plan".to_string(),
            "A test subscription plan".to_string(),
            PricingModel::Fixed {
                amount: 99.0,
                currency: "USD".to_string(),
            },
            BillingCycle::Monthly,
        );

        let tenant_id = Uuid::new_v4();
        let mut subscription = Subscription::new(tenant_id, "test_plan".to_string(), &plan);

        // Test immediate cancellation
        subscription.cancel(false);
        assert_eq!(subscription.status, SubscriptionStatus::Cancelled);
        assert!(subscription.cancelled_at.is_some());

        // Test cancellation at period end
        let mut subscription2 = Subscription::new(tenant_id, "test_plan".to_string(), &plan);
        subscription2.cancel(true);
        assert_eq!(subscription2.status, SubscriptionStatus::Active);
        assert!(subscription2.cancel_at_period_end);
        assert!(subscription2.cancelled_at.is_some());
    }

    #[test]
    fn test_subscription_manager() {
        let mut manager = SubscriptionManager::new();
        
        let plan = SubscriptionPlan::new(
            "test_plan".to_string(),
            "Test Plan".to_string(),
            "A test subscription plan".to_string(),
            PricingModel::Fixed {
                amount: 99.0,
                currency: "USD".to_string(),
            },
            BillingCycle::Monthly,
        );

        manager.add_plan(plan);

        let tenant_id = Uuid::new_v4();
        let subscription = manager.create_subscription(tenant_id, "test_plan".to_string()).unwrap();

        assert_eq!(subscription.tenant_id, tenant_id);
        assert_eq!(subscription.plan_id, "test_plan");

        // Test getting tenant subscription
        let retrieved = manager.get_tenant_subscription(&tenant_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().tenant_id, tenant_id);
    }
}

#[cfg(test)]
mod usage_tracking_tests {
    use super::*;

    #[test]
    fn test_usage_record_creation() {
        let tenant_id = Uuid::new_v4();
        let record = UsageRecord::new(
            tenant_id,
            "api_calls".to_string(),
            100,
            "calls".to_string(),
        ).with_cost(10.0);

        assert_eq!(record.tenant_id, tenant_id);
        assert_eq!(record.resource_type, "api_calls");
        assert_eq!(record.quantity, 100);
        assert_eq!(record.unit, "calls");
        assert_eq!(record.cost, Some(10.0));
    }

    #[test]
    fn test_usage_limit() {
        let tenant_id = Uuid::new_v4();
        let mut limit = UsageLimit::new(
            tenant_id,
            "api_calls".to_string(),
            UsageLimitType::Hard,
            1000,
            Duration::from_secs(30 * 24 * 60 * 60), // 30 days
        );

        assert_eq!(limit.tenant_id, tenant_id);
        assert_eq!(limit.resource_type, "api_calls");
        assert_eq!(limit.limit_value, 1000);
        assert!(!limit.is_exceeded());
        assert!(!limit.is_warning_threshold_reached());

        // Test usage tracking
        limit.current_usage = 500;
        assert!(!limit.is_exceeded());
        assert!(!limit.is_warning_threshold_reached());

        limit.current_usage = 850;
        assert!(!limit.is_exceeded());
        assert!(limit.is_warning_threshold_reached());

        limit.current_usage = 1100;
        assert!(limit.is_exceeded());
        assert!(limit.is_warning_threshold_reached());

        // Test usage percentage
        limit.current_usage = 750;
        assert_eq!(limit.usage_percentage(), 75.0);
    }

    #[test]
    fn test_usage_tracker() {
        let mut tracker = UsageTracker::new();
        let tenant_id = Uuid::new_v4();

        // Add usage limit
        let limit = UsageLimit::new(
            tenant_id,
            "api_calls".to_string(),
            UsageLimitType::Soft,
            1000,
            Duration::from_secs(30 * 24 * 60 * 60),
        );
        tracker.add_usage_limit(limit);

        // Record usage
        let record = UsageRecord::new(
            tenant_id,
            "api_calls".to_string(),
            100,
            "calls".to_string(),
        ).with_cost(10.0);

        tracker.record_usage(record).unwrap();

        // Check usage limit
        let limit_check = tracker.check_usage_limit(&tenant_id, "api_calls").unwrap();
        assert!(limit_check.allowed);
        assert!(limit_check.exceeded_limits.is_empty());
        assert!(limit_check.warning_limits.is_empty());

        // Record more usage to trigger warning
        for _ in 0..8 {
            let record = UsageRecord::new(
                tenant_id,
                "api_calls".to_string(),
                100,
                "calls".to_string(),
            ).with_cost(10.0);
            tracker.record_usage(record).unwrap();
        }

        let limit_check = tracker.check_usage_limit(&tenant_id, "api_calls").unwrap();
        assert!(limit_check.allowed); // Soft limit allows operation
        assert!(limit_check.exceeded_limits.is_empty());
        assert!(!limit_check.warning_limits.is_empty()); // Should trigger warning
    }

    #[test]
    fn test_usage_stats_generation() {
        let mut tracker = UsageTracker::new();
        let tenant_id = Uuid::new_v4();

        // Record multiple usage entries
        for i in 1..=10 {
            let record = UsageRecord::new(
                tenant_id,
                "api_calls".to_string(),
                i * 10,
                "calls".to_string(),
            ).with_cost(i as f64);
            tracker.record_usage(record).unwrap();
        }

        let period = (
            SystemTime::now() - Duration::from_secs(24 * 60 * 60),
            SystemTime::now(),
        );

        let stats = tracker.get_usage_stats(&tenant_id, period).unwrap();
        assert_eq!(stats.tenant_id, tenant_id);
        assert!(stats.usage_by_resource.contains_key("api_calls"));

        let api_stats = &stats.usage_by_resource["api_calls"];
        assert_eq!(api_stats.resource_type, "api_calls");
        assert_eq!(api_stats.total_usage, 550); // Sum of 10+20+...+100
        assert_eq!(api_stats.usage_count, 10);
        assert_eq!(api_stats.peak_usage, 100);
        assert_eq!(api_stats.total_cost, 55.0); // Sum of 1+2+...+10
    }
}

#[cfg(test)]
mod billing_engine_tests {
    use super::*;

    #[test]
    fn test_pricing_rule_creation() {
        let rule = PricingRule {
            id: "api_pricing".to_string(),
            name: "API Pricing".to_string(),
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
            effective_from: SystemTime::now() - Duration::from_secs(24 * 60 * 60),
            effective_until: None,
        };

        assert_eq!(rule.resource_type, "api_calls");
        assert!(rule.enabled);
    }

    #[test]
    fn test_billing_engine_cost_calculation() {
        let mut engine = BillingEngine::new();
        
        // Add pricing rule
        let rule = PricingRule {
            id: "api_pricing".to_string(),
            name: "API Pricing".to_string(),
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
            effective_from: SystemTime::now() - Duration::from_secs(24 * 60 * 60),
            effective_until: None,
        };
        engine.add_pricing_rule(rule);

        // Create usage stats
        let tenant_id = Uuid::new_v4();
        let mut usage_by_resource = HashMap::new();
        usage_by_resource.insert("api_calls".to_string(), ResourceUsageStats {
            resource_type: "api_calls".to_string(),
            total_usage: 1000,
            average_usage: 100.0,
            peak_usage: 200,
            usage_count: 10,
            total_cost: 0.0,
            average_cost: 0.0,
            unit: "calls".to_string(),
            daily_usage: Vec::new(),
        });

        let usage_stats = UsageStats {
            tenant_id,
            period: (SystemTime::now() - Duration::from_secs(24 * 60 * 60), SystemTime::now()),
            usage_by_resource,
            total_cost: 0.0,
            currency: "USD".to_string(),
            generated_at: SystemTime::now(),
        };

        // Create subscription
        let subscription = Subscription::new(
            tenant_id,
            "test_plan".to_string(),
            &SubscriptionPlan::new(
                "test_plan".to_string(),
                "Test Plan".to_string(),
                "Test".to_string(),
                PricingModel::Fixed { amount: 99.0, currency: "USD".to_string() },
                BillingCycle::Monthly,
            ),
        );

        // Calculate cost
        let billing_items = engine.calculate_usage_cost(&tenant_id, &usage_stats, &subscription).unwrap();
        assert_eq!(billing_items.len(), 1);
        assert_eq!(billing_items[0].resource_type, "api_calls");
        assert_eq!(billing_items[0].total_amount, 10.0); // 1000 * 0.01
    }

    #[test]
    fn test_invoice_generation() {
        let mut engine = BillingEngine::new();
        let tenant_id = Uuid::new_v4();

        let billing_items = vec![
            BillingItem {
                id: Uuid::new_v4().to_string(),
                name: "API Calls".to_string(),
                description: "API usage".to_string(),
                resource_type: "api_calls".to_string(),
                quantity: 1000,
                unit_price: 0.01,
                total_amount: 10.0,
                billing_period: (SystemTime::now() - Duration::from_secs(24 * 60 * 60), SystemTime::now()),
                currency: "USD".to_string(),
                metadata: HashMap::new(),
            }
        ];

        let invoice = engine.generate_invoice(
            &tenant_id,
            billing_items,
            (SystemTime::now() - Duration::from_secs(24 * 60 * 60), SystemTime::now()),
            30,
        ).unwrap();

        assert_eq!(invoice.tenant_id, tenant_id);
        assert_eq!(invoice.subtotal, 10.0);
        assert_eq!(invoice.tax_amount, 1.0); // 10% tax
        assert_eq!(invoice.total_amount, 11.0);
        assert_eq!(invoice.status, InvoiceStatus::Pending);
        assert_eq!(invoice.items.len(), 1);
    }
}

#[cfg(test)]
mod payment_processor_tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_mock_payment_processor() {
        let processor = MockPaymentProcessor::new().with_failure_rate(0.0); // No failures for test

        let payment_request = PaymentRequest::new(
            "invoice_123".to_string(),
            Uuid::new_v4(),
            100.0,
            "USD".to_string(),
            PaymentMethod::CreditCard {
                card_number: "4111111111111111".to_string(),
                expiry_month: 12,
                expiry_year: 2025,
                cardholder_name: "Test User".to_string(),
            },
            "Test payment".to_string(),
        );

        let result = processor.process_payment(payment_request).await.unwrap();
        assert_eq!(result.status, PaymentStatus::Succeeded);
        assert_eq!(result.amount, 100.0);
        assert!(result.transaction_id.is_some());
    }

    #[tokio::test]
    async fn test_payment_method_validation() {
        let processor = MockPaymentProcessor::new();

        // Valid credit card
        let valid_card = PaymentMethod::CreditCard {
            card_number: "4111111111111111".to_string(),
            expiry_month: 12,
            expiry_year: 2025,
            cardholder_name: "Test User".to_string(),
        };
        assert!(processor.validate_payment_method(&valid_card).await.unwrap());

        // Invalid credit card (expired)
        let expired_card = PaymentMethod::CreditCard {
            card_number: "4111111111111111".to_string(),
            expiry_month: 12,
            expiry_year: 2020,
            cardholder_name: "Test User".to_string(),
        };
        assert!(!processor.validate_payment_method(&expired_card).await.unwrap());

        // Valid bank transfer
        let bank_transfer = PaymentMethod::BankTransfer {
            account_number: "123456789".to_string(),
            routing_number: "987654321".to_string(),
            bank_name: "Test Bank".to_string(),
        };
        assert!(processor.validate_payment_method(&bank_transfer).await.unwrap());
    }
}
