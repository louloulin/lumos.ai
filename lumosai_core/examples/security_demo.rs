//! 安全系统演示应用
//! 
//! 展示Lumos.ai企业级安全功能的完整使用示例

use std::collections::HashMap;
use chrono::Utc;
use lumosai_core::security::*;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🔒 Lumos.ai 企业级安全系统演示");
    println!("=====================================\n");
    
    // 1. 初始化安全框架
    println!("1. 初始化安全框架...");
    let config = SecurityConfig::default();
    let mut security = SecurityFramework::new(config).await?;
    println!("✅ 安全框架初始化完成\n");
    
    // 2. 演示数据加密
    println!("2. 数据加密演示...");
    let sensitive_data = "机密文档：2024年战略规划".as_bytes();
    println!("原始数据: {:?}", String::from_utf8_lossy(sensitive_data));
    
    let encrypted = security.encrypt_data(sensitive_data).await?;
    println!("✅ 数据加密成功，加密后大小: {} bytes", encrypted.len());
    
    let decrypted = security.decrypt_data(&encrypted).await?;
    println!("✅ 数据解密成功: {:?}", String::from_utf8_lossy(&decrypted));
    println!();
    
    // 3. 演示零信任访问控制
    println!("3. 零信任访问控制演示...");
    
    // 模拟用户访问请求
    let access_request = AccessRequest {
        user_id: "alice@company.com".to_string(),
        resource: "financial_reports".to_string(),
        action: "read".to_string(),
        context: SecurityContext {
            user_id: Some("alice@company.com".to_string()),
            session_id: Some("sess_abc123".to_string()),
            ip_address: "192.168.1.100".to_string(),
            user_agent: Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64)".to_string()),
            request_path: "/api/financial/reports".to_string(),
            request_method: "GET".to_string(),
            headers: HashMap::from([
                ("Authorization".to_string(), "Bearer token123".to_string()),
                ("X-Forwarded-For".to_string(), "192.168.1.100".to_string()),
            ]),
            timestamp: Utc::now(),
        },
    };
    
    println!("用户: {}", access_request.user_id);
    println!("请求资源: {}", access_request.resource);
    println!("请求动作: {}", access_request.action);
    println!("来源IP: {}", access_request.context.ip_address);
    
    let decision = security.verify_access(&access_request).await?;
    match decision {
        AccessDecision::Allow => {
            println!("✅ 访问已授权");
        }
        AccessDecision::Deny { reason } => {
            println!("❌ 访问被拒绝: {}", reason);
        }
        AccessDecision::Conditional { conditions } => {
            println!("⚠️ 条件访问: {:?}", conditions);
        }
    }
    println!();
    
    // 4. 演示威胁检测
    println!("4. 威胁检测演示...");
    
    // 模拟可疑请求
    let suspicious_context = SecurityContext {
        user_id: Some("attacker".to_string()),
        session_id: None,
        ip_address: "192.168.100.100".to_string(),
        user_agent: Some("sqlmap/1.0".to_string()),
        request_path: "/api/users?id=1' UNION SELECT password FROM users--".to_string(),
        request_method: "GET".to_string(),
        headers: HashMap::new(),
        timestamp: Utc::now(),
    };
    
    println!("检测可疑请求:");
    println!("  路径: {}", suspicious_context.request_path);
    println!("  User-Agent: {:?}", suspicious_context.user_agent);
    
    let threats = security.detect_threats(&suspicious_context).await?;
    if threats.is_empty() {
        println!("✅ 未检测到威胁");
    } else {
        println!("⚠️ 检测到 {} 个威胁:", threats.len());
        for threat in &threats {
            println!("  - {}: {} (严重程度: {:?})", 
                threat.threat_type, 
                threat.description, 
                threat.severity
            );
        }
    }
    println!();
    
    // 5. 演示审计日志
    println!("5. 审计日志演示...");
    
    // 记录各种安全事件
    let events = vec![
        SecurityEvent::LoginAttempt {
            user_id: "alice@company.com".to_string(),
            success: true,
            ip_address: "192.168.1.100".to_string(),
            timestamp: Utc::now(),
        },
        SecurityEvent::PermissionCheck {
            user_id: "alice@company.com".to_string(),
            resource: "financial_reports".to_string(),
            action: "read".to_string(),
            granted: true,
            timestamp: Utc::now(),
        },
        SecurityEvent::DataAccess {
            user_id: "alice@company.com".to_string(),
            resource_type: "financial_report".to_string(),
            resource_id: "Q4_2023".to_string(),
            action: "download".to_string(),
            timestamp: Utc::now(),
        },
    ];
    
    for event in events {
        security.log_security_event(event).await?;
    }
    println!("✅ 安全事件已记录到审计日志");
    println!();
    
    // 6. 演示合规性检查
    println!("6. 合规性检查演示...");
    
    let standards = vec![
        ComplianceStandard::SOC2,
        ComplianceStandard::GDPR,
    ];
    
    for standard in standards {
        println!("检查 {:?} 合规性...", standard);
        let report = security.check_compliance(standard).await?;
        
        println!("  合规分数: {:.2}%", report.overall_score * 100.0);
        println!("  总要求数: {}", report.total_requirements);
        println!("  合规要求: {}", report.compliant_requirements);
        println!("  违规数量: {}", report.violations.len());
        
        if !report.violations.is_empty() {
            println!("  违规详情:");
            for violation in &report.violations {
                println!("    - {}: {}", violation.rule_name, violation.description);
            }
        }
        println!();
    }
    
    // 7. 演示网络安全
    println!("7. 网络安全演示...");
    
    let network_config = NetworkSecurityConfig::default();
    let mut network_security = NetworkSecurityManager::new(&network_config).await?;
    
    // 模拟网络请求
    let network_request = NetworkRequest {
        source_ip: "203.0.113.1".parse()?,
        destination_ip: "10.0.0.1".parse()?,
        source_port: 12345,
        destination_port: 443,
        protocol: Protocol::TCP,
        payload_size: 2048,
        timestamp: Utc::now(),
    };
    
    println!("检查网络请求:");
    println!("  源IP: {}", network_request.source_ip);
    println!("  目标端口: {}", network_request.destination_port);
    println!("  协议: {:?}", network_request.protocol);
    
    let network_decision = network_security.check_request(&network_request).await?;
    match network_decision {
        NetworkDecision::Allow => {
            println!("✅ 网络请求已允许");
        }
        NetworkDecision::Block { reason } => {
            println!("❌ 网络请求被阻断: {}", reason);
        }
        NetworkDecision::RateLimit { retry_after } => {
            println!("⚠️ 请求被限流，请在 {} 秒后重试", retry_after);
        }
        NetworkDecision::Monitor { alerts } => {
            println!("👁️ 请求被监控，告警: {:?}", alerts);
        }
    }
    println!();
    
    // 8. 获取整体安全状态
    println!("8. 安全状态总览...");
    let security_status = security.get_security_status().await?;
    let network_status = network_security.get_status().await?;
    
    println!("🔐 加密状态:");
    println!("  当前密钥ID: {}", security_status.encryption_status.current_key_id);
    println!("  加密算法: {:?}", security_status.encryption_status.algorithm);
    println!("  密钥年龄: {} 小时", security_status.encryption_status.key_age_hours);
    
    println!("\n🛡️ 零信任状态:");
    println!("  活跃会话: {}", security_status.zero_trust_status.active_sessions);
    println!("  受信设备: {}", security_status.zero_trust_status.trusted_devices);
    println!("  活跃策略: {}", security_status.zero_trust_status.active_policies);
    
    println!("\n⚠️ 威胁检测:");
    println!("  当前威胁级别: {:?}", security_status.threat_level);
    
    println!("\n📋 合规状态:");
    println!("  整体合规分数: {:.2}%", security_status.compliance_status.overall_compliance_score * 100.0);
    println!("  活跃违规: {}", security_status.compliance_status.active_violations);
    
    println!("\n🌐 网络安全:");
    println!("  防火墙启用: {}", network_status.firewall_enabled);
    println!("  DDoS防护启用: {}", network_status.ddos_protection_enabled);
    println!("  活跃防火墙规则: {}", network_status.active_firewall_rules);
    println!("  被阻断IP数量: {}", network_status.blocked_ips_count);
    
    println!("\n🎉 安全系统演示完成！");
    println!("Lumos.ai 提供企业级的全方位安全保护，包括:");
    println!("  ✅ 端到端数据加密");
    println!("  ✅ 零信任架构访问控制");
    println!("  ✅ 实时威胁检测与响应");
    println!("  ✅ 完整的审计日志记录");
    println!("  ✅ 多标准合规性监控");
    println!("  ✅ 网络层安全防护");
    
    Ok(())
}

