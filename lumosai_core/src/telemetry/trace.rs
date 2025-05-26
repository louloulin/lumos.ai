use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use async_trait::async_trait;

/// 执行追踪数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    /// 追踪ID
    pub trace_id: String,
    /// 代理ID
    pub agent_id: String,
    /// 执行步骤
    pub steps: Vec<TraceStep>,
    /// 总执行时长
    pub total_duration_ms: u64,
    /// 开始时间
    pub start_time: u64,
    /// 结束时间
    pub end_time: u64,
    /// 是否成功
    pub success: bool,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
    /// 追踪标签
    pub tags: HashMap<String, String>,
    /// 父追踪ID（用于分布式追踪）
    pub parent_trace_id: Option<String>,
    /// 根追踪ID
    pub root_trace_id: String,
}

/// 追踪步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceStep {
    /// 步骤ID
    pub step_id: String,
    /// 步骤名称
    pub name: String,
    /// 步骤类型
    pub step_type: StepType,
    /// 开始时间
    pub start_time: u64,
    /// 结束时间
    pub end_time: u64,
    /// 执行时长（毫秒）
    pub duration_ms: u64,
    /// 是否成功
    pub success: bool,
    /// 错误信息
    pub error: Option<String>,
    /// 输入数据
    pub input: Option<serde_json::Value>,
    /// 输出数据
    pub output: Option<serde_json::Value>,
    /// 步骤元数据
    pub metadata: HashMap<String, serde_json::Value>,
    /// 子步骤
    pub children: Vec<TraceStep>,
}

/// 步骤类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    /// LLM调用
    LlmCall,
    /// 工具调用
    ToolCall,
    /// 内存操作
    MemoryOperation,
    /// 数据处理
    DataProcessing,
    /// 验证
    Validation,
    /// 转换
    Transformation,
    /// 自定义步骤
    Custom(String),
}

/// 追踪收集器trait
#[async_trait]
pub trait TraceCollector: Send + Sync {
    /// 开始新的追踪
    async fn start_trace(&self, agent_id: String, metadata: HashMap<String, serde_json::Value>) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 结束追踪
    async fn end_trace(&self, trace_id: &str, success: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// 添加追踪步骤
    async fn add_trace_step(&self, trace_id: &str, step: TraceStep) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// 获取追踪信息
    async fn get_trace(&self, trace_id: &str) -> Result<Option<ExecutionTrace>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 搜索追踪
    async fn search_traces(&self, query: TraceQuery) -> Result<Vec<ExecutionTrace>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 获取追踪统计
    async fn get_trace_stats(&self, agent_id: Option<&str>, from_time: Option<u64>, to_time: Option<u64>) -> Result<TraceStats, Box<dyn std::error::Error + Send + Sync>>;
}

/// 追踪查询参数
#[derive(Debug, Clone)]
pub struct TraceQuery {
    /// 代理ID
    pub agent_id: Option<String>,
    /// 开始时间
    pub start_time: Option<u64>,
    /// 结束时间
    pub end_time: Option<u64>,
    /// 是否成功
    pub success: Option<bool>,
    /// 最小执行时长（毫秒）
    pub min_duration_ms: Option<u64>,
    /// 最大执行时长（毫秒）
    pub max_duration_ms: Option<u64>,
    /// 标签过滤
    pub tags: HashMap<String, String>,
    /// 限制结果数量
    pub limit: Option<usize>,
    /// 偏移量
    pub offset: Option<usize>,
}

/// 追踪统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceStats {
    /// 总追踪数量
    pub total_traces: u64,
    /// 成功追踪数量
    pub successful_traces: u64,
    /// 失败追踪数量
    pub failed_traces: u64,
    /// 平均执行时间
    pub avg_duration_ms: f64,
    /// P50执行时间
    pub p50_duration_ms: u64,
    /// P95执行时间
    pub p95_duration_ms: u64,
    /// P99执行时间
    pub p99_duration_ms: u64,
    /// 最慢的追踪
    pub slowest_traces: Vec<String>,
    /// 步骤类型统计
    pub step_type_stats: HashMap<String, u64>,
}

/// 追踪构建器
pub struct TraceBuilder {
    trace: ExecutionTrace,
    current_step: Option<TraceStep>,
}

impl TraceBuilder {
    /// 创建新的追踪构建器
    pub fn new(agent_id: String) -> Self {
        let trace_id = Uuid::new_v4().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
            
        Self {
            trace: ExecutionTrace {
                trace_id: trace_id.clone(),
                agent_id,
                steps: Vec::new(),
                total_duration_ms: 0,
                start_time: now,
                end_time: now,
                success: false,
                metadata: HashMap::new(),
                tags: HashMap::new(),
                parent_trace_id: None,
                root_trace_id: trace_id,
            },
            current_step: None,
        }
    }
    
