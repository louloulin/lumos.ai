//! Comprehensive unit tests for the telemetry system

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::telemetry::{
    AgentMetrics, ToolMetrics, MemoryMetrics, ExecutionContext, MetricValue,
    TokenUsage, MetricsCollector, TraceCollector, ExecutionTrace, TraceStep, StepType, TraceBuilder,
    InMemoryMetricsCollector, FileSystemMetricsCollector, OtelMetricsCollector, OtelConfig
};

#[tokio::test]
async fn test_agent_metrics_creation() {
    let execution_id = Uuid::new_v4().to_string();
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    let agent_metrics = AgentMetrics {
        agent_name: "test_agent".to_string(),
        execution_id: execution_id.clone(),
        start_time,
        end_time: start_time + 1000,
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
            custom.insert("test_metric".to_string(), MetricValue::Integer(42));
            custom
        },
        context: ExecutionContext {
            session_id: Some("test_session".to_string()),
            user_id: Some("test_user".to_string()),
            request_id: Some("test_request".to_string()),
            environment: "test".to_string(),
            version: Some("1.0.0".to_string()),
        },
    };
    
    assert_eq!(agent_metrics.agent_name, "test_agent");
    assert_eq!(agent_metrics.execution_id, execution_id);
    assert_eq!(agent_metrics.execution_time_ms, 1000);
    assert!(agent_metrics.success);
    assert_eq!(agent_metrics.token_usage.total_tokens, 150);
    assert_eq!(agent_metrics.tool_calls_count, 2);
    assert_eq!(agent_metrics.memory_operations, 1);
    assert_eq!(agent_metrics.error_count, 0);
}

#[tokio::test]
async fn test_in_memory_metrics_collector() {
    let collector = InMemoryMetricsCollector::new();
    
    // Create test agent metrics
    let agent_metrics = AgentMetrics {
        agent_name: "test_agent".to_string(),
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
        tool_calls_count: 1,
        memory_operations: 0,
        error_count: 0,
        custom_metrics: HashMap::new(),
        context: ExecutionContext {
            session_id: None,
            user_id: None,
            request_id: None,
            environment: "test".to_string(),
            version: None,
        },
    };
    
    // Record the metrics
    collector.record_agent_execution(agent_metrics.clone()).await.unwrap();
    
    // Get metrics summary
    let summary = collector.get_metrics_summary(Some("test_agent"), None, None).await.unwrap();
    assert_eq!(summary.total_executions, 1);
    assert_eq!(summary.successful_executions, 1);
    assert_eq!(summary.failed_executions, 0);
    assert_eq!(summary.total_tokens_used, 150);
    
    // Get agent performance
    let performance = collector.get_agent_performance("test_agent").await.unwrap();
    assert_eq!(performance.agent_name, "test_agent");
    assert_eq!(performance.executions_last_24h, 1);
    assert_eq!(performance.success_rate_24h, 1.0);
}

#[tokio::test]
async fn test_tool_metrics_collection() {
    let collector = InMemoryMetricsCollector::new();
    
    let tool_metrics = ToolMetrics {
        tool_name: "test_tool".to_string(),
        execution_time_ms: 500,
        success: true,
        error: None,
        input_size_bytes: 100,
        output_size_bytes: 200,
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
    };
    
    collector.record_tool_execution(tool_metrics).await.unwrap();
    
    // Verify tool metrics were recorded (would need additional query methods)
    // This is a basic test to ensure no panics occur
}

#[tokio::test]
async fn test_memory_metrics_collection() {
    let collector = InMemoryMetricsCollector::new();
    
    let memory_metrics = MemoryMetrics {
        operation_type: "get".to_string(),
        execution_time_ms: 10,
        success: true,
        key: Some("test_key".to_string()),
        data_size_bytes: Some(1024),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
    };
    
    collector.record_memory_operation(memory_metrics).await.unwrap();
    
    // Verify memory metrics were recorded
    // This is a basic test to ensure no panics occur
}

#[tokio::test]
async fn test_execution_trace() {
    let collector = InMemoryMetricsCollector::new();
    
    // Start a trace
    let trace_id = collector.start_trace(
        "test_agent".to_string(),
        {
            let mut metadata = HashMap::new();
            metadata.insert("test_key".to_string(), serde_json::Value::String("test_value".to_string()));
            metadata
        }
    ).await.unwrap();
    
    // Add trace steps
    let mut step1 = TraceStep::new("Test LLM call".to_string(), StepType::LlmCall);
    step1.duration_ms = 100;
    step1.success = true;
    collector.add_trace_step(&trace_id, step1).await.unwrap();
    
    let mut step2 = TraceStep::new("Test tool call".to_string(), StepType::ToolCall);
    step2.metadata.insert("tool_name".to_string(), serde_json::Value::String("test_tool".to_string()));
    step2.duration_ms = 200;
    step2.success = true;
    collector.add_trace_step(&trace_id, step2).await.unwrap();
    
    // Complete the trace
    collector.end_trace(&trace_id, true).await.unwrap();
    
    // Get trace stats
    let stats = collector.get_trace_stats(Some("test_agent"), None, None).await.unwrap();
    assert_eq!(stats.total_traces, 1);
    assert_eq!(stats.successful_traces, 1);
    assert_eq!(stats.failed_traces, 0);
}

