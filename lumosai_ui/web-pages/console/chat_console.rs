/*!
# Chat Console Component

主聊天控制台组件，提供完整的AI对话界面。

## 功能特性

- **实时对话**: 流式AI响应显示
- **消息历史**: 完整的对话历史记录
- **多媒体支持**: 文件上传和语音输入
- **响应式布局**: 适配不同屏幕尺寸
*/

use dioxus::prelude::*;
use crate::console::*;
use crate::console::message_stream::MessageStream;
use crate::console::chat_form::ChatForm;

/// 聊天控制台主组件
#[component]
pub fn ChatConsole() -> Element {
    rsx! {
        div {
            class: "flex flex-col h-full bg-base-100",

            // 聊天头部
            ChatHeader {
                conversation: None
            }

            // 消息区域
            div {
                class: "flex-1 overflow-hidden",
                MessageStream {
                    conversation: None,
                    pending_state: PendingChatState::None,
                    is_streaming: false
                }
            }

            // 输入区域
            div {
                class: "border-t border-base-300 bg-base-100",
                ChatForm {
                    is_locked: false,
                    on_send: move |content: String| {
                        send_message(content);
                    }
                }
            }
        }
    }
}

/// 聊天头部组件
#[component]
fn ChatHeader(conversation: Option<ChatConversation>) -> Element {
    rsx! {
        div {
            class: "flex items-center justify-between p-4 border-b border-base-300 bg-base-100",
            
            div {
                class: "flex items-center space-x-3",
                
                // AI头像
                div {
                    class: "w-10 h-10 bg-primary rounded-full flex items-center justify-center",
                    span {
                        class: "text-primary-content font-bold text-lg",
                        "🤖"
                    }
                }
                
                div {
                    h2 {
                        class: "text-lg font-semibold text-base-content",
                        if let Some(ref conv) = conversation {
                            "{conv.title}"
                        } else {
                            "LumosAI Assistant"
                        }
                    }
                    p {
                        class: "text-sm text-base-content/70",
                        "AI助手 • 在线"
                    }
                }
            }
            
            // 操作按钮
            div {
                class: "flex items-center space-x-2",
                
                button {
                    class: "btn btn-ghost btn-sm",
                    title: "新建对话",
                    onclick: move |_| {
                        // TODO: 实现新建对话
                    },
                    "➕"
                }
                
                button {
                    class: "btn btn-ghost btn-sm",
                    title: "对话历史",
                    onclick: move |_| {
                        // TODO: 显示对话历史
                    },
                    "📋"
                }
                
                button {
                    class: "btn btn-ghost btn-sm",
                    title: "设置",
                    onclick: move |_| {
                        // TODO: 显示设置
                    },
                    "⚙️"
                }
            }
        }
    }
}

/// 空状态组件
#[component]
pub fn EmptyState() -> Element {
    rsx! {
        div {
            class: "flex flex-col items-center justify-center h-full p-8 text-center",
            
            div {
                class: "w-24 h-24 bg-primary/10 rounded-full flex items-center justify-center mb-6",
                span {
                    class: "text-4xl",
                    "💬"
                }
            }
            
            h3 {
                class: "text-xl font-semibold text-base-content mb-2",
                "开始新的对话"
            }
            
            p {
                class: "text-base-content/70 mb-6 max-w-md",
                "向AI助手提问任何问题，获得智能回答和帮助。支持文本、文件和语音输入。"
            }
            
            div {
                class: "flex flex-wrap gap-2 justify-center",
                
                SuggestionChip {
                    text: "解释量子计算",
                    icon: "🔬"
                }
                
                SuggestionChip {
                    text: "写一首诗",
                    icon: "✍️"
                }
                
                SuggestionChip {
                    text: "编程帮助",
                    icon: "💻"
                }
                
                SuggestionChip {
                    text: "数据分析",
                    icon: "📊"
                }
            }
        }
    }
}

/// 建议芯片组件
#[component]
fn SuggestionChip(text: &'static str, icon: &'static str) -> Element {
    rsx! {
        button {
            class: "btn btn-outline btn-sm gap-2 hover:btn-primary",
            onclick: move |_| {
                send_message(text.to_string());
            },
            span { "{icon}" }
            span { "{text}" }
        }
    }
}

/// 加载状态组件
#[component]
pub fn LoadingState() -> Element {
    rsx! {
        div {
            class: "flex items-center justify-center p-8",
            
            div {
                class: "flex items-center space-x-3",
                
                div {
                    class: "loading loading-spinner loading-md text-primary"
                }
                
                span {
                    class: "text-base-content/70",
                    "正在加载对话..."
                }
            }
        }
    }
}

/// 错误状态组件
#[component]
pub fn ErrorState(error_message: String) -> Element {
    rsx! {
        div {
            class: "flex flex-col items-center justify-center p-8 text-center",
            
            div {
                class: "w-16 h-16 bg-error/10 rounded-full flex items-center justify-center mb-4",
                span {
                    class: "text-2xl text-error",
                    "⚠️"
                }
            }
            
            h3 {
                class: "text-lg font-semibold text-base-content mb-2",
                "连接错误"
            }
            
            p {
                class: "text-base-content/70 mb-4 max-w-md",
                "{error_message}"
            }
            
            button {
                class: "btn btn-primary btn-sm",
                onclick: move |_| {
                    // TODO: 重试连接
                },
                "重试"
            }
        }
    }
}
