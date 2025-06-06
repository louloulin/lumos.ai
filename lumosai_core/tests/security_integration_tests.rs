//! 安全系统集成测试

use std::collections::HashMap;
use chrono::Utc;
use lumosai_core::security::*;

#[tokio::test]
async fn test_security_framework_integration() {
    // 创建安全框架
    let config = SecurityConfig::default();
    let mut security = SecurityFramework::new(config).await.unwrap();
    
    // 测试数据加密和解密
    let test_data = b"sensitive information";
    let encrypted = security.encrypt_data(test_data).await.unwrap();
    let decrypted = security.decrypt_data(&encrypted).await.unwrap();
    assert_eq!(test_data, decrypted.as_slice());
    
    // 测试访问验证
    let access_request = AccessRequest {
        user_id: "test_user".to_string(),
        resource: "sensitive_resource".to_string(),
        action: "read".to_string(),
        context: SecurityContext {
            user_id: Some("test_user".to_string()),
            session_id: None,
            ip_address: "192.168.1.1".to_string(),
            user_agent: Some("TestAgent/1.0".to_string()),
            request_path: "/api/sensitive".to_string(),
            request_method: "GET".to_string(),
            headers: HashMap::new(),
            timestamp: Utc::now(),
        },
    };
    
    let decision = security.verify_access(&access_request).await.unwrap();
    match decision {
        AccessDecision::Allow => println!("Access granted"),
        AccessDecision::Deny { reason } => println!("Access denied: {}", reason),
        AccessDecision::Conditional { conditions } => println!("Conditional access: {:?}", conditions),
    }
    
    // 测试威胁检测
    let threats = security.detect_threats(&access_request.context).await.unwrap();
    println!("Detected {} threats", threats.len());
    
    // 测试安全事件记录
    let security_event = SecurityEvent::LoginAttempt {
        user_id: "test_user".to_string(),
        success: true,
        ip_address: "192.168.1.1".to_string(),
        timestamp: Utc::now(),
    };
    
    security.log_security_event(security_event).await.unwrap();
    
    // 测试合规性检查
    let compliance_report = security.check_compliance(ComplianceStandard::SOC2).await.unwrap();
    println!("Compliance score: {}", compliance_report.overall_score);
    
    // 获取安全状态
    let status = security.get_security_status().await.unwrap();
    println!("Security status: {:?}", status);
}

#[tokio::test]
async fn test_encryption_manager() {
    let config = EncryptionConfig::default();
    let mut manager = EncryptionManager::new(&config).await.unwrap();
    
    // 测试基本加密解密
    let data = b"Hello, World!";
    let encrypted = manager.encrypt(data).await.unwrap();
    let decrypted = manager.decrypt(&encrypted).await.unwrap();
    assert_eq!(data, decrypted.as_slice());
    
    // 测试密钥轮换
    let old_key_id = manager.key_id.clone();
    manager.rotate_key().await.unwrap();
    assert_ne!(old_key_id, manager.key_id);
    
    // 测试状态获取
    let status = manager.get_status().await.unwrap();
    assert!(status.transport_encryption_enabled);
    assert!(status.storage_encryption_enabled);
}

#[tokio::test]
async fn test_zero_trust_engine() {
    let config = ZeroTrustConfig::default();
    let mut engine = ZeroTrustEngine::new(&config).await.unwrap();
    
    // 创建测试会话
    let context = SecurityContext {
        user_id: Some("test_user".to_string()),
        session_id: None,
        ip_address: "192.168.1.1".to_string(),
        user_agent: Some("TestAgent/1.0".to_string()),
        request_path: "/api/test".to_string(),
        request_method: "GET".to_string(),
        headers: HashMap::new(),
        timestamp: Utc::now(),
    };
    
    let session_id = engine.create_session("test_user", &context).await.unwrap();
    
    // 测试访问验证
    let mut request = AccessRequest {
        user_id: "test_user".to_string(),
        resource: "test_resource".to_string(),
        action: "read".to_string(),
        context: context.clone(),
    };
    request.context.session_id = Some(session_id);
    
    let decision = engine.verify_access(&request).await.unwrap();
    println!("Access decision: {:?}", decision);
    
    // 测试状态获取
    let status = engine.get_status().await.unwrap();
    assert!(status.active_sessions > 0);
}

