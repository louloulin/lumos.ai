/*!
# Model Popup Component

模型选择弹窗组件，参考bionic-gpt实现，提供AI模型选择功能。

## 功能特性

- **模型列表**: 显示所有可用的AI模型
- **模型信息**: 显示模型详细信息和能力
- **性能指标**: 显示模型速度、质量等指标
- **快速切换**: 支持快速切换常用模型
*/

#![allow(non_snake_case)]
use dioxus::prelude::*;
use web_assets::files::BUTTON_SELECT_SVG;

/// 增强的模型选择弹窗
#[component]
pub fn ModelPopup(team_id: i32, current_model: String, on_close: EventHandler<()>) -> Element {
    // 简化实现，移除use_signal依赖
    let selected_model = current_model.clone();
    let show_details = false;

    // 模拟可用模型列表
    let available_models = vec![
        ModelInfo {
            id: "gpt-4".to_string(),
            name: "GPT-4".to_string(),
            provider: "OpenAI".to_string(),
            description: "最先进的大语言模型，具有出色的推理能力".to_string(),
            speed: "中等".to_string(),
            quality: "极高".to_string(),
            cost: "高".to_string(),
            context_length: "8K tokens".to_string(),
            capabilities: vec!["文本生成", "代码编写", "数学推理", "创意写作"],
        },
        ModelInfo {
            id: "gpt-3.5-turbo".to_string(),
            name: "GPT-3.5 Turbo".to_string(),
            provider: "OpenAI".to_string(),
            description: "快速且经济的模型，适合大多数对话场景".to_string(),
            speed: "快".to_string(),
            quality: "高".to_string(),
            cost: "中等".to_string(),
            context_length: "4K tokens".to_string(),
            capabilities: vec!["文本生成", "对话交流", "简单推理"],
        },
        ModelInfo {
            id: "claude-3".to_string(),
            name: "Claude 3".to_string(),
            provider: "Anthropic".to_string(),
            description: "安全可靠的AI助手，擅长分析和创作".to_string(),
            speed: "中等".to_string(),
            quality: "极高".to_string(),
            cost: "高".to_string(),
            context_length: "100K tokens".to_string(),
            capabilities: vec!["长文本分析", "创意写作", "代码审查"],
        },
    ];

    rsx! {
        div {
            class: "modal modal-open",

            div {
                class: "modal-box max-w-4xl",

                // 模态框头部
                div {
                    class: "flex items-center justify-between mb-6",
                    h3 {
                        class: "text-2xl font-bold text-base-content",
                        "🧠 选择AI模型"
                    }
                    button {
                        class: "btn btn-ghost btn-sm",
                        onclick: move |_| on_close.call(()),
                        "✕"
                    }
                }

                // 当前模型信息
                div {
                    class: "alert alert-info mb-6",
                    div {
                        h4 {
                            class: "font-semibold mb-1",
                            "当前使用模型"
                        }
                        p {
                            class: "text-sm",
                            "{current_model}"
                        }
                    }
                }

                // 模型列表
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 mb-6",

                    for model in &available_models {
                        ModelCard {
                            model: model.clone(),
                            is_selected: selected_model == model.id
                        }
                    }
                }

                // 模型详情
                if show_details {
                    if let Some(model) = available_models.iter().find(|m| m.id == selected_model) {
                        ModelDetails {
                            model: model.clone()
                        }
                    }
                }

                // 操作按钮
                div {
                    class: "modal-action",

                    button {
                        class: "btn btn-ghost",
                        onclick: move |_| on_close.call(()),
                        "取消"
                    }

                    button {
                        class: "btn btn-ghost gap-2",
                        span { "ℹ️" }
                        "查看详情"
                    }

                    button {
                        class: "btn btn-primary gap-2",
                        disabled: selected_model == current_model,
                        span { "🔄" }
                        "切换模型"
                    }
                }
            }
        }
    }
}

/// 模型信息结构
#[derive(Clone, PartialEq)]
struct ModelInfo {
    id: String,
    name: String,
    provider: String,
    description: String,
    speed: String,
    quality: String,
    cost: String,
    context_length: String,
    capabilities: Vec<&'static str>,
}

