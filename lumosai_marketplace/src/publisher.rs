//! 工具发布器实现

use async_trait::async_trait;
use std::path::Path;
use uuid::Uuid;
use chrono::Utc;

use crate::models::ToolPackage;
use crate::validator::{ToolValidator, ValidationResult};
use crate::security::{SecurityScanner, SecurityAuditResult};
use crate::error::{MarketplaceError, Result};

/// 发布请求
#[derive(Debug, Clone)]
pub struct PublishRequest {
    /// 工具包数据
    pub package: ToolPackage,
    
    /// 工具包文件路径
    pub package_path: String,
    
    /// 是否跳过验证
    pub skip_validation: bool,
    
    /// 是否跳过安全扫描
    pub skip_security_scan: bool,
    
    /// 发布者信息
    pub publisher_info: PublisherInfo,
}

/// 发布者信息
#[derive(Debug, Clone)]
pub struct PublisherInfo {
    /// 发布者ID
    pub id: String,
    
    /// 发布者名称
    pub name: String,
    
    /// 发布者邮箱
    pub email: String,
    
    /// API密钥
    pub api_key: Option<String>,
}

/// 发布结果
#[derive(Debug, Clone)]
pub struct PublishResult {
    /// 是否成功
    pub success: bool,
    
    /// 工具包ID
    pub package_id: Option<Uuid>,
    
    /// 验证结果
    pub validation_result: Option<ValidationResult>,
    
    /// 安全审计结果
    pub security_audit: Option<SecurityAuditResult>,
    
    /// 错误信息
    pub error_message: Option<String>,
    
    /// 发布时间
    pub published_at: chrono::DateTime<chrono::Utc>,
}

/// 工具发布器trait
#[async_trait]
pub trait ToolPublisher: Send + Sync {
    /// 发布工具包
    async fn publish(&self, request: PublishRequest) -> Result<PublishResult>;
    
    /// 验证发布权限
    async fn verify_publish_permission(&self, publisher: &PublisherInfo, package: &ToolPackage) -> Result<bool>;
    
    /// 处理工具包文件
    async fn process_package_file(&self, file_path: &str) -> Result<ToolPackage>;
    
    /// 上传工具包文件
    async fn upload_package_file(&self, package_id: Uuid, file_path: &str) -> Result<String>;
}

/// 默认工具发布器
pub struct DefaultToolPublisher {
    validator: Box<dyn ToolValidator>,
    security_scanner: Box<dyn SecurityScanner>,
    storage_path: String,
}

impl DefaultToolPublisher {
    /// 创建新的默认发布器
    pub fn new(
        validator: Box<dyn ToolValidator>,
        security_scanner: Box<dyn SecurityScanner>,
        storage_path: String,
    ) -> Self {
        Self {
            validator,
            security_scanner,
            storage_path,
        }
    }
    
    /// 验证包文件格式
    async fn validate_package_file(&self, file_path: &str) -> Result<()> {
        let path = Path::new(file_path);
        
        // 检查文件是否存在
        if !path.exists() {
            return Err(MarketplaceError::publish("工具包文件不存在"));
        }
        
        // 检查文件扩展名
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("tar") | Some("gz") | Some("zip") => Ok(()),
            _ => Err(MarketplaceError::publish("不支持的文件格式，请使用 .tar.gz 或 .zip")),
        }
    }
    
    /// 提取包元数据
    async fn extract_package_metadata(&self, file_path: &str) -> Result<ToolPackage> {
        // 这里应该实现实际的文件解析逻辑
        // 目前返回一个示例包
        self.validate_package_file(file_path).await?;
        
        // TODO: 实现实际的包解析逻辑
        Err(MarketplaceError::publish("包解析功能尚未实现"))
    }
}

#[async_trait]
impl ToolPublisher for DefaultToolPublisher {
    async fn publish(&self, request: PublishRequest) -> Result<PublishResult> {
        let mut result = PublishResult {
            success: false,
            package_id: None,
            validation_result: None,
            security_audit: None,
            error_message: None,
            published_at: Utc::now(),
        };
        
        // 验证发布权限
        if !self.verify_publish_permission(&request.publisher_info, &request.package).await? {
            result.error_message = Some("发布权限验证失败".to_string());
            return Ok(result);
        }
        
        // 处理工具包文件
        let mut package = match self.process_package_file(&request.package_path).await {
            Ok(pkg) => pkg,
            Err(e) => {
                result.error_message = Some(format!("处理工具包文件失败: {}", e));
                return Ok(result);
            }
        };
        
        // 验证工具包
        if !request.skip_validation {
            match self.validator.validate(&package).await {
                Ok(validation_result) => {
                    if !validation_result.passed {
                        result.validation_result = Some(validation_result);
                        result.error_message = Some("工具包验证失败".to_string());
                        return Ok(result);
                    }
                    result.validation_result = Some(validation_result);
                }
                Err(e) => {
                    result.error_message = Some(format!("验证过程出错: {}", e));
                    return Ok(result);
                }
            }
        }
        
        // 安全扫描
        if !request.skip_security_scan {
            match self.security_scanner.scan_package(&package).await {
                Ok(security_audit) => {
                    if security_audit.security_level == crate::models::SecurityLevel::Dangerous ||
                       security_audit.security_level == crate::models::SecurityLevel::Malicious {
                        result.security_audit = Some(security_audit);
                        result.error_message = Some("安全扫描发现严重问题".to_string());
                        return Ok(result);
                    }
                    package.security_audit = Some(security_audit.clone());
                    result.security_audit = Some(security_audit);
                }
                Err(e) => {
                    result.error_message = Some(format!("安全扫描出错: {}", e));
                    return Ok(result);
                }
            }
        }
        
        // 上传工具包文件
        match self.upload_package_file(package.id, &request.package_path).await {
            Ok(_) => {
                // 标记为已发布
                package.published = true;
                package.published_at = Some(Utc::now());
                
                result.success = true;
                result.package_id = Some(package.id);
            }
            Err(e) => {
                result.error_message = Some(format!("上传文件失败: {}", e));
            }
        }
        
        Ok(result)
    }
    
