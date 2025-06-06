//! 安全扫描器实现

use async_trait::async_trait;
use regex::Regex;
use std::collections::HashSet;

use crate::models::{ToolPackage, SecurityAuditResult, SecurityLevel, SecurityIssue, SecurityIssueType, Severity};
use crate::error::{MarketplaceError, Result};

/// 安全扫描器trait
#[async_trait]
pub trait SecurityScanner: Send + Sync {
    /// 扫描工具包
    async fn scan_package(&self, package: &ToolPackage) -> Result<SecurityAuditResult>;
    
    /// 扫描代码
    async fn scan_code(&self, code: &str, file_path: &str) -> Result<Vec<SecurityIssue>>;
    
    /// 扫描依赖
    async fn scan_dependencies(&self, dependencies: &std::collections::HashMap<String, String>) -> Result<Vec<SecurityIssue>>;
    
    /// 扫描权限
    async fn scan_permissions(&self, permissions: &[crate::models::Permission]) -> Result<Vec<SecurityIssue>>;
}

/// 默认安全扫描器
pub struct DefaultSecurityScanner {
    dangerous_patterns: Vec<Regex>,
    suspicious_keywords: HashSet<String>,
    known_vulnerabilities: HashSet<String>,
}

impl DefaultSecurityScanner {
    /// 创建新的默认安全扫描器
    pub fn new() -> Self {
        let dangerous_patterns = vec![
            Regex::new(r"eval\s*\(").unwrap(),
            Regex::new(r"exec\s*\(").unwrap(),
            Regex::new(r"system\s*\(").unwrap(),
            Regex::new(r"shell_exec\s*\(").unwrap(),
            Regex::new(r"unsafe\s*\{").unwrap(),
            Regex::new(r"std::process::Command").unwrap(),
            Regex::new(r"std::fs::remove_dir_all").unwrap(),
            Regex::new(r"std::fs::remove_file").unwrap(),
            Regex::new(r"std::env::set_var").unwrap(),
            Regex::new(r"reqwest::get").unwrap(),
            Regex::new(r"tokio::process::Command").unwrap(),
        ];
        
        let suspicious_keywords = [
            "password", "secret", "token", "key", "api_key",
            "private_key", "access_token", "auth_token",
            "credential", "passwd", "pwd", "admin",
            "root", "sudo", "chmod", "chown",
        ].iter().map(|s| s.to_string()).collect();
        
        let known_vulnerabilities = [
            "CVE-2021-44228", // Log4j
            "CVE-2022-22965", // Spring4Shell
            "CVE-2021-45046", // Log4j 2.15.0
        ].iter().map(|s| s.to_string()).collect();
        
        Self {
            dangerous_patterns,
            suspicious_keywords,
            known_vulnerabilities,
        }
    }
    
    /// 计算安全分数
    fn calculate_security_score(&self, issues: &[SecurityIssue]) -> u8 {
        let mut score = 100u8;
        
        for issue in issues {
            let deduction = match issue.severity {
                Severity::Critical => 40,
                Severity::High => 25,
                Severity::Medium => 15,
                Severity::Low => 5,
                Severity::Info => 1,
            };
            
            score = score.saturating_sub(deduction);
        }
        
        score
    }
    
    /// 确定安全级别
    fn determine_security_level(&self, issues: &[SecurityIssue]) -> SecurityLevel {
        let has_critical = issues.iter().any(|i| i.severity == Severity::Critical);
        let has_high = issues.iter().any(|i| i.severity == Severity::High);
        let medium_count = issues.iter().filter(|i| i.severity == Severity::Medium).count();
        
        if has_critical {
            SecurityLevel::Malicious
        } else if has_high || medium_count > 5 {
            SecurityLevel::Dangerous
        } else if medium_count > 0 || issues.iter().any(|i| i.severity == Severity::Low) {
            SecurityLevel::Warning
        } else {
            SecurityLevel::Safe
        }
    }
    
    /// 生成安全报告
    fn generate_security_report(&self, issues: &[SecurityIssue], score: u8, level: &SecurityLevel) -> String {
        let mut report = format!("安全扫描报告\n");
        report.push_str(&format!("安全分数: {}/100\n", score));
        report.push_str(&format!("安全级别: {:?}\n\n", level));
        
        if issues.is_empty() {
            report.push_str("✅ 未发现安全问题\n");
        } else {
            report.push_str(&format!("发现 {} 个安全问题:\n\n", issues.len()));
            
            for (i, issue) in issues.iter().enumerate() {
                report.push_str(&format!(
                    "{}. [{:?}] {:?}: {}\n",
                    i + 1,
                    issue.severity,
                    issue.issue_type,
                    issue.description
                ));
                
                if let Some(file) = &issue.file_path {
                    report.push_str(&format!("   文件: {}", file));
                    if let Some(line) = issue.line_number {
                        report.push_str(&format!(":{}", line));
                    }
                    report.push('\n');
                }
                
                if let Some(fix) = &issue.fix_suggestion {
                    report.push_str(&format!("   修复建议: {}\n", fix));
                }
                report.push('\n');
            }
        }
        
        report
    }
}

