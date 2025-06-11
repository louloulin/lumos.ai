/*!
# Message Stream Component

消息流组件，负责显示聊天消息和流式响应。
*/

use dioxus::prelude::*;
use crate::console::*;
use crate::console::chat_console::EmptyState;

/// 消息流组件
#[component]
pub fn MessageStream(
    conversation: Option<ChatConversation>,
    pending_state: PendingChatState,
    is_streaming: bool
) -> Element {
    rsx! {
        div {
            class: "flex-1 overflow-y-auto p-4 space-y-4",
            id: "message-stream",

            // 空状态
            div {
                class: "flex items-center justify-center h-full",
                EmptyState {}
            }
        }
    }
}
