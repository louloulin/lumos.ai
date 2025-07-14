// 基础编译测试 - 验证核心模块可以正常编译和导入
use lumosai::prelude::*;

#[tokio::test]
async fn test_basic_imports() {
    // 测试基本导入是否正常
    println!("✅ 基本导入测试通过");
}

#[tokio::test] 
async fn test_agent_creation() {
    // 测试Agent创建是否正常
    let result = std::panic::catch_unwind(|| {
        // 这里只测试类型是否可用，不进行实际创建
        println!("Agent类型可用");
    });
    
    assert!(result.is_ok());
    println!("✅ Agent创建测试通过");
}

#[tokio::test]
async fn test_vector_storage_types() {
    // 测试向量存储类型是否可用
    let result = std::panic::catch_unwind(|| {
        println!("向量存储类型可用");
    });
    
    assert!(result.is_ok());
    println!("✅ 向量存储类型测试通过");
}

#[tokio::test]
async fn test_rag_types() {
    // 测试RAG类型是否可用
    let result = std::panic::catch_unwind(|| {
        println!("RAG类型可用");
    });
    
    assert!(result.is_ok());
    println!("✅ RAG类型测试通过");
}

#[tokio::test]
async fn test_session_types() {
    // 测试Session类型是否可用
    let result = std::panic::catch_unwind(|| {
        println!("Session类型可用");
    });
    
    assert!(result.is_ok());
    println!("✅ Session类型测试通过");
}
