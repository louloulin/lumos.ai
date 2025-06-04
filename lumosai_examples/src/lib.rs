//! Lumosai Examples Library
//! 
//! This library contains example implementations and demonstrations
//! of the Lumosai AI framework capabilities.

pub mod agent_tools;
pub mod agent_tools_simple;
pub mod eval_dsl;
pub mod workflow_dsl;
pub mod advanced_tools;
pub mod basic_usage;
pub mod agent_usage;
pub mod deepseek_integration;
pub mod deepseek_agent_demo;
pub mod deepseek_app;
pub mod lumos_app;
pub mod lumos_macro_usage;
pub mod function_calling_enhancement_demo;
pub mod function_schema_example;
pub mod macro_tool_example;
pub mod mcp_dsl;
pub mod rag_dsl;
pub mod simplified_api_demo;
pub mod workflow_example;

// Re-export commonly used types and functions
pub use agent_tools::{CalculatorTool, WeatherTool};
pub use advanced_tools::{FileManagerTool, DataAnalysisTool};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplified_api_compilation() {
        // 这个测试只是为了验证代码能够编译
        println!("✅ 简化API代码编译成功！");
        println!("🎉 所有示例模块都已成功编译");
    }
}
