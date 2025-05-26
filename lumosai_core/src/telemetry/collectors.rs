use crate::telemetry::metrics::*;
use crate::telemetry::trace::*;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use serde_json;

/// 内存中的指标收集器实现
#[derive(Debug, Clone)]
pub struct InMemoryMetricsCollector {
    /// 代理执行指标存储
    agent_metrics: Arc<RwLock<Vec<AgentMetrics>>>,
    /// 工具执行指标存储
    tool_metrics: Arc<RwLock<Vec<ToolMetrics>>>,
    /// 内存操作指标存储
    memory_metrics: Arc<RwLock<Vec<MemoryMetrics>>>,
    /// 执行追踪存储
    traces: Arc<RwLock<HashMap<String, ExecutionTrace>>>,
}

impl InMemoryMetricsCollector {
    /// 创建新的内存指标收集器
    pub fn new() -> Self {
        Self {
            agent_metrics: Arc::new(RwLock::new(Vec::new())),
            tool_metrics: Arc::new(RwLock::new(Vec::new())),
            memory_metrics: Arc::new(RwLock::new(Vec::new())),
            traces: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 清空所有指标数据
    pub async fn clear_all(&self) {
        self.agent_metrics.write().await.clear();
        self.tool_metrics.write().await.clear();
        self.memory_metrics.write().await.clear();
        self.traces.write().await.clear();
    }
    
    /// 获取所有代理指标
    pub async fn get_all_agent_metrics(&self) -> Vec<AgentMetrics> {
        self.agent_metrics.read().await.clone()
    }
    
    /// 获取所有工具指标
    pub async fn get_all_tool_metrics(&self) -> Vec<ToolMetrics> {
        self.tool_metrics.read().await.clone()
    }
    
    /// 获取所有内存指标
    pub async fn get_all_memory_metrics(&self) -> Vec<MemoryMetrics> {
        self.memory_metrics.read().await.clone()
    }
}

#[async_trait]
impl MetricsCollector for InMemoryMetricsCollector {
    async fn record_agent_execution(&self, metrics: AgentMetrics) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.agent_metrics.write().await.push(metrics);
        Ok(())
    }
    
    async fn record_tool_execution(&self, metrics: ToolMetrics) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.tool_metrics.write().await.push(metrics);
        Ok(())
    }
    
