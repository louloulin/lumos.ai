//! Agent状态管理器 - 负责Agent的状态管理和维护
//!
//! 这个组件负责管理Agent的运行状态、会话状态和统计信息。

use crate::agent::types::{AgentState as AgentStateType, AgentStatus};
use crate::agent::{Message, ToolCall};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Agent状态
///
/// 管理Agent的运行状态、会话历史和统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    /// Agent当前状态
    pub status: AgentStatus,
    /// 会话历史消息
    pub conversation_history: Vec<Message>,
    /// 消息处理数量
    pub message_count: usize,
    /// 总token使用量
    pub total_tokens: usize,
    /// 工具调用次数
    pub tool_calls_count: usize,
    /// 平均响应时间（毫秒）
    pub average_response_time: f64,
    /// 最后活动时间
    pub last_activity: DateTime<Utc>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 会话ID
    pub session_id: Option<String>,
    /// 用户ID
    pub user_id: Option<String>,
    /// 自定义状态数据
    pub custom_data: HashMap<String, serde_json::Value>,
    /// 错误计数
    pub error_count: usize,
    /// 最后错误
    pub last_error: Option<String>,
    /// 性能指标
    pub metrics: AgentMetrics,
}

/// Agent性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    /// 响应时间统计
    pub response_times: VecDeque<f64>,
    /// 内存使用统计
    pub memory_usage: VecDeque<usize>,
    /// CPU使用统计
    pub cpu_usage: VecDeque<f64>,
    /// 工具调用延迟
    pub tool_call_latencies: HashMap<String, VecDeque<f64>>,
    /// 成功率统计
    pub success_rate: f64,
    /// 超时统计
    pub timeout_count: usize,
}

impl Default for AgentMetrics {
    fn default() -> Self {
        Self {
            response_times: VecDeque::new(),
            memory_usage: VecDeque::new(),
            cpu_usage: VecDeque::new(),
            tool_call_latencies: HashMap::new(),
            success_rate: 0.0,
            timeout_count: 0,
        }
    }
}

