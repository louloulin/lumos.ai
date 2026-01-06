/*!
# API Keys Management

API密钥管理界面，参考bionic-gpt实现，提供API密钥的创建、管理和监控功能。

## 功能特性

- **密钥管理**: 创建、查看、删除API密钥
- **使用统计**: 显示API调用次数和令牌使用量
- **权限控制**: 基于RBAC的权限管理
- **安全展示**: 安全的密钥显示和复制功能
*/

#![allow(non_snake_case)]
use crate::app_layout::{Layout, SideBar};
use crate::types::Rbac;
use dioxus::prelude::*;

// 临时类型定义
#[derive(Clone, Debug, PartialEq)]
pub struct ApiKey {
    pub id: i32,
    pub name: String,
    pub key_preview: String,
    pub created_at: String,
    pub last_used: Option<String>,
    pub usage_count: i64,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TokenUsage {
    pub date: String,
    pub tokens_used: i64,
    pub requests_count: i64,
}

/// API密钥管理页面
#[component]
pub fn ApiKeysPage(team_id: i32, rbac: Rbac) -> Element {
    // 模拟数据
    let api_keys = vec![
        ApiKey {
            id: 1,
            name: "Production API Key".to_string(),
            key_preview: "lm_*********************abc123".to_string(),
            created_at: "2024-01-15".to_string(),
            last_used: Some("2024-01-20 10:30".to_string()),
            usage_count: 1250,
            is_active: true,
        },
        ApiKey {
            id: 2,
            name: "Development API Key".to_string(),
            key_preview: "lm_*********************def456".to_string(),
            created_at: "2024-01-10".to_string(),
            last_used: Some("2024-01-19 15:45".to_string()),
            usage_count: 89,
            is_active: true,
        },
        ApiKey {
            id: 3,
            name: "Test API Key".to_string(),
            key_preview: "lm_*********************ghi789".to_string(),
            created_at: "2024-01-05".to_string(),
            last_used: None,
            usage_count: 0,
            is_active: false,
        },
    ];

    let token_usage = vec![
        TokenUsage {
            date: "2024-01-20".to_string(),
            tokens_used: 15420,
            requests_count: 89,
        },
        TokenUsage {
            date: "2024-01-19".to_string(),
            tokens_used: 12350,
            requests_count: 67,
        },
        TokenUsage {
            date: "2024-01-18".to_string(),
            tokens_used: 18900,
            requests_count: 102,
        },
    ];

    rsx! {
        Layout {
            section_class: "p-4",
            selected_item: SideBar::ApiKeys,
            team_id,
            rbac: rbac.clone(),
            title: "API Keys",
            header: rsx! {
                div {
                    class: "flex items-center justify-between",
                    h3 {
                        class: "text-2xl font-bold",
                        "🔑 API Keys"
                    }
                    div {
                        class: "flex gap-4",
                        if rbac.can_use_api_keys() {
                            button {
                                class: "btn btn-primary gap-2",
                                onclick: move |_| {
                                    // TODO: 打开创建API密钥模态框
                                },
                                span { "➕" }
                                "Create API Key"
                            }
                        }
                    }
                }
            },

            div {
                class: "space-y-6",

                // 使用统计概览
                UsageOverview {
                    token_usage: token_usage.clone()
                }

                // API密钥列表
                ApiKeysList {
                    api_keys: api_keys.clone(),
                    rbac: rbac.clone()
                }

                // 使用指南
                UsageGuide {}
            }
        }
    }
}

/// 使用统计概览组件
#[component]
fn UsageOverview(token_usage: Vec<TokenUsage>) -> Element {
    let total_tokens: i64 = token_usage.iter().map(|u| u.tokens_used).sum();
    let total_requests: i64 = token_usage.iter().map(|u| u.requests_count).sum();

    rsx! {
        div {
            class: "grid grid-cols-1 md:grid-cols-3 gap-6",

            // 总令牌使用量
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-lg font-semibold text-base-content/80",
                                "Total Tokens"
                            }
                            p {
                                class: "text-3xl font-bold text-primary",
                                "{total_tokens}"
                            }
                        }
                        div {
                            class: "text-4xl",
                            "🪙"
                        }
                    }
                    p {
                        class: "text-sm text-base-content/60",
                        "Last 7 days"
                    }
                }
            }

            // 总请求数
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-lg font-semibold text-base-content/80",
                                "API Requests"
                            }
                            p {
                                class: "text-3xl font-bold text-secondary",
                                "{total_requests}"
                            }
                        }
                        div {
                            class: "text-4xl",
                            "📊"
                        }
                    }
                    p {
                        class: "text-sm text-base-content/60",
                        "Last 7 days"
                    }
                }
            }

            // 平均每日使用
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-lg font-semibold text-base-content/80",
                                "Daily Average"
                            }
                            p {
                                class: "text-3xl font-bold text-accent",
                                "{total_tokens / 7}"
                            }
                        }
                        div {
                            class: "text-4xl",
                            "📈"
                        }
                    }
                    p {
                        class: "text-sm text-base-content/60",
                        "Tokens per day"
                    }
                }
            }
        }
    }
}

