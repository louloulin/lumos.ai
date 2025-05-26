//! Integration tests for the complete telemetry system
//! 
//! These tests verify that all telemetry components work together correctly,
//! including agent execution with telemetry, memory operations monitoring,
//! and real-time metrics collection.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

use crate::agent::{Agent, AgentConfig, Base, Component};
use crate::memory::{BasicWorkingMemory, SemanticMemory, WorkingMemory};
use crate::llm::{LlmProvider, LlmConfig, LlmResponse};
use crate::tools::{Tool, ToolResult};
use crate::telemetry::{
    AgentMetrics, ToolMetrics, MemoryMetrics, ExecutionContext, MetricValue,
    TokenUsage, MetricsCollector, TraceCollector, ExecutionTrace, TraceStep, StepType,
    InMemoryMetricsCollector, FileSystemMetricsCollector
};
use crate::logger::{Logger, LogLevel, ConsoleLogger};

// Mock implementations for testing

#[derive(Clone)]
pub struct MockLlmProvider {
    pub delay_ms: u64,
    pub should_fail: bool,
}

impl MockLlmProvider {
    pub fn new() -> Self {
        Self {
            delay_ms: 100,
            should_fail: false,
        }
    }
    
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }
    
    pub fn with_failure(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }
}

#[async_trait::async_trait]
impl LlmProvider for MockLlmProvider {
    async fn generate(&self, _prompt: &str, _config: Option<LlmConfig>) -> Result<LlmResponse, Box<dyn std::error::Error + Send + Sync>> {
        sleep(Duration::from_millis(self.delay_ms)).await;
        
        if self.should_fail {
            return Err("Mock LLM failure".into());
        }
        
        Ok(LlmResponse {
            content: "Mock response".to_string(),
            usage: Some(TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            }),
            model: "mock-model".to_string(),
            finish_reason: Some("stop".to_string()),
        })
    }
}

#[derive(Clone)]
pub struct MockTool {
    pub name: String,
    pub delay_ms: u64,
    pub should_fail: bool,
}

impl MockTool {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            delay_ms: 50,
            should_fail: false,
        }
    }
    
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }
    
    pub fn with_failure(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }
}

#[async_trait::async_trait]
impl Tool for MockTool {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        "Mock tool for testing"
    }
    
    async fn execute(&self, _input: serde_json::Value) -> ToolResult {
        sleep(Duration::from_millis(self.delay_ms)).await;
        
        if self.should_fail {
            return ToolResult::error("Mock tool failure");
        }
        
        ToolResult::success(serde_json::json!({
            "result": "mock tool output",
            "tool": self.name
        }))
    }
}

/// Test complete agent execution flow with telemetry
#[tokio::test]
async fn test_agent_execution_with_telemetry() {
    let collector = Arc::new(InMemoryMetricsCollector::new());
    let logger = Arc::new(ConsoleLogger::new(LogLevel::Debug));
    
    // Create mock LLM provider
    let llm_provider = Arc::new(MockLlmProvider::new().with_delay(100));
    
    // Create mock tools
    let mut tools = HashMap::new();
    tools.insert("calculator".to_string(), Box::new(MockTool::new("calculator").with_delay(50)) as Box<dyn Tool>);
    tools.insert("weather".to_string(), Box::new(MockTool::new("weather").with_delay(75)) as Box<dyn Tool>);
    
    // Create agent with telemetry
    let mut agent = Agent::new(AgentConfig {
        name: "test_agent".to_string(),
        description: "Test agent for telemetry integration".to_string(),
        llm_config: LlmConfig::default(),
        max_iterations: 3,
        memory_config: None,
    });
    
    agent.set_llm_provider(llm_provider);
    agent.set_tools(tools);
    agent.set_logger(logger.clone());
    agent.set_metrics_collector(collector.clone());
    
    // Create execution context
    let context = ExecutionContext {
        session_id: Some("test_session".to_string()),
        user_id: Some("test_user".to_string()),
        request_id: Some(Uuid::new_v4().to_string()),
        environment: "test".to_string(),
        version: Some("1.0.0".to_string()),
    };
    
    // Execute agent
    let start_time = std::time::Instant::now();
    let result = agent.execute("Calculate the weather temperature", Some(context)).await;
    let execution_time = start_time.elapsed();
    
    // Verify execution succeeded
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.content.is_empty());
    
    // Verify metrics were collected
    let summary = collector.get_metrics_summary(Some("test_agent"), None, None).await.unwrap();
    assert_eq!(summary.total_executions, 1);
    assert_eq!(summary.successful_executions, 1);
    assert_eq!(summary.failed_executions, 0);
    assert!(summary.average_execution_time_ms > 0.0);
    assert!(summary.total_tokens_used > 0);
    
    // Verify agent performance metrics
    let performance = collector.get_agent_performance("test_agent").await.unwrap();
    assert_eq!(performance.agent_name, "test_agent");
    assert_eq!(performance.executions_last_24h, 1);
    assert_eq!(performance.success_rate_24h, 1.0);
    assert!(performance.avg_execution_time_ms > 0.0);
    
    println!("✅ Agent execution with telemetry test passed");
    println!("   Execution time: {:?}", execution_time);
    println!("   Total tokens: {}", summary.total_tokens_used);
    println!("   Average execution time: {:.2}ms", summary.average_execution_time_ms);
}

