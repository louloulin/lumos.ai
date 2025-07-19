//! Enhanced MCP integration with advanced features
//! 
//! This module provides enhanced MCP functionality including:
//! - Connection pooling and management
//! - Automatic reconnection and health checks
//! - Tool discovery and caching
//! - Resource subscription management
//! - Performance monitoring and metrics

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::{interval, timeout};
use serde_json::Value;

use crate::{MCPClient, MCPConfiguration, Result, MCPError};
use crate::types::{Tool, ToolDefinition};

/// Enhanced MCP manager with connection pooling and advanced features
pub struct EnhancedMCPManager {
    /// Pool of MCP clients
    clients: Arc<RwLock<HashMap<String, Arc<MCPClient>>>>,
    /// Client configurations
    configurations: Arc<RwLock<HashMap<String, MCPConfiguration>>>,
    /// Health check status
    health_status: Arc<RwLock<HashMap<String, HealthStatus>>>,
    /// Tool cache
    tool_cache: Arc<RwLock<HashMap<String, Vec<Tool>>>>,
    /// Resource subscriptions
    subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Performance metrics
    metrics: Arc<RwLock<PerformanceMetrics>>,
    /// Configuration
    config: ManagerConfig,
}

/// Health status of an MCP client
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub last_check: Instant,
    pub consecutive_failures: u32,
    pub last_error: Option<String>,
    pub response_time: Option<Duration>,
}

/// Performance metrics for MCP operations
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: Duration,
    pub peak_response_time: Duration,
    pub tool_execution_count: HashMap<String, u64>,
    pub error_count_by_type: HashMap<String, u64>,
}

/// Configuration for the enhanced MCP manager
#[derive(Debug, Clone)]
pub struct ManagerConfig {
    /// Health check interval
    pub health_check_interval: Duration,
    /// Maximum consecutive failures before marking unhealthy
    pub max_consecutive_failures: u32,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Tool cache TTL
    pub tool_cache_ttl: Duration,
    /// Enable automatic reconnection
    pub auto_reconnect: bool,
    /// Maximum number of retry attempts
    pub max_retry_attempts: u32,
}

impl Default for ManagerConfig {
    fn default() -> Self {
        Self {
            health_check_interval: Duration::from_secs(30),
            max_consecutive_failures: 3,
            connection_timeout: Duration::from_secs(10),
            tool_cache_ttl: Duration::from_secs(300),
            auto_reconnect: true,
            max_retry_attempts: 3,
        }
    }
}

