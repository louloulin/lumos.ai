/*!
# Enhanced Assistants Management

增强的AI助手管理界面，参考bionic-gpt实现。

## 功能特性

- **助手网格视图**: 现代化的助手卡片展示
- **智能搜索**: 支持名称、描述、标签搜索
- **分类过滤**: 按可见性、类型、状态过滤
- **批量操作**: 支持批量启用/禁用、删除等
- **性能监控**: 助手使用统计和性能指标
- **模板系统**: 预设的助手模板快速创建
*/

#![allow(non_snake_case)]
use dioxus::prelude::*;
use crate::app_layout::{Layout, SideBar};
use crate::types::{Rbac, BionicToolDefinition};

// 临时类型定义
#[derive(Clone, Debug)]
pub struct Prompt {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub visibility: String,
    pub category_id: Option<i32>,
}

#[derive(Clone, Debug)]
pub struct Category {
    pub id: i32,
    pub name: String,
}

/// 增强的助手管理页面
#[component]
pub fn EnhancedAssistants(
    team_id: i32,
    rbac: Rbac,
    prompts: Vec<Prompt>,
    categories: Vec<Category>,
    available_tools: Vec<BionicToolDefinition>,
) -> Element {
    // 简化实现，移除use_signal依赖
    let search_query = String::new();
    let filter_category = -1i32; // -1 表示全部
    let filter_visibility = "all".to_string();
    let view_mode = "grid".to_string(); // grid 或 list
    let selected_assistants = Vec::<i32>::new();
    let show_create_modal = false;
    let show_templates = false;
    let show_stats = false;

    // 过滤助手列表
    let filtered_prompts = prompts.iter()
        .filter(|prompt| {
            let matches_search = search_query.is_empty() ||
                prompt.name.to_lowercase().contains(&search_query.to_lowercase()) ||
                prompt.description.as_ref().map_or(false, |desc|
                    desc.to_lowercase().contains(&search_query.to_lowercase()));

            let matches_category = filter_category == -1 ||
                prompt.category_id == Some(filter_category);

            let matches_visibility = match filter_visibility.as_str() {
                "public" => prompt.visibility == "Public",
                "private" => prompt.visibility == "Private",
                _ => true
            };

            matches_search && matches_category && matches_visibility
        })
        .collect::<Vec<_>>();

    rsx! {
        Layout {
            section_class: "p-6 bg-base-50 min-h-screen",
            selected_item: SideBar::Prompts,
            team_id,
            rbac: rbac.clone(),
            title: "AI助手管理".to_string(),
            header: rsx! {
                div {
                    class: "flex items-center justify-between",
                    div {
                        h1 {
                            class: "text-3xl font-bold text-base-content",
                            "🤖 AI助手管理"
                        }
                        p {
                            class: "text-base-content/60 mt-1",
                            "创建、管理和优化您的AI助手"
                        }
                    }
                    div {
                        class: "flex items-center space-x-2",
                        button {
                            class: "btn btn-ghost btn-sm",
                            "📊 统计"
                        }
                        if rbac.can_create_assistant() {
                            button {
                                class: "btn btn-primary gap-2",
                                span { "➕" }
                                "创建助手"
                            }
                        }
                    }
                }
            },

            div {
                class: "space-y-6",

                // 统计面板
                AssistantStatsPanel {
                    prompts: prompts.clone(),
                    categories: categories.clone(),
                    available_tools: available_tools.clone()
                }

                // 工具栏
                div {
                    class: "bg-base-100 rounded-lg shadow-sm border border-base-300 p-4",

                    div {
                        class: "flex flex-col lg:flex-row justify-between items-start lg:items-center gap-4",

                        // 左侧：操作按钮
                        div {
                            class: "flex flex-wrap gap-2",

                            button {
                                class: "btn btn-ghost btn-sm gap-2",
                                span { "📋" }
                                "模板"
                            }

                            button {
                                class: "btn btn-ghost btn-sm gap-2",
                                span { "📥" }
                                "导入"
                            }

                            button {
                                class: "btn btn-ghost btn-sm gap-2",
                                span { "📤" }
                                "导出"
                            }

                            if !selected_assistants.is_empty() {
                                div {
                                    class: "divider divider-horizontal"
                                }

                                button {
                                    class: "btn btn-warning btn-sm gap-2",
                                    span { "🗑️" }
                                    "删除选中 ({selected_assistants.len()})"
                                }
                            }
                        }

                        // 右侧：搜索和过滤
                        div {
                            class: "flex flex-wrap gap-2 items-center",

                            // 分类过滤
                            select {
                                class: "select select-bordered select-sm",
                                value: filter_category.to_string(),
                                option { value: "-1", "全部分类" }
                                for category in &categories {
                                    option {
                                        value: "{category.id}",
                                        "{category.name}"
                                    }
                                }
                            }

                            // 可见性过滤
                            select {
                                class: "select select-bordered select-sm",
                                value: filter_visibility.clone(),
                                option { value: "all", "全部助手" }
                                option { value: "public", "公开助手" }
                                option { value: "private", "私有助手" }
                            }

                            // 搜索框
                            div {
                                class: "form-control",
                                input {
                                    r#type: "text",
                                    placeholder: "搜索助手...",
                                    class: "input input-bordered input-sm w-64",
                                    value: search_query.clone()
                                }
                            }

                            // 视图切换
                            div {
                                class: "btn-group",
                                button {
                                    class: if view_mode == "grid" { "btn btn-sm btn-active" } else { "btn btn-sm" },
                                    "⊞"
                                }
                                button {
                                    class: if view_mode == "list" { "btn btn-sm btn-active" } else { "btn btn-sm" },
                                    "☰"
                                }
                            }
                        }
                    }
                }

                // 助手列表/网格
                if filtered_prompts.is_empty() {
                    EmptyAssistantsState {
                        has_assistants: !prompts.is_empty(),
                        search_query: search_query.clone()
                    }
                } else {
                    match view_mode.as_str() {
                        "list" => rsx! {
                            AssistantListView {
                                prompts: filtered_prompts,
                                team_id,
                                rbac: rbac.clone(),
                                selected_assistants: selected_assistants.clone()
                            }
                        },
                        _ => rsx! {
                            AssistantGridView {
                                prompts: filtered_prompts,
                                team_id,
                                rbac: rbac.clone(),
                                available_tools: available_tools.clone(),
                                selected_assistants: selected_assistants.clone()
                            }
                        }
                    }
                }

                // 模态框（简化实现）
                if show_create_modal {
                    div {
                        class: "modal modal-open",
                        div {
                            class: "modal-box",
                            "创建助手模态框"
                        }
                    }
                }

                if show_templates {
                    div {
                        class: "modal modal-open",
                        div {
                            class: "modal-box",
                            "助手模板模态框"
                        }
                    }
                }

                if show_stats {
                    div {
                        class: "modal modal-open",
                        div {
                            class: "modal-box",
                            "统计模态框"
                        }
                    }
                }
            }
        }
    }
}