/// Test memory operations with telemetry integration
#[tokio::test]
async fn test_memory_operations_with_telemetry() {
    let collector = Arc::new(InMemoryMetricsCollector::new());
    
    // Test working memory with telemetry
    let mut working_memory = BasicWorkingMemory::new().with_metrics_collector(collector.clone());
    
    // Test memory operations
    working_memory.update("key1", serde_json::json!({"data": "value1"})).await.unwrap();
    working_memory.update("key2", serde_json::json!({"data": "value2"})).await.unwrap();
    
    let value = working_memory.get("key1").await.unwrap();
    assert!(value.is_some());
    
    working_memory.set_value("direct_key", "direct_value".to_string()).await.unwrap();
    let direct_value = working_memory.get_value("direct_key").await.unwrap();
    assert_eq!(direct_value, Some("direct_value".to_string()));
    
    // Test semantic memory with telemetry
    let mut semantic_memory = SemanticMemory::new("test_collection").with_metrics_collector(collector.clone());
    
    let entry_id = semantic_memory.add_entry("Test content for semantic search").await.unwrap();
    let search_results = semantic_memory.search("test content", Some(5)).await.unwrap();
    assert!(!search_results.is_empty());
    
    let retrieved_entry = semantic_memory.get_entry(&entry_id).await.unwrap();
    assert!(retrieved_entry.is_some());
    
    // Give some time for metrics to be recorded
    sleep(Duration::from_millis(100)).await;
    
    // Verify memory metrics were collected
    let summary = collector.get_metrics_summary(None, None, None).await.unwrap();
    assert!(summary.total_memory_operations > 0);
    
    println!("✅ Memory operations with telemetry test passed");
    println!("   Total memory operations: {}", summary.total_memory_operations);
}

/// Test concurrent agent executions with telemetry
#[tokio::test]
async fn test_concurrent_executions_with_telemetry() {
    let collector = Arc::new(InMemoryMetricsCollector::new());
    let logger = Arc::new(ConsoleLogger::new(LogLevel::Info));
    
    let num_agents = 5;
    let mut handles = Vec::new();
    
    for i in 0..num_agents {
        let collector_clone = collector.clone();
        let logger_clone = logger.clone();
        
        let handle = tokio::spawn(async move {
            let llm_provider = Arc::new(MockLlmProvider::new().with_delay(50 + i * 10));
            
            let mut tools = HashMap::new();
            tools.insert(
                format!("tool_{}", i),
                Box::new(MockTool::new(&format!("tool_{}", i)).with_delay(30)) as Box<dyn Tool>
            );
            
            let mut agent = Agent::new(AgentConfig {
                name: format!("agent_{}", i),
                description: format!("Test agent {} for concurrent telemetry", i),
                llm_config: LlmConfig::default(),
                max_iterations: 2,
                memory_config: None,
            });
            
            agent.set_llm_provider(llm_provider);
            agent.set_tools(tools);
            agent.set_logger(logger_clone);
            agent.set_metrics_collector(collector_clone);
            
            let context = ExecutionContext {
                session_id: Some(format!("session_{}", i)),
                user_id: Some(format!("user_{}", i)),
                request_id: Some(Uuid::new_v4().to_string()),
                environment: "test".to_string(),
                version: Some("1.0.0".to_string()),
            };
            
            agent.execute(&format!("Task {}", i), Some(context)).await
        });
        
        handles.push(handle);
    }
    
    // Wait for all executions to complete
    let mut successful_executions = 0;
    for handle in handles {
        let result = handle.await.unwrap();
        if result.is_ok() {
            successful_executions += 1;
        }
    }
    
    // Verify all executions were recorded
    assert_eq!(successful_executions, num_agents);
    
    let summary = collector.get_metrics_summary(None, None, None).await.unwrap();
    assert_eq!(summary.total_executions, num_agents as u64);
    assert_eq!(summary.successful_executions, num_agents as u64);
    assert_eq!(summary.failed_executions, 0);
    
    println!("✅ Concurrent executions with telemetry test passed");
    println!("   Total concurrent executions: {}", num_agents);
    println!("   Success rate: 100%");
}

