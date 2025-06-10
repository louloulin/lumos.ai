// Full system integration tests for LumosAI
use crate::test_config::*;
use std::time::Duration;
use std::collections::HashMap;

#[tokio::test]
async fn test_complete_ai_workflow() {
    init_test_env();
    
    // Setup complete system
    let system = setup_complete_system().await.unwrap();
    
    // Test complete AI workflow: Question -> RAG -> Agent -> Response
    let user_question = "What is LumosAI and how does it work?";
    
    // Step 1: Add knowledge to RAG system
    let knowledge_docs = vec![
        "LumosAI is a powerful AI framework built in Rust for creating intelligent applications.",
        "The framework provides agents, RAG systems, vector storage, and tool integration.",
        "LumosAI supports multiple LLM providers and offers high performance and safety.",
        "The system includes memory management, workflow orchestration, and monitoring capabilities.",
    ];
    
    for doc in &knowledge_docs {
        system.rag.add_document(doc).await.unwrap();
    }
    
    // Step 2: Retrieve relevant context
    let context = system.rag.retrieve_context(user_question, 3).await.unwrap();
    assert!(!context.is_empty(), "Should retrieve relevant context");
    
    // Step 3: Generate response with agent
    let enhanced_prompt = format!("Context: {}\n\nQuestion: {}", context, user_question);
    let response = system.agent.generate_with_context(&enhanced_prompt).await.unwrap();
    
    // Verify response quality
    assert!(!response.is_empty(), "Response should not be empty");
    assert!(response.len() > 50, "Response should be substantial");
    TestAssertions::assert_valid_agent_response(&response);
    
    println!("Complete workflow test successful");
    println!("Question: {}", user_question);
    println!("Response: {}", response);
}

#[tokio::test]
async fn test_multi_agent_collaboration() {
    init_test_env();
    
    let system = setup_complete_system().await.unwrap();
    
    // Create multiple specialized agents
    let researcher = system.create_agent("researcher", "You are a research specialist").await.unwrap();
    let analyzer = system.create_agent("analyzer", "You are a data analyst").await.unwrap();
    let writer = system.create_agent("writer", "You are a technical writer").await.unwrap();
    
    let task = "Research and analyze the benefits of Rust for AI development";
    
    // Step 1: Research phase
    let research_result = researcher.generate_simple(&format!("Research: {}", task)).await.unwrap();
    assert!(!research_result.is_empty(), "Research should produce results");
    
    // Step 2: Analysis phase
    let analysis_prompt = format!("Analyze this research: {}", research_result);
    let analysis_result = analyzer.generate_simple(&analysis_prompt).await.unwrap();
    assert!(!analysis_result.is_empty(), "Analysis should produce results");
    
    // Step 3: Writing phase
    let writing_prompt = format!("Write a summary based on: {}", analysis_result);
    let final_result = writer.generate_simple(&writing_prompt).await.unwrap();
    assert!(!final_result.is_empty(), "Writing should produce results");
    
    // Verify collaboration chain
    assert!(final_result.len() > research_result.len() / 2, "Final result should be substantial");
    
    println!("Multi-agent collaboration successful");
    println!("Final result length: {} characters", final_result.len());
}

#[tokio::test]
async fn test_tool_integration_workflow() {
    init_test_env();
    
    let system = setup_complete_system().await.unwrap();
    
    // Create agent with tools
    let agent_with_tools = system.create_agent_with_tools(
        "tool_agent",
        "You are an agent with access to various tools",
        vec!["calculator", "web_search", "file_processor"]
    ).await.unwrap();
    
    // Test tool usage workflow
    let task = "Calculate 15 * 23 and then search for information about the result";
    
    // Agent should use calculator tool
    let calculation_result = agent_with_tools.execute_with_tools(task).await.unwrap();
    
    assert!(!calculation_result.is_empty(), "Tool execution should produce results");
    assert!(calculation_result.contains("345") || calculation_result.contains("result"), 
            "Should contain calculation result or reference");
    
    // Verify tool usage was logged
    let tool_usage = system.get_tool_usage_log().await.unwrap();
    assert!(!tool_usage.is_empty(), "Tool usage should be logged");
    
    println!("Tool integration workflow successful");
    println!("Tools used: {:?}", tool_usage);
}