/// 助手统计面板
#[component]
fn AssistantStatsPanel(
    prompts: Vec<Prompt>,
    categories: Vec<Category>,
    available_tools: Vec<BionicToolDefinition>,
) -> Element {
    let total_assistants = prompts.len();
    let public_assistants = prompts.iter().filter(|p| p.visibility == "Public").count();
    let private_assistants = prompts.iter().filter(|p| p.visibility == "Private").count();
    let active_categories = categories.len();

    rsx! {
        div {
            class: "stats stats-horizontal shadow-sm bg-base-100 border border-base-300 w-full",

            div {
                class: "stat",
                div {
                    class: "stat-figure text-primary",
                    span {
                        class: "text-3xl",
                        "🤖"
                    }
                }
                div {
                    class: "stat-title",
                    "总助手数"
                }
                div {
                    class: "stat-value text-primary",
                    "{total_assistants}"
                }
                div {
                    class: "stat-desc",
                    "已创建的AI助手"
                }
            }

            div {
                class: "stat",
                div {
                    class: "stat-figure text-success",
                    span {
                        class: "text-3xl",
                        "🌐"
                    }
                }
                div {
                    class: "stat-title",
                    "公开助手"
                }
                div {
                    class: "stat-value text-success",
                    "{public_assistants}"
                }
                div {
                    class: "stat-desc",
                    "可供团队使用"
                }
            }

            div {
                class: "stat",
                div {
                    class: "stat-figure text-warning",
                    span {
                        class: "text-3xl",
                        "🔒"
                    }
                }
                div {
                    class: "stat-title",
                    "私有助手"
                }
                div {
                    class: "stat-value text-warning",
                    "{private_assistants}"
                }
                div {
                    class: "stat-desc",
                    "仅个人使用"
                }
            }

            div {
                class: "stat",
                div {
                    class: "stat-figure text-info",
                    span {
                        class: "text-3xl",
                        "🛠️"
                    }
                }
                div {
                    class: "stat-title",
                    "可用工具"
                }
                div {
                    class: "stat-value text-info",
                    "{available_tools.len()}"
                }
                div {
                    class: "stat-desc",
                    "AI工具集成"
                }
            }
        }
    }
}