#[tokio::test]
async fn test_trace_builder() {
    let mut builder = TraceBuilder::new("test_agent".to_string());
    
    // Start a step
    builder.start_step("Starting LLM call".to_string(), StepType::LlmCall);
    
    // Set step input
    builder.set_step_input(serde_json::json!({
        "model": "gpt-4",
        "temperature": 0.7
    }));
    
    // Complete the step
    builder.end_step(true);
    
    // Start another step
    builder.start_step("Tool execution".to_string(), StepType::ToolCall);
    builder.set_step_input(serde_json::json!({
        "tool_name": "calculator"
    }));
    builder.end_step(true);
    
    // Build the trace
    let trace = builder.build();
    
    assert_eq!(trace.agent_id, "test_agent");
    assert_eq!(trace.steps.len(), 2);
    assert!(trace.success);
    assert!(trace.total_duration_ms > 0);
    
    // Check first step
    assert!(matches!(trace.steps[0].step_type, StepType::LlmCall));
    assert_eq!(        trace.steps[0].name, "Starting LLM call");
    assert!(trace.steps[0].success);
    
    // Check second step
    assert!(matches!(trace.steps[1].step_type, StepType::ToolCall));
    assert_eq!(        trace.steps[1].name, "Tool execution");
    assert!(trace.steps[1].success);
}

#[tokio::test]
async fn test_filesystem_metrics_collector() {
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new().unwrap();
    let collector = FileSystemMetricsCollector::new(temp_dir.path().to_path_buf()).unwrap();
    
    let agent_metrics = AgentMetrics {
        agent_name: "fs_test_agent".to_string(),
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
        tool_calls_count: 1,
        memory_operations: 0,
        error_count: 0,
        custom_metrics: HashMap::new(),
        context: ExecutionContext {
            session_id: None,
            user_id: None,
            request_id: None,
            environment: "test".to_string(),
            version: None,
        },
    };
    
    collector.record_agent_execution(agent_metrics).await.unwrap();
    
    let summary = collector.get_metrics_summary(Some("fs_test_agent"), None, None).await.unwrap();
    assert_eq!(summary.total_executions, 1);
    assert_eq!(summary.successful_executions, 1);
}

