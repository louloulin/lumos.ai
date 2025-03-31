use lumosai_core::{
    memory::{WorkingMemoryConfig, create_working_memory},
    Result,
};
use serde_json::json;

#[tokio::test]
async fn test_basic_working_memory() -> Result<()> {
    // 创建工作内存配置
    let working_memory_config = WorkingMemoryConfig {
        enabled: true,
        template: Some(r#"{"initial_key": "initial_value"}"#.to_string()),
        content_type: Some("application/json".to_string()),
        max_capacity: Some(1024),
    };

    // 创建基本工作内存
    let memory = create_working_memory(&working_memory_config)?;
    
    // 检查初始内容
    let content = memory.get().await?;
    assert_eq!(content.content.get("initial_key").unwrap(), &json!("initial_value"));
    
    // 更新内容
    memory.set_value("test_key", json!("updated_value")).await?;
    
    // 检查更新后的内容
    let updated_content = memory.get().await?;
    assert_eq!(updated_content.content.get("test_key").unwrap(), &json!("updated_value"));
    
    // 初始值应该仍然存在
    assert_eq!(updated_content.content.get("initial_key").unwrap(), &json!("initial_value"));
    
    println!("Working memory updated successfully");
    
    Ok(())
} 