/*!
# Dashboard Page

主页仪表板组件，提供系统概览、快速操作和最近活动展示。

## 功能特性

- **统计概览**: 使用情况统计和关键指标
- **快速操作**: 常用功能的快速入口
- **最近活动**: 最近的对话和助手使用记录
- **系统状态**: 服务状态和配额使用情况
- **欢迎引导**: 新用户引导和帮助信息
*/

#![allow(non_snake_case)]
use dioxus::prelude::*;
use crate::app_layout::{Layout, SideBar};
use crate::types::Rbac;

// 临时类型定义
#[derive(Clone, Debug, PartialEq)]
pub struct DashboardStats {
    pub total_conversations: i64,
    pub total_messages: i64,
    pub active_assistants: i64,
    pub tokens_used_today: i64,
    pub api_calls_today: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecentActivity {
    pub id: i64,
    pub activity_type: String,
    pub title: String,
    pub description: String,
    pub timestamp: String,
    pub icon: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct QuickAction {
    pub title: String,
    pub description: String,
    pub icon: String,
    pub href: String,
    pub color: String,
}

/// 主页仪表板页面
#[component]
pub fn DashboardPage(
    team_id: i32,
    rbac: Rbac,
) -> Element {
    // 模拟数据
    let stats = DashboardStats {
        total_conversations: 156,
        total_messages: 1240,
        active_assistants: 8,
        tokens_used_today: 15420,
        api_calls_today: 89,
    };

    let recent_activities = vec![
        RecentActivity {
            id: 1,
            activity_type: "conversation".to_string(),
            title: "New conversation with Code Assistant".to_string(),
            description: "Started a new coding session".to_string(),
            timestamp: "2 minutes ago".to_string(),
            icon: "💬".to_string(),
        },
        RecentActivity {
            id: 2,
            activity_type: "assistant".to_string(),
            title: "Created Writing Helper assistant".to_string(),
            description: "New assistant for content creation".to_string(),
            timestamp: "1 hour ago".to_string(),
            icon: "🤖".to_string(),
        },
        RecentActivity {
            id: 3,
            activity_type: "dataset".to_string(),
            title: "Uploaded product documentation".to_string(),
            description: "45 documents processed successfully".to_string(),
            timestamp: "3 hours ago".to_string(),
            icon: "📄".to_string(),
        },
    ];

    let quick_actions = vec![
        QuickAction {
            title: "Start New Chat".to_string(),
            description: "Begin a conversation with AI".to_string(),
            icon: "💬".to_string(),
            href: format!("/app/team/{}/console", team_id),
            color: "btn-primary".to_string(),
        },
        QuickAction {
            title: "Create Assistant".to_string(),
            description: "Build a custom AI assistant".to_string(),
            icon: "🤖".to_string(),
            href: format!("/app/team/{}/prompts/new", team_id),
            color: "btn-secondary".to_string(),
        },
        QuickAction {
            title: "Upload Documents".to_string(),
            description: "Add knowledge to your AI".to_string(),
            icon: "📁".to_string(),
            href: format!("/app/team/{}/datasets", team_id),
            color: "btn-accent".to_string(),
        },
        QuickAction {
            title: "View Analytics".to_string(),
            description: "Check usage and performance".to_string(),
            icon: "📊".to_string(),
            href: format!("/app/team/{}/audit_trail", team_id),
            color: "btn-info".to_string(),
        },
    ];

    rsx! {
        Layout {
            section_class: "p-6",
            selected_item: SideBar::None,
            team_id,
            rbac: rbac.clone(),
            title: "Dashboard",
            header: rsx! {
                div {
                    class: "flex items-center justify-between",
                    h3 {
                        class: "text-3xl font-bold",
                        "🌟 Welcome to LumosAI"
                    }
                    div {
                        class: "text-sm text-base-content/60",
                        "Today is a great day to build something amazing!"
                    }
                }
            },

            div {
                class: "space-y-8",

                // 统计概览
                StatsOverview {
                    stats: stats.clone()
                }

                // 快速操作
                QuickActions {
                    actions: quick_actions.clone()
                }

                // 最近活动
                RecentActivities {
                    activities: recent_activities.clone()
                }

                // 欢迎引导
                WelcomeGuide {
                    rbac: rbac.clone()
                }
            }
        }
    }
}

/// 统计概览组件
#[component]
fn StatsOverview(stats: DashboardStats) -> Element {
    rsx! {
        div {
            class: "grid grid-cols-1 md:grid-cols-5 gap-6",

            // 总对话数
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-lg font-semibold text-base-content/80",
                                "Conversations"
                            }
                            p {
                                class: "text-3xl font-bold text-primary",
                                "{stats.total_conversations}"
                            }
                        }
                        div {
                            class: "text-4xl",
                            "💬"
                        }
                    }
                    p {
                        class: "text-sm text-base-content/60",
                        "Total conversations"
                    }
                }
            }

            // 总消息数
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-lg font-semibold text-base-content/80",
                                "Messages"
                            }
                            p {
                                class: "text-3xl font-bold text-secondary",
                                "{stats.total_messages}"
                            }
                        }
                        div {
                            class: "text-4xl",
                            "📝"
                        }
                    }
                    p {
                        class: "text-sm text-base-content/60",
                        "Messages exchanged"
                    }
                }
            }

            // 活跃助手
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-lg font-semibold text-base-content/80",
                                "Assistants"
                            }
                            p {
                                class: "text-3xl font-bold text-accent",
                                "{stats.active_assistants}"
                            }
                        }
                        div {
                            class: "text-4xl",
                            "🤖"
                        }
                    }
                    p {
                        class: "text-sm text-base-content/60",
                        "Active assistants"
                    }
                }
            }

            // 今日令牌使用
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-lg font-semibold text-base-content/80",
                                "Tokens Today"
                            }
                            p {
                                class: "text-3xl font-bold text-warning",
                                "{stats.tokens_used_today}"
                            }
                        }
                        div {
                            class: "text-4xl",
                            "🪙"
                        }
                    }
                    p {
                        class: "text-sm text-base-content/60",
                        "Tokens consumed"
                    }
                }
            }

            // 今日API调用
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-lg font-semibold text-base-content/80",
                                "API Calls"
                            }
                            p {
                                class: "text-3xl font-bold text-info",
                                "{stats.api_calls_today}"
                            }
                        }
                        div {
                            class: "text-4xl",
                            "🔌"
                        }
                    }
                    p {
                        class: "text-sm text-base-content/60",
                        "API calls today"
                    }
                }
            }
        }
    }
}