    async fn record_memory_operation(&self, metrics: MemoryMetrics) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.memory_metrics.write().await.push(metrics);
        Ok(())
    }
    
    async fn get_metrics_summary(&self, agent_name: Option<&str>, from_time: Option<u64>, to_time: Option<u64>) -> Result<MetricsSummary, Box<dyn std::error::Error + Send + Sync>> {
        let metrics = self.agent_metrics.read().await;
        
        // 过滤指标
        let filtered_metrics: Vec<&AgentMetrics> = metrics.iter()
            .filter(|m| {
                if let Some(name) = agent_name {
                    if m.agent_name != name {
                        return false;
                    }
                }
                if let Some(from) = from_time {
                    if m.start_time < from {
                        return false;
                    }
                }
                if let Some(to) = to_time {
                    if m.end_time > to {
                        return false;
                    }
                }
                true
            })
            .collect();
        
        if filtered_metrics.is_empty() {
            return Ok(MetricsSummary {
                total_executions: 0,
                successful_executions: 0,
                failed_executions: 0,
                avg_execution_time_ms: 0.0,
                min_execution_time_ms: 0,
                max_execution_time_ms: 0,
                total_tokens_used: 0,
                avg_tokens_per_execution: 0.0,
                tool_call_stats: HashMap::new(),
                time_range: TimeRange {
                    start: from_time.unwrap_or(0),
                    end: to_time.unwrap_or(0),
                },
            });
        }
        
        let total_executions = filtered_metrics.len() as u64;
        let successful_executions = filtered_metrics.iter().filter(|m| m.success).count() as u64;
        let failed_executions = total_executions - successful_executions;
        
        let execution_times: Vec<u64> = filtered_metrics.iter().map(|m| m.execution_time_ms).collect();
        let avg_execution_time_ms = execution_times.iter().sum::<u64>() as f64 / execution_times.len() as f64;
        let min_execution_time_ms = *execution_times.iter().min().unwrap_or(&0);
        let max_execution_time_ms = *execution_times.iter().max().unwrap_or(&0);
        
        let total_tokens_used: u64 = filtered_metrics.iter().map(|m| m.token_usage.total_tokens as u64).sum();
        let avg_tokens_per_execution = total_tokens_used as f64 / total_executions as f64;
        
        // 统计工具调用
        let mut tool_call_stats = HashMap::new();
        for metric in &filtered_metrics {
            *tool_call_stats.entry("total_calls".to_string()).or_insert(0) += metric.tool_calls_count as u64;
        }
        
        Ok(MetricsSummary {
            total_executions,
            successful_executions,
            failed_executions,
            avg_execution_time_ms,
            min_execution_time_ms,
            max_execution_time_ms,
            total_tokens_used,
            avg_tokens_per_execution,
            tool_call_stats,
            time_range: TimeRange {
                start: from_time.unwrap_or(0),
                end: to_time.unwrap_or(u64::MAX),
            },
        })
    }
    
    async fn get_agent_performance(&self, agent_name: &str) -> Result<AgentPerformance, Box<dyn std::error::Error + Send + Sync>> {
        let metrics = self.agent_metrics.read().await;
        let tool_metrics = self.tool_metrics.read().await;
        
        // 过滤指定代理的指标
        let agent_metrics: Vec<&AgentMetrics> = metrics.iter()
            .filter(|m| m.agent_name == agent_name)
            .collect();
        
        if agent_metrics.is_empty() {
            return Err("No metrics found for the specified agent".into());
        }
        
        // 计算最近24小时的指标
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let twenty_four_hours_ago = now - (24 * 60 * 60 * 1000);
        
        let recent_metrics: Vec<&AgentMetrics> = agent_metrics.iter()
            .filter(|m| m.start_time >= twenty_four_hours_ago)
            .copied()
            .collect();
        
        let executions_last_24h = recent_metrics.len() as u64;
        let successful_last_24h = recent_metrics.iter().filter(|m| m.success).count();
        let success_rate_24h = if executions_last_24h > 0 {
            successful_last_24h as f64 / executions_last_24h as f64
        } else {
            0.0
        };
        
        let avg_response_time_24h = if !recent_metrics.is_empty() {
            recent_metrics.iter().map(|m| m.execution_time_ms).sum::<u64>() as f64 / recent_metrics.len() as f64
        } else {
            0.0
        };
        
        // 生成趋势数据（简化版本）
        let error_rate_trend = vec![(now, 1.0 - success_rate_24h)];
        let performance_trend = vec![(now, avg_response_time_24h)];
        
        // 统计最常用工具
        let mut tool_usage = HashMap::new();
        for tool_metric in tool_metrics.iter() {
            *tool_usage.entry(tool_metric.tool_name.clone()).or_insert(0u64) += 1;
        }
        
        let mut top_tools: Vec<(String, u64)> = tool_usage.into_iter().collect();
        top_tools.sort_by(|a, b| b.1.cmp(&a.1));
        top_tools.truncate(10);
        
        Ok(AgentPerformance {
            agent_name: agent_name.to_string(),
            executions_last_24h,
            success_rate_24h,
            avg_response_time_24h,
            error_rate_trend,
            performance_trend,
            top_tools,
            resource_usage: ResourceUsage {
                avg_memory_mb: 50.0, // 示例数据
                peak_memory_mb: 100.0,
                cpu_usage_percent: 25.0,
            },
        })
    }
}

#[async_trait]
impl TraceCollector for InMemoryMetricsCollector {
    async fn start_trace(&self, agent_id: String, metadata: HashMap<String, serde_json::Value>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut trace = ExecutionTrace::new(agent_id);
        trace.metadata = metadata;
        
        let trace_id = trace.trace_id.clone();
        self.traces.write().await.insert(trace_id.clone(), trace);
        
        Ok(trace_id)
    }
    
    async fn end_trace(&self, trace_id: &str, success: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut traces = self.traces.write().await;
        if let Some(trace) = traces.get_mut(trace_id) {
            trace.success = success;
            trace.end_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            trace.total_duration_ms = trace.end_time - trace.start_time;
        }
        Ok(())
    }
    
