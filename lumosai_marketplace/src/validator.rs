//! 工具验证器实现

use async_trait::async_trait;
use regex::Regex;
use std::collections::HashSet;

use crate::models::{ToolPackage, ValidationRule, Severity, SecurityIssue, SecurityIssueType};
use crate::error::{MarketplaceError, Result};

/// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// 是否通过验证
    pub passed: bool,
    
    /// 验证分数 (0-100)
    pub score: u8,
    
    /// 发现的问题
    pub issues: Vec<ValidationIssue>,
    
    /// 验证报告
    pub report: String,
}

/// 验证问题
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    /// 问题类型
    pub issue_type: ValidationIssueType,
    
    /// 严重程度
    pub severity: Severity,
    
    /// 问题描述
    pub description: String,
    
    /// 修复建议
    pub fix_suggestion: Option<String>,
}

/// 验证问题类型
#[derive(Debug, Clone)]
pub enum ValidationIssueType {
    /// 元数据问题
    Metadata,
    /// 依赖问题
    Dependency,
    /// 许可证问题
    License,
    /// 安全问题
    Security,
    /// 性能问题
    Performance,
    /// 兼容性问题
    Compatibility,
}

/// 工具验证器trait
#[async_trait]
pub trait ToolValidator: Send + Sync {
    /// 验证工具包
    async fn validate(&self, package: &ToolPackage) -> Result<ValidationResult>;
    
    /// 验证元数据
    async fn validate_metadata(&self, package: &ToolPackage) -> Result<Vec<ValidationIssue>>;
    
    /// 验证依赖
    async fn validate_dependencies(&self, package: &ToolPackage) -> Result<Vec<ValidationIssue>>;
    
    /// 验证许可证
    async fn validate_license(&self, package: &ToolPackage) -> Result<Vec<ValidationIssue>>;
    
    /// 验证安全性
    async fn validate_security(&self, package: &ToolPackage) -> Result<Vec<ValidationIssue>>;
}

/// 默认工具验证器
pub struct DefaultToolValidator {
    allowed_licenses: HashSet<String>,
    forbidden_keywords: HashSet<String>,
    name_regex: Regex,
    version_regex: Regex,
}

impl DefaultToolValidator {
    /// 创建新的默认验证器
    pub fn new() -> Result<Self> {
        let allowed_licenses = [
            "MIT", "Apache-2.0", "BSD-3-Clause", "ISC", 
            "GPL-3.0", "LGPL-3.0", "MPL-2.0"
        ].iter().map(|s| s.to_string()).collect();
        
        let forbidden_keywords = [
            "eval", "exec", "system", "shell", "unsafe",
            "password", "secret", "token", "key"
        ].iter().map(|s| s.to_string()).collect();
        
        let name_regex = Regex::new(r"^[a-zA-Z0-9_-]+$")
            .map_err(|e| MarketplaceError::validation(format!("Invalid regex: {}", e)))?;
        
        let version_regex = Regex::new(r"^\d+\.\d+\.\d+")
            .map_err(|e| MarketplaceError::validation(format!("Invalid regex: {}", e)))?;
        
        Ok(Self {
            allowed_licenses,
            forbidden_keywords,
            name_regex,
            version_regex,
        })
    }
    
    /// 计算验证分数
    fn calculate_score(&self, issues: &[ValidationIssue]) -> u8 {
        let mut score = 100u8;
        
        for issue in issues {
            let deduction = match issue.severity {
                Severity::Critical => 30,
                Severity::High => 20,
                Severity::Medium => 10,
                Severity::Low => 5,
                Severity::Info => 1,
            };
            
            score = score.saturating_sub(deduction);
        }
        
        score
    }
    
    /// 生成验证报告
    fn generate_report(&self, issues: &[ValidationIssue], score: u8) -> String {
        let mut report = format!("验证分数: {}/100\n\n", score);
        
        if issues.is_empty() {
            report.push_str("✅ 所有验证检查都通过了！\n");
        } else {
            report.push_str(&format!("发现 {} 个问题:\n\n", issues.len()));
            
            for (i, issue) in issues.iter().enumerate() {
                report.push_str(&format!(
                    "{}. [{:?}] {:?}: {}\n",
                    i + 1,
                    issue.severity,
                    issue.issue_type,
                    issue.description
                ));
                
                if let Some(fix) = &issue.fix_suggestion {
                    report.push_str(&format!("   建议: {}\n", fix));
                }
                report.push('\n');
            }
        }
        
        report
    }
}

#[async_trait]
impl ToolValidator for DefaultToolValidator {
    async fn validate(&self, package: &ToolPackage) -> Result<ValidationResult> {
        let mut all_issues = Vec::new();
        
        // 执行各种验证
        all_issues.extend(self.validate_metadata(package).await?);
        all_issues.extend(self.validate_dependencies(package).await?);
        all_issues.extend(self.validate_license(package).await?);
        all_issues.extend(self.validate_security(package).await?);
        
        let score = self.calculate_score(&all_issues);
        let passed = score >= 70; // 70分以上算通过
        let report = self.generate_report(&all_issues, score);
        
        Ok(ValidationResult {
            passed,
            score,
            issues: all_issues,
            report,
        })
    }
    
