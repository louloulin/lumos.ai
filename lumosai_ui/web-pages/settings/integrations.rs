/*!
# Integrations Management

集成管理界面，参考bionic-gpt实现，提供第三方服务集成管理功能。

## 功能特性

- **集成列表**: 显示所有可用的第三方集成
- **配置管理**: 配置和管理集成设置
- **状态监控**: 监控集成状态和健康度
- **使用统计**: 集成使用情况统计
*/

#![allow(non_snake_case)]
use dioxus::prelude::*;
use crate::app_layout::{Layout, SideBar};
use crate::types::Rbac;

// 临时类型定义
#[derive(Clone, Debug, PartialEq)]
pub struct Integration {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub provider: String,
    pub category: String,
    pub icon: String,
    pub is_enabled: bool,
    pub is_configured: bool,
    pub last_sync: Option<String>,
    pub usage_count: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IntegrationCategory {
    pub name: String,
    pub description: String,
    pub integrations: Vec<Integration>,
}

/// 集成管理页面
#[component]
pub fn IntegrationsPage(
    team_id: i32,
    rbac: Rbac,
) -> Element {
    // 模拟数据
    let integration_categories = vec![
        IntegrationCategory {
            name: "AI Models".to_string(),
            description: "Connect to external AI model providers".to_string(),
            integrations: vec![
                Integration {
                    id: 1,
                    name: "OpenAI GPT".to_string(),
                    description: "Access GPT-4 and other OpenAI models".to_string(),
                    provider: "OpenAI".to_string(),
                    category: "AI Models".to_string(),
                    icon: "🤖".to_string(),
                    is_enabled: true,
                    is_configured: true,
                    last_sync: Some("2024-01-20 10:30".to_string()),
                    usage_count: 1250,
                },
                Integration {
                    id: 2,
                    name: "Claude".to_string(),
                    description: "Anthropic's Claude AI assistant".to_string(),
                    provider: "Anthropic".to_string(),
                    category: "AI Models".to_string(),
                    icon: "🧠".to_string(),
                    is_enabled: false,
                    is_configured: false,
                    last_sync: None,
                    usage_count: 0,
                },
            ],
        },
        IntegrationCategory {
            name: "Data Sources".to_string(),
            description: "Connect to external data sources".to_string(),
            integrations: vec![
                Integration {
                    id: 3,
                    name: "Google Drive".to_string(),
                    description: "Access files from Google Drive".to_string(),
                    provider: "Google".to_string(),
                    category: "Data Sources".to_string(),
                    icon: "📁".to_string(),
                    is_enabled: true,
                    is_configured: true,
                    last_sync: Some("2024-01-20 09:15".to_string()),
                    usage_count: 89,
                },
                Integration {
                    id: 4,
                    name: "Notion".to_string(),
                    description: "Connect to Notion workspace".to_string(),
                    provider: "Notion".to_string(),
                    category: "Data Sources".to_string(),
                    icon: "📝".to_string(),
                    is_enabled: false,
                    is_configured: false,
                    last_sync: None,
                    usage_count: 0,
                },
            ],
        },
        IntegrationCategory {
            name: "Communication".to_string(),
            description: "Connect to communication platforms".to_string(),
            integrations: vec![
                Integration {
                    id: 5,
                    name: "Slack".to_string(),
                    description: "Integrate with Slack workspace".to_string(),
                    provider: "Slack".to_string(),
                    category: "Communication".to_string(),
                    icon: "💬".to_string(),
                    is_enabled: true,
                    is_configured: true,
                    last_sync: Some("2024-01-20 08:45".to_string()),
                    usage_count: 456,
                },
                Integration {
                    id: 6,
                    name: "Discord".to_string(),
                    description: "Connect to Discord server".to_string(),
                    provider: "Discord".to_string(),
                    category: "Communication".to_string(),
                    icon: "🎮".to_string(),
                    is_enabled: false,
                    is_configured: false,
                    last_sync: None,
                    usage_count: 0,
                },
            ],
        },
    ];

    rsx! {
        Layout {
            section_class: "p-4",
            selected_item: SideBar::Integrations,
            team_id,
            rbac: rbac.clone(),
            title: "Integrations",
            header: rsx! {
                div {
                    class: "flex items-center justify-between",
                    h3 {
                        class: "text-2xl font-bold",
                        "🔗 Integrations"
                    }
                    div {
                        class: "flex gap-4",
                        if rbac.can_manage_integrations() {
                            button {
                                class: "btn btn-primary gap-2",
                                onclick: move |_| {
                                    // TODO: 打开添加集成模态框
                                },
                                span { "➕" }
                                "Add Integration"
                            }
                        }
                    }
                }
            },

            div {
                class: "space-y-8",

                // 集成概览
                IntegrationsOverview {
                    integration_categories: integration_categories.clone()
                }

                // 集成分类
                for category in &integration_categories {
                    IntegrationCategorySection {
                        category: category.clone(),
                        rbac: rbac.clone()
                    }
                }

                // 集成指南
                IntegrationGuide {}
            }
        }
    }
}

/// 集成概览组件
#[component]
fn IntegrationsOverview(integration_categories: Vec<IntegrationCategory>) -> Element {
    let total_integrations: usize = integration_categories.iter()
        .map(|cat| cat.integrations.len())
        .sum();
    
    let enabled_integrations: usize = integration_categories.iter()
        .flat_map(|cat| &cat.integrations)
        .filter(|int| int.is_enabled)
        .count();
    
    let total_usage: i64 = integration_categories.iter()
        .flat_map(|cat| &cat.integrations)
        .map(|int| int.usage_count)
        .sum();

    rsx! {
        div {
            class: "grid grid-cols-1 md:grid-cols-3 gap-6",

            // 总集成数
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-lg font-semibold text-base-content/80",
                                "Total Integrations"
                            }
                            p {
                                class: "text-3xl font-bold text-primary",
                                "{total_integrations}"
                            }
                        }
                        div {
                            class: "text-4xl",
                            "🔗"
                        }
                    }
                    p {
                        class: "text-sm text-base-content/60",
                        "Available integrations"
                    }
                }
            }

            // 已启用集成
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-lg font-semibold text-base-content/80",
                                "Active Integrations"
                            }
                            p {
                                class: "text-3xl font-bold text-success",
                                "{enabled_integrations}"
                            }
                        }
                        div {
                            class: "text-4xl",
                            "✅"
                        }
                    }
                    p {
                        class: "text-sm text-base-content/60",
                        "Currently enabled"
                    }
                }
            }

            // 总使用量
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-lg font-semibold text-base-content/80",
                                "Total Usage"
                            }
                            p {
                                class: "text-3xl font-bold text-secondary",
                                "{total_usage}"
                            }
                        }
                        div {
                            class: "text-4xl",
                            "📊"
                        }
                    }
                    p {
                        class: "text-sm text-base-content/60",
                        "API calls made"
                    }
                }
            }
        }
    }
}