/// API密钥列表组件
#[component]
fn ApiKeysList(api_keys: Vec<ApiKey>, rbac: Rbac) -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-header p-6 border-b border-base-300",
                h4 {
                    class: "text-xl font-bold",
                    "API Keys ({api_keys.len()})"
                }
            }
            div {
                class: "card-body p-0",
                if api_keys.is_empty() {
                    EmptyApiKeys {}
                } else {
                    div {
                        class: "overflow-x-auto",
                        table {
                            class: "table table-zebra w-full",
                            thead {
                                tr {
                                    th { "Name" }
                                    th { "Key Preview" }
                                    th { "Created" }
                                    th { "Last Used" }
                                    th { "Usage" }
                                    th { "Status" }
                                    if rbac.can_use_api_keys() {
                                        th {
                                            class: "text-right",
                                            "Actions"
                                        }
                                    }
                                }
                            }
                            tbody {
                                for api_key in &api_keys {
                                    ApiKeyRow {
                                        api_key: api_key.clone(),
                                        rbac: rbac.clone()
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

/// API密钥行组件
#[component]
fn ApiKeyRow(api_key: ApiKey, rbac: Rbac) -> Element {
    rsx! {
        tr {
            td {
                div {
                    class: "font-medium",
                    "{api_key.name}"
                }
            }
            td {
                div {
                    class: "font-mono text-sm bg-base-200 px-2 py-1 rounded",
                    "{api_key.key_preview}"
                }
            }
            td {
                div {
                    class: "text-sm text-base-content/70",
                    "{api_key.created_at}"
                }
            }
            td {
                if let Some(last_used) = &api_key.last_used {
                    div {
                        class: "text-sm text-base-content/70",
                        "{last_used}"
                    }
                } else {
                    div {
                        class: "text-sm text-base-content/50",
                        "Never"
                    }
                }
            }
            td {
                div {
                    class: "text-sm",
                    "{api_key.usage_count} calls"
                }
            }
            td {
                if api_key.is_active {
                    div {
                        class: "badge badge-success",
                        "Active"
                    }
                } else {
                    div {
                        class: "badge badge-error",
                        "Inactive"
                    }
                }
            }
            if rbac.can_use_api_keys() {
                td {
                    class: "text-right",
                    div {
                        class: "flex justify-end gap-2",
                        button {
                            class: "btn btn-ghost btn-xs",
                            title: "Copy Key",
                            "📋"
                        }
                        button {
                            class: "btn btn-ghost btn-xs text-error",
                            title: "Delete Key",
                            "🗑️"
                        }
                    }
                }
            }
        }
    }
}

/// 空状态组件
#[component]
fn EmptyApiKeys() -> Element {
    rsx! {
        div {
            class: "text-center py-12",
            div {
                class: "text-6xl mb-4",
                "🔑"
            }
            h4 {
                class: "text-xl font-semibold mb-2",
                "No API Keys"
            }
            p {
                class: "text-base-content/60 mb-4",
                "Create your first API key to start using the Lumos AI API"
            }
            button {
                class: "btn btn-primary gap-2",
                span { "➕" }
                "Create API Key"
            }
        }
    }
}

/// 使用指南组件
#[component]
fn UsageGuide() -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-body",
                h4 {
                    class: "text-xl font-bold mb-4",
                    "📚 API Usage Guide"
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
                            li { "• Create an API key for your application" }
                            li { "• Include the key in your request headers" }
                            li { "• Start making API calls to Lumos AI" }
                        }
                    }

                    div {
                        h5 {
                            class: "font-semibold mb-2",
                            "🔒 Security Best Practices"
                        }
                        ul {
                            class: "space-y-1 text-sm text-base-content/70",
                            li { "• Keep your API keys secure and private" }
                            li { "• Rotate keys regularly" }
                            li { "• Use different keys for different environments" }
                        }
                    }
                }
            }
        }
    }
}
