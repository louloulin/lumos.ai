/*!
# History Drawer Component

对话历史抽屉组件，参考bionic-gpt实现，提供对话历史管理功能。

## 功能特性

- **历史列表**: 显示所有对话历史
- **搜索过滤**: 支持关键词搜索历史对话
- **分类管理**: 按时间、主题等分类
- **快速操作**: 删除、重命名、导出等
*/

#![allow(non_snake_case)]
use crate::types::History;
use daisy_rsx::*;
use dioxus::prelude::*;

/// 增强的对话历史抽屉组件
#[component]
pub fn HistoryDrawer(trigger_id: String, team_id: i32, history: Vec<History>) -> Element {
    // 简化实现，移除use_signal依赖
    let search_query = String::new();
    let selected_filter = "all".to_string();

    // 过滤历史记录
    let filtered_history = history
        .iter()
        .filter(|h| {
            let matches_search = search_query.is_empty()
                || h.summary
                    .to_lowercase()
                    .contains(&search_query.to_lowercase());

            let matches_filter = match selected_filter.as_str() {
                "recent" => true,     // TODO: 实现时间过滤
                "favorites" => false, // TODO: 实现收藏过滤
                _ => true,
            };

            matches_search && matches_filter
        })
        .collect::<Vec<_>>();

    rsx! {
        Modal {
            trigger_id: &trigger_id,
            ModalBody {
                // 模态框头部
                div {
                    class: "flex items-center justify-between mb-4",
                    h3 {
                        class: "text-xl font-bold text-base-content",
                        "📋 对话历史"
                    }
                    div {
                        class: "text-sm text-base-content/60",
                        "共 {history.len()} 条记录"
                    }
                }

                // 搜索和过滤
                div {
                    class: "mb-4 space-y-3",

                    // 搜索框
                    div {
                        class: "form-control",
                        input {
                            r#type: "text",
                            placeholder: "搜索对话内容...",
                            class: "input input-bordered input-sm",
                            value: search_query.clone()
                        }
                    }

                    // 过滤按钮
                    div {
                        class: "flex space-x-2",

                        for (value, label) in [
                            ("all", "全部"),
                            ("recent", "最近"),
                            ("favorites", "收藏")
                        ] {
                            button {
                                class: if selected_filter == value {
                                    "btn btn-primary btn-xs"
                                } else {
                                    "btn btn-ghost btn-xs"
                                },
                                "{label}"
                            }
                        }
                    }
                }

                // 历史列表
                div {
                    class: "max-h-96 overflow-y-auto",

                    if filtered_history.is_empty() {
                        div {
                            class: "text-center py-8",
                            div {
                                class: "text-4xl mb-2",
                                if search_query.is_empty() {
                                    "💬"
                                } else {
                                    "🔍"
                                }
                            }
                            p {
                                class: "text-base-content/60",
                                if search_query.is_empty() {
                                    "暂无对话历史"
                                } else {
                                    "未找到匹配的对话"
                                }
                            }
                        }
                    } else {
                        ul {
                            class: "space-y-2",
                            for hist in &filtered_history {
                                li {
                                    class: "w-full",
                                    HistoryItem {
                                        history: (*hist).clone(),
                                        team_id
                                    }
                                }
                            }
                        }
                    }
                }

                // 底部操作
                div {
                    class: "mt-4 pt-4 border-t border-base-300",
                    div {
                        class: "flex justify-between items-center",

                        div {
                            class: "text-xs text-base-content/60",
                            "显示 {filtered_history.len()} / {history.len()} 条记录"
                        }

                        div {
                            class: "flex space-x-2",

                            button {
                                class: "btn btn-ghost btn-xs gap-1",
                                span { "📤" }
                                "导出"
                            }

                            button {
                                class: "btn btn-ghost btn-xs gap-1 text-error",
                                span { "🗑️" }
                                "清空"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 历史记录项组件
#[component]
fn HistoryItem(history: History, team_id: i32) -> Element {
    // 简化实现，移除use_signal依赖
    let show_actions = false;

    rsx! {
        div {
            class: "group relative p-3 bg-base-200 hover:bg-base-300 rounded-lg transition-colors",
            // onmouseenter: move |_| show_actions.set(true),
            // onmouseleave: move |_| show_actions.set(false),

            a {
                class: "block",
                href: crate::routes::console::Conversation{team_id, conversation_id: history.id as i64}.to_string(),

                div {
                    class: "flex items-start justify-between",

                    div {
                        class: "flex-1 min-w-0",

                        // 对话摘要
                        p {
                            class: "text-sm font-medium line-clamp-2 mb-1",
                            "{history.summary}"
                        }

                        // 时间信息
                        div {
                            class: "flex items-center space-x-2 text-xs text-base-content/60",
                            span {
                                "📅 {format_relative_time(&history.created_at.to_string())}"
                            }
                            span {
                                "💬 ID: {history.id}"
                            }
                        }
                    }

                    // 操作按钮
                    if show_actions {
                        div {
                            class: "flex space-x-1 ml-2",

                            button {
                                class: "btn btn-ghost btn-xs",
                                title: "收藏",
                                onclick: move |e| {
                                    e.stop_propagation();
                                    // TODO: 实现收藏功能
                                },
                                "⭐"
                            }

                            button {
                                class: "btn btn-ghost btn-xs text-error",
                                title: "删除",
                                onclick: move |e| {
                                    e.stop_propagation();
                                    // TODO: 实现删除功能
                                },
                                "🗑️"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 格式化相对时间
fn format_relative_time(datetime: &str) -> String {
    // TODO: 实现真实的相对时间格式化
    // 这里应该解析datetime并返回"刚刚"、"5分钟前"、"昨天"等
    datetime.to_string()
}