    async fn validate_metadata(&self, package: &ToolPackage) -> Result<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        
        // 验证名称
        if package.name.is_empty() {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Metadata,
                severity: Severity::Critical,
                description: "工具包名称不能为空".to_string(),
                fix_suggestion: Some("请提供有效的工具包名称".to_string()),
            });
        } else if !self.name_regex.is_match(&package.name) {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Metadata,
                severity: Severity::High,
                description: "工具包名称包含无效字符".to_string(),
                fix_suggestion: Some("名称只能包含字母、数字、下划线和连字符".to_string()),
            });
        }
        
        // 验证描述
        if package.description.is_empty() {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Metadata,
                severity: Severity::High,
                description: "工具包描述不能为空".to_string(),
                fix_suggestion: Some("请提供详细的工具包描述".to_string()),
            });
        } else if package.description.len() < 20 {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Metadata,
                severity: Severity::Medium,
                description: "工具包描述过短".to_string(),
                fix_suggestion: Some("描述应该至少包含20个字符".to_string()),
            });
        }
        
        // 验证作者
        if package.author.is_empty() {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Metadata,
                severity: Severity::Medium,
                description: "作者信息不能为空".to_string(),
                fix_suggestion: Some("请提供作者姓名或组织名称".to_string()),
            });
        }
        
        // 验证版本
        if !self.version_regex.is_match(&package.version.to_string()) {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Metadata,
                severity: Severity::High,
                description: "版本号格式无效".to_string(),
                fix_suggestion: Some("请使用语义化版本号格式 (如: 1.0.0)".to_string()),
            });
        }
        
        // 验证关键词
        if package.keywords.is_empty() {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Metadata,
                severity: Severity::Low,
                description: "缺少关键词".to_string(),
                fix_suggestion: Some("添加相关关键词有助于工具被发现".to_string()),
            });
        }
        
        // 验证分类
        if package.categories.is_empty() {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Metadata,
                severity: Severity::Medium,
                description: "缺少分类信息".to_string(),
                fix_suggestion: Some("请为工具包选择合适的分类".to_string()),
            });
        }
        
        Ok(issues)
    }
    
    async fn validate_dependencies(&self, package: &ToolPackage) -> Result<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        
        // 检查依赖数量
        if package.dependencies.len() > 50 {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Dependency,
                severity: Severity::Medium,
                description: "依赖数量过多".to_string(),
                fix_suggestion: Some("考虑减少不必要的依赖".to_string()),
            });
        }
        
        // 检查循环依赖（简化检查）
        for (dep_name, _version) in &package.dependencies {
            if dep_name == &package.name {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::Dependency,
                    severity: Severity::Critical,
                    description: "检测到自依赖".to_string(),
                    fix_suggestion: Some("移除对自身的依赖".to_string()),
                });
            }
        }
        
        Ok(issues)
    }
    
    async fn validate_license(&self, package: &ToolPackage) -> Result<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        
        if package.license.is_empty() {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::License,
                severity: Severity::Critical,
                description: "缺少许可证信息".to_string(),
                fix_suggestion: Some("请指定有效的开源许可证".to_string()),
            });
        } else if !self.allowed_licenses.contains(&package.license) {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::License,
                severity: Severity::High,
                description: format!("不支持的许可证: {}", package.license),
                fix_suggestion: Some("请使用支持的开源许可证 (MIT, Apache-2.0, 等)".to_string()),
            });
        }
        
        Ok(issues)
    }
    
    async fn validate_security(&self, package: &ToolPackage) -> Result<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        
        // 检查描述中的敏感关键词
        let description_lower = package.description.to_lowercase();
        for keyword in &self.forbidden_keywords {
            if description_lower.contains(keyword) {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::Security,
                    severity: Severity::Medium,
                    description: format!("描述中包含敏感关键词: {}", keyword),
                    fix_suggestion: Some("请避免在描述中使用敏感关键词".to_string()),
                });
            }
        }
        
        // 检查工具清单中的权限
        for tool_def in &package.manifest.tools {
            if tool_def.requires_auth && tool_def.permissions.is_empty() {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::Security,
                    severity: Severity::High,
                    description: format!("工具 '{}' 需要认证但未声明权限", tool_def.name),
                    fix_suggestion: Some("请明确声明所需的权限".to_string()),
                });
            }
        }
        
        Ok(issues)
    }
}

impl Default for DefaultToolValidator {
    fn default() -> Self {
        Self::new().expect("Failed to create default validator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_validator_creation() {
        let validator = DefaultToolValidator::new().unwrap();
        assert!(!validator.allowed_licenses.is_empty());
    }
    
    #[tokio::test]
    async fn test_metadata_validation() {
        let validator = DefaultToolValidator::new().unwrap();
        let package = create_test_package();
        
        let issues = validator.validate_metadata(&package).await.unwrap();
        // 应该没有严重问题
        assert!(!issues.iter().any(|i| matches!(i.severity, Severity::Critical)));
    }
    
    #[tokio::test]
    async fn test_invalid_package_validation() {
        let validator = DefaultToolValidator::new().unwrap();
        let mut package = create_test_package();
        
        // 创建无效的包
        package.name = "".to_string(); // 空名称
        package.license = "INVALID".to_string(); // 无效许可证
        
        let result = validator.validate(&package).await.unwrap();
        assert!(!result.passed);
        assert!(result.score < 70);
        assert!(!result.issues.is_empty());
    }
    
    fn create_test_package() -> ToolPackage {
        use chrono::Utc;
        use semver::Version;
        use uuid::Uuid;
        
        ToolPackage {
            id: Uuid::new_v4(),
            name: "test_tool".to_string(),
            version: Version::new(1, 0, 0),
            description: "This is a comprehensive test tool description with sufficient length".to_string(),
            author: "Test Author".to_string(),
            author_email: Some("test@example.com".to_string()),
            license: "MIT".to_string(),
            homepage: None,
            repository: None,
            keywords: vec!["test".to_string(), "utility".to_string()],
            categories: vec![crate::models::ToolCategory::Utility],
            dependencies: HashMap::new(),
            lumos_version: "0.1.0".to_string(),
            manifest: crate::models::ToolManifest {
                tools: vec![],
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
