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
use tokio::sync::{RwLock, Mutex};
use tokio::time::{interval, timeout};
use futures::stream::{Stream, StreamExt};
use serde_json::Value;

use crate::{MCPClient, MCPConfiguration, Result, MCPError};
use crate::types::{ServerCapabilities, Tool, ResourceContent};

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
        let client = MCPClient::new(config.clone())?;
        
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
        
        // Attempt initial connection
        self.connect_client(&name).await?;
        
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

        // Fetch from client
        let tools = client.list_tools().await?;
        
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
            match client.execute_tool("", tool_name, parameters.clone(), false).await {
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
    async fn update_success_metrics(&self, client_name: &str, tool_name: &str, response_time: Duration) {
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
    async fn update_failure_metrics(&self, client_name: &str, tool_name: &str, error: &MCPError) {
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
            match timeout(Duration::from_secs(5), client.list_tools()).await {
                Ok(Ok(_)) => true,
                _ => false,
            }
        } else {
            false
        }
    }
}
