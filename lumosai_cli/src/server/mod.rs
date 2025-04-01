pub mod ui_server;
pub mod api_server;

use crate::error::CliResult;
use colored::Colorize;

/// 初始化服务器模块
pub fn init() -> CliResult<()> {
    println!("{}", "初始化服务器模块...".bright_blue());
    Ok(())
} 