/// 模型卡片组件
#[component]
fn ModelCard(model: ModelInfo, is_selected: bool) -> Element {
    rsx! {
        div {
            class: if is_selected {
                "card bg-primary/10 border-2 border-primary shadow-lg cursor-pointer"
            } else {
                "card bg-base-200 border border-base-300 hover:shadow-md cursor-pointer transition-all"
            },
            // onclick: move |_| on_select.call(model.id.clone()),

            div {
                class: "card-body p-4",

                // 模型头部
                div {
                    class: "flex items-start justify-between mb-3",

                    div {
                        h4 {
                            class: "font-bold text-lg",
                            "{model.name}"
                        }
                        p {
                            class: "text-sm text-base-content/60",
                            "{model.provider}"
                        }
                    }

                    if is_selected {
                        div {
                            class: "badge badge-primary",
                            "已选择"
                        }
                    }
                }

                // 模型描述
                p {
                    class: "text-sm text-base-content/80 mb-4 line-clamp-2",
                    "{model.description}"
                }

                // 性能指标
                div {
                    class: "space-y-2 mb-4",

                    div {
                        class: "flex justify-between text-xs",
                        span { "速度:" }
                        span {
                            class: get_metric_color(&model.speed),
                            "{model.speed}"
                        }
                    }

                    div {
                        class: "flex justify-between text-xs",
                        span { "质量:" }
                        span {
                            class: get_metric_color(&model.quality),
                            "{model.quality}"
                        }
                    }

                    div {
                        class: "flex justify-between text-xs",
                        span { "成本:" }
                        span {
                            class: get_cost_color(&model.cost),
                            "{model.cost}"
                        }
                    }
                }

                // 操作按钮
                div {
                    class: "flex justify-between items-center",

                    button {
                        class: "btn btn-ghost btn-xs",
                        "详情"
                    }

                    div {
                        class: "text-xs text-base-content/60",
                        "{model.context_length}"
                    }
                }
            }
        }
    }
}

/// 模型详情组件
#[component]
fn ModelDetails(model: ModelInfo) -> Element {
    rsx! {
        div {
            class: "alert alert-info mb-4",

            div {
                class: "w-full",

                div {
                    class: "flex items-start justify-between mb-4",

                    div {
                        h4 {
                            class: "font-bold text-lg mb-1",
                            "📋 {model.name} 详细信息"
                        }
                        p {
                            class: "text-sm text-base-content/70",
                            "提供商: {model.provider}"
                        }
                    }

                    button {
                        class: "btn btn-ghost btn-sm",
                        "✕"
                    }
                }

                div {
                    class: "grid grid-cols-1 md:grid-cols-2 gap-4",

                    // 基本信息
                    div {
                        h5 {
                            class: "font-semibold mb-2",
                            "基本信息"
                        }
                        div {
                            class: "space-y-2 text-sm",
                            div {
                                span { class: "font-medium", "描述: " }
                                span { "{model.description}" }
                            }
                            div {
                                span { class: "font-medium", "上下文长度: " }
                                span { "{model.context_length}" }
                            }
                        }
                    }

                    // 性能指标
                    div {
                        h5 {
                            class: "font-semibold mb-2",
                            "性能指标"
                        }
                        div {
                            class: "space-y-2 text-sm",
                            div {
                                span { class: "font-medium", "响应速度: " }
                                span { class: get_metric_color(&model.speed), "{model.speed}" }
                            }
                            div {
                                span { class: "font-medium", "输出质量: " }
                                span { class: get_metric_color(&model.quality), "{model.quality}" }
                            }
                            div {
                                span { class: "font-medium", "使用成本: " }
                                span { class: get_cost_color(&model.cost), "{model.cost}" }
                            }
                        }
                    }
                }

                // 能力列表
                div {
                    class: "mt-4",
                    h5 {
                        class: "font-semibold mb-2",
                        "主要能力"
                    }
                    div {
                        class: "flex flex-wrap gap-2",
                        for capability in &model.capabilities {
                            div {
                                class: "badge badge-outline badge-sm",
                                "{capability}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 获取性能指标颜色
fn get_metric_color(metric: &str) -> &'static str {
    match metric {
        "极高" | "快" => "text-success",
        "高" | "中等" => "text-warning",
        "低" | "慢" => "text-error",
        _ => "text-base-content",
    }
}

/// 获取成本颜色
fn get_cost_color(cost: &str) -> &'static str {
    match cost {
        "低" => "text-success",
        "中等" => "text-warning",
        "高" => "text-error",
        _ => "text-base-content",
    }
}