/// 演示高级安全场景
async fn demo_advanced_security_scenarios() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n🚀 高级安全场景演示");
    println!("====================");
    
    let config = SecurityConfig::default();
    let mut security = SecurityFramework::new(config).await?;
    
    // 场景1: 多因素认证流程
    println!("\n场景1: 多因素认证流程");
    let mfa_request = AccessRequest {
        user_id: "admin@company.com".to_string(),
        resource: "admin_panel".to_string(),
        action: "access".to_string(),
        context: SecurityContext {
            user_id: Some("admin@company.com".to_string()),
            session_id: None,
            ip_address: "203.0.113.50".to_string(), // 外部IP
            user_agent: Some("Mozilla/5.0".to_string()),
            request_path: "/admin".to_string(),
            request_method: "GET".to_string(),
            headers: HashMap::new(),
            timestamp: Utc::now(),
        },
    };
    
    let decision = security.verify_access(&mfa_request).await?;
    println!("MFA决策: {:?}", decision);
    
    // 场景2: 数据泄露检测
    println!("\n场景2: 数据泄露检测");
    let data_exfiltration_context = SecurityContext {
        user_id: Some("insider@company.com".to_string()),
        session_id: Some("sess_xyz789".to_string()),
        ip_address: "192.168.1.200".to_string(),
        user_agent: Some("curl/7.68.0".to_string()),
        request_path: "/api/export/all_users".to_string(),
        request_method: "POST".to_string(),
        headers: HashMap::from([
            ("Content-Length".to_string(), "1048576".to_string()), // 1MB
        ]),
        timestamp: Utc::now(),
    };
    
    let threats = security.detect_threats(&data_exfiltration_context).await?;
    println!("检测到的威胁: {} 个", threats.len());
    
    // 场景3: 合规违规处理
    println!("\n场景3: 合规违规处理");
    let compliance_violation = SecurityEvent::ComplianceViolation {
        standard: "GDPR".to_string(),
        rule: "Art.32".to_string(),
        severity: ComplianceSeverity::Critical,
        details: "Unencrypted personal data transmission".to_string(),
        timestamp: Utc::now(),
    };
    
    security.log_security_event(compliance_violation).await?;
    println!("✅ 合规违规事件已记录并触发自动响应");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_security_demo() {
        // 测试演示应用的核心功能
        let config = SecurityConfig::default();
        let security = SecurityFramework::new(config).await;
        assert!(security.is_ok());
    }
}