impl AgentState {
    /// 创建新的Agent状态
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            status: AgentStatus::Idle,
            conversation_history: Vec::new(),
            message_count: 0,
            total_tokens: 0,
            tool_calls_count: 0,
            average_response_time: 0.0,
            last_activity: now,
            created_at: now,
            session_id: None,
            user_id: None,
            custom_data: HashMap::new(),
            error_count: 0,
            last_error: None,
            metrics: AgentMetrics::default(),
        }
    }

    /// 创建带会话ID的Agent状态
    pub fn with_session(session_id: String) -> Self {
        let mut state = Self::new();
        state.session_id = Some(session_id);
        state
    }

    /// 设置状态
    pub fn set_status(&mut self, status: AgentStatus) {
        self.status = status;
        self.last_activity = Utc::now();
    }

    /// 获取状态
    pub fn status(&self) -> AgentStatus {
        self.status
    }

    /// 添加消息到历史记录
    pub fn add_message(&mut self, message: Message) {
        self.conversation_history.push(message);
        self.message_count += 1;
        self.last_activity = Utc::now();

        // 限制历史记录长度
        if self.conversation_history.len() > 1000 {
            self.conversation_history.remove(0);
        }
    }

    /// 获取会话历史
    pub fn conversation_history(&self) -> &[Message] {
        &self.conversation_history
    }

    /// 清空会话历史
    pub fn clear_history(&mut self) {
        self.conversation_history.clear();
        self.last_activity = Utc::now();
    }

    /// 设置会话ID
    pub fn set_session_id(&mut self, session_id: String) {
        self.session_id = Some(session_id);
    }

    /// 获取会话ID
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// 设置用户ID
    pub fn set_user_id(&mut self, user_id: String) {
        self.user_id = Some(user_id);
    }

    /// 获取用户ID
    pub fn user_id(&self) -> Option<&str> {
        self.user_id.as_deref()
    }

    /// 记录token使用
    pub fn record_token_usage(&mut self, tokens: usize) {
        self.total_tokens += tokens;
        self.last_activity = Utc::now();
    }

    /// 记录工具调用
    pub fn record_tool_call(&mut self, tool_name: &str, duration_ms: f64) {
        self.tool_calls_count += 1;
        self.last_activity = Utc::now();

        // 记录工具调用延迟
        let latencies = self
            .metrics
            .tool_call_latencies
            .entry(tool_name.to_string())
            .or_insert_with(|| VecDeque::new());

        latencies.push_back(duration_ms);

        // 限制记录数量
        if latencies.len() > 100 {
            latencies.pop_front();
        }
    }

    /// 记录响应时间
    pub fn record_response_time(&mut self, response_time_ms: f64) {
        self.metrics.response_times.push_back(response_time_ms);

        // 限制记录数量
        if self.metrics.response_times.len() > 1000 {
            self.metrics.response_times.pop_front();
        }

        // 计算平均响应时间
        if !self.metrics.response_times.is_empty() {
            let sum: f64 = self.metrics.response_times.iter().sum();
            self.average_response_time = sum / self.metrics.response_times.len() as f64;
        }
    }

    /// 记录内存使用
    pub fn record_memory_usage(&mut self, memory_bytes: usize) {
        self.metrics.memory_usage.push_back(memory_bytes);

        // 限制记录数量
        if self.metrics.memory_usage.len() > 100 {
            self.metrics.memory_usage.pop_front();
        }
    }

    /// 记录错误
    pub fn record_error(&mut self, error: String) {
        self.error_count += 1;
        self.last_error = Some(error);
        self.last_activity = Utc::now();
    }

    /// 记录超时
    pub fn record_timeout(&mut self) {
        self.metrics.timeout_count += 1;
        self.last_activity = Utc::now();
    }

    /// 计算成功率
    pub fn update_success_rate(&mut self, success: bool) {
        if self.message_count > 0 {
            let successful_calls = if success {
                self.message_count.saturating_sub(self.error_count)
            } else {
                self.message_count.saturating_sub(self.error_count + 1)
            };

            self.metrics.success_rate = successful_calls as f64 / self.message_count as f64;
        }
    }

    /// 获取工具调用统计
    pub fn get_tool_stats(&self, tool_name: &str) -> Option<ToolStats> {
        let latencies = self.metrics.tool_call_latencies.get(tool_name)?;

        if latencies.is_empty() {
            return None;
        }

        let count = latencies.len();
        let sum: f64 = latencies.iter().sum();
        let avg = sum / count as f64;
        let min = latencies.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = latencies.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        Some(ToolStats {
            count,
            average_latency_ms: avg,
            min_latency_ms: min,
            max_latency_ms: max,
        })
    }

    /// 获取性能摘要
    pub fn get_performance_summary(&self) -> PerformanceSummary {
        let avg_response_time = if self.metrics.response_times.is_empty() {
            0.0
        } else {
            let sum: f64 = self.metrics.response_times.iter().sum();
            sum / self.metrics.response_times.len() as f64
        };

        let avg_memory_usage = if self.metrics.memory_usage.is_empty() {
            0
        } else {
            let sum: usize = self.metrics.memory_usage.iter().sum();
            sum / self.metrics.memory_usage.len()
        };

        PerformanceSummary {
            uptime_seconds: (Utc::now() - self.created_at).num_seconds() as u64,
            message_count: self.message_count,
            total_tokens: self.total_tokens,
            tool_calls_count: self.tool_calls_count,
            average_response_time_ms: avg_response_time,
            average_memory_usage_bytes: avg_memory_usage,
            success_rate: self.metrics.success_rate,
            error_count: self.error_count,
            timeout_count: self.metrics.timeout_count,
        }
    }

    /// 检查是否健康
    pub fn is_healthy(&self) -> bool {
        // 检查错误率
        if self.message_count > 0 && self.error_count as f64 / self.message_count as f64 > 0.1 {
            return false;
        }

        // 检查超时率
        if self.tool_calls_count > 0
            && self.metrics.timeout_count as f64 / self.tool_calls_count as f64 > 0.05
        {
            return false;
        }

        // 检查响应时间
        if self.average_response_time > 10000.0 {
            // 10秒
            return false;
        }

        true
    }

    /// 获取状态快照
    pub fn snapshot(&self) -> AgentStateSnapshot {
        AgentStateSnapshot {
            status: self.status,
            message_count: self.message_count,
            total_tokens: self.total_tokens,
            tool_calls_count: self.tool_calls_count,
            average_response_time: self.average_response_time,
            last_activity: self.last_activity,
            session_id: self.session_id.clone(),
            user_id: self.user_id.clone(),
            error_count: self.error_count,
            success_rate: self.metrics.success_rate,
            uptime_seconds: (Utc::now() - self.created_at).num_seconds() as u64,
        }
    }

    /// 重置状态
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// 序列化状态
    pub fn serialize(&self) -> crate::error::Result<String> {
        serde_json::to_string(self).map_err(|e| crate::error::Error::Serialization {
            message: format!("Failed to serialize agent state: {}", e),
        })
    }

    /// 反序列化状态
    pub fn deserialize(data: &str) -> crate::error::Result<Self> {
        serde_json::from_str(data).map_err(|e| crate::error::Error::Serialization {
            message: format!("Failed to deserialize agent state: {}", e),
        })
    }
}