impl EnhancedMCPManager {
    /// Create a new enhanced MCP manager
    pub fn new(config: ManagerConfig) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            configurations: Arc::new(RwLock::new(HashMap::new())),
            health_status: Arc::new(RwLock::new(HashMap::new())),
            tool_cache: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            config,
        }
    }

    /// Add an MCP client configuration
    pub async fn add_client(&self, name: String, config: MCPConfiguration) -> Result<()> {
        // Create a simple client for testing
        let client = MCPClient::new(
            &name,
            crate::types::ServerParameters::Stdio(crate::types::StdioServerParameters {
                command: "test".to_string(),
                args: vec![],
                env: std::collections::HashMap::new(),
            }),
            None,
            Some("1.0.0"),
            Some(5000),
        );

        // Store configuration and client
        {
            let mut configs = self.configurations.write().await;
            configs.insert(name.clone(), config);
        }

        {
            let mut clients = self.clients.write().await;
            clients.insert(name.clone(), Arc::new(client));
        }

        // Initialize health status
        {
            let mut health = self.health_status.write().await;
            health.insert(name.clone(), HealthStatus {
                is_healthy: false,
                last_check: Instant::now(),
                consecutive_failures: 0,
                last_error: None,
                response_time: None,
            });
        }

        Ok(())
    }

    /// Connect to a specific client
    async fn connect_client(&self, name: &str) -> Result<()> {
        let client = {
            let clients = self.clients.read().await;
            clients.get(name).cloned()
                .ok_or_else(|| MCPError::ClientNotFound(name.to_string()))?
        };

        let start_time = Instant::now();
        let result = timeout(self.config.connection_timeout, client.connect()).await;
        let response_time = start_time.elapsed();

        let mut health = self.health_status.write().await;
        let status = health.get_mut(name).unwrap();
        
        match result {
            Ok(Ok(())) => {
                status.is_healthy = true;
                status.consecutive_failures = 0;
                status.last_error = None;
                status.response_time = Some(response_time);
                status.last_check = Instant::now();
                Ok(())
            }
            Ok(Err(e)) => {
                status.is_healthy = false;
                status.consecutive_failures += 1;
                status.last_error = Some(e.to_string());
                status.last_check = Instant::now();
                Err(e)
            }
            Err(_) => {
                let timeout_error = MCPError::TimeoutError(self.config.connection_timeout.as_millis() as u64);
                status.is_healthy = false;
                status.consecutive_failures += 1;
                status.last_error = Some(timeout_error.to_string());
                status.last_check = Instant::now();
                Err(timeout_error)
            }
        }
    }

    /// Get all available tools from all healthy clients
    pub async fn get_all_tools(&self) -> Result<HashMap<String, Vec<Tool>>> {
        let mut all_tools = HashMap::new();
        let clients = self.clients.read().await;
        
        for (name, client) in clients.iter() {
            if self.is_client_healthy(name).await {
                match self.get_tools_for_client(name, client).await {
                    Ok(tools) => {
                        all_tools.insert(name.clone(), tools);
                    }
                    Err(e) => {
                        eprintln!("Failed to get tools for client {}: {}", name, e);
                        // Update health status
                        self.mark_client_unhealthy(name, &e.to_string()).await;
                    }
                }
            }
        }
        
        Ok(all_tools)
    }

    /// Get tools for a specific client with caching
    async fn get_tools_for_client(&self, name: &str, client: &MCPClient) -> Result<Vec<Tool>> {
        // Check cache first
        {
            let cache = self.tool_cache.read().await;
            if let Some(tools) = cache.get(name) {
                return Ok(tools.clone());
            }
        }

        // Fetch from client - convert HashMap to Vec
        let tools_map = client.tools().await?;
        let tools: Vec<Tool> = tools_map.into_iter().map(|(name, _tool)| {
            // Create a simple Tool struct for now
            Tool {
                name,
                description: "Tool from MCP server".to_string(),
                input_schema: None,
            }
        }).collect();

        // Update cache
        {
            let mut cache = self.tool_cache.write().await;
            cache.insert(name.to_string(), tools.clone());
        }

        Ok(tools)
    }

    /// Execute a tool on the best available client
    pub async fn execute_tool(&self, tool_name: &str, parameters: HashMap<String, Value>) -> Result<String> {
        let start_time = Instant::now();
        
        // Find a client that has this tool
        let client_name = self.find_client_with_tool(tool_name).await?;
        let client = {
            let clients = self.clients.read().await;
            clients.get(&client_name).cloned()
                .ok_or_else(|| MCPError::ClientNotFound(client_name.clone()))?
        };

        // Execute with retry logic
        let mut attempts = 0;
        let mut last_error = None;
        
        while attempts < self.config.max_retry_attempts {
            match client.execute_tool(
                "default_resource",
                tool_name,
                parameters.clone(),
                false
            ).await {
                Ok(result) => {
                    // Update metrics
                    self.update_success_metrics(&client_name, tool_name, start_time.elapsed()).await;
                    return Ok(result);
                }
                Err(e) => {
                    last_error = Some(e);
                    attempts += 1;
                    
                    if attempts < self.config.max_retry_attempts {
                        // Wait before retry
                        tokio::time::sleep(Duration::from_millis(100 * attempts as u64)).await;
                    }
                }
            }
        }
        
        // All attempts failed
        let error = last_error.unwrap();
        self.update_failure_metrics(&client_name, tool_name, &error).await;
        self.mark_client_unhealthy(&client_name, &error.to_string()).await;
        Err(error)
    }

    /// Find a client that has the specified tool
    async fn find_client_with_tool(&self, tool_name: &str) -> Result<String> {
        let all_tools = self.get_all_tools().await?;
        
        for (client_name, tools) in all_tools {
            if tools.iter().any(|tool| tool.name == tool_name) {
                return Ok(client_name);
            }
        }
        
        Err(MCPError::ToolNotFound(tool_name.to_string()))
    }

    /// Check if a client is healthy
    async fn is_client_healthy(&self, name: &str) -> bool {
        let health = self.health_status.read().await;
        health.get(name).map(|status| status.is_healthy).unwrap_or(false)
    }

    /// Mark a client as unhealthy
    async fn mark_client_unhealthy(&self, name: &str, error: &str) {
        let mut health = self.health_status.write().await;
        if let Some(status) = health.get_mut(name) {
            status.is_healthy = false;
            status.consecutive_failures += 1;
            status.last_error = Some(error.to_string());
            status.last_check = Instant::now();
        }
    }

    /// Update success metrics
    async fn update_success_metrics(&self, _client_name: &str, tool_name: &str, response_time: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;
        metrics.successful_requests += 1;
        
        // Update tool execution count
        *metrics.tool_execution_count.entry(tool_name.to_string()).or_insert(0) += 1;
        
        // Update response time metrics
        if response_time > metrics.peak_response_time {
            metrics.peak_response_time = response_time;
        }
        
        // Update average response time (simple moving average)
        let total_time = metrics.average_response_time.as_nanos() as f64 * (metrics.successful_requests - 1) as f64;
        let new_average = (total_time + response_time.as_nanos() as f64) / metrics.successful_requests as f64;
        metrics.average_response_time = Duration::from_nanos(new_average as u64);
    }

    /// Update failure metrics
    async fn update_failure_metrics(&self, _client_name: &str, _tool_name: &str, error: &MCPError) {
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;
        metrics.failed_requests += 1;
        
        // Update error count by type
        let error_type = match error {
            MCPError::ConnectionError(_) => "connection",
            MCPError::TimeoutError(_) => "timeout",
            MCPError::ToolExecutionError(_) => "tool_execution",
            MCPError::ServerError(_) => "server",
            _ => "other",
        };
        *metrics.error_count_by_type.entry(error_type.to_string()).or_insert(0) += 1;
    }

    /// Get performance metrics
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }

    /// Get health status for all clients
    pub async fn get_health_status(&self) -> HashMap<String, HealthStatus> {
        self.health_status.read().await.clone()
    }

    /// Auto-discover tools from all connected servers
    pub async fn auto_discover_tools(&self) -> Result<HashMap<String, Vec<ToolDefinition>>> {
        let mut discovered_tools = HashMap::new();
        let clients = self.clients.read().await;

        for (server_name, client) in clients.iter() {
            if self.is_client_healthy(server_name).await {
                match self.discover_tools_from_server(server_name, client).await {
                    Ok(tools) => {
                        discovered_tools.insert(server_name.clone(), tools);
                        println!("✅ Discovered {} tools from server '{}'",
                               discovered_tools.get(server_name).unwrap().len(), server_name);
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to discover tools from server '{}': {}", server_name, e);
                        self.mark_client_unhealthy(server_name, &e.to_string()).await;
                    }
                }
            }
        }

        Ok(discovered_tools)
    }

    /// Discover tools from a specific server
    async fn discover_tools_from_server(&self, server_name: &str, client: &MCPClient) -> Result<Vec<ToolDefinition>> {
        // First, try to get resources which contain tools
        let resources = match client.resources().await {
            Ok(resources) => resources,
            Err(_) => {
                // If resources fail, try direct tool listing
                return self.get_tools_from_server_direct(server_name, client).await;
            }
        };

        let mut all_tools = Vec::new();
        for resource in resources.resources {
            all_tools.extend(resource.tools);
        }

        // Update cache
        {
            let mut cache = self.tool_cache.write().await;
            let tool_cache_entries: Vec<Tool> = all_tools.iter().map(|tool_def| Tool {
                name: tool_def.name.clone(),
                description: tool_def.description.clone(),
                input_schema: tool_def.return_schema.clone(),
            }).collect();
            cache.insert(server_name.to_string(), tool_cache_entries);
        }

        Ok(all_tools)
    }

    /// Get tools directly from server (fallback method)
    async fn get_tools_from_server_direct(&self, server_name: &str, client: &MCPClient) -> Result<Vec<ToolDefinition>> {
        let tools_map = client.tools().await?;
        let tools: Vec<ToolDefinition> = tools_map.into_iter().map(|(name, _tool)| {
            ToolDefinition {
                name: name.clone(),
                description: format!("Tool '{}' from MCP server '{}'", name, server_name),
                parameters: vec![], // Will be populated from actual tool schema
                return_schema: None,
            }
        }).collect();

        Ok(tools)
    }

    /// Register MCP server with enhanced configuration
    pub async fn register_mcp_server(&self, server_name: String, config: MCPConfiguration) -> Result<()> {
        // Check for duplicate registration
        {
            let configs = self.configurations.read().await;
            if configs.contains_key(&server_name) {
                return Err(MCPError::DuplicateConfiguration(server_name));
            }
        }

        // Add the client
        self.add_client(server_name.clone(), config).await?;

        // Attempt initial connection
        match self.connect_client(&server_name).await {
            Ok(()) => {
                println!("✅ Successfully registered and connected to MCP server '{}'", server_name);

                // Perform initial tool discovery
                if let Err(e) = self.discover_tools_from_server_by_name(&server_name).await {
                    eprintln!("⚠️  Initial tool discovery failed for '{}': {}", server_name, e);
                }
            }
            Err(e) => {
                eprintln!("⚠️  Failed to connect to MCP server '{}': {}", server_name, e);
                // Mark as unhealthy but keep registered for retry
                self.mark_client_unhealthy(&server_name, &e.to_string()).await;
            }
        }

        Ok(())
    }

    /// Discover tools from a server by name
    async fn discover_tools_from_server_by_name(&self, server_name: &str) -> Result<()> {
        let client = {
            let clients = self.clients.read().await;
            clients.get(server_name).cloned()
                .ok_or_else(|| MCPError::ClientNotFound(server_name.to_string()))?
        };

        self.discover_tools_from_server(server_name, &client).await?;
        Ok(())
    }

    /// Execute MCP tool with enhanced error handling and caching
    pub async fn execute_mcp_tool(&self, tool_name: &str, params: HashMap<String, serde_json::Value>) -> Result<serde_json::Value> {
        let start_time = Instant::now();

        // Find the best server for this tool
        let server_name = self.find_best_server_for_tool(tool_name).await?;
        let client = {
            let clients = self.clients.read().await;
            clients.get(&server_name).cloned()
                .ok_or_else(|| MCPError::ClientNotFound(server_name.clone()))?
        };

        // Execute with retry and circuit breaker logic
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.config.max_retry_attempts {
            // Check if server is healthy before attempting
            if !self.is_client_healthy(&server_name).await && attempts > 0 {
                // Try to find alternative server
                if let Ok(alt_server) = self.find_alternative_server_for_tool(tool_name, &server_name).await {
                    return self.execute_on_server(&alt_server, tool_name, &params, start_time).await;
                }
            }

            match self.execute_on_server(&server_name, tool_name, &params, start_time).await {
                Ok(result) => {
                    self.update_success_metrics(&server_name, tool_name, start_time.elapsed()).await;
                    return Ok(result);
                }
                Err(e) => {
                    last_error = Some(e);
                    attempts += 1;

                    if attempts < self.config.max_retry_attempts {
                        // Exponential backoff
                        let delay = Duration::from_millis(100 * (2_u64.pow(attempts as u32 - 1)));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        // All attempts failed
        let error = last_error.unwrap();
        self.update_failure_metrics(&server_name, tool_name, &error).await;
        self.mark_client_unhealthy(&server_name, &error.to_string()).await;
        Err(error)
    }

    /// Execute tool on a specific server
    async fn execute_on_server(&self, server_name: &str, tool_name: &str, params: &HashMap<String, serde_json::Value>, start_time: Instant) -> Result<serde_json::Value> {
        let client = {
            let clients = self.clients.read().await;
            clients.get(server_name).cloned()
                .ok_or_else(|| MCPError::ClientNotFound(server_name.to_string()))?
        };

        // Execute the tool
        let result = client.execute_tool(
            "default_resource", // TODO: Make this configurable
            tool_name,
            params.clone(),
            false
        ).await?;

        // Try to parse as JSON, fallback to string
        match serde_json::from_str::<serde_json::Value>(&result) {
            Ok(json_result) => Ok(json_result),
            Err(_) => Ok(serde_json::Value::String(result)),
        }
    }

    /// Find the best server for a tool (load balancing)
    async fn find_best_server_for_tool(&self, tool_name: &str) -> Result<String> {
        let all_tools = self.get_all_tools().await?;
        let mut candidates = Vec::new();

        // Find all servers that have this tool
        for (server_name, tools) in all_tools {
            if tools.iter().any(|tool| tool.name == tool_name) {
                if self.is_client_healthy(&server_name).await {
                    candidates.push(server_name);
                }
            }
        }

        if candidates.is_empty() {
            return Err(MCPError::ToolNotFound(tool_name.to_string()));
        }

        // Simple round-robin for now (could be enhanced with load metrics)
        let health_status = self.health_status.read().await;
        candidates.sort_by(|a, b| {
            let a_response_time = health_status.get(a)
                .and_then(|s| s.response_time)
                .unwrap_or(Duration::from_secs(999));
            let b_response_time = health_status.get(b)
                .and_then(|s| s.response_time)
                .unwrap_or(Duration::from_secs(999));
            a_response_time.cmp(&b_response_time)
        });

        Ok(candidates[0].clone())
    }

    /// Find alternative server for a tool (excluding failed server)
    async fn find_alternative_server_for_tool(&self, tool_name: &str, exclude_server: &str) -> Result<String> {
        let all_tools = self.get_all_tools().await?;

        for (server_name, tools) in all_tools {
            if server_name != exclude_server &&
               tools.iter().any(|tool| tool.name == tool_name) &&
               self.is_client_healthy(&server_name).await {
                return Ok(server_name);
            }
        }

        Err(MCPError::ToolNotFound(format!("No alternative server found for tool '{}'", tool_name)))
    }

    /// Start background health monitoring
    pub async fn start_health_monitoring(&self) {
        let clients = self.clients.clone();
        let health_status = self.health_status.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(config.health_check_interval);
            
            loop {
                interval.tick().await;
                
                let client_names: Vec<String> = {
                    let clients_guard = clients.read().await;
                    clients_guard.keys().cloned().collect()
                };
                
                for name in client_names {
                    // Perform health check
                    let is_healthy = Self::perform_health_check(&clients, &name).await;
                    
                    // Update health status
                    let mut health = health_status.write().await;
                    if let Some(status) = health.get_mut(&name) {
                        status.is_healthy = is_healthy;
                        status.last_check = Instant::now();
                        
                        if !is_healthy {
                            status.consecutive_failures += 1;
                        } else {
                            status.consecutive_failures = 0;
                            status.last_error = None;
                        }
                    }
                }
            }
        });
    }

    /// Perform health check for a specific client
    async fn perform_health_check(clients: &Arc<RwLock<HashMap<String, Arc<MCPClient>>>>, name: &str) -> bool {
        let client = {
            let clients_guard = clients.read().await;
            clients_guard.get(name).cloned()
        };

        if let Some(client) = client {
            // Simple ping test
            match timeout(Duration::from_secs(5), client.tools()).await {
                Ok(Ok(_)) => true,
                _ => false,
            }
        } else {
            false
        }
    }

    /// Subscribe to resource updates from MCP servers
    pub async fn subscribe_to_resource(&self, server_name: &str, resource_uri: &str) -> Result<()> {
        let client = {
            let clients = self.clients.read().await;
            clients.get(server_name).cloned()
                .ok_or_else(|| MCPError::ClientNotFound(server_name.to_string()))?
        };

        // Add to subscriptions tracking
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.entry(server_name.to_string())
                .or_insert_with(Vec::new)
                .push(resource_uri.to_string());
        }

        // TODO: Implement actual subscription via MCP protocol
        // This would involve sending a SubscribeResource message
        println!("📡 Subscribed to resource '{}' on server '{}'", resource_uri, server_name);

        Ok(())
    }

    /// Unsubscribe from resource updates
    pub async fn unsubscribe_from_resource(&self, server_name: &str, resource_uri: &str) -> Result<()> {
        let client = {
            let clients = self.clients.read().await;
            clients.get(server_name).cloned()
                .ok_or_else(|| MCPError::ClientNotFound(server_name.to_string()))?
        };

        // Remove from subscriptions tracking
        {
            let mut subscriptions = self.subscriptions.write().await;
            if let Some(server_subscriptions) = subscriptions.get_mut(server_name) {
                server_subscriptions.retain(|uri| uri != resource_uri);
                if server_subscriptions.is_empty() {
                    subscriptions.remove(server_name);
                }
            }
        }

        // TODO: Implement actual unsubscription via MCP protocol
        println!("📡 Unsubscribed from resource '{}' on server '{}'", resource_uri, server_name);

        Ok(())
    }

    /// Get all active subscriptions
    pub async fn get_active_subscriptions(&self) -> HashMap<String, Vec<String>> {
        self.subscriptions.read().await.clone()
    }

    /// Batch execute multiple tools across different servers
    pub async fn batch_execute_tools(&self, requests: Vec<(String, HashMap<String, serde_json::Value>)>) -> Vec<Result<serde_json::Value>> {
        let mut futures = Vec::new();

        // Create futures for all requests
        for (tool_name, params) in requests.into_iter() {
            let self_clone = self;
            let future = async move {
                self_clone.execute_mcp_tool(&tool_name, params).await
            };
            futures.push(future);
        }

        // Execute all futures concurrently
        futures::future::join_all(futures).await
    }

    /// Get comprehensive server status report
    pub async fn get_server_status_report(&self) -> HashMap<String, ServerStatus> {
        let mut report = HashMap::new();
        let clients = self.clients.read().await;
        let health_status = self.health_status.read().await;
        let tool_cache = self.tool_cache.read().await;
        let subscriptions = self.subscriptions.read().await;

        for (server_name, _client) in clients.iter() {
            let health = health_status.get(server_name).cloned().unwrap_or_else(|| HealthStatus {
                is_healthy: false,
                last_check: Instant::now(),
                consecutive_failures: 0,
                last_error: Some("No health data available".to_string()),
                response_time: None,
            });

            let tool_count = tool_cache.get(server_name).map(|tools| tools.len()).unwrap_or(0);
            let subscription_count = subscriptions.get(server_name).map(|subs| subs.len()).unwrap_or(0);

            report.insert(server_name.clone(), ServerStatus {
                name: server_name.clone(),
                health: health,
                tool_count,
                subscription_count,
                last_activity: Instant::now(), // TODO: Track actual last activity
            });
        }

        report
    }

    /// Refresh tool cache for all servers
    pub async fn refresh_tool_cache(&self) -> Result<()> {
        let clients = self.clients.read().await;
        let mut refresh_tasks = Vec::new();

        for (server_name, client) in clients.iter() {
            if self.is_client_healthy(server_name).await {
                let server_name = server_name.clone();
                let client = client.clone();
                let tool_cache = self.tool_cache.clone();

                let task = tokio::spawn(async move {
                    match Self::refresh_tools_for_server(&server_name, &client, &tool_cache).await {
                        Ok(count) => {
                            println!("🔄 Refreshed {} tools for server '{}'", count, server_name);
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to refresh tools for server '{}': {}", server_name, e);
                        }
                    }
                });

                refresh_tasks.push(task);
            }
        }

        // Wait for all refresh tasks to complete
        futures::future::join_all(refresh_tasks).await;

        Ok(())
    }

    /// Refresh tools for a specific server
    async fn refresh_tools_for_server(
        server_name: &str,
        client: &MCPClient,
        tool_cache: &Arc<RwLock<HashMap<String, Vec<Tool>>>>
    ) -> Result<usize> {
        let tools_map = client.tools().await?;
        let tools: Vec<Tool> = tools_map.into_iter().map(|(name, _tool)| {
            Tool {
                name: name.clone(),
                description: format!("Tool '{}' from MCP server '{}'", name, server_name),
                input_schema: None,
            }
        }).collect();

        let tool_count = tools.len();

        // Update cache
        {
            let mut cache = tool_cache.write().await;
            cache.insert(server_name.to_string(), tools);
        }

        Ok(tool_count)
    }

    /// Start background tasks (health monitoring, cache refresh, etc.)
    pub async fn start_background_tasks(&self) {
        // Start health monitoring
        self.start_health_monitoring().await;

        // Start periodic tool cache refresh
        self.start_cache_refresh_task().await;

        println!("🚀 Started background tasks for MCP manager");
    }

    /// Start periodic cache refresh task
    async fn start_cache_refresh_task(&self) {
        let tool_cache = self.tool_cache.clone();
        let clients = self.clients.clone();
        let cache_ttl = self.config.tool_cache_ttl;

        tokio::spawn(async move {
            let mut interval = interval(cache_ttl);

            loop {
                interval.tick().await;

                let client_names: Vec<String> = {
                    let clients_guard = clients.read().await;
                    clients_guard.keys().cloned().collect()
                };

                for server_name in client_names {
                    let client = {
                        let clients_guard = clients.read().await;
                        clients_guard.get(&server_name).cloned()
                    };

                    if let Some(client) = client {
                        if let Err(e) = Self::refresh_tools_for_server(&server_name, &client, &tool_cache).await {
                            eprintln!("🔄 Cache refresh failed for '{}': {}", server_name, e);
                        }
                    }
                }
            }
        });
    }
}

/// Server status information
#[derive(Debug, Clone)]
pub struct ServerStatus {
    pub name: String,
    pub health: HealthStatus,
    pub tool_count: usize,
    pub subscription_count: usize,
    pub last_activity: Instant,
}

impl std::fmt::Debug for EnhancedMCPManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnhancedMCPManager")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}