/// Test error scenarios and telemetry recording
#[tokio::test]
async fn test_error_scenarios_with_telemetry() {
    let collector = Arc::new(InMemoryMetricsCollector::new());
    let logger = Arc::new(ConsoleLogger::new(LogLevel::Debug));
    
    // Test LLM failure
    let llm_provider = Arc::new(MockLlmProvider::new().with_failure(true));
    let mut tools = HashMap::new();
    tools.insert("test_tool".to_string(), Box::new(MockTool::new("test_tool")) as Box<dyn Tool>);
    
    let mut agent = Agent::new(AgentConfig {
        name: "failing_agent".to_string(),
        description: "Agent that fails for testing".to_string(),
        llm_config: LlmConfig::default(),
        max_iterations: 1,
        memory_config: None,
    });
    
    agent.set_llm_provider(llm_provider);
    agent.set_tools(tools);
    agent.set_logger(logger.clone());
    agent.set_metrics_collector(collector.clone());
    
    let context = ExecutionContext {
        session_id: Some("error_session".to_string()),
        user_id: Some("error_user".to_string()),
        request_id: Some(Uuid::new_v4().to_string()),
        environment: "test".to_string(),
        version: Some("1.0.0".to_string()),
    };
    
    // Execute and expect failure
    let result = agent.execute("This should fail", Some(context)).await;
    assert!(result.is_err());
    
    // Test tool failure
    let llm_provider_success = Arc::new(MockLlmProvider::new());
    let mut failing_tools = HashMap::new();
    failing_tools.insert(
        "failing_tool".to_string(),
        Box::new(MockTool::new("failing_tool").with_failure(true)) as Box<dyn Tool>
    );
    
    let mut agent2 = Agent::new(AgentConfig {
        name: "tool_failing_agent".to_string(),
        description: "Agent with failing tool".to_string(),
        llm_config: LlmConfig::default(),
        max_iterations: 2,
        memory_config: None,
    });
    
    agent2.set_llm_provider(llm_provider_success);
    agent2.set_tools(failing_tools);
    agent2.set_logger(logger);
    agent2.set_metrics_collector(collector.clone());
    
    let context2 = ExecutionContext {
        session_id: Some("tool_error_session".to_string()),
        user_id: Some("tool_error_user".to_string()),
        request_id: Some(Uuid::new_v4().to_string()),
        environment: "test".to_string(),
        version: Some("1.0.0".to_string()),
    };
    
    let result2 = agent2.execute("Use the failing tool", Some(context2)).await;
    // This might succeed with error handling, depending on implementation
    
    // Verify error metrics were recorded
    let summary = collector.get_metrics_summary(None, None, None).await.unwrap();
    assert!(summary.total_executions >= 1);
    assert!(summary.failed_executions >= 1);
    
    println!("✅ Error scenarios with telemetry test passed");
    println!("   Total executions: {}", summary.total_executions);
    println!("   Failed executions: {}", summary.failed_executions);
    println!("   Success rate: {:.2}%", 
        (summary.successful_executions as f64 / summary.total_executions as f64) * 100.0
    );
}