/// 快速操作组件
#[component]
fn QuickActions(actions: Vec<QuickAction>) -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-body",
                h4 {
                    class: "text-xl font-bold mb-6",
                    "🚀 Quick Actions"
                }
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4",
                    for action in &actions {
                        a {
                            href: "{action.href}",
                            class: "card bg-base-200 hover:bg-base-300 transition-colors cursor-pointer",
                            div {
                                class: "card-body p-4 text-center",
                                div {
                                    class: "text-4xl mb-2",
                                    "{action.icon}"
                                }
                                h5 {
                                    class: "font-bold mb-1",
                                    "{action.title}"
                                }
                                p {
                                    class: "text-sm text-base-content/70",
                                    "{action.description}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 最近活动组件
#[component]
fn RecentActivities(activities: Vec<RecentActivity>) -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-body",
                h4 {
                    class: "text-xl font-bold mb-6",
                    "📋 Recent Activity"
                }
                div {
                    class: "space-y-4",
                    for activity in &activities {
                        div {
                            class: "flex items-start space-x-4 p-4 bg-base-200 rounded-lg",
                            div {
                                class: "text-2xl",
                                "{activity.icon}"
                            }
                            div {
                                class: "flex-1",
                                h5 {
                                    class: "font-semibold",
                                    "{activity.title}"
                                }
                                p {
                                    class: "text-sm text-base-content/70",
                                    "{activity.description}"
                                }
                                p {
                                    class: "text-xs text-base-content/50 mt-1",
                                    "{activity.timestamp}"
                                }
                            }
                        }
                    }
                }
                div {
                    class: "text-center mt-6",
                    button {
                        class: "btn btn-outline btn-sm",
                        "View All Activity"
                    }
                }
            }
        }
    }
}

/// 欢迎引导组件
#[component]
fn WelcomeGuide(rbac: Rbac) -> Element {
    rsx! {
        div {
            class: "card bg-gradient-to-r from-primary/10 to-secondary/10 shadow-lg",
            div {
                class: "card-body",
                h4 {
                    class: "text-xl font-bold mb-4",
                    "🎯 Getting Started"
                }
                div {
                    class: "grid grid-cols-1 md:grid-cols-3 gap-6",
                    
                    div {
                        h5 {
                            class: "font-semibold mb-2",
                            "1️⃣ Start Chatting"
                        }
                        p {
                            class: "text-sm text-base-content/70",
                            "Begin your AI journey by starting a conversation. Ask questions, get help, or explore ideas."
                        }
                    }
                    
                    div {
                        h5 {
                            class: "font-semibold mb-2",
                            "2️⃣ Create Assistants"
                        }
                        p {
                            class: "text-sm text-base-content/70",
                            "Build custom AI assistants tailored to your specific needs and workflows."
                        }
                    }
                    
                    div {
                        h5 {
                            class: "font-semibold mb-2",
                            "3️⃣ Upload Knowledge"
                        }
                        p {
                            class: "text-sm text-base-content/70",
                            "Enhance your AI with your own documents and data for more accurate responses."
                        }
                    }
                }
                
                div {
                    class: "text-center mt-6",
                    button {
                        class: "btn btn-primary gap-2",
                        span { "📚" }
                        "View Documentation"
                    }
                }
            }
        }
    }
}