    async fn add_trace_step(&self, trace_id: &str, step: TraceStep) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut traces = self.traces.write().await;
        if let Some(trace) = traces.get_mut(trace_id) {
            trace.steps.push(step);
        }
        Ok(())
    }
    
    async fn get_trace(&self, trace_id: &str) -> Result<Option<ExecutionTrace>, Box<dyn std::error::Error + Send + Sync>> {
        let traces = self.traces.read().await;
        Ok(traces.get(trace_id).cloned())
    }
    
    async fn search_traces(&self, query: TraceQuery) -> Result<Vec<ExecutionTrace>, Box<dyn std::error::Error + Send + Sync>> {
        let traces = self.traces.read().await;
        
        let filtered_traces: Vec<ExecutionTrace> = traces.values()
            .filter(|trace| {
                if let Some(ref agent_id) = query.agent_id {
                    if &trace.agent_id != agent_id {
                        return false;
                    }
                }
                if let Some(start_time) = query.start_time {
                    if trace.start_time < start_time {
                        return false;
                    }
                }
                if let Some(end_time) = query.end_time {
                    if trace.end_time > end_time {
                        return false;
                    }
                }
                if let Some(success) = query.success {
                    if trace.success != success {
                        return false;
                    }
                }
                if let Some(min_duration) = query.min_duration_ms {
                    if trace.total_duration_ms < min_duration {
                        return false;
                    }
                }
                if let Some(max_duration) = query.max_duration_ms {
                    if trace.total_duration_ms > max_duration {
                        return false;
                    }
                }
                // 检查标签过滤
                for (key, value) in &query.tags {
                    if trace.tags.get(key) != Some(value) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        
        let mut result = filtered_traces;
        
        // 应用偏移量和限制
        if let Some(offset) = query.offset {
            if offset < result.len() {
                result = result.into_iter().skip(offset).collect();
            } else {
                result.clear();
            }
        }
        
        if let Some(limit) = query.limit {
            result.truncate(limit);
        }
        
        Ok(result)
    }
    
    async fn get_trace_stats(&self, agent_id: Option<&str>, from_time: Option<u64>, to_time: Option<u64>) -> Result<TraceStats, Box<dyn std::error::Error + Send + Sync>> {
        let traces = self.traces.read().await;
        
        let filtered_traces: Vec<&ExecutionTrace> = traces.values()
            .filter(|trace| {
                if let Some(aid) = agent_id {
                    if trace.agent_id != aid {
                        return false;
                    }
                }
                if let Some(from) = from_time {
                    if trace.start_time < from {
                        return false;
                    }
                }
                if let Some(to) = to_time {
                    if trace.end_time > to {
                        return false;
                    }
                }
                true
            })
            .collect();
        
        if filtered_traces.is_empty() {
            return Ok(TraceStats {
                total_traces: 0,
                successful_traces: 0,
                failed_traces: 0,
                avg_duration_ms: 0.0,
                p50_duration_ms: 0,
                p95_duration_ms: 0,
                p99_duration_ms: 0,
                slowest_traces: Vec::new(),
                step_type_stats: HashMap::new(),
            });
        }
        
        let total_traces = filtered_traces.len() as u64;
        let successful_traces = filtered_traces.iter().filter(|t| t.success).count() as u64;
        let failed_traces = total_traces - successful_traces;
        
        let mut durations: Vec<u64> = filtered_traces.iter().map(|t| t.total_duration_ms).collect();
        durations.sort();
        
        let avg_duration_ms = durations.iter().sum::<u64>() as f64 / durations.len() as f64;
        
        let p50_index = durations.len() / 2;
        let p95_index = (durations.len() as f64 * 0.95) as usize;
        let p99_index = (durations.len() as f64 * 0.99) as usize;
        
        let p50_duration_ms = durations.get(p50_index).copied().unwrap_or(0);
        let p95_duration_ms = durations.get(p95_index).copied().unwrap_or(0);
        let p99_duration_ms = durations.get(p99_index).copied().unwrap_or(0);
        
        // 获取最慢的追踪
        let mut slowest_traces: Vec<(String, u64)> = filtered_traces.iter()
            .map(|t| (t.trace_id.clone(), t.total_duration_ms))
            .collect();
        slowest_traces.sort_by(|a, b| b.1.cmp(&a.1));
        slowest_traces.truncate(10);
        let slowest_trace_ids: Vec<String> = slowest_traces.into_iter().map(|(id, _)| id).collect();
        
        // 统计步骤类型
        let mut step_type_stats = HashMap::new();
        for trace in &filtered_traces {
            for step in &trace.steps {
                let step_type_name = match &step.step_type {
                    StepType::LlmCall => "llm_call",
                    StepType::ToolCall => "tool_call",
                    StepType::MemoryOperation => "memory_operation",
                    StepType::DataProcessing => "data_processing",
                    StepType::Validation => "validation",
                    StepType::Transformation => "transformation",
                    StepType::Custom(name) => name,
                }.to_string();
                
                *step_type_stats.entry(step_type_name).or_insert(0) += 1;
            }
        }
        
        Ok(TraceStats {
            total_traces,
            successful_traces,
            failed_traces,
            avg_duration_ms,
            p50_duration_ms,
            p95_duration_ms,
            p99_duration_ms,
            slowest_traces: slowest_trace_ids,
            step_type_stats,
        })
    }
}

impl Default for InMemoryMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// 文件系统指标收集器
#[derive(Debug)]
pub struct FileSystemMetricsCollector {
    /// 数据目录路径
    data_dir: std::path::PathBuf,
    /// 内存缓存
    cache: InMemoryMetricsCollector,
}

impl FileSystemMetricsCollector {
    /// 创建新的文件系统指标收集器
    pub fn new<P: AsRef<std::path::Path>>(data_dir: P) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let data_dir = data_dir.as_ref().to_path_buf();
        
        // 确保目录存在
        std::fs::create_dir_all(&data_dir)?;
        
        Ok(Self {
            data_dir,
            cache: InMemoryMetricsCollector::new(),
        })
    }
    
    /// 保存指标到文件
    async fn save_to_file<T: serde::Serialize>(&self, filename: &str, data: &T) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let file_path = self.data_dir.join(filename);
        let json_data = serde_json::to_string_pretty(data)?;
        tokio::fs::write(file_path, json_data).await?;
        Ok(())
    }
    
    /// 从文件加载指标
    async fn load_from_file<T: serde::de::DeserializeOwned>(&self, filename: &str) -> Result<Option<T>, Box<dyn std::error::Error + Send + Sync>> {
        let file_path = self.data_dir.join(filename);
        
        if !file_path.exists() {
            return Ok(None);
        }
        
        let json_data = tokio::fs::read_to_string(file_path).await?;
        let data: T = serde_json::from_str(&json_data)?;
        Ok(Some(data))
    }
}

