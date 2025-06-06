//! 内置工具模块
//!
//! 提供常用的内置工具，包括文件操作、网络请求、数据处理等

use std::path::PathBuf;
use crate::tool::Tool;

/// 内置工具配置
#[derive(Debug, Clone)]
pub struct BuiltinToolsConfig {
    /// 文件操作配置
    pub file_ops: Option<FileOpsConfig>,
    /// HTTP客户端配置
    pub http_client: Option<HttpClientConfig>,
    /// 数据处理配置
    pub data_processing: Option<DataProcessingConfig>,
}

/// 文件操作工具配置
#[derive(Debug, Clone)]
pub struct FileOpsConfig {
    /// 允许的根目录
    pub allowed_paths: Vec<PathBuf>,
    /// 是否允许读取隐藏文件
    pub allow_hidden_files: bool,
    /// 是否允许覆盖现有文件
    pub allow_overwrite: bool,
}

/// HTTP客户端工具配置
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// 允许的域名列表（空表示允许所有）
    pub allowed_domains: Vec<String>,
    /// 请求超时时间（秒）
    pub timeout_seconds: u64,
    /// 最大响应大小（字节）
    pub max_response_size: usize,
    /// 用户代理字符串
    pub user_agent: String,
}

/// 数据处理工具配置
#[derive(Debug, Clone)]
pub struct DataProcessingConfig {
    /// 最大处理数据大小（字节）
    pub max_data_size: usize,
    /// 是否启用高级功能
    pub enable_advanced_features: bool,
}

impl Default for BuiltinToolsConfig {
    fn default() -> Self {
        Self {
            file_ops: Some(FileOpsConfig::default()),
            http_client: Some(HttpClientConfig::default()),
            data_processing: Some(DataProcessingConfig::default()),
        }
    }
}

impl Default for FileOpsConfig {
    fn default() -> Self {
        Self {
            allowed_paths: vec![std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))],
            allow_hidden_files: false,
            allow_overwrite: true,
        }
    }
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            allowed_domains: Vec::new(), // 允许所有域名
            timeout_seconds: 30,
            max_response_size: 10 * 1024 * 1024, // 10MB
            user_agent: "Lumosai-Agent/1.0".to_string(),
        }
    }
}

impl Default for DataProcessingConfig {
    fn default() -> Self {
        Self {
            max_data_size: 50 * 1024 * 1024, // 50MB
            enable_advanced_features: true,
        }
    }
}

// Import the new tool modules
pub mod web;
pub mod file;
pub mod data;
pub mod system;
pub mod math;

// Re-export tool creation functions
pub use web::*;
pub use file::*;
pub use data::*;
pub use system::*;
pub use math::*;

/// 创建所有内置工具
///
/// 根据配置创建并返回所有可用的内置工具
pub fn create_all_builtin_tools(_config: &BuiltinToolsConfig) -> Vec<Box<dyn Tool>> {
    vec![
        // Web tools
        Box::new(create_http_request_tool()),
        Box::new(create_web_scraper_tool()),
        Box::new(create_json_api_tool()),
        Box::new(create_url_validator_tool()),

        // File tools
        Box::new(create_file_reader_tool()),
        Box::new(create_file_writer_tool()),
        Box::new(create_directory_lister_tool()),
        Box::new(create_file_info_tool()),

        // Data tools
        Box::new(create_json_parser_tool()),
        Box::new(create_csv_parser_tool()),
        Box::new(create_data_transformer_tool()),
        Box::new(create_excel_reader_tool()),
        Box::new(create_pdf_parser_tool()),
        Box::new(create_data_validator_tool()),
        Box::new(create_data_cleaner_tool()),
        Box::new(create_enhanced_data_transformer_tool()),
        Box::new(create_schema_generator_tool()),

        // System tools
        Box::new(create_datetime_tool()),
        Box::new(create_uuid_generator_tool()),
        Box::new(create_hash_generator_tool()),

        // Math tools
        Box::new(create_calculator_tool()),
        Box::new(create_statistics_tool()),
    ]
}

/// 创建安全的内置工具集
///
/// 创建一个安全配置的工具集，适用于生产环境
pub fn create_safe_builtin_tools(_workspace_path: PathBuf) -> Vec<Box<dyn Tool>> {
    vec![
        // Safe tools for production
        Box::new(create_json_parser_tool()),
        Box::new(create_csv_parser_tool()),
        Box::new(create_data_transformer_tool()),
        Box::new(create_data_validator_tool()),
        Box::new(create_data_cleaner_tool()),
        Box::new(create_enhanced_data_transformer_tool()),
        Box::new(create_schema_generator_tool()),
        Box::new(create_datetime_tool()),
        Box::new(create_uuid_generator_tool()),
        Box::new(create_calculator_tool()),
        Box::new(create_statistics_tool()),
        // Note: File and web tools excluded for security
    ]
}

/// 创建开发环境的内置工具集
///
/// 创建一个适用于开发环境的工具集，权限较为宽松
pub fn create_dev_builtin_tools() -> Vec<Box<dyn Tool>> {
    // Include all tools for development
    let config = BuiltinToolsConfig::default();
    create_all_builtin_tools(&config)
}

/// 获取工具的分类信息
pub fn get_tool_categories() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("文件操作", vec!["file_reader", "file_writer", "directory_lister", "file_info"]),
        ("网络请求", vec!["http_request", "web_scraper", "json_api", "url_validator"]),
        ("数据处理", vec![
            "json_parser", "csv_parser", "data_transformer", "excel_reader",
            "pdf_parser", "data_validator", "data_cleaner", "enhanced_data_transformer",
            "schema_generator"
        ]),
        ("系统工具", vec!["datetime", "uuid_generator", "hash_generator"]),
        ("数学计算", vec!["calculator", "statistics"]),
    ]
}