/// 工具统计信息
#[derive(Debug, Clone)]
pub struct ToolStats {
    /// 调用次数
    pub count: usize,
    /// 平均延迟（毫秒）
    pub average_latency_ms: f64,
    /// 最小延迟（毫秒）
    pub min_latency_ms: f64,
    /// 最大延迟（毫秒）
    pub max_latency_ms: f64,
}

/// 性能摘要
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    /// 运行时间（秒）
    pub uptime_seconds: u64,
    /// 消息数量
    pub message_count: usize,
    /// 总token数
    pub total_tokens: usize,
    /// 工具调用次数
    pub tool_calls_count: usize,
    /// 平均响应时间（毫秒）
    pub average_response_time_ms: f64,
    /// 平均内存使用（字节）
    pub average_memory_usage_bytes: usize,
    /// 成功率
    pub success_rate: f64,
    /// 错误数量
    pub error_count: usize,
    /// 超时数量
    pub timeout_count: usize,
}

/// Agent状态快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStateSnapshot {
    /// 状态
    pub status: AgentStatus,
    /// 消息数量
    pub message_count: usize,
    /// 总token数
    pub total_tokens: usize,
    /// 工具调用次数
    pub tool_calls_count: usize,
    /// 平均响应时间
    pub average_response_time: f64,
    /// 最后活动时间
    pub last_activity: DateTime<Utc>,
    /// 会话ID
    pub session_id: Option<String>,
    /// 用户ID
    pub user_id: Option<String>,
    /// 错误数量
    pub error_count: usize,
    /// 成功率
    pub success_rate: f64,
    /// 运行时间（秒）
    pub uptime_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_state_creation() {
        let state = AgentState::new();
        assert_eq!(state.status(), AgentStatus::Idle);
        assert_eq!(state.message_count, 0);
        assert_eq!(state.total_tokens, 0);
        assert!(state.is_healthy());
    }

    #[test]
    fn test_agent_state_with_session() {
        let state = AgentState::with_session("test-session".to_string());
        assert_eq!(state.session_id(), Some("test-session"));
    }

    #[test]
    fn test_message_addition() {
        let mut state = AgentState::new();
        let message = Message {
            role: crate::agent::types::MessageRole::User,
            content: "Hello".to_string(),
            ..Default::default()
        };

        state.add_message(message);
        assert_eq!(state.message_count, 1);
        assert_eq!(state.conversation_history.len(), 1);
    }

    #[test]
    fn test_token_usage_recording() {
        let mut state = AgentState::new();
        state.record_token_usage(100);
        assert_eq!(state.total_tokens, 100);
    }

    #[test]
    fn test_tool_call_recording() {
        let mut state = AgentState::new();
        state.record_tool_call("test_tool", 50.0);
        assert_eq!(state.tool_calls_count, 1);

        let stats = state.get_tool_stats("test_tool");
        assert!(stats.is_some());
        assert_eq!(stats.unwrap().count, 1);
    }

    #[test]
    fn test_response_time_recording() {
        let mut state = AgentState::new();
        state.record_response_time(100.0);
        state.record_response_time(200.0);

        assert_eq!(state.average_response_time, 150.0);
    }

    #[test]
    fn test_health_check() {
        let mut state = AgentState::new();
        assert!(state.is_healthy());

        // 模拟高错误率
        state.error_count = 100;
        state.message_count = 50;
        assert!(!state.is_healthy());
    }

    #[test]
    fn test_serialization() {
        let state = AgentState::new();
        let serialized = state.serialize().unwrap();
        let deserialized = AgentState::deserialize(&serialized).unwrap();

        assert_eq!(state.status(), deserialized.status());
        assert_eq!(state.message_count, deserialized.message_count);
    }
}