#[async_trait]
impl MetricsCollector for FileSystemMetricsCollector {
    async fn record_agent_execution(&self, metrics: AgentMetrics) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 先保存到内存缓存
        self.cache.record_agent_execution(metrics.clone()).await?;
        
        // 然后保存到文件
        let filename = format!("agent_metrics_{}.json", metrics.execution_id);
        self.save_to_file(&filename, &metrics).await?;
        
        Ok(())
    }
    
    async fn record_tool_execution(&self, metrics: ToolMetrics) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.cache.record_tool_execution(metrics.clone()).await?;
        
        let filename = format!("tool_metrics_{}_{}.json", metrics.tool_name, metrics.timestamp);
        self.save_to_file(&filename, &metrics).await?;
        
        Ok(())
    }
    
    async fn record_memory_operation(&self, metrics: MemoryMetrics) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.cache.record_memory_operation(metrics.clone()).await?;
        
        let filename = format!("memory_metrics_{}_{}.json", metrics.operation_type, metrics.timestamp);
        self.save_to_file(&filename, &metrics).await?;
        
        Ok(())
    }
    
    async fn get_metrics_summary(&self, agent_name: Option<&str>, from_time: Option<u64>, to_time: Option<u64>) -> Result<MetricsSummary, Box<dyn std::error::Error + Send + Sync>> {
        // 使用内存缓存进行快速查询
        self.cache.get_metrics_summary(agent_name, from_time, to_time).await
    }
    
    async fn get_agent_performance(&self, agent_name: &str) -> Result<AgentPerformance, Box<dyn std::error::Error + Send + Sync>> {
        self.cache.get_agent_performance(agent_name).await
    }
}

#[async_trait]
impl TraceCollector for FileSystemMetricsCollector {
    async fn start_trace(&self, agent_id: String, metadata: HashMap<String, serde_json::Value>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.cache.start_trace(agent_id, metadata).await
    }
    
    async fn end_trace(&self, trace_id: &str, success: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.cache.end_trace(trace_id, success).await?;
        
        // 保存完整的追踪到文件
        if let Some(trace) = self.cache.get_trace(trace_id).await? {
            let filename = format!("trace_{}.json", trace_id);
            self.save_to_file(&filename, &trace).await?;
        }
        
        Ok(())
    }
    
    async fn add_trace_step(&self, trace_id: &str, step: TraceStep) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.cache.add_trace_step(trace_id, step).await
    }
    
    async fn get_trace(&self, trace_id: &str) -> Result<Option<ExecutionTrace>, Box<dyn std::error::Error + Send + Sync>> {
        // 先检查内存缓存
        if let Some(trace) = self.cache.get_trace(trace_id).await? {
            return Ok(Some(trace));
        }
        
        // 如果缓存中没有，尝试从文件加载
        let filename = format!("trace_{}.json", trace_id);
        self.load_from_file(&filename).await
    }
    
    async fn search_traces(&self, query: TraceQuery) -> Result<Vec<ExecutionTrace>, Box<dyn std::error::Error + Send + Sync>> {
        self.cache.search_traces(query).await
    }
    
    async fn get_trace_stats(&self, agent_id: Option<&str>, from_time: Option<u64>, to_time: Option<u64>) -> Result<TraceStats, Box<dyn std::error::Error + Send + Sync>> {
        self.cache.get_trace_stats(agent_id, from_time, to_time).await
    }
}