#[async_trait]
impl SecurityScanner for DefaultSecurityScanner {
    async fn scan_package(&self, package: &ToolPackage) -> Result<SecurityAuditResult> {
        let mut all_issues = Vec::new();
        
        // 扫描依赖
        all_issues.extend(self.scan_dependencies(&package.dependencies).await?);
        
        // 扫描权限
        for tool_def in &package.manifest.tools {
            all_issues.extend(self.scan_permissions(&tool_def.permissions).await?);
        }
        
        // 扫描元数据中的敏感信息
        let metadata_code = format!("{} {} {}", package.name, package.description, package.author);
        all_issues.extend(self.scan_code(&metadata_code, "metadata").await?);
        
        let score = self.calculate_security_score(&all_issues);
        let security_level = self.determine_security_level(&all_issues);
        let report = self.generate_security_report(&all_issues, score, &security_level);
        
        Ok(SecurityAuditResult {
            audit_time: chrono::Utc::now(),
            security_level,
            issues: all_issues,
            score,
            report,
            auditor_version: "1.0.0".to_string(),
        })
    }
    
    async fn scan_code(&self, code: &str, file_path: &str) -> Result<Vec<SecurityIssue>> {
        let mut issues = Vec::new();
        
        // 扫描危险模式
        for (line_num, line) in code.lines().enumerate() {
            for pattern in &self.dangerous_patterns {
                if pattern.is_match(line) {
                    issues.push(SecurityIssue {
                        issue_type: SecurityIssueType::CodeInjection,
                        severity: Severity::High,
                        description: format!("检测到危险代码模式: {}", pattern.as_str()),
                        file_path: Some(file_path.to_string()),
                        line_number: Some(line_num as u32 + 1),
                        fix_suggestion: Some("请避免使用危险的代码模式".to_string()),
                    });
                }
            }
            
            // 扫描敏感关键词
            let line_lower = line.to_lowercase();
            for keyword in &self.suspicious_keywords {
                if line_lower.contains(keyword) {
                    issues.push(SecurityIssue {
                        issue_type: SecurityIssueType::HardcodedSecret,
                        severity: Severity::Medium,
                        description: format!("检测到敏感关键词: {}", keyword),
                        file_path: Some(file_path.to_string()),
                        line_number: Some(line_num as u32 + 1),
                        fix_suggestion: Some("请避免在代码中硬编码敏感信息".to_string()),
                    });
                }
            }
        }
        
        Ok(issues)
    }
    
    async fn scan_dependencies(&self, dependencies: &std::collections::HashMap<String, String>) -> Result<Vec<SecurityIssue>> {
        let mut issues = Vec::new();
        
        for (dep_name, version) in dependencies {
            // 检查已知漏洞
            if self.known_vulnerabilities.iter().any(|vuln| dep_name.contains(vuln)) {
                issues.push(SecurityIssue {
                    issue_type: SecurityIssueType::UnsafeDependency,
                    severity: Severity::Critical,
                    description: format!("依赖 {} 包含已知安全漏洞", dep_name),
                    file_path: None,
                    line_number: None,
                    fix_suggestion: Some("请更新到安全版本或移除此依赖".to_string()),
                });
            }
            
            // 检查版本格式
            if version.contains("*") || version.contains("^") || version.contains("~") {
                issues.push(SecurityIssue {
                    issue_type: SecurityIssueType::UnsafeDependency,
                    severity: Severity::Low,
                    description: format!("依赖 {} 使用了不精确的版本号: {}", dep_name, version),
                    file_path: None,
                    line_number: None,
                    fix_suggestion: Some("建议使用精确的版本号以确保安全性".to_string()),
                });
            }
        }
        
        Ok(issues)
    }
    