#[tokio::test]
async fn test_threat_detector() {
    let config = ThreatDetectionConfig::default();
    let mut detector = ThreatDetector::new(&config).await.unwrap();
    
    // 测试SQL注入检测
    let context = SecurityContext {
        user_id: Some("test_user".to_string()),
        session_id: None,
        ip_address: "192.168.1.1".to_string(),
        user_agent: None,
        request_path: "/api/users?id=1' UNION SELECT * FROM passwords--".to_string(),
        request_method: "GET".to_string(),
        headers: HashMap::new(),
        timestamp: Utc::now(),
    };
    
    let alerts = detector.detect(&context).await.unwrap();
    assert!(!alerts.is_empty());
    assert!(alerts.iter().any(|a| a.threat_type == "SQL Injection Attempt"));
    
    // 测试XSS检测
    let xss_context = SecurityContext {
        user_id: Some("test_user".to_string()),
        session_id: None,
        ip_address: "192.168.1.1".to_string(),
        user_agent: None,
        request_path: "/api/comment".to_string(),
        request_method: "POST".to_string(),
        headers: HashMap::from([
            ("content".to_string(), "<script>alert('xss')</script>".to_string()),
        ]),
        timestamp: Utc::now(),
    };
    
    let xss_alerts = detector.detect(&xss_context).await.unwrap();
    assert!(!xss_alerts.is_empty());
    
    // 测试威胁级别
    let threat_level = detector.get_current_threat_level().await.unwrap();
    println!("Current threat level: {:?}", threat_level);
}

#[tokio::test]
async fn test_audit_logger() {
    let config = AuditConfig::default();
    let mut logger = AuditLogger::new(&config).await.unwrap();
    
    // 测试安全事件记录
    let security_event = SecurityEvent::LoginAttempt {
        user_id: "test_user".to_string(),
        success: true,
        ip_address: "192.168.1.1".to_string(),
        timestamp: Utc::now(),
    };
    
    logger.log_event(security_event).await.unwrap();
    
    // 测试权限检查事件
    let permission_event = SecurityEvent::PermissionCheck {
        user_id: "test_user".to_string(),
        resource: "sensitive_data".to_string(),
        action: "read".to_string(),
        granted: true,
        timestamp: Utc::now(),
    };
    
    logger.log_event(permission_event).await.unwrap();
    
    // 测试数据访问事件
    let data_access_event = SecurityEvent::DataAccess {
        user_id: "test_user".to_string(),
        resource_type: "user_profile".to_string(),
        resource_id: "12345".to_string(),
        action: "update".to_string(),
        timestamp: Utc::now(),
    };
    
    logger.log_event(data_access_event).await.unwrap();
    
    // 测试合规报告生成
    let start_time = Utc::now() - chrono::Duration::hours(24);
    let end_time = Utc::now();
    let report = logger.generate_compliance_report(start_time, end_time, "SOC2").await.unwrap();
    
    assert_eq!(report.standard, "SOC2");
    assert!(report.generated_at <= Utc::now());
}

#[tokio::test]
async fn test_compliance_monitor() {
    let config = ComplianceConfig::default();
    let mut monitor = ComplianceMonitor::new(&config).await.unwrap();
    
    // 测试SOC2合规检查
    let soc2_report = monitor.check_compliance(ComplianceStandard::SOC2).await.unwrap();
    assert!(matches!(soc2_report.standard, ComplianceStandard::SOC2));
    assert!(soc2_report.overall_score >= 0.0 && soc2_report.overall_score <= 1.0);
    
    // 测试GDPR合规检查
    let gdpr_report = monitor.check_compliance(ComplianceStandard::GDPR).await.unwrap();
    assert!(matches!(gdpr_report.standard, ComplianceStandard::GDPR));
    
    // 测试合规状态
    let status = monitor.get_status().await.unwrap();
    assert!(status.overall_compliance_score >= 0.0 && status.overall_compliance_score <= 1.0);
    assert!(!status.standards_status.is_empty());
}