    async fn verify_publish_permission(&self, publisher: &PublisherInfo, _package: &ToolPackage) -> Result<bool> {
        // 简单的权限验证逻辑
        if publisher.id.is_empty() || publisher.name.is_empty() || publisher.email.is_empty() {
            return Ok(false);
        }
        
        // TODO: 实现更复杂的权限验证逻辑
        // - 检查API密钥
        // - 验证发布者身份
        // - 检查包名称权限
        
        Ok(true)
    }
    
    async fn process_package_file(&self, file_path: &str) -> Result<ToolPackage> {
        self.extract_package_metadata(file_path).await
    }
    
    async fn upload_package_file(&self, package_id: Uuid, file_path: &str) -> Result<String> {
        let source_path = Path::new(file_path);
        let file_name = source_path.file_name()
            .ok_or_else(|| MarketplaceError::publish("无效的文件路径"))?;
        
        let storage_dir = Path::new(&self.storage_path);
        std::fs::create_dir_all(storage_dir)?;
        
        let dest_path = storage_dir.join(format!("{}_{}", package_id, file_name.to_string_lossy()));
        
        // 复制文件到存储目录
        std::fs::copy(source_path, &dest_path)?;
        
        Ok(dest_path.to_string_lossy().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validator::DefaultToolValidator;
    use crate::security::DefaultSecurityScanner;
    use tempfile::{TempDir, NamedTempFile};
    use std::io::Write;
    
    #[tokio::test]
    async fn test_publisher_creation() {
        let validator = Box::new(DefaultToolValidator::new().unwrap());
        let scanner = Box::new(DefaultSecurityScanner::new());
        let temp_dir = TempDir::new().unwrap();
        
        let publisher = DefaultToolPublisher::new(
            validator,
            scanner,
            temp_dir.path().to_string_lossy().to_string(),
        );
        
        // 测试权限验证
        let publisher_info = PublisherInfo {
            id: "test_publisher".to_string(),
            name: "Test Publisher".to_string(),
            email: "test@example.com".to_string(),
            api_key: None,
        };
        
        let package = create_test_package();
        let has_permission = publisher.verify_publish_permission(&publisher_info, &package).await.unwrap();
        assert!(has_permission);
    }
    
    #[tokio::test]
    async fn test_package_file_validation() {
        let validator = Box::new(DefaultToolValidator::new().unwrap());
        let scanner = Box::new(DefaultSecurityScanner::new());
        let temp_dir = TempDir::new().unwrap();
        
        let publisher = DefaultToolPublisher::new(
            validator,
            scanner,
            temp_dir.path().to_string_lossy().to_string(),
        );
        
        // 创建测试文件
        let mut temp_file = NamedTempFile::with_suffix(".tar.gz").unwrap();
        temp_file.write_all(b"test content").unwrap();
        
        let result = publisher.validate_package_file(temp_file.path().to_str().unwrap()).await;
        assert!(result.is_ok());
        
        // 测试不存在的文件
        let result = publisher.validate_package_file("/nonexistent/file.tar.gz").await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_invalid_publisher_info() {
        let validator = Box::new(DefaultToolValidator::new().unwrap());
        let scanner = Box::new(DefaultSecurityScanner::new());
        let temp_dir = TempDir::new().unwrap();
        
        let publisher = DefaultToolPublisher::new(
            validator,
            scanner,
            temp_dir.path().to_string_lossy().to_string(),
        );
        
        // 测试无效的发布者信息
        let invalid_publisher_info = PublisherInfo {
            id: "".to_string(), // 空ID
            name: "Test Publisher".to_string(),
            email: "test@example.com".to_string(),
            api_key: None,
        };
        
        let package = create_test_package();
        let has_permission = publisher.verify_publish_permission(&invalid_publisher_info, &package).await.unwrap();
        assert!(!has_permission);
    }
    
    fn create_test_package() -> ToolPackage {
        use chrono::Utc;
        use semver::Version;
        use std::collections::HashMap;
        
        ToolPackage {
            id: Uuid::new_v4(),
            name: "test_tool".to_string(),
            version: Version::new(1, 0, 0),
            description: "Test tool description".to_string(),
            author: "Test Author".to_string(),
            author_email: Some("test@example.com".to_string()),
            license: "MIT".to_string(),
            homepage: None,
            repository: None,
            keywords: vec!["test".to_string()],
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
