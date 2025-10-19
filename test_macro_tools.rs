use lumosai_core::tool::builtin::macro_tools::*;
use lumosai_core::tool::{Tool, ToolExecutionContext, ToolExecutionOptions};
use serde_json::{json, Value};
use tempfile::TempDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用 tokio runtime
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        println!("🧪 测试宏驱动的工具实现");

        // 创建临时目录用于测试
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        let test_file = temp_path.join("test.txt");

        // 创建执行上下文
        let context = ToolExecutionContext {
            thread_id: Some("test-thread".to_string()),
            resource_id: Some("test-resource".to_string()),
            run_id: Some("test-run".to_string()),
            tool_call_id: Some("test-call".to_string()),
            messages: None,
            abort_signal: None,
        };

        let options = ToolExecutionOptions {
            context: None,
            validate_params: true,
            validate_output: false,
        };
    
    println!("\n📝 测试文件写入工具");
    let write_tool = write_file_tool();
    println!("工具ID: {}", write_tool.id());
    println!("工具描述: {}", write_tool.description());
    
    let write_params = json!({
        "path": test_file.to_string_lossy(),
        "content": "Hello, LumosAI Macro Tools!",
        "encoding": "utf-8"
    });
    
    let write_result = write_tool.execute(write_params, context.clone(), &options).await?;
    println!("写入结果: {}", write_result);
    
    println!("\n📖 测试文件读取工具");
    let read_tool = read_file_tool();
    println!("工具ID: {}", read_tool.id());
    println!("工具描述: {}", read_tool.description());
    
    let read_params = json!({
        "path": test_file.to_string_lossy(),
        "encoding": "utf-8"
    });
    
    let read_result = read_tool.execute(read_params, context.clone(), &options).await?;
    println!("读取结果: {}", read_result);
    
    println!("\n📁 测试目录列表工具");
    let list_tool = list_directory_tool();
    println!("工具ID: {}", list_tool.id());
    println!("工具描述: {}", list_tool.description());
    
    let list_params = json!({
        "path": temp_path.to_string_lossy(),
        "recursive": false
    });
    
    let list_result = list_tool.execute(list_params, context.clone(), &options).await?;
    println!("目录列表结果: {}", list_result);
    
    println!("\n📊 测试文件信息工具");
    let info_tool = get_file_info_tool();
    println!("工具ID: {}", info_tool.id());
    println!("工具描述: {}", info_tool.description());
    
    let info_params = json!({
        "path": test_file.to_string_lossy()
    });
    
    let info_result = info_tool.execute(info_params, context.clone(), &options).await?;
    println!("文件信息结果: {}", info_result);
    
        println!("\n✅ 所有宏驱动工具测试完成！");
        println!("🎉 宏驱动的工具实现成功验证！");

        Ok::<(), Box<dyn std::error::Error>>(())
    })
}
