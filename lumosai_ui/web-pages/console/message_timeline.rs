/*!
# Message Timeline Component

消息时间线组件，提供对话历史管理功能。

## 功能特性

- **对话列表**: 显示所有历史对话
- **搜索过滤**: 按内容搜索对话
- **分组显示**: 按时间分组显示
- **快速操作**: 删除、重命名等操作
*/

use dioxus::prelude::*;

/// 消息时间线组件
#[component]
pub fn MessageTimeline() -> Element {
    
    rsx! {
        div {
            class: "flex flex-col h-full bg-base-100",

            // 头部
            div {
                class: "p-4 border-b border-base-300",

                h2 {
                    class: "text-lg font-semibold mb-3",
                    "对话历史"
                }

                // 搜索框
                div {
                    class: "relative",
                    input {
                        class: "input input-bordered w-full pl-10",
                        placeholder: "搜索对话...",
                        r#type: "text"
                    }
                    div {
                        class: "absolute left-3 top-1/2 transform -translate-y-1/2 text-base-content/50",
                        "🔍"
                    }
                }
            }

            // 对话列表
            div {
                class: "flex-1 overflow-y-auto",

                EmptyTimelineState {
                    has_search: false
                }
            }

            // 底部操作
            div {
                class: "p-4 border-t border-base-300",

                button {
                    class: "btn btn-primary w-full",
                    onclick: move |_| {
                        // TODO: 创建新对话
                    },
                    "➕ 新建对话"
                }
            }
        }
    }
}



/// 空时间线状态组件
#[component]
fn EmptyTimelineState(has_search: bool) -> Element {
    rsx! {
        div {
            class: "flex flex-col items-center justify-center h-full p-8 text-center",
            
            div {
                class: "w-16 h-16 bg-base-300 rounded-full flex items-center justify-center mb-4",
                span {
                    class: "text-2xl",
                    if has_search { "🔍" } else { "💬" }
                }
            }
            
            h3 {
                class: "text-lg font-semibold mb-2",
                if has_search {
                    "未找到匹配的对话"
                } else {
                    "暂无对话历史"
                }
            }
            
            p {
                class: "text-base-content/60 text-sm",
                if has_search {
                    "尝试使用不同的关键词搜索"
                } else {
                    "开始新的对话来创建历史记录"
                }
            }
        }
    }
}


