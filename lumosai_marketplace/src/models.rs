//! 工具市场数据模型定义

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use semver::Version;
use std::collections::HashMap;
use serde_json::Value;

/// 工具包元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPackage {
    /// 工具包ID
    pub id: Uuid,
    
    /// 工具包名称
    pub name: String,
    
    /// 版本
    pub version: Version,
    
    /// 描述
    pub description: String,
    
    /// 作者
    pub author: String,
    
    /// 作者邮箱
    pub author_email: Option<String>,
    
    /// 许可证
    pub license: String,
    
    /// 主页URL
    pub homepage: Option<String>,
    
    /// 仓库URL
    pub repository: Option<String>,
    
    /// 关键词
    pub keywords: Vec<String>,
    
    /// 分类
    pub categories: Vec<ToolCategory>,
    
    /// 依赖
    pub dependencies: HashMap<String, String>,
    
    /// Lumos版本要求
    pub lumos_version: String,
    
    /// 工具清单
    pub manifest: ToolManifest,
    
    /// 额外元数据
    pub metadata: HashMap<String, Value>,
    
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    
    /// 发布时间
    pub published_at: Option<DateTime<Utc>>,
    
    /// 下载次数
    pub download_count: u64,
    
    /// 评分
    pub rating: f64,
    
    /// 评分数量
    pub rating_count: u32,
    
    /// 是否已发布
    pub published: bool,
    
    /// 是否已验证
    pub verified: bool,
    
    /// 安全扫描结果
    pub security_audit: Option<SecurityAuditResult>,
    
    /// 性能基准测试结果
    pub performance_benchmark: Option<PerformanceBenchmark>,
}

/// 工具分类
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ToolCategory {
    /// 网络工具
    Web,
    /// 文件操作
    File,
    /// 数据处理
    Data,
    /// AI相关
    AI,
    /// 系统工具
    System,
    /// 数学计算
    Math,
    /// 加密工具
    Crypto,
    /// 数据库
    Database,
    /// API工具
    API,
    /// 实用工具
    Utility,
    /// 自定义
    Custom,
}

impl ToolCategory {
    /// 获取分类的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            ToolCategory::Web => "网络工具",
            ToolCategory::File => "文件操作",
            ToolCategory::Data => "数据处理",
            ToolCategory::AI => "AI相关",
            ToolCategory::System => "系统工具",
            ToolCategory::Math => "数学计算",
            ToolCategory::Crypto => "加密工具",
            ToolCategory::Database => "数据库",
            ToolCategory::API => "API工具",
            ToolCategory::Utility => "实用工具",
            ToolCategory::Custom => "自定义",
        }
    }
    
    /// 获取分类的图标
    pub fn emoji(&self) -> &'static str {
        match self {
            ToolCategory::Web => "🌐",
            ToolCategory::File => "📁",
            ToolCategory::Data => "📊",
            ToolCategory::AI => "🤖",
            ToolCategory::System => "⚙️",
            ToolCategory::Math => "🔢",
            ToolCategory::Crypto => "🔐",
            ToolCategory::Database => "🗄️",
            ToolCategory::API => "🔌",
            ToolCategory::Utility => "🛠️",
            ToolCategory::Custom => "🎨",
        }
    }
}

/// 工具清单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManifest {
    /// 工具定义列表
    pub tools: Vec<ToolDefinition>,
    
    /// 入口点
    pub entry_point: String,
    
    /// 导出的符号
    pub exports: Vec<String>,
    
    /// 权限要求
    pub permissions: Vec<Permission>,
    
    /// 配置模式
    pub config_schema: Option<Value>,
    
    /// 最小Rust版本
    pub rust_version: Option<String>,
    
    /// 构建脚本
    pub build_script: Option<String>,
}

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// 工具名称
    pub name: String,
    
    /// 工具描述
    pub description: String,
    
    /// 参数定义
    pub parameters: Vec<ParameterDefinition>,
    
    /// 返回值定义
    pub returns: ReturnDefinition,
    
    /// 使用示例
    pub examples: Vec<ToolExample>,
    
    /// 标签
    pub tags: Vec<String>,
    
    /// 是否为异步工具
    pub async_tool: bool,
    
    /// 是否需要认证
    pub requires_auth: bool,
    
    /// 权限要求
    pub permissions: Vec<Permission>,
}

/// 参数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDefinition {
    /// 参数名称
    pub name: String,
    
    /// 参数描述
    pub description: String,
    
    /// 参数类型
    pub r#type: String,
    
    /// 是否必需
    pub required: bool,
    
    /// 默认值
    pub default: Option<Value>,
    
    /// 验证规则
    pub validation: Option<ValidationRule>,
    
    /// 示例值
    pub examples: Vec<Value>,
}

/// 返回值定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnDefinition {
    /// 返回类型
    pub r#type: String,
    
    /// 返回描述
    pub description: String,
    
    /// JSON模式
    pub schema: Option<Value>,
    
    /// 示例返回值
    pub examples: Vec<Value>,
}

/// 工具使用示例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExample {
    /// 示例标题
    pub title: String,
    
    /// 示例描述
    pub description: String,
    
    /// 输入参数
    pub input: Value,
    
    /// 期望输出
    pub output: Value,
    
    /// 代码示例
    pub code: Option<String>,
}