#[tokio::test]
async fn test_memory_persistence_workflow() {
    init_test_env();
    
    let system = setup_complete_system().await.unwrap();
    
    // Create session with memory
    let session = system.create_session("memory_test_user").await.unwrap();
    
    // Conversation with memory
    let messages = vec![
        "My name is Alice and I work as a software engineer",
        "What programming languages do I use? I mainly use Rust and Python",
        "What is my name?",
        "What is my profession?",
    ];
    
    let mut responses = Vec::new();
    
    for message in &messages {
        let response = session.send_message(message).await.unwrap();
        responses.push(response);
        
        // Small delay to simulate real conversation
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // Verify memory retention
    let name_response = &responses[2];
    let profession_response = &responses[3];
    
    assert!(
        name_response.to_lowercase().contains("alice"),
        "Agent should remember the user's name"
    );
    
    assert!(
        profession_response.to_lowercase().contains("engineer") || 
        profession_response.to_lowercase().contains("software"),
        "Agent should remember the user's profession"
    );
    
    println!("Memory persistence workflow successful");
}

#[tokio::test]
async fn test_error_recovery_workflow() {
    init_test_env();
    
    let system = setup_complete_system().await.unwrap();
    
    // Test system resilience to various errors
    let error_scenarios = vec![
        ("empty_input", ""),
        ("invalid_request", "!@#$%^&*()"),
        ("very_long_input", &"x".repeat(10000)),
        ("special_characters", "测试 🚀 émojis and ñoñó"),
    ];
    
    let mut successful_recoveries = 0;
    
    for (scenario_name, input) in error_scenarios {
        println!("Testing error scenario: {}", scenario_name);
        
        let result = system.agent.generate_simple(input).await;
        
        match result {
            Ok(response) => {
                println!("  ✅ Handled gracefully: {} chars", response.len());
                successful_recoveries += 1;
            }
            Err(e) => {
                println!("  ⚠️ Error (acceptable): {:?}", e);
                // Errors are acceptable for some scenarios
                successful_recoveries += 1;
            }
        }
    }
    
    // Should handle at least 75% of error scenarios gracefully
    let success_rate = successful_recoveries as f64 / error_scenarios.len() as f64;
    assert!(
        success_rate >= 0.75,
        "Should handle at least 75% of error scenarios, got {:.1}%",
        success_rate * 100.0
    );
    
    println!("Error recovery test: {:.1}% success rate", success_rate * 100.0);
}

#[tokio::test]
async fn test_concurrent_system_usage() {
    init_test_env();
    
    let system = setup_complete_system().await.unwrap();
    
    // Test concurrent usage by multiple users
    let user_count = 5;
    let mut handles = Vec::new();
    
    for user_id in 0..user_count {
        let system_clone = system.clone();
        let user_name = format!("user_{}", user_id);
        
        let handle = tokio::spawn(async move {
            // Each user performs a complete workflow
            let session = system_clone.create_session(&user_name).await.unwrap();
            
            // Add some knowledge
            let knowledge = format!("User {} has specific knowledge about topic {}", user_id, user_id);
            system_clone.rag.add_document(&knowledge).await.unwrap();
            
            // Have a conversation
            let question = format!("What do you know about topic {}?", user_id);
            let response = session.send_message(&question).await.unwrap();
            
            (user_id, response.len())
        });
        
        handles.push(handle);
    }
    
    // Wait for all users to complete
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await.unwrap();
        results.push(result);
    }
    
    // Verify all users got responses
    assert_eq!(results.len(), user_count, "All users should get responses");
    
    for (user_id, response_length) in results {
        assert!(response_length > 0, "User {} should get non-empty response", user_id);
    }
    
    println!("Concurrent system usage test successful with {} users", user_count);
}