#[tokio::test]
async fn test_otel_metrics_collector() {
    let config = OtelConfig {
        service_name: "test_service".to_string(),
        service_version: Some("1.0.0".to_string()),
        otlp_endpoint: Some("http://localhost:4318/v1/traces".to_string()),
        sampling_rate: 1.0,
        enable_metrics: true,
        enable_traces: true,
        enable_logs: false,
        batch_size: 10,
        export_timeout_ms: 5000,
        resource_attributes: HashMap::new(),
    };
    
    let inner_collector = Arc::new(InMemoryMetricsCollector::new());
    
    // Create a mock exporter for testing
    struct MockOtelExporter;
    
    #[async_trait::async_trait]
    impl crate::telemetry::otel::OtelExporter for MockOtelExporter {
        async fn export_spans(&self, _spans: Vec<crate::telemetry::otel::OtelSpan>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        
        async fn export_metrics(&self, _metrics: Vec<crate::telemetry::otel::OtelMetric>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        
        async fn force_flush(&self, _timeout: std::time::Duration) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        
        async fn shutdown(&self, _timeout: std::time::Duration) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
    }
    
    let mock_exporter = Box::new(MockOtelExporter);
    let otel_collector = OtelMetricsCollector::new(
        Box::new((*inner_collector).clone()),
        mock_exporter,
        config,
    );
    
    let agent_metrics = AgentMetrics {
        agent_name: "otel_test_agent".to_string(),
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
        tool_calls_count: 1,
        memory_operations: 0,
        error_count: 0,
        custom_metrics: HashMap::new(),
        context: ExecutionContext {
            session_id: None,
            user_id: None,
            request_id: None,
            environment: "test".to_string(),
            version: None,
        },
    };
    
    // This should not panic and should delegate to inner collector
    otel_collector.record_agent_execution(agent_metrics).await.unwrap();
    
    let summary = otel_collector.get_metrics_summary(Some("otel_test_agent"), None, None).await.unwrap();
    assert_eq!(summary.total_executions, 1);
}

#[tokio::test]
async fn test_metrics_filtering_by_time_range() {
    let collector = InMemoryMetricsCollector::new();
    
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
    let hour_ago = now - (60 * 60 * 1000);
    let two_hours_ago = now - (2 * 60 * 60 * 1000);
    
    // Add metrics from different time periods
    let old_metrics = AgentMetrics {
        agent_name: "test_agent".to_string(),
        execution_id: Uuid::new_v4().to_string(),
        start_time: two_hours_ago,
        end_time: two_hours_ago + 1000,
        execution_time_ms: 1000,
        success: true,
        token_usage: TokenUsage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
        },
        tool_calls_count: 1,
        memory_operations: 0,
        error_count: 0,
        custom_metrics: HashMap::new(),
        context: ExecutionContext {
            session_id: None,
            user_id: None,
            request_id: None,
            environment: "test".to_string(),
            version: None,
        },
    };
    
    let recent_metrics = AgentMetrics {
        agent_name: "test_agent".to_string(),
        execution_id: Uuid::new_v4().to_string(),
        start_time: hour_ago,
        end_time: hour_ago + 1000,
        execution_time_ms: 1000,
        success: true,
        token_usage: TokenUsage {
            prompt_tokens: 200,
            completion_tokens: 100,
            total_tokens: 300,
        },
        tool_calls_count: 2,
        memory_operations: 1,
        error_count: 0,
        custom_metrics: HashMap::new(),
        context: ExecutionContext {
            session_id: None,
            user_id: None,
            request_id: None,
            environment: "test".to_string(),
            version: None,
        },
    };
    
    collector.record_agent_execution(old_metrics).await.unwrap();
    collector.record_agent_execution(recent_metrics).await.unwrap();
    
    // Get all metrics
    let all_summary = collector.get_metrics_summary(Some("test_agent"), None, None).await.unwrap();
    assert_eq!(all_summary.total_executions, 2);
    assert_eq!(all_summary.total_tokens_used, 450);
    
    // Get recent metrics only
    let recent_summary = collector.get_metrics_summary(
        Some("test_agent"), 
        Some(hour_ago - 1000), 
        None
    ).await.unwrap();
    assert_eq!(recent_summary.total_executions, 1);
    assert_eq!(recent_summary.total_tokens_used, 300);
}

#[tokio::test]
async fn test_error_handling() {
    let collector = InMemoryMetricsCollector::new();
    
    // Test getting performance for non-existent agent
    let result = collector.get_agent_performance("non_existent_agent").await;
    assert!(result.is_err());
    
    // Test getting trace stats with no traces
    let stats = collector.get_trace_stats(Some("non_existent_agent"), None, None).await.unwrap();
    assert_eq!(stats.total_traces, 0);
    assert_eq!(stats.successful_traces, 0);
    assert_eq!(stats.failed_traces, 0);
}

#[tokio::test]
async fn test_concurrent_metrics_collection() {
    use tokio::task::JoinSet;
    
    let collector = Arc::new(InMemoryMetricsCollector::new());
    let mut tasks = JoinSet::new();
    
    // Spawn multiple concurrent tasks to record metrics
    for i in 0..10 {
        let collector_clone = collector.clone();
        tasks.spawn(async move {
            let agent_metrics = AgentMetrics {
                agent_name: format!("test_agent_{}", i),
                execution_id: Uuid::new_v4().to_string(),
                start_time: 1000 + i as u64,
                end_time: 2000 + i as u64,
                execution_time_ms: 1000,
                success: i % 2 == 0, // Half successful, half failed
                token_usage: TokenUsage {
                    prompt_tokens: 100 + i as u32,
                    completion_tokens: 50 + i as u32,
                    total_tokens: 150 + (i as u32 * 2),
                },
                tool_calls_count: i,
                memory_operations: 0,
                error_count: if i % 2 == 0 { 0 } else { 1 },
                custom_metrics: HashMap::new(),
                context: ExecutionContext {
                    session_id: None,
                    user_id: None,
                    request_id: None,
                    environment: "test".to_string(),
                    version: None,
                },
            };
            
            collector_clone.record_agent_execution(agent_metrics).await.unwrap();
        });
    }
    
    // Wait for all tasks to complete
    while let Some(_) = tasks.join_next().await {}
    
    // Verify all metrics were recorded
    let summary = collector.get_metrics_summary(None, None, None).await.unwrap();
    assert_eq!(summary.total_executions, 10);
    assert_eq!(summary.successful_executions, 5);
    assert_eq!(summary.failed_executions, 5);
}
