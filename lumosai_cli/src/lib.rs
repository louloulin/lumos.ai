pub mod commands;
pub mod error;
pub mod util;
pub mod server;

/// Lumosai CLI工具主要版本号
pub const LUMOSAI_CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Lumosai CLI工具的名称
pub const LUMOSAI_CLI_NAME: &str = env!("CARGO_PKG_NAME");

/// Lumosai CLI工具的描述
pub const LUMOSAI_CLI_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Lumosai CLI工具的作者
pub const LUMOSAI_CLI_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

/// 获取完整的版本信息
pub fn version_info() -> String {
    format!(
        "{} v{}",
        LUMOSAI_CLI_NAME,
        LUMOSAI_CLI_VERSION
    )
}

/// 获取完整的版本字符串，包括额外信息
pub fn full_version_string() -> String {
    format!(
        "{}\n{}\n作者: {}",
        version_info(),
        LUMOSAI_CLI_DESCRIPTION,
        LUMOSAI_CLI_AUTHORS
    )
} 