/// 获取工具的详细信息
pub fn get_tool_info(tool_id: &str) -> Option<ToolInfo> {
    match tool_id {
        "file_read" => Some(ToolInfo {
            id: "file_read",
            name: "文件读取",
            description: "读取指定文件的内容",
            category: "文件操作",
            risk_level: "低",
            required_permissions: vec!["文件系统读取"],
        }),
        "file_write" => Some(ToolInfo {
            id: "file_write",
            name: "文件写入",
            description: "将内容写入到指定文件",
            category: "文件操作",
            risk_level: "中",
            required_permissions: vec!["文件系统写入"],
        }),
        "directory_list" => Some(ToolInfo {
            id: "directory_list",
            name: "目录列表",
            description: "列出指定目录中的文件和子目录",
            category: "文件操作",
            risk_level: "低",
            required_permissions: vec!["文件系统读取"],
        }),
        "http_request" => Some(ToolInfo {
            id: "http_request",
            name: "HTTP请求",
            description: "发送HTTP请求并获取响应",
            category: "网络请求",
            risk_level: "中",
            required_permissions: vec!["网络访问"],
        }),
        "json_api" => Some(ToolInfo {
            id: "json_api",
            name: "JSON API请求",
            description: "发送JSON格式的API请求",
            category: "网络请求",
            risk_level: "中",
            required_permissions: vec!["网络访问"],
        }),
        "json_processor" => Some(ToolInfo {
            id: "json_processor",
            name: "JSON处理器",
            description: "处理JSON数据，支持解析、查询、修改等操作",
            category: "数据处理",
            risk_level: "低",
            required_permissions: vec![],
        }),
        "text_processor" => Some(ToolInfo {
            id: "text_processor",
            name: "文本处理器",
            description: "处理文本数据，支持搜索、替换、分割等操作",
            category: "数据处理",
            risk_level: "低",
            required_permissions: vec![],
        }),
        "data_converter" => Some(ToolInfo {
            id: "data_converter",
            name: "数据转换器",
            description: "在不同数据格式之间进行转换",
            category: "数据处理",
            risk_level: "低",
            required_permissions: vec![],
        }),
        "excel_reader" => Some(ToolInfo {
            id: "excel_reader",
            name: "Excel读取器",
            description: "读取Excel文件(.xlsx, .xls)数据",
            category: "数据处理",
            risk_level: "低",
            required_permissions: vec!["文件系统读取"],
        }),
        "pdf_parser" => Some(ToolInfo {
            id: "pdf_parser",
            name: "PDF解析器",
            description: "从PDF文件提取文本、表格和元数据",
            category: "数据处理",
            risk_level: "低",
            required_permissions: vec!["文件系统读取"],
        }),
        "data_validator" => Some(ToolInfo {
            id: "data_validator",
            name: "数据验证器",
            description: "根据模式验证数据结构和类型",
            category: "数据处理",
            risk_level: "低",
            required_permissions: vec![],
        }),
        "data_cleaner" => Some(ToolInfo {
            id: "data_cleaner",
            name: "数据清洗器",
            description: "清洗和标准化数据",
            category: "数据处理",
            risk_level: "低",
            required_permissions: vec![],
        }),
        "enhanced_data_transformer" => Some(ToolInfo {
            id: "enhanced_data_transformer",
            name: "增强数据转换器",
            description: "执行复杂的数据转换和格式转换",
            category: "数据处理",
            risk_level: "低",
            required_permissions: vec![],
        }),
        "schema_generator" => Some(ToolInfo {
            id: "schema_generator",
            name: "模式生成器",
            description: "从样本数据自动生成数据模式",
            category: "数据处理",
            risk_level: "低",
            required_permissions: vec![],
        }),
        _ => None,
    }
}

/// 工具信息结构
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub risk_level: &'static str,
    pub required_permissions: Vec<&'static str>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_all_builtin_tools() {
        let config = BuiltinToolsConfig::default();
        let tools = create_all_builtin_tools(&config);

        // 应该包含所有22个内置工具
        assert_eq!(tools.len(), 22);
    }

    #[test]
    fn test_safe_builtin_tools() {
        let workspace = PathBuf::from("/tmp/test");
        let tools = create_safe_builtin_tools(workspace);

        // 安全工具集应该包含11个工具（排除文件和网络工具）
        assert_eq!(tools.len(), 11);
    }

    #[test]
    fn test_dev_builtin_tools() {
        let tools = create_dev_builtin_tools();

        // 开发工具集应该包含所有22个工具
        assert_eq!(tools.len(), 22);
    }

    #[test]
    fn test_tool_categories() {
        let categories = get_tool_categories();
        assert_eq!(categories.len(), 5);
        assert_eq!(categories[0].0, "文件操作");
        assert_eq!(categories[1].0, "网络请求");
        assert_eq!(categories[2].0, "数据处理");
        assert_eq!(categories[3].0, "系统工具");
        assert_eq!(categories[4].0, "数学计算");
    }

    #[test]
    fn test_tool_info() {
        let info = get_tool_info("file_read").unwrap();
        assert_eq!(info.id, "file_read");
        assert_eq!(info.category, "文件操作");
        assert_eq!(info.risk_level, "低");

        assert!(get_tool_info("unknown_tool").is_none());
    }
}
