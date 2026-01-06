//! # 推理轨迹可视化
//!
//! 记录和可视化 Agent 的思维过程。

use serde::{Deserialize, Serialize};

/// 轨迹事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    /// 事件 ID
    pub id: String,
    /// 事件类型
    pub event_type: String,
    /// 时间戳
    pub timestamp: u64,
    /// 内容
    pub content: String,
    /// 父事件 ID
    pub parent_id: Option<String>,
}

/// 推理轨迹
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningTrace {
    /// 轨迹 ID
    pub id: String,
    /// 事件列表
    pub events: Vec<TraceEvent>,
}

/// 轨迹可视化
pub struct TraceVisualization;

impl TraceVisualization {
    pub fn format_trace(trace: &ReasoningTrace) -> String {
        let mut output = String::new();
        output.push_str(&format!("Trace ID: {}\n", trace.id));
        output.push_str("Events:\n");

        for event in &trace.events {
            output.push_str(&format!(
                "- [{}] {}: {}\n",
                event.timestamp, event.event_type, event.content
            ));
        }

        output
    }

    pub fn format_mermaid(trace: &ReasoningTrace) -> String {
        let mut output = String::from("graph TD\n");

        for (i, event) in trace.events.iter().enumerate() {
            let node_id = format!("node{}", i);
            let label = format!("{}: {}", event.event_type,
                event.content.chars().take(30).collect::<String>());

            output.push_str(&format!("    {}[\"{}\"]\n", node_id, label));

            if let Some(parent_id) = &event.parent_id {
                output.push_str(&format!("    {} --> {}\n", parent_id, node_id));
            }
        }

        output
    }
}