    /// 设置父追踪ID
    pub fn with_parent_trace(mut self, parent_trace_id: String) -> Self {
        self.trace.parent_trace_id = Some(parent_trace_id.clone());
        self.trace.root_trace_id = parent_trace_id;
        self
    }
    
    /// 添加标签
    pub fn with_tag(mut self, key: String, value: String) -> Self {
        self.trace.tags.insert(key, value);
        self
    }
    
    /// 添加元数据
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.trace.metadata.insert(key, value);
        self
    }
    
    /// 开始新步骤
    pub fn start_step(&mut self, name: String, step_type: StepType) -> &mut Self {
        let step_id = Uuid::new_v4().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
            
        self.current_step = Some(TraceStep {
            step_id,
            name,
            step_type,
            start_time: now,
            end_time: now,
            duration_ms: 0,
            success: false,
            error: None,
            input: None,
            output: None,
            metadata: HashMap::new(),
            children: Vec::new(),
        });
        
        self
    }
    
    /// 设置步骤输入
    pub fn set_step_input(&mut self, input: serde_json::Value) -> &mut Self {
        if let Some(ref mut step) = self.current_step {
            step.input = Some(input);
        }
        self
    }
    
    /// 设置步骤输出
    pub fn set_step_output(&mut self, output: serde_json::Value) -> &mut Self {
        if let Some(ref mut step) = self.current_step {
            step.output = Some(output);
        }
        self
    }
    
    /// 设置步骤错误
    pub fn set_step_error(&mut self, error: String) -> &mut Self {
        if let Some(ref mut step) = self.current_step {
            step.error = Some(error);
            step.success = false;
        }
        self
    }
    
    /// 完成当前步骤
    pub fn end_step(&mut self, success: bool) -> &mut Self {
        if let Some(mut step) = self.current_step.take() {
            step.end_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            step.duration_ms = step.end_time - step.start_time;
            step.success = success;
            
            self.trace.steps.push(step);
        }
        self
    }
    
    /// 完成追踪
    pub fn finish(mut self, success: bool) -> ExecutionTrace {
        self.trace.end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        self.trace.total_duration_ms = self.trace.end_time - self.trace.start_time;
        self.trace.success = success;
        
        self.trace
    }
    
    /// 获取追踪ID
    pub fn trace_id(&self) -> &str {
        &self.trace.trace_id
    }
}

impl ExecutionTrace {
    /// 创建新的追踪
    pub fn new(agent_id: String) -> Self {
        let trace_id = Uuid::new_v4().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
            
        Self {
            trace_id: trace_id.clone(),
            agent_id,
            steps: Vec::new(),
            total_duration_ms: 0,
            start_time: now,
            end_time: now,
            success: false,
            metadata: HashMap::new(),
            tags: HashMap::new(),
            parent_trace_id: None,
            root_trace_id: trace_id,
        }
    }
    
    /// 获取所有失败的步骤
    pub fn get_failed_steps(&self) -> Vec<&TraceStep> {
        self.steps.iter().filter(|step| !step.success).collect()
    }
    
    /// 获取最慢的步骤
    pub fn get_slowest_step(&self) -> Option<&TraceStep> {
        self.steps.iter().max_by_key(|step| step.duration_ms)
    }
    
    /// 计算总的工具调用次数
    pub fn count_tool_calls(&self) -> usize {
        self.steps.iter()
            .filter(|step| matches!(step.step_type, StepType::ToolCall))
            .count()
    }
    
    /// 获取错误摘要
    pub fn get_error_summary(&self) -> Vec<String> {
        self.steps.iter()
            .filter_map(|step| step.error.as_ref())
            .cloned()
            .collect()
    }
}

impl TraceStep {
    /// 创建新的追踪步骤
    pub fn new(name: String, step_type: StepType) -> Self {
        let step_id = Uuid::new_v4().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
            
        Self {
            step_id,
            name,
            step_type,
            start_time: now,
            end_time: now,
            duration_ms: 0,
            success: false,
            error: None,
            input: None,
            output: None,
            metadata: HashMap::new(),
            children: Vec::new(),
        }
    }
    
    /// 添加子步骤
    pub fn add_child(&mut self, child: TraceStep) {
        self.children.push(child);
    }
    
    /// 设置持续时间
    pub fn set_duration(&mut self, duration: Duration) {
        self.duration_ms = duration.as_millis() as u64;
        self.end_time = self.start_time + self.duration_ms;
    }
}

impl Default for TraceQuery {
    fn default() -> Self {
        Self {
            agent_id: None,
            start_time: None,
            end_time: None,
            success: None,
            min_duration_ms: None,
            max_duration_ms: None,
            tags: HashMap::new(),
            limit: Some(100),
            offset: None,
        }
    }
}