/// Test telemetry data persistence and retrieval
#[tokio::test]
async fn test_telemetry_persistence() {
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new().unwrap();
    let fs_collector = Arc::new(
        FileSystemMetricsCollector::new(temp_dir.path().to_path_buf())
            .await
            .unwrap()
    );
    
    // Record some metrics
    let agent_metrics = AgentMetrics {
        agent_name: "persistent_agent".to_string(),
        execution_id: Uuid::new_v4().to_string(),
        start_time: 1000,
        end_time: 2000,
        execution_time_ms: 1000,
        success: true,
        token_usage: TokenUsage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
        },
        tool_calls_count: 2,
        memory_operations: 1,
        error_count: 0,
        custom_metrics: {
            let mut custom = HashMap::new();
            custom.insert("test_metric".to_string(), MetricValue::Float(42.5));
            custom
        },
        context: ExecutionContext {
            session_id: Some("persist_session".to_string()),
            user_id: Some("persist_user".to_string()),
            request_id: Some(Uuid::new_v4().to_string()),
            environment: "test".to_string(),
            version: Some("1.0.0".to_string()),
        },
    };
    
    fs_collector.record_agent_execution(agent_metrics).await.unwrap();
    
    // Create new collector instance to test persistence
    let fs_collector2 = Arc::new(
        FileSystemMetricsCollector::new(temp_dir.path().to_path_buf())
            .await
            .unwrap()
    );
    
    // Verify data persisted
    let summary = fs_collector2.get_metrics_summary(Some("persistent_agent"), None, None).await.unwrap();
    assert_eq!(summary.total_executions, 1);
    assert_eq!(summary.successful_executions, 1);
    assert_eq!(summary.total_tokens_used, 150);
    
    let performance = fs_collector2.get_agent_performance("persistent_agent").await.unwrap();
    assert_eq!(performance.agent_name, "persistent_agent");
    assert_eq!(performance.executions_last_24h, 1);
    
    println!("✅ Telemetry persistence test passed");
    println!("   Data successfully persisted and retrieved from filesystem");
}

/// Test real-time metrics aggregation
#[tokio::test]
async fn test_realtime_metrics_aggregation() {
    let collector = Arc::new(InMemoryMetricsCollector::new());
    
    // Simulate real-time data collection
    let mut handles = Vec::new();
    let num_concurrent_agents = 3;
    
    for i in 0..num_concurrent_agents {
        let collector_clone = collector.clone();
        
        let handle = tokio::spawn(async move {
            for j in 0..5 {
                let agent_metrics = AgentMetrics {
                    agent_name: format!("realtime_agent_{}", i),
                    execution_id: Uuid::new_v4().to_string(),
                    start_time: j * 1000,
                    end_time: (j + 1) * 1000,
                    execution_time_ms: 1000,
                    success: j % 2 == 0, // Alternate success/failure
                    token_usage: TokenUsage {
                        prompt_tokens: 10 + j,
                        completion_tokens: 20 + j,
                        total_tokens: 30 + (j * 2),
                    },
                    tool_calls_count: j,
                    memory_operations: j / 2,
                    error_count: if j % 2 == 0 { 0 } else { 1 },
                    custom_metrics: HashMap::new(),
                    context: ExecutionContext {
                        session_id: Some(format!("realtime_session_{}_{}", i, j)),
                        user_id: Some(format!("realtime_user_{}", i)),
                        request_id: Some(Uuid::new_v4().to_string()),
                        environment: "test".to_string(),
                        version: Some("1.0.0".to_string()),
                    },
                };
                
                collector_clone.record_agent_execution(agent_metrics).await.unwrap();
                
                // Small delay to simulate real-time data
                sleep(Duration::from_millis(10)).await;
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all data collection to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify aggregated metrics
    let total_summary = collector.get_metrics_summary(None, None, None).await.unwrap();
    assert_eq!(total_summary.total_executions, (num_concurrent_agents * 5) as u64);
    assert!(total_summary.successful_executions > 0);
    assert!(total_summary.failed_executions > 0);
    assert!(total_summary.total_tokens_used > 0);
    
    // Verify per-agent metrics
    for i in 0..num_concurrent_agents {
        let agent_name = format!("realtime_agent_{}", i);
        let agent_performance = collector.get_agent_performance(&agent_name).await.unwrap();
        assert_eq!(agent_performance.agent_name, agent_name);
        assert_eq!(agent_performance.executions_last_24h, 5);
        assert_eq!(agent_performance.success_rate_24h, 0.6); // 3 success out of 5
    }
    
    println!("✅ Real-time metrics aggregation test passed");
    println!("   Total executions: {}", total_summary.total_executions);
    println!("   Success rate: {:.2}%", 
        (total_summary.successful_executions as f64 / total_summary.total_executions as f64) * 100.0
    );
}

/// Integration test runner
pub async fn run_all_integration_tests() {
    println!("🚀 Starting Lumosai Telemetry Integration Tests");
    println!("================================================");
    
    test_agent_execution_with_telemetry().await;
    test_memory_operations_with_telemetry().await;
    test_concurrent_executions_with_telemetry().await;
    test_error_scenarios_with_telemetry().await;
    test_telemetry_persistence().await;
    test_realtime_metrics_aggregation().await;
    
    println!("================================================");
    println!("✅ All Lumosai Telemetry Integration Tests Passed!");
    println!("🎉 Telemetry system is fully operational and ready for production");
}