#[tokio::test]
async fn test_network_security_manager() {
    let config = NetworkSecurityConfig::default();
    let mut manager = NetworkSecurityManager::new(&config).await.unwrap();
    
    // 测试网络请求检查
    let request = NetworkRequest {
        source_ip: "192.168.1.1".parse().unwrap(),
        destination_ip: "10.0.0.1".parse().unwrap(),
        source_port: 12345,
        destination_port: 80,
        protocol: Protocol::TCP,
        payload_size: 1024,
        timestamp: Utc::now(),
    };
    
    let decision = manager.check_request(&request).await.unwrap();
    match decision {
        NetworkDecision::Allow => println!("Network request allowed"),
        NetworkDecision::Block { reason } => println!("Network request blocked: {}", reason),
        NetworkDecision::RateLimit { retry_after } => println!("Rate limited, retry after: {}", retry_after),
        NetworkDecision::Monitor { alerts } => println!("Monitoring with alerts: {:?}", alerts),
    }
    
    // 测试IP阻断
    let malicious_ip = "192.168.100.100".parse().unwrap();
    manager.block_ip(malicious_ip, Some(chrono::Duration::hours(24))).await.unwrap();
    
    // 测试网络安全策略应用
    let policy = NetworkSecurityPolicy {
        name: "Test Policy".to_string(),
        firewall_rules: vec![],
        rate_limits: RateLimitingConfig::default(),
        geo_restrictions: GeoFilteringConfig::default(),
        enabled: true,
    };
    
    manager.apply_policy(policy).await.unwrap();
    
    // 测试状态获取
    let status = manager.get_status().await.unwrap();
    assert!(status.firewall_enabled);
    assert!(status.ddos_protection_enabled);
}

#[tokio::test]
async fn test_end_to_end_security_scenario() {
    // 模拟完整的安全场景
    let config = SecurityConfig::default();
    let mut security = SecurityFramework::new(config).await.unwrap();
    
    // 1. 用户登录
    let login_event = SecurityEvent::LoginAttempt {
        user_id: "alice".to_string(),
        success: true,
        ip_address: "203.0.113.1".to_string(), // 外部IP
        timestamp: Utc::now(),
    };
    security.log_security_event(login_event).await.unwrap();
    
    // 2. 访问敏感资源
    let access_request = AccessRequest {
        user_id: "alice".to_string(),
        resource: "financial_data".to_string(),
        action: "read".to_string(),
        context: SecurityContext {
            user_id: Some("alice".to_string()),
            session_id: Some("session_123".to_string()),
            ip_address: "203.0.113.1".to_string(),
            user_agent: Some("Mozilla/5.0".to_string()),
            request_path: "/api/financial/reports".to_string(),
            request_method: "GET".to_string(),
            headers: HashMap::new(),
            timestamp: Utc::now(),
        },
    };
    
    // 3. 零信任验证
    let decision = security.verify_access(&access_request).await.unwrap();
    println!("Access decision: {:?}", decision);
    
    // 4. 威胁检测
    let threats = security.detect_threats(&access_request.context).await.unwrap();
    if !threats.is_empty() {
        println!("Threats detected: {}", threats.len());
        for threat in &threats {
            println!("  - {}: {}", threat.threat_type, threat.description);
        }
    }
    
    // 5. 数据加密
    let sensitive_data = b"Financial report: Q4 2023 revenue $1.2M";
    let encrypted_data = security.encrypt_data(sensitive_data).await.unwrap();
    println!("Data encrypted successfully");
    
    // 6. 审计记录
    let data_access_event = SecurityEvent::DataAccess {
        user_id: "alice".to_string(),
        resource_type: "financial_report".to_string(),
        resource_id: "q4_2023".to_string(),
        action: "read".to_string(),
        timestamp: Utc::now(),
    };
    security.log_security_event(data_access_event).await.unwrap();
    
    // 7. 合规检查
    let compliance_report = security.check_compliance(ComplianceStandard::SOC2).await.unwrap();
    println!("SOC2 compliance score: {}", compliance_report.overall_score);
    
    // 8. 获取整体安全状态
    let status = security.get_security_status().await.unwrap();
    println!("Overall security status: {:?}", status.threat_level);
    
    // 验证所有操作都成功完成
    assert!(encrypted_data.len() > sensitive_data.len()); // 加密数据应该更大（包含元数据）
    assert!(compliance_report.overall_score >= 0.0);
}
