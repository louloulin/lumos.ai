//! Memory 单元测试
//!
//! P0-3 任务：增加 Memory 模块的单元测试覆盖率
//! 测试范围：
//! - Memory 创建和配置
//! - Memory 存储和检索
//! - Working Memory 操作
//! - Memory 配置验证

use lumosai_core::llm::{Message, MockLlmProvider, Role};
use lumosai_core::memory::{
    BasicMemory, Memory, MemoryConfig, UnifiedMemory, WorkingMemory, WorkingMemoryConfig,
    create_working_memory,
};
use serde_json::{json, Value};
use std::sync::Arc;

/// 测试 1: BasicMemory 创建
#[test]
fn test_basic_memory_creation() {
    let memory = BasicMemory::new(None, None);
    // Memory 创建成功即可
    let _ = memory;
}

/// 测试 2: BasicMemory 存储消息
#[tokio::test]
async fn test_basic_memory_store() {
    let memory = BasicMemory::new(None, None);
    let message = Message::new(Role::User, "Hello".to_string(), None, None);

    let result = memory.store(&message).await;
    assert!(result.is_ok());
}

/// 测试 3: BasicMemory 检索消息
#[tokio::test]
async fn test_basic_memory_retrieve() {
    let memory = BasicMemory::new(None, None);
    let config = MemoryConfig::default();

    let result = memory.retrieve(&config).await;
    assert!(result.is_ok());
}

/// 测试 4: MemoryConfig 默认值
#[test]
fn test_memory_config_default() {
    let config = MemoryConfig::default();

    assert!(config.enabled);
    assert!(config.store_id.is_none());
    assert!(config.namespace.is_none());
    assert!(config.last_messages.is_none());
}

/// 测试 5: MemoryConfig 自定义配置
#[test]
fn test_memory_config_custom() {
    let config = MemoryConfig {
        enabled: true,
        store_id: Some("test_store".to_string()),
        namespace: Some("test_namespace".to_string()),
        last_messages: Some(10),
        ..Default::default()
    };

    assert!(config.enabled);
    assert_eq!(config.store_id, Some("test_store".to_string()));
    assert_eq!(config.namespace, Some("test_namespace".to_string()));
    assert_eq!(config.last_messages, Some(10));
}

/// 测试 6: WorkingMemoryConfig 创建
#[test]
fn test_working_memory_config_creation() {
    let config = WorkingMemoryConfig {
        enabled: true,
        template: None,
        content_type: Some("application/json".to_string()),
        max_capacity: Some(1024),
    };

    assert!(config.enabled);
    assert_eq!(config.content_type, Some("application/json".to_string()));
    assert_eq!(config.max_capacity, Some(1024));
}

/// 测试 7: WorkingMemory 创建
#[test]
fn test_working_memory_creation() {
    let config = WorkingMemoryConfig {
        enabled: true,
        template: None,
        content_type: Some("application/json".to_string()),
        max_capacity: Some(1024),
    };

    let result = create_working_memory(&config);
    assert!(result.is_ok());
}

/// 测试 8: WorkingMemory 设置和获取值
#[tokio::test]
async fn test_working_memory_set_get() {
    let config = WorkingMemoryConfig {
        enabled: true,
        template: None,
        content_type: Some("application/json".to_string()),
        max_capacity: Some(1024),
    };

    let memory = create_working_memory(&config).unwrap();

    // 设置值
    let result = memory
        .set_value("test_key", Value::String("test_value".to_string()))
        .await;
    assert!(result.is_ok());

    // 获取值
    let value = memory.get_value("test_key").await.unwrap();
    assert_eq!(value, Some(Value::String("test_value".to_string())));
}

/// 测试 9: WorkingMemory 删除值
#[tokio::test]
async fn test_working_memory_delete() {
    let config = WorkingMemoryConfig {
        enabled: true,
        template: None,
        content_type: Some("application/json".to_string()),
        max_capacity: Some(1024),
    };

    let memory = create_working_memory(&config).unwrap();

    // 设置值
    memory
        .set_value("test_key", Value::String("test_value".to_string()))
        .await
        .unwrap();

    // 删除值
    let result = memory.delete_value("test_key").await;
    assert!(result.is_ok());

    // 验证已删除
    let value = memory.get_value("test_key").await.unwrap();
    assert_eq!(value, None);
}

