//! SOP (Standard Operating Procedure) 类型定义
//!
//! 为现有 Agent 系统添加 SOP 协作能力，基于 MetaGPT 的设计理念

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::llm::Message;

/// Agent 行动类型
///
/// 定义 Agent 在 SOP 流程中可以执行的行动
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentAction {
    /// 回复消息
    Reply {
        /// 回复内容
        content: String,
    },

    /// 发送消息给特定 Agent
    Send {
        /// 消息类型
        msg_type: String,
        /// 接收者 ID（None 表示广播）
        receiver: Option<String>,
        /// 消息内容
        content: Value,
    },

    /// 调用工具
    ToolCall {
        /// 工具名称
        tool_name: String,
        /// 工具参数
        arguments: Value,
    },

    /// 委派任务给其他 Agent
    Delegate {
        /// 目标 Agent ID
        target_agent: String,
        /// 任务描述
        task: String,
        /// 任务参数
        params: Value,
    },

    /// 等待（暂不执行）
    Wait {
        /// 等待原因
        reason: String,
    },

    /// 完成任务
    Finish {
        /// 最终结果
        result: Value,
    },

    /// 无操作
    NoOp,
}

impl AgentAction {
    /// 创建回复行动
    pub fn reply(content: impl Into<String>) -> Self {
        Self::Reply {
            content: content.into(),
        }
    }

    /// 创建发送行动
    pub fn send(
        msg_type: impl Into<String>,
        receiver: Option<impl Into<String>>,
        content: Value,
    ) -> Self {
        Self::Send {
            msg_type: msg_type.into(),
            receiver: receiver.map(|r| r.into()),
            content,
        }
    }

    /// 创建工具调用行动
    pub fn tool_call(tool_name: impl Into<String>, arguments: Value) -> Self {
        Self::ToolCall {
            tool_name: tool_name.into(),
            arguments,
        }
    }

    /// 创建委派行动
    pub fn delegate(
        target_agent: impl Into<String>,
        task: impl Into<String>,
        params: Value,
    ) -> Self {
        Self::Delegate {
            target_agent: target_agent.into(),
            task: task.into(),
            params,
        }
    }

    /// 创建等待行动
    pub fn wait(reason: impl Into<String>) -> Self {
        Self::Wait {
            reason: reason.into(),
        }
    }

    /// 创建完成行动
    pub fn finish(result: Value) -> Self {
        Self::Finish { result }
    }
}

/// SOP 消息类型
///
/// 扩展现有的 Message 系统，添加 SOP 特定的元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SopMessage {
    /// 消息 ID
    pub id: String,

    /// 消息类型（用于 watch 匹配）
    pub msg_type: String,

    /// 发送者 Agent ID
    pub sender: String,

    /// 接收者 Agent ID（None 表示广播）
    pub receiver: Option<String>,

    /// 消息内容
    pub content: Value,

    /// 元数据
    pub metadata: HashMap<String, Value>,

    /// 时间戳
    pub timestamp: i64,
}

impl SopMessage {
    /// 创建新消息
    pub fn new(
        msg_type: impl Into<String>,
        sender: impl Into<String>,
        receiver: Option<impl Into<String>>,
        content: Value,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            msg_type: msg_type.into(),
            sender: sender.into(),
            receiver: receiver.map(|r| r.into()),
            content,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// 创建广播消息
    pub fn broadcast(
        msg_type: impl Into<String>,
        sender: impl Into<String>,
        content: Value,
    ) -> Self {
        Self::new(msg_type, sender, None::<String>, content)
    }

    /// 检查消息是否发送给指定接收者
    pub fn is_for(&self, receiver_id: &str) -> bool {
        match &self.receiver {
            Some(r) => r == receiver_id,
            None => true, // 广播消息对所有人可见
        }
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// 转换为 LLM Message
    pub fn to_llm_message(&self) -> Message {
        Message {
            role: crate::llm::Role::User,
            content: serde_json::to_string(&self.content).unwrap_or_default(),
            metadata: Some(self.metadata.clone()),
            name: Some(self.sender.clone()),
        }
    }
}

/// SOP 执行模式
///
/// 定义 Agent 团队的执行策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SopExecutionMode {
    /// React 模式：事件驱动，Agent 响应消息并执行
    React,

    /// ByOrder 模式：按预定义顺序依次执行
    ByOrder,

    /// PlanAndAct 模式：先规划再执行
    PlanAndAct,
}

impl Default for SopExecutionMode {
    fn default() -> Self {
        Self::React
    }
}

/// SOP 统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SopStats {
    /// 总消息数
    pub total_messages: usize,

    /// 按类型分组的消息数
    pub message_types: HashMap<String, usize>,

    /// 活跃 Agent 数
    pub active_agents: usize,

    /// 完成的 Agent 数
    pub completed_agents: usize,
}

impl Default for SopStats {
    fn default() -> Self {
        Self {
            total_messages: 0,
            message_types: HashMap::new(),
            active_agents: 0,
            completed_agents: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_agent_action_creation() {
        let action = AgentAction::reply("test");
        assert!(matches!(action, AgentAction::Reply { .. }));

        let action = AgentAction::send("test_type", Some("receiver"), json!({}));
        assert!(matches!(action, AgentAction::Send { .. }));
    }

    #[test]
    fn test_sop_message_creation() {
        let msg = SopMessage::new("test", "sender", Some("receiver"), json!({"data": "test"}));
        assert_eq!(msg.msg_type, "test");
        assert_eq!(msg.sender, "sender");
        assert!(msg.is_for("receiver"));
    }

    #[test]
    fn test_broadcast_message() {
        let msg = SopMessage::broadcast("test", "sender", json!({}));
        assert!(msg.is_for("anyone"));
        assert!(msg.receiver.is_none());
    }
}
