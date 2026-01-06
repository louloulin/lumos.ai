/*!
# Tools Modal Component

工具选择和配置模态框组件，参考bionic-gpt实现。

## 功能特性

- **工具选择**: 启用/禁用可用工具
- **工具配置**: 配置工具参数
- **权限控制**: 基于用户权限显示工具
- **实时预览**: 工具功能预览和测试
*/

#![allow(non_snake_case)]
use crate::routes;
use crate::types::BionicToolDefinition;
use daisy_rsx::*;
use dioxus::prelude::*;

/// 工具模态框组件
#[component]
pub fn ToolsModal(
    enabled_tools: Vec<String>,
    available_tools: Vec<BionicToolDefinition>,
) -> Element {
    // 简化实现，移除use_signal依赖
    let selected_tools = enabled_tools.clone();
    let show_tool_details: Option<String> = None;

    rsx! {
        form {
            action: routes::console::SetTools{}.to_string(),
            method: "post",
            Modal {
                trigger_id: "tool-modal",
                ModalBody {
                    // 模态框头部
                    div {
                        class: "flex items-center justify-between mb-6",
                        h3 {
                            class: "text-2xl font-bold text-base-content",
                            "🛠️ AI工具管理"
                        }
                        div {
                            class: "text-sm text-base-content/60",
                            "选择和配置AI助手可用的工具"
                        }
                    }

                    // 工具统计
                    div {
                        class: "stats stats-horizontal shadow mb-6 w-full",

                        div {
                            class: "stat",
                            div {
                                class: "stat-title",
                                "可用工具"
                            }
                            div {
                                class: "stat-value text-primary text-2xl",
                                "{available_tools.len()}"
                            }
                        }

                        div {
                            class: "stat",
                            div {
                                class: "stat-title",
                                "已启用"
                            }
                            div {
                                class: "stat-value text-success text-2xl",
                                "{selected_tools.len()}"
                            }
                        }

                        div {
                            class: "stat",
                            div {
                                class: "stat-title",
                                "状态"
                            }
                            div {
                                class: "stat-value text-sm",
                                if selected_tools.is_empty() {
                                    "🔴 无工具"
                                } else {
                                    "🟢 已配置"
                                }
                            }
                        }
                    }

                    // 工具列表
                    div {
                        class: "space-y-3 mb-6 max-h-96 overflow-y-auto",

                        for tool in &available_tools {
                            ToolCard {
                                tool: tool.clone(),
                                is_enabled: selected_tools.contains(&tool.function.name)
                            }
                        }
                    }

                    // 工具详情面板
                    if let Some(ref tool_name) = show_tool_details {
                        if let Some(tool) = available_tools.iter().find(|t| &t.function.name == tool_name) {
                            ToolDetailsPanel {
                                tool: tool.clone()
                            }
                        }
                    }

                    ModalAction {
                        button {
                            class: "btn btn-ghost",
                            r#type: "button",
                            "重置"
                        }
                        Button {
                            button_type: ButtonType::Submit,
                            button_scheme: ButtonScheme::Primary,
                            "保存配置"
                        }
                    }
                }
            }
        }
    }
}

/// 工具卡片组件
#[component]
fn ToolCard(tool: BionicToolDefinition, is_enabled: bool) -> Element {
    rsx! {
        div {
            class: "card bg-base-200 shadow-sm border border-base-300 hover:shadow-md transition-all duration-200",

            div {
                class: "card-body p-4",

                // 工具头部
                div {
                    class: "flex items-start justify-between mb-3",

                    div {
                        class: "flex items-center space-x-3",
                        span {
                            class: "text-2xl",
                            "{get_tool_icon(&tool.function.name)}"
                        }
                        div {
                            h4 {
                                class: "font-semibold text-base-content",
                                "{tool.function.name}"
                            }
                            p {
                                class: "text-xs text-base-content/60",
                                "{get_tool_category(&tool.function.name)}"
                            }
                        }
                    }

                    input {
                        r#type: "checkbox",
                        name: "tools",
                        value: "{tool.function.name}",
                        class: "toggle toggle-primary",
                        checked: is_enabled
                    }
                }

                // 工具描述
                p {
                    class: "text-sm text-base-content/80 mb-3 line-clamp-2",
                    "{tool.function.description.as_deref().unwrap_or(\"暂无描述\")}"
                }

                // 工具参数信息
                if let Some(ref _params) = tool.function.parameters {
                    div {
                        class: "mb-3",
                        div {
                            class: "text-xs text-base-content/60 mb-1",
                            "参数配置："
                        }
                        div {
                            class: "flex flex-wrap gap-1",
                            span {
                                class: "badge badge-outline badge-xs",
                                "可配置"
                            }
                        }
                    }
                }

                // 操作按钮
                div {
                    class: "flex justify-between items-center",

                    div {
                        class: "flex space-x-2",
                        button {
                            class: "btn btn-ghost btn-xs",
                            r#type: "button",
                            "详情"
                        }
                        if is_enabled {
                            button {
                                class: "btn btn-ghost btn-xs text-primary",
                                r#type: "button",
                                "测试"
                            }
                        }
                    }

                    div {
                        class: "badge badge-sm",
                        class: if is_enabled { "badge-success" } else { "badge-ghost" },
                        if is_enabled { "✅ 已启用" } else { "⭕ 未启用" }
                    }
                }
            }
        }
    }
}

/// 工具详情面板组件
#[component]
fn ToolDetailsPanel(tool: BionicToolDefinition) -> Element {
    rsx! {
        div {
            class: "alert alert-info mb-4",

            div {
                class: "flex items-start justify-between w-full",

                div {
                    class: "flex-1",
                    h4 {
                        class: "font-bold text-lg mb-2",
                        "🔍 {tool.function.name} 详情"
                    }

                    p {
                        class: "text-sm mb-3",
                        "{tool.function.description.as_deref().unwrap_or(\"暂无详细描述\")}"
                    }

                    if let Some(ref _params) = tool.function.parameters {
                        div {
                            class: "space-y-2",
                            h5 {
                                class: "font-semibold text-sm",
                                "参数配置："
                            }
                            div {
                                class: "bg-base-100 p-3 rounded-lg",
                                p {
                                    class: "text-xs text-base-content/70",
                                    "此工具支持参数配置，可以根据需要调整工具行为。"
                                }
                            }
                        }
                    }
                }

                button {
                    class: "btn btn-ghost btn-sm",
                    r#type: "button",
                    "✕"
                }
            }
        }
    }
}

/// 获取工具图标
fn get_tool_icon(tool_name: &str) -> &'static str {
    match tool_name {
        "calculator" => "🧮",
        "current_time" => "⏰",
        "system_info" => "💻",
        "file_search" => "🔍",
        "web_search" => "🌐",
        "code_executor" => "⚡",
        "image_generator" => "🎨",
        "translator" => "🌍",
        _ => "🛠️",
    }
}

/// 获取工具分类
fn get_tool_category(tool_name: &str) -> &'static str {
    match tool_name {
        "calculator" => "数学计算",
        "current_time" => "时间工具",
        "system_info" => "系统工具",
        "file_search" => "文件工具",
        "web_search" => "网络工具",
        "code_executor" => "代码工具",
        "image_generator" => "创意工具",
        "translator" => "语言工具",
        _ => "通用工具",
    }
}

/// 测试工具功能
fn test_tool(tool_name: &str) {
    // TODO: 实现工具测试功能
    println!("测试工具: {}", tool_name);
}