    async fn scan_permissions(&self, permissions: &[crate::models::Permission]) -> Result<Vec<SecurityIssue>> {
        let mut issues = Vec::new();
        
        for permission in permissions {
            match permission.risk_level() {
                crate::models::RiskLevel::High => {
                    issues.push(SecurityIssue {
                        issue_type: SecurityIssueType::PermissionAbuse,
                        severity: Severity::High,
                        description: format!("请求高风险权限: {}", permission.display_name()),
                        file_path: None,
                        line_number: None,
                        fix_suggestion: Some("请确保此权限是必需的，并在文档中说明用途".to_string()),
                    });
                }
                crate::models::RiskLevel::Medium => {
                    issues.push(SecurityIssue {
                        issue_type: SecurityIssueType::PermissionAbuse,
                        severity: Severity::Medium,
                        description: format!("请求中等风险权限: {}", permission.display_name()),
                        file_path: None,
                        line_number: None,
                        fix_suggestion: Some("请在文档中说明此权限的用途".to_string()),
                    });
                }
                crate::models::RiskLevel::Low => {
                    // 低风险权限不报告问题
                }
            }
        }
        
        Ok(issues)
    }
}

impl Default for DefaultSecurityScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_security_scanner_creation() {
        let scanner = DefaultSecurityScanner::new();
        assert!(!scanner.dangerous_patterns.is_empty());
        assert!(!scanner.suspicious_keywords.is_empty());
    }
    
    #[tokio::test]
    async fn test_code_scanning() {
        let scanner = DefaultSecurityScanner::new();
        
        // 测试危险代码
        let dangerous_code = "let result = eval(user_input);";
        let issues = scanner.scan_code(dangerous_code, "test.rs").await.unwrap();
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| matches!(i.severity, Severity::High)));
        
        // 测试安全代码
        let safe_code = "let result = calculate(input);";
        let issues = scanner.scan_code(safe_code, "test.rs").await.unwrap();
        assert!(issues.is_empty());
    }
    
    #[tokio::test]
    async fn test_dependency_scanning() {
        let scanner = DefaultSecurityScanner::new();
        
        let mut dependencies = HashMap::new();
        dependencies.insert("safe_crate".to_string(), "1.0.0".to_string());
        dependencies.insert("unsafe_crate".to_string(), "^1.0.0".to_string());
        
        let issues = scanner.scan_dependencies(&dependencies).await.unwrap();
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.description.contains("不精确的版本号")));
    }
    
    #[tokio::test]
    async fn test_permission_scanning() {
        let scanner = DefaultSecurityScanner::new();
        
        let permissions = vec![
            crate::models::Permission::FileRead,
            crate::models::Permission::SystemCommand,
        ];
        
        let issues = scanner.scan_permissions(&permissions).await.unwrap();
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.description.contains("高风险权限")));
    }
    
    #[tokio::test]
    async fn test_package_scanning() {
        let scanner = DefaultSecurityScanner::new();
        let package = create_test_package();
        
        let result = scanner.scan_package(&package).await.unwrap();
        assert!(result.score <= 100);
        assert!(!result.report.is_empty());
    }
    
    fn create_test_package() -> ToolPackage {
        use chrono::Utc;
        use semver::Version;
        use uuid::Uuid;
        
        ToolPackage {
            id: Uuid::new_v4(),
            name: "test_tool".to_string(),
            version: Version::new(1, 0, 0),
            description: "Test tool with password in description".to_string(),
            author: "Test Author".to_string(),
            author_email: Some("test@example.com".to_string()),
            license: "MIT".to_string(),
            homepage: None,
            repository: None,
            keywords: vec!["test".to_string()],
            categories: vec![crate::models::ToolCategory::Utility],
            dependencies: {
                let mut deps = HashMap::new();
                deps.insert("unsafe_dep".to_string(), "^1.0.0".to_string());
                deps
            },
            lumos_version: "0.1.0".to_string(),
            manifest: crate::models::ToolManifest {
                tools: vec![crate::models::ToolDefinition {
                    name: "test_tool".to_string(),
                    description: "Test tool".to_string(),
                    parameters: vec![],
                    returns: crate::models::ReturnDefinition {
                        r#type: "string".to_string(),
                        description: "Result".to_string(),
                        schema: None,
                        examples: vec![],
                    },
                    examples: vec![],
                    tags: vec![],
                    async_tool: false,
                    requires_auth: false,
                    permissions: vec![crate::models::Permission::SystemCommand],
                }],
                entry_point: "main.rs".to_string(),
                exports: vec![],
                permissions: vec![],
                config_schema: None,
                rust_version: None,
                build_script: None,
            },
            metadata: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            published_at: None,
            download_count: 0,
            rating: 0.0,
            rating_count: 0,
            published: false,
            verified: false,
            security_audit: None,
            performance_benchmark: None,
        }
    }
}