/// 测试 10: WorkingMemory 清空
#[tokio::test]
async fn test_working_memory_clear() {
    let config = WorkingMemoryConfig {
        enabled: true,
        template: None,
        content_type: Some("application/json".to_string()),
        max_capacity: Some(1024),
    };

    let memory = create_working_memory(&config).unwrap();

    // 设置多个值
    memory
        .set_value("key1", Value::String("value1".to_string()))
        .await
        .unwrap();
    memory
        .set_value("key2", Value::String("value2".to_string()))
        .await
        .unwrap();

    // 清空
    let result = memory.clear().await;
    assert!(result.is_ok());

    // 验证已清空
    let value1 = memory.get_value("key1").await.unwrap();
    let value2 = memory.get_value("key2").await.unwrap();
    assert_eq!(value1, None);
    assert_eq!(value2, None);
}

/// 测试 11: UnifiedMemory 基础内存
#[test]
fn test_unified_memory_basic() {
    let memory = UnifiedMemory::basic();
    // Memory 创建成功即可
    let _ = memory;
}

/// 测试 12: UnifiedMemory 工作内存
#[test]
fn test_unified_memory_working() {
    let memory = UnifiedMemory::working(1000);
    // Memory 创建成功即可
    let _ = memory;
}

/// 测试 13: Message 创建
#[test]
fn test_message_creation() {
    let message = Message::new(Role::User, "Hello, world!".to_string(), None, None);

    assert_eq!(message.role, Role::User);
    assert_eq!(message.content, "Hello, world!");
}

/// 测试 14: Message 不同角色
#[test]
fn test_message_different_roles() {
    let user_msg = Message::new(Role::User, "User message".to_string(), None, None);
    let assistant_msg = Message::new(
        Role::Assistant,
        "Assistant message".to_string(),
        None,
        None,
    );
    let system_msg = Message::new(Role::System, "System message".to_string(), None, None);

    assert_eq!(user_msg.role, Role::User);
    assert_eq!(assistant_msg.role, Role::Assistant);
    assert_eq!(system_msg.role, Role::System);
}

/// 测试 15: WorkingMemory 不同数据类型
#[tokio::test]
async fn test_working_memory_different_types() {
    let config = WorkingMemoryConfig {
        enabled: true,
        template: None,
        content_type: Some("application/json".to_string()),
        max_capacity: Some(1024),
    };

    let memory = create_working_memory(&config).unwrap();

    // 字符串
    memory
        .set_value("string", Value::String("test".to_string()))
        .await
        .unwrap();

    // 数字
    memory
        .set_value("number", Value::Number(serde_json::Number::from(42)))
        .await
        .unwrap();

    // 布尔值
    memory.set_value("bool", Value::Bool(true)).await.unwrap();

    // 数组
    memory
        .set_value("array", json!(["a", "b", "c"]))
        .await
        .unwrap();

    // 对象
    memory
        .set_value("object", json!({"key": "value"}))
        .await
        .unwrap();

    // 验证所有值
    assert_eq!(
        memory.get_value("string").await.unwrap(),
        Some(Value::String("test".to_string()))
    );
    assert_eq!(
        memory.get_value("number").await.unwrap(),
        Some(Value::Number(serde_json::Number::from(42)))
    );
    assert_eq!(
        memory.get_value("bool").await.unwrap(),
        Some(Value::Bool(true))
    );
    assert_eq!(
        memory.get_value("array").await.unwrap(),
        Some(json!(["a", "b", "c"]))
    );
    assert_eq!(
        memory.get_value("object").await.unwrap(),
        Some(json!({"key": "value"}))
    );
}

/// 测试 16: WorkingMemory 更新值
#[tokio::test]
async fn test_working_memory_update() {
    let config = WorkingMemoryConfig {
        enabled: true,
        template: None,
        content_type: Some("application/json".to_string()),
        max_capacity: Some(1024),
    };

    let memory = create_working_memory(&config).unwrap();

    // 设置初始值
    memory
        .set_value("key", Value::String("initial".to_string()))
        .await
        .unwrap();

    // 更新值
    memory
        .set_value("key", Value::String("updated".to_string()))
        .await
        .unwrap();

    // 验证更新
    let value = memory.get_value("key").await.unwrap();
    assert_eq!(value, Some(Value::String("updated".to_string())));
}