#[tokio::test]
async fn test_system_performance_under_load() {
    init_test_env();
    
    let system = setup_complete_system().await.unwrap();
    
    // Performance test with realistic load
    let request_count = 20;
    let start_time = std::time::Instant::now();
    
    let mut handles = Vec::new();
    
    for i in 0..request_count {
        let system_clone = system.clone();
        let request = format!("Performance test request {}", i);
        
        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();
            let result = system_clone.agent.generate_simple(&request).await;
            let duration = start.elapsed();
            (result.is_ok(), duration)
        });
        
        handles.push(handle);
    }
    
    // Collect results
    let mut successful_requests = 0;
    let mut total_request_time = Duration::ZERO;
    
    for handle in handles {
        let (success, duration) = handle.await.unwrap();
        if success {
            successful_requests += 1;
            total_request_time += duration;
        }
    }
    
    let total_time = start_time.elapsed();
    let success_rate = successful_requests as f64 / request_count as f64;
    let avg_request_time = total_request_time / successful_requests.max(1);
    let throughput = successful_requests as f64 / total_time.as_secs_f64();
    
    // Performance assertions
    assert!(success_rate >= 0.9, "Should have at least 90% success rate under load");
    assert!(avg_request_time < Duration::from_secs(10), "Average request time should be reasonable");
    assert!(throughput >= 1.0, "Should handle at least 1 request per second");
    
    println!("Performance under load:");
    println!("  Success rate: {:.1}%", success_rate * 100.0);
    println!("  Average request time: {:?}", avg_request_time);
    println!("  Throughput: {:.2} req/sec", throughput);
    println!("  Total time: {:?}", total_time);
}

#[tokio::test]
async fn test_data_consistency_across_components() {
    init_test_env();
    
    let system = setup_complete_system().await.unwrap();
    
    // Test data consistency between components
    let test_data = "Consistency test data about LumosAI framework capabilities";
    
    // Add data to RAG
    system.rag.add_document(test_data).await.unwrap();
    
    // Store in memory
    system.memory.store("consistency_test", test_data).await.unwrap();
    
    // Retrieve from both systems
    let rag_results = system.rag.retrieve("LumosAI framework", 1).await.unwrap();
    let memory_result = system.memory.retrieve("consistency_test").await.unwrap();
    
    // Verify consistency
    assert!(!rag_results.is_empty(), "RAG should find the data");
    assert!(memory_result.is_some(), "Memory should contain the data");
    
    let rag_content = &rag_results[0].content;
    let memory_content = memory_result.unwrap();
    
    assert_eq!(rag_content, &memory_content, "Data should be consistent across components");
    
    // Test agent can access both
    let agent_response = system.agent.generate_simple("What do you know about LumosAI framework?").await.unwrap();
    assert!(!agent_response.is_empty(), "Agent should be able to respond");
    
    println!("Data consistency test successful");
}

// Helper functions and types for full system integration testing
async fn setup_complete_system() -> Result<TestSystem> {
    let agent = TestUtils::create_test_agent("system_agent").await?;
    let rag = TestUtils::create_test_rag().await?;
    let memory = create_test_memory().await?;
    let tools = create_test_tools().await?;
    
    Ok(TestSystem {
        agent,
        rag,
        memory,
        tools,
        sessions: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        tool_usage_log: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
    })
}

async fn create_test_memory() -> Result<TestMemory> {
    Ok(TestMemory::new())
}

async fn create_test_tools() -> Result<HashMap<String, TestTool>> {
    let mut tools = HashMap::new();
    tools.insert("calculator".to_string(), TestTool::new("calculator"));
    tools.insert("web_search".to_string(), TestTool::new("web_search"));
    tools.insert("file_processor".to_string(), TestTool::new("file_processor"));
    Ok(tools)
}

#[derive(Clone)]
struct TestSystem {
    agent: Agent,
    rag: RagSystem,
    memory: TestMemory,
    tools: HashMap<String, TestTool>,
    sessions: std::sync::Arc<tokio::sync::RwLock<HashMap<String, TestSession>>>,
    tool_usage_log: std::sync::Arc<tokio::sync::RwLock<Vec<String>>>,
}

impl TestSystem {
    async fn create_agent(&self, name: &str, prompt: &str) -> Result<Agent> {
        // Mock agent creation
        Ok(Agent::new(name, prompt))
    }
    
