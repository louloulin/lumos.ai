//! ContextFS功能验证测试
//!
//! 这个文件用于验证ContextFS实现的基本功能，
//! 独立于其他测试以确保ContextFS核心功能正常工作。

use crate::contextfs::*;
use crate::error::Result;

#[tokio::test]
async fn test_contextfs_basic_operations() -> Result<()> {
    let fs = InMemoryContextFS::new();

    // 创建Agent上下文
    let agent_id = ContextId {
        context_type: ContextType::Agent,
        entity_id: "test-agent".to_string(),
        sub_path: Some("config".to_string()),
    };

    let agent_context = Context::new(
        agent_id.clone(),
        ContextContent::Structured(serde_json::json!({
            "name": "Test Agent",
            "instructions": "You are a helpful assistant.",
            "model": "gpt-4"
        }))
    );

    let path = "/agent/test-agent/config";

    // 测试写入
    fs.write_context(path, agent_context.clone()).await?;

    // 测试读取
    let read_context = fs.read_context(path).await?;
    assert_eq!(read_context.id, agent_id);
    assert_eq!(read_context.version, 1);

    // 测试存在性检查
    assert!(fs.exists_context(path).await?);

    // 测试列出
    let contexts = fs.list_context("/agent/").await?;
    assert_eq!(contexts.len(), 1);
    assert_eq!(contexts[0], agent_id);

    // 测试更新
    let mut updated_context = read_context;
    updated_context.update_content(ContextContent::Structured(serde_json::json!({
        "name": "Test Agent",
        "instructions": "You are a helpful assistant with updated instructions.",
        "model": "gpt-4"
    })));
    fs.write_context(path, updated_context).await?;

    let updated_read = fs.read_context(path).await?;
    assert_eq!(updated_read.version, 2);

    // 测试删除
    fs.delete_context(path).await?;
    assert!(!fs.exists_context(path).await?);

    Ok(())
}

#[tokio::test]
async fn test_contextfs_semantic_search() -> Result<()> {
    let fs = InMemoryContextFS::new();

    // 创建多个上下文
    let agent1 = Context::new(
        ContextId {
            context_type: ContextType::Agent,
            entity_id: "agent1".to_string(),
            sub_path: None,
        },
        ContextContent::Text("I am a helpful AI assistant specializing in programming.".to_string())
    );

    let agent2 = Context::new(
        ContextId {
            context_type: ContextType::Agent,
            entity_id: "agent2".to_string(),
            sub_path: None,
        },
        ContextContent::Text("I am a creative writing assistant who helps with stories.".to_string())
    );

    let memory1 = Context::new(
        ContextId {
            context_type: ContextType::Memory,
            entity_id: "mem1".to_string(),
            sub_path: None,
        },
        ContextContent::Text("The user asked about Rust programming and I helped them.".to_string())
    );

    // 写入上下文
    fs.write_context("/agent/agent1", agent1).await?;
    fs.write_context("/agent/agent2", agent2).await?;
    fs.write_context("/memory/mem1", memory1).await?;

    // 搜索编程相关的上下文
    let query = SemanticQuery {
        query: "programming".to_string(),
        context_types: None,
        entity_ids: None,
        metadata_filters: None,
        threshold: None,
        limit: Some(10),
        sort_by: None,
    };

    let results = fs.search_context(&query).await?;
    assert_eq!(results.len(), 2); // 应该找到agent1和memory1

    // 只搜索Agent类型的上下文
    let agent_query = SemanticQuery {
        query: "assistant".to_string(),
        context_types: Some(vec![ContextType::Agent]),
        entity_ids: None,
        metadata_filters: None,
        threshold: None,
        limit: Some(10),
        sort_by: None,
    };

    let agent_results = fs.search_context(&agent_query).await?;
    assert_eq!(agent_results.len(), 2); // 应该找到两个agent

    Ok(())
}

#[tokio::test]
async fn test_contextfs_path_resolution() -> Result<()> {
    // 测试路径解析
    let agent_path = "/agent/test-agent";
    let agent_id = ContextPathResolver::parse_path(agent_path)?;
    assert_eq!(agent_id.context_type, ContextType::Agent);
    assert_eq!(agent_id.entity_id, "test-agent");
    assert_eq!(agent_id.sub_path, None);

    let reconstructed = ContextPathResolver::to_path(&agent_id);
    assert_eq!(reconstructed, agent_path);

    // 测试带子路径的解析
    let memory_path = "/memory/user123/session456";
    let memory_id = ContextPathResolver::parse_path(memory_path)?;
    assert_eq!(memory_id.context_type, ContextType::Memory);
    assert_eq!(memory_id.entity_id, "user123");
    assert_eq!(memory_id.sub_path, Some("session456".to_string()));

    // 测试系统路径
    let sys_path = "/sys/agentmem/config/database";
    let sys_id = ContextPathResolver::parse_path(sys_path)?;
    assert_eq!(sys_id.context_type, ContextType::System);
    assert_eq!(sys_id.entity_id, "config");
    assert_eq!(sys_id.sub_path, Some("database".to_string()));

    Ok(())
}

#[test]
fn test_context_metadata_and_relations() {
    let mut context = Context::new(
        ContextId {
            context_type: ContextType::Agent,
            entity_id: "test".to_string(),
            sub_path: None,
        },
        ContextContent::Text("Test content".to_string())
    );

    // 测试元数据
    context.add_metadata("author", serde_json::Value::String("AI Assistant".to_string()));
    context.add_metadata("version", serde_json::Value::String("1.0".to_string()));

    assert_eq!(context.metadata.len(), 2);
    assert_eq!(context.metadata.get("author"),
               Some(&serde_json::Value::String("AI Assistant".to_string())));

    // 测试关系
    let relation = ContextRelation {
        relation_type: "depends_on".to_string(),
        target_id: ContextId {
            context_type: ContextType::Memory,
            entity_id: "memory1".to_string(),
            sub_path: None,
        },
        weight: 0.8,
        properties: std::collections::HashMap::new(),
    };

    context = context.with_relation(relation);
    assert_eq!(context.relations.len(), 1);
    assert_eq!(context.relations[0].relation_type, "depends_on");
    assert_eq!(context.relations[0].weight, 0.8);

    // 测试版本控制
    let initial_version = context.version;
    context.update_content(ContextContent::Text("Updated content".to_string()));
    assert_eq!(context.version, initial_version + 1);
}

#[test]
fn test_context_content_types() {
    // 测试文本内容
    let text_content = ContextContent::Text("Hello world".to_string());
    assert!(InMemoryContextFS::matches_query(&text_content, "world"));
    assert!(!InMemoryContextFS::matches_query(&text_content, "nonexistent"));

    // 测试代码内容
    let code_content = ContextContent::Code {
        language: "rust".to_string(),
        content: "fn main() { println!(\"Hello\"); }".to_string(),
    };
    assert!(InMemoryContextFS::matches_query(&code_content, "println"));
    assert!(!InMemoryContextFS::matches_query(&code_content, "python"));

    // 测试结构化内容
    let structured_content = ContextContent::Structured(serde_json::json!("Hello world"));
    assert!(InMemoryContextFS::matches_query(&structured_content, "world"));

    // 测试多模态内容
    let multi_content = ContextContent::MultiModal(vec![
        ContextContent::Text("First part".to_string()),
        ContextContent::Text("Second part".to_string()),
    ]);
    assert!(InMemoryContextFS::matches_query(&multi_content, "First"));
    assert!(InMemoryContextFS::matches_query(&multi_content, "Second"));
    assert!(!InMemoryContextFS::matches_query(&multi_content, "nonexistent"));
}



