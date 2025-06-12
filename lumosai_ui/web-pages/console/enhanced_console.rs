/*!
# Enhanced Console Component

增强的AI助手控制台，参考bionic-gpt的assistant_console.rs实现。

## 功能特性

- **完整的AI对话界面**: 集成聊天、工具、历史等功能
- **工具调用支持**: 显示和管理AI工具调用
- **实时流式响应**: 支持流式AI回复
- **多媒体支持**: 文件上传、语音输入等
- **权限控制**: 基于用户权限的功能访问
*/

#![allow(non_snake_case)]
use dioxus::prelude::*;
use crate::app_layout::{Layout, SideBar};
use crate::types::{Rbac, BionicToolDefinition};
use crate::console::{ChatWithChunks, PendingChatState};
use crate::console::console_stream::ConsoleStream;
use crate::console::chat_input::ChatInput;
use crate::console::tools_modal::ToolsModal;
// use crate::console::history_drawer::HistoryDrawer;
// use crate::console::model_popup::ModelPopup;

// 临时类型定义
#[derive(Clone, Debug, PartialEq)]
pub struct SinglePrompt {
    pub name: String,
    pub model_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Capability {
    pub name: String,
}

/// 增强的助手控制台组件
#[component]
pub fn EnhancedAssistantConsole(
    team_id: i32,
    conversation_id: Option<i64>,
    rbac: Rbac,
    chat_history: Vec<ChatWithChunks>,
    pending_chat_state: PendingChatState,
    prompt: SinglePrompt,
    selected_item: SideBar,
    title: String,
    header: Element,
    is_tts_disabled: bool,
    capabilities: Vec<Capability>,
    enabled_tools: Vec<String>,
    available_tools: Vec<BionicToolDefinition>,
) -> Element {
    let _has_pending_chat = !matches!(&pending_chat_state, PendingChatState::None);
    // 简化实现，移除use_signal依赖
    let show_tools_modal = false;
    let _show_history_drawer = false;
    let _show_model_popup = false;

    rsx! {
        Layout {
            section_class: "h-screen flex flex-col",
            selected_item,
            team_id,
            rbac: rbac.clone(),
            title: title.clone(),
            header,

            // 主控制台区域
            div {
                class: "flex-1 flex flex-col bg-base-100 relative",

                // 控制台头部工具栏
                ConsoleHeader {
                    prompt: prompt.clone(),
                    conversation_id,
                    enabled_tools: enabled_tools.clone(),
                    available_tools: available_tools.clone(),
                    capabilities: capabilities.clone(),
                    // 简化事件处理
                }

                // 聊天流区域
                div {
                    class: "flex-1 overflow-hidden",
                    ConsoleStream {
                        team_id,
                        chat_history: chat_history.clone(),
                        pending_chat_state: pending_chat_state.clone(),
                        is_tts_disabled,
                        rbac: rbac.clone()
                    }
                }

                // 输入表单区域
                ChatInput {
                    team_id,
                    conversation_id,
                    rbac: rbac.clone(),
                    disabled: _has_pending_chat
                }

                // 模态框和抽屉（简化实现）
                if show_tools_modal {
                    div {
                        class: "modal modal-open",
                        div {
                            class: "modal-box relative",
                            ToolsModal {
                                enabled_tools: enabled_tools.clone(),
                                available_tools: available_tools.clone()
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 控制台头部工具栏
#[component]
fn ConsoleHeader(
    prompt: SinglePrompt,
    conversation_id: Option<i64>,
    enabled_tools: Vec<String>,
    available_tools: Vec<BionicToolDefinition>,
    capabilities: Vec<Capability>,
) -> Element {
    rsx! {
        div {
            class: "flex items-center justify-between p-4 border-b border-base-300 bg-base-50",

            // 左侧：助手信息
            div {
                class: "flex items-center space-x-4",

                // 助手头像
                div {
                    class: "avatar",
                    div {
                        class: "w-12 h-12 rounded-full bg-primary flex items-center justify-center",
                        span {
                            class: "text-primary-content text-xl",
                            "🤖"
                        }
                    }
                }

                // 助手信息
                div {
                    h2 {
                        class: "text-lg font-semibold text-base-content",
                        "{prompt.name}"
                    }
                    div {
                        class: "flex items-center space-x-2 text-sm text-base-content/70",
                        span {
                            "模型: {prompt.model_name.as_ref().map(|s| s.as_str()).unwrap_or(\"默认\")}"
                        }
                        span { "•" }
                        span {
                            "工具: {enabled_tools.len()}/{available_tools.len()}"
                        }
                        if conversation_id.is_some() {
                            span { "•" }
                            span {
                                "对话ID: {conversation_id.unwrap()}"
                            }
                        }
                    }
                }
            }

            // 右侧：操作按钮
            div {
                class: "flex items-center space-x-2",

                // 工具管理按钮
                button {
                    class: "btn btn-ghost btn-sm gap-2",
                    title: "管理工具",
                    span { "🛠️" }
                    span { "工具" }
                    if !enabled_tools.is_empty() {
                        div {
                            class: "badge badge-primary badge-xs",
                            "{enabled_tools.len()}"
                        }
                    }
                }

                // 对话历史按钮
                button {
                    class: "btn btn-ghost btn-sm gap-2",
                    title: "对话历史",
                    span { "📋" }
                    span { "历史" }
                }

                // 模型选择按钮
                button {
                    class: "btn btn-ghost btn-sm gap-2",
                    title: "选择模型",
                    span { "🧠" }
                    span { "模型" }
                }

                // 设置按钮
                div {
                    class: "dropdown dropdown-end",
                    button {
                        class: "btn btn-ghost btn-sm",
                        title: "更多设置",
                        span { "⚙️" }
                    }
                    ul {
                        class: "dropdown-content menu p-2 shadow bg-base-100 rounded-box w-52",
                        li {
                            a {
                                href: "#",
                                "🎨 主题设置"
                            }
                        }
                        li {
                            a {
                                href: "#",
                                "🔊 语音设置"
                            }
                        }
                        li {
                            a {
                                href: "#",
                                "📤 导出对话"
                            }
                        }
                        li {
                            a {
                                href: "#",
                                "🗑️ 清空对话"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 控制台状态指示器
#[component]
fn ConsoleStatusIndicator(
    pending_state: PendingChatState,
    enabled_tools_count: usize,
    total_tools_count: usize,
) -> Element {
    rsx! {
        div {
            class: "flex items-center space-x-4 text-xs text-base-content/60",

            // 连接状态
            div {
                class: "flex items-center space-x-1",
                div {
                    class: "w-2 h-2 rounded-full bg-success animate-pulse"
                }
                span { "已连接" }
            }

            // 处理状态
            match pending_state {
                PendingChatState::None => rsx! {
                    div {
                        class: "flex items-center space-x-1",
                        div {
                            class: "w-2 h-2 rounded-full bg-base-300"
                        }
                        span { "就绪" }
                    }
                },
                _ => rsx! {
                    div {
                        class: "flex items-center space-x-1",
                        div {
                            class: "w-2 h-2 rounded-full bg-warning animate-pulse"
                        }
                        span { "处理中" }
                    }
                }
            }

            // 工具状态
            div {
                class: "flex items-center space-x-1",
                span { "工具: {enabled_tools_count}/{total_tools_count}" }
            }
        }
    }
}