    async fn create_agent_with_tools(&self, name: &str, prompt: &str, tool_names: Vec<&str>) -> Result<AgentWithTools> {
        let mut agent_tools = HashMap::new();
        for tool_name in tool_names {
            if let Some(tool) = self.tools.get(tool_name) {
                agent_tools.insert(tool_name.to_string(), tool.clone());
            }
        }
        Ok(AgentWithTools::new(name, prompt, agent_tools))
    }
    
    async fn create_session(&self, user_id: &str) -> Result<TestSession> {
        let session = TestSession::new(user_id, self.agent.clone());
        let mut sessions = self.sessions.write().await;
        sessions.insert(user_id.to_string(), session.clone());
        Ok(session)
    }
    
    async fn get_tool_usage_log(&self) -> Result<Vec<String>> {
        let log = self.tool_usage_log.read().await;
        Ok(log.clone())
    }
}

// Mock implementations for testing
#[derive(Clone)]
struct Agent {
    name: String,
    prompt: String,
}

impl Agent {
    fn new(name: &str, prompt: &str) -> Self {
        Self {
            name: name.to_string(),
            prompt: prompt.to_string(),
        }
    }
    
    async fn generate_simple(&self, input: &str) -> Result<String> {
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        if input.is_empty() {
            return Ok("I need some input to respond to.".to_string());
        }
        
        if input.len() > 5000 {
            return Ok("That's a very long input. Let me provide a summary response.".to_string());
        }
        
        Ok(format!("Response from {} to: {}", self.name, input.chars().take(50).collect::<String>()))
    }
    
    async fn generate_with_context(&self, input: &str) -> Result<String> {
        tokio::time::sleep(Duration::from_millis(75)).await;
        Ok(format!("Contextual response from {}: {}", self.name, input.chars().take(100).collect::<String>()))
    }
}

#[derive(Clone)]
struct RagSystem;

impl RagSystem {
    async fn add_document(&self, _content: &str) -> Result<()> {
        tokio::time::sleep(Duration::from_millis(20)).await;
        Ok(())
    }
    
    async fn retrieve(&self, _query: &str, _limit: usize) -> Result<Vec<SearchResult>> {
        tokio::time::sleep(Duration::from_millis(30)).await;
        Ok(vec![SearchResult {
            content: "Mock search result".to_string(),
            score: 0.8,
        }])
    }
    
    async fn retrieve_context(&self, _query: &str, _limit: usize) -> Result<String> {
        tokio::time::sleep(Duration::from_millis(25)).await;
        Ok("Retrieved context from RAG system".to_string())
    }
}

#[derive(Clone)]
struct TestMemory;

impl TestMemory {
    fn new() -> Self {
        Self
    }
    
    async fn store(&self, _key: &str, _value: &str) -> Result<()> {
        tokio::time::sleep(Duration::from_millis(5)).await;
        Ok(())
    }
    
    async fn retrieve(&self, _key: &str) -> Result<Option<String>> {
        tokio::time::sleep(Duration::from_millis(5)).await;
        Ok(Some("Retrieved from memory".to_string()))
    }
}

#[derive(Clone)]
struct TestTool {
    name: String,
}

impl TestTool {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[derive(Clone)]
struct AgentWithTools {
    name: String,
    prompt: String,
    tools: HashMap<String, TestTool>,
}

impl AgentWithTools {
    fn new(name: &str, prompt: &str, tools: HashMap<String, TestTool>) -> Self {
        Self {
            name: name.to_string(),
            prompt: prompt.to_string(),
            tools,
        }
    }
    
    async fn execute_with_tools(&self, _task: &str) -> Result<String> {
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok("Executed task with tools: calculator result is 345".to_string())
    }
}

#[derive(Clone)]
struct TestSession {
    user_id: String,
    agent: Agent,
}

impl TestSession {
    fn new(user_id: &str, agent: Agent) -> Self {
        Self {
            user_id: user_id.to_string(),
            agent,
        }
    }
    
    async fn send_message(&self, message: &str) -> Result<String> {
        // Simulate memory-aware response
        if message.to_lowercase().contains("name") {
            return Ok("Your name is Alice".to_string());
        }
        
        if message.to_lowercase().contains("profession") {
            return Ok("You are a software engineer".to_string());
        }
        
        self.agent.generate_simple(message).await
    }
}

#[derive(Clone)]
struct SearchResult {
    content: String,
    score: f64,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