/// 验证规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// 最小值
    pub min: Option<f64>,
    
    /// 最大值
    pub max: Option<f64>,
    
    /// 最小长度
    pub min_length: Option<usize>,
    
    /// 最大长度
    pub max_length: Option<usize>,
    
    /// 正则表达式模式
    pub pattern: Option<String>,
    
    /// 枚举值
    pub enum_values: Option<Vec<Value>>,
    
    /// 自定义验证器
    pub custom_validator: Option<String>,
}

/// 权限定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
    /// 文件系统读取
    FileRead,
    /// 文件系统写入
    FileWrite,
    /// 网络访问
    Network,
    /// 系统命令执行
    SystemCommand,
    /// 环境变量访问
    Environment,
    /// 数据库访问
    Database,
    /// 加密操作
    Crypto,
    /// 用户数据访问
    UserData,
    /// 管理员权限
    Admin,
    /// 自定义权限
    Custom(String),
}

impl Permission {
    /// 获取权限的显示名称
    pub fn display_name(&self) -> String {
        match self {
            Permission::FileRead => "文件读取".to_string(),
            Permission::FileWrite => "文件写入".to_string(),
            Permission::Network => "网络访问".to_string(),
            Permission::SystemCommand => "系统命令".to_string(),
            Permission::Environment => "环境变量".to_string(),
            Permission::Database => "数据库访问".to_string(),
            Permission::Crypto => "加密操作".to_string(),
            Permission::UserData => "用户数据".to_string(),
            Permission::Admin => "管理员权限".to_string(),
            Permission::Custom(name) => format!("自定义: {}", name),
        }
    }
    
    /// 获取权限的风险级别
    pub fn risk_level(&self) -> RiskLevel {
        match self {
            Permission::FileRead | Permission::Environment => RiskLevel::Low,
            Permission::FileWrite | Permission::Network | Permission::Database => RiskLevel::Medium,
            Permission::SystemCommand | Permission::Crypto | Permission::UserData | Permission::Admin => RiskLevel::High,
            Permission::Custom(_) => RiskLevel::Medium,
        }
    }
}

/// 风险级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    /// 低风险
    Low,
    /// 中等风险
    Medium,
    /// 高风险
    High,
}

/// 安全审计结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditResult {
    /// 审计时间
    pub audit_time: DateTime<Utc>,
    
    /// 安全级别
    pub security_level: SecurityLevel,
    
    /// 发现的问题
    pub issues: Vec<SecurityIssue>,
    
    /// 审计分数 (0-100)
    pub score: u8,
    
    /// 审计报告
    pub report: String,
    
    /// 审计器版本
    pub auditor_version: String,
}

/// 安全级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecurityLevel {
    /// 安全
    Safe,
    /// 警告
    Warning,
    /// 危险
    Dangerous,
    /// 恶意
    Malicious,
}

/// 安全问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    /// 问题类型
    pub issue_type: SecurityIssueType,
    
    /// 严重程度
    pub severity: Severity,
    
    /// 问题描述
    pub description: String,
    
    /// 文件位置
    pub file_path: Option<String>,
    
    /// 行号
    pub line_number: Option<u32>,
    
    /// 修复建议
    pub fix_suggestion: Option<String>,
}

/// 安全问题类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityIssueType {
    /// 代码注入
    CodeInjection,
    /// 路径遍历
    PathTraversal,
    /// 不安全的依赖
    UnsafeDependency,
    /// 硬编码密钥
    HardcodedSecret,
    /// 不安全的网络请求
    UnsafeNetworkRequest,
    /// 权限滥用
    PermissionAbuse,
    /// 恶意代码
    MaliciousCode,
    /// 其他
    Other(String),
}

/// 严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// 信息
    Info,
    /// 低
    Low,
    /// 中等
    Medium,
    /// 高
    High,
    /// 严重
    Critical,
}

/// 性能基准测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBenchmark {
    /// 测试时间
    pub benchmark_time: DateTime<Utc>,
    
    /// 平均执行时间（毫秒）
    pub avg_execution_time_ms: f64,
    
    /// 内存使用量（字节）
    pub memory_usage_bytes: u64,
    
    /// CPU使用率（百分比）
    pub cpu_usage_percent: f64,
    
    /// 吞吐量（操作/秒）
    pub throughput_ops_per_sec: f64,
    
    /// 测试环境
    pub test_environment: TestEnvironment,
    
    /// 基准测试分数
    pub benchmark_score: u32,
}

/// 测试环境
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEnvironment {
    /// 操作系统
    pub os: String,
    
    /// CPU型号
    pub cpu: String,
    
    /// 内存大小（GB）
    pub memory_gb: u32,
    
    /// Rust版本
    pub rust_version: String,
    
    /// 编译器优化级别
    pub optimization_level: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tool_category_display() {
        assert_eq!(ToolCategory::Web.display_name(), "网络工具");
        assert_eq!(ToolCategory::Web.emoji(), "🌐");
    }
    
    #[test]
    fn test_permission_risk_level() {
        assert_eq!(Permission::FileRead.risk_level(), RiskLevel::Low);
        assert_eq!(Permission::SystemCommand.risk_level(), RiskLevel::High);
    }
    
    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
        assert!(Severity::Low > Severity::Info);
    }
}