/// 集成分类区块组件
#[component]
fn IntegrationCategorySection(
    category: IntegrationCategory,
    rbac: Rbac,
) -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-header p-6 border-b border-base-300",
                h4 {
                    class: "text-xl font-bold",
                    "{category.name}"
                }
                p {
                    class: "text-base-content/70 mt-1",
                    "{category.description}"
                }
            }
            div {
                class: "card-body",
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                    for integration in &category.integrations {
                        IntegrationCard {
                            integration: integration.clone(),
                            rbac: rbac.clone()
                        }
                    }
                }
            }
        }
    }
}

/// 集成卡片组件
#[component]
fn IntegrationCard(
    integration: Integration,
    rbac: Rbac,
) -> Element {
    rsx! {
        div {
            class: if integration.is_enabled {
                "card bg-base-100 border-2 border-success shadow-md"
            } else {
                "card bg-base-100 border border-base-300 shadow-md"
            },
            div {
                class: "card-body p-4",
                
                // 头部
                div {
                    class: "flex items-start justify-between mb-3",
                    div {
                        class: "flex items-center space-x-3",
                        div {
                            class: "text-3xl",
                            "{integration.icon}"
                        }
                        div {
                            h5 {
                                class: "font-bold",
                                "{integration.name}"
                            }
                            p {
                                class: "text-xs text-base-content/60",
                                "by {integration.provider}"
                            }
                        }
                    }
                    div {
                        if integration.is_enabled {
                            div {
                                class: "badge badge-success badge-sm",
                                "Enabled"
                            }
                        } else {
                            div {
                                class: "badge badge-neutral badge-sm",
                                "Disabled"
                            }
                        }
                    }
                }
                
                // 描述
                p {
                    class: "text-sm text-base-content/70 mb-4 line-clamp-2",
                    "{integration.description}"
                }
                
                // 统计信息
                if integration.is_enabled {
                    div {
                        class: "space-y-2 mb-4",
                        div {
                            class: "flex justify-between text-xs text-base-content/60",
                            span { "Usage:" }
                            span { "{integration.usage_count} calls" }
                        }
                        if let Some(last_sync) = &integration.last_sync {
                            div {
                                class: "flex justify-between text-xs text-base-content/60",
                                span { "Last sync:" }
                                span { "{last_sync}" }
                            }
                        }
                    }
                }
                
                // 操作按钮
                div {
                    class: "flex justify-between items-center",
                    if integration.is_enabled {
                        button {
                            class: "btn btn-outline btn-sm",
                            "Configure"
                        }
                    } else {
                        button {
                            class: "btn btn-primary btn-sm",
                            disabled: !rbac.can_manage_integrations(),
                            "Enable"
                        }
                    }
                    
                    if rbac.can_manage_integrations() {
                        div {
                            class: "dropdown dropdown-end",
                            button {
                                class: "btn btn-ghost btn-xs",
                                "⋮"
                            }
                            ul {
                                class: "dropdown-content menu p-2 shadow bg-base-100 rounded-box w-52",
                                li {
                                    a {
                                        "View Details"
                                    }
                                }
                                if integration.is_enabled {
                                    li {
                                        a {
                                            "Settings"
                                        }
                                    }
                                    li {
                                        a {
                                            class: "text-warning",
                                            "Disable"
                                        }
                                    }
                                } else {
                                    li {
                                        a {
                                            class: "text-success",
                                            "Enable"
                                        }
                                    }
                                }
                                li {
                                    a {
                                        class: "text-error",
                                        "Remove"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 集成指南组件
#[component]
fn IntegrationGuide() -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-body",
                h4 {
                    class: "text-xl font-bold mb-4",
                    "📚 Integration Guide"
                }
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                    
                    div {
                        h5 {
                            class: "font-semibold mb-2",
                            "🚀 Getting Started"
                        }
                        ul {
                            class: "space-y-1 text-sm text-base-content/70",
                            li { "• Choose the integration you want to enable" }
                            li { "• Follow the setup instructions" }
                            li { "• Configure the integration settings" }
                            li { "• Test the connection" }
                        }
                    }
                    
                    div {
                        h5 {
                            class: "font-semibold mb-2",
                            "🔒 Security & Privacy"
                        }
                        ul {
                            class: "space-y-1 text-sm text-base-content/70",
                            li { "• All integrations use secure OAuth 2.0" }
                            li { "• Your data is encrypted in transit" }
                            li { "• You can revoke access anytime" }
                            li { "• We follow industry best practices" }
                        }
                    }
                }
            }
        }
    }
}
