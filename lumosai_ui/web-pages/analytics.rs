/*!
# Analytics & Reporting Page

高级分析和报告功能，参考bionic-gpt实现，提供企业级的数据分析和可视化功能。

## 功能特性

- **使用分析**: 详细的使用情况统计和趋势分析
- **性能监控**: AI模型性能和响应时间监控
- **成本分析**: 令牌使用成本和预算管理
- **用户行为**: 用户活动模式和偏好分析
- **报告生成**: 自动化报告生成和导出功能
*/

#![allow(non_snake_case)]
use crate::app_layout::{Layout, SideBar};
use crate::charts::{ApiRequestChartCard, TokenUsageChartCard};
use crate::types::Rbac;
use dioxus::prelude::*;

// 分析数据类型定义
#[derive(Clone, Debug, PartialEq)]
pub struct AnalyticsOverview {
    pub total_users: i64,
    pub total_conversations: i64,
    pub total_tokens: i64,
    pub total_cost: f64,
    pub avg_response_time: f64,
    pub success_rate: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModelPerformance {
    pub model_name: String,
    pub total_requests: i64,
    pub avg_response_time: f64,
    pub success_rate: f64,
    pub total_tokens: i64,
    pub cost: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserActivity {
    pub user_id: i32,
    pub user_name: String,
    pub conversations: i64,
    pub messages: i64,
    pub tokens_used: i64,
    pub last_active: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CostBreakdown {
    pub category: String,
    pub amount: f64,
    pub percentage: f64,
    pub trend: String, // "up", "down", "stable"
}

/// 分析页面
#[component]
pub fn AnalyticsPage(team_id: i32, rbac: Rbac) -> Element {
    // 模拟数据
    let overview = AnalyticsOverview {
        total_users: 45,
        total_conversations: 1250,
        total_tokens: 2500000,
        total_cost: 125.50,
        avg_response_time: 1.2,
        success_rate: 98.5,
    };

    let model_performance = vec![
        ModelPerformance {
            model_name: "GPT-4".to_string(),
            total_requests: 850,
            avg_response_time: 1.5,
            success_rate: 99.2,
            total_tokens: 1800000,
            cost: 90.0,
        },
        ModelPerformance {
            model_name: "DeepSeek-Chat".to_string(),
            total_requests: 400,
            avg_response_time: 0.8,
            success_rate: 97.5,
            total_tokens: 700000,
            cost: 35.5,
        },
    ];

    let top_users = vec![
        UserActivity {
            user_id: 1,
            user_name: "张三".to_string(),
            conversations: 156,
            messages: 890,
            tokens_used: 450000,
            last_active: "2 hours ago".to_string(),
        },
        UserActivity {
            user_id: 2,
            user_name: "李四".to_string(),
            conversations: 98,
            messages: 567,
            tokens_used: 280000,
            last_active: "1 day ago".to_string(),
        },
        UserActivity {
            user_id: 3,
            user_name: "王五".to_string(),
            conversations: 76,
            messages: 423,
            tokens_used: 210000,
            last_active: "3 hours ago".to_string(),
        },
    ];

    let cost_breakdown = vec![
        CostBreakdown {
            category: "GPT-4 API".to_string(),
            amount: 90.0,
            percentage: 71.6,
            trend: "up".to_string(),
        },
        CostBreakdown {
            category: "DeepSeek API".to_string(),
            amount: 35.5,
            percentage: 28.3,
            trend: "stable".to_string(),
        },
        CostBreakdown {
            category: "Storage".to_string(),
            amount: 0.1,
            percentage: 0.1,
            trend: "down".to_string(),
        },
    ];

    rsx! {
        Layout {
            section_class: "p-4",
            selected_item: SideBar::AuditTrail,
            team_id,
            rbac: rbac.clone(),
            title: "Analytics & Reports",
            header: rsx! {
                div {
                    class: "flex items-center justify-between",
                    h3 {
                        class: "text-2xl font-bold",
                        "📊 Analytics & Reports"
                    }
                    div {
                        class: "flex space-x-2",
                        button {
                            class: "btn btn-outline btn-sm",
                            "📅 Date Range"
                        }
                        button {
                            class: "btn btn-primary btn-sm",
                            "📄 Export Report"
                        }
                    }
                }
            },

            div {
                class: "max-w-7xl mx-auto space-y-6",

                // 概览统计
                OverviewStats {
                    overview: overview.clone()
                }

                // 图表区域
                div {
                    class: "grid grid-cols-1 lg:grid-cols-2 gap-6",

                    // 令牌使用趋势图
                    TokenUsageChartCard {
                        data: vec![], // 这里应该传入真实数据
                        title: "Token Usage Trends".to_string()
                    }

                    // API请求趋势图
                    ApiRequestChartCard {
                        data: vec![], // 这里应该传入真实数据
                        title: "API Request Trends".to_string()
                    }
                }

                // 模型性能分析
                ModelPerformanceCard {
                    models: model_performance.clone()
                }

                // 用户活动分析
                UserActivityCard {
                    users: top_users.clone()
                }

                // 成本分析
                CostAnalysisCard {
                    breakdown: cost_breakdown.clone()
                }
            }
        }
    }
}

/// 概览统计组件
#[component]
fn OverviewStats(overview: AnalyticsOverview) -> Element {
    rsx! {
        div {
            class: "grid grid-cols-1 md:grid-cols-3 lg:grid-cols-6 gap-4",

            // 总用户数
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body p-4",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-sm font-medium text-base-content/70",
                                "Total Users"
                            }
                            p {
                                class: "text-2xl font-bold text-primary",
                                "{overview.total_users}"
                            }
                        }
                        div {
                            class: "text-2xl",
                            "👥"
                        }
                    }
                }
            }

            // 总对话数
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body p-4",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-sm font-medium text-base-content/70",
                                "Conversations"
                            }
                            p {
                                class: "text-2xl font-bold text-secondary",
                                "{overview.total_conversations}"
                            }
                        }
                        div {
                            class: "text-2xl",
                            "💬"
                        }
                    }
                }
            }

            // 总令牌数
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body p-4",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-sm font-medium text-base-content/70",
                                "Total Tokens"
                            }
                            p {
                                class: "text-2xl font-bold text-accent",
                                "{overview.total_tokens / 1000}K"
                            }
                        }
                        div {
                            class: "text-2xl",
                            "🪙"
                        }
                    }
                }
            }

            // 总成本
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body p-4",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-sm font-medium text-base-content/70",
                                "Total Cost"
                            }
                            p {
                                class: "text-2xl font-bold text-warning",
                                "${overview.total_cost:.2}"
                            }
                        }
                        div {
                            class: "text-2xl",
                            "💰"
                        }
                    }
                }
            }

            // 平均响应时间
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body p-4",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-sm font-medium text-base-content/70",
                                "Avg Response"
                            }
                            p {
                                class: "text-2xl font-bold text-info",
                                "{overview.avg_response_time:.1}s"
                            }
                        }
                        div {
                            class: "text-2xl",
                            "⚡"
                        }
                    }
                }
            }

            // 成功率
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body p-4",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-sm font-medium text-base-content/70",
                                "Success Rate"
                            }
                            p {
                                class: "text-2xl font-bold text-success",
                                "{overview.success_rate:.1}%"
                            }
                        }
                        div {
                            class: "text-2xl",
                            "✅"
                        }
                    }
                }
            }
        }
    }
}

/// 模型性能分析组件
#[component]
fn ModelPerformanceCard(models: Vec<ModelPerformance>) -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-body",
                h4 {
                    class: "text-xl font-bold mb-4",
                    "🤖 Model Performance Analysis"
                }
                div {
                    class: "overflow-x-auto",
                    table {
                        class: "table table-zebra w-full",
                        thead {
                            tr {
                                th { "Model" }
                                th { "Requests" }
                                th { "Avg Response Time" }
                                th { "Success Rate" }
                                th { "Tokens Used" }
                                th { "Cost" }
                            }
                        }
                        tbody {
                            for model in &models {
                                tr {
                                    td {
                                        div {
                                            class: "font-semibold",
                                            "{model.model_name}"
                                        }
                                    }
                                    td { "{model.total_requests}" }
                                    td {
                                        span {
                                            class: if model.avg_response_time < 1.0 { "text-success" } else { "text-warning" },
                                            "{model.avg_response_time:.2}s"
                                        }
                                    }
                                    td {
                                        span {
                                            class: if model.success_rate > 98.0 { "text-success" } else { "text-warning" },
                                            "{model.success_rate:.1}%"
                                        }
                                    }
                                    td { "{model.total_tokens / 1000}K" }
                                    td {
                                        span {
                                            class: "font-semibold",
                                            "${model.cost:.2}"
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
}

/// 用户活动分析组件
#[component]
fn UserActivityCard(users: Vec<UserActivity>) -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-body",
                h4 {
                    class: "text-xl font-bold mb-4",
                    "👤 Top Active Users"
                }
                div {
                    class: "space-y-3",
                    for user in &users {
                        div {
                            class: "flex items-center justify-between p-4 bg-base-200 rounded-lg",
                            div {
                                class: "flex items-center space-x-3",
                                div {
                                    class: "avatar placeholder",
                                    div {
                                        class: "bg-primary text-primary-content rounded-full w-10",
                                        span {
                                            class: "text-sm",
                                            "{user.user_name.chars().next().unwrap_or('U')}"
                                        }
                                    }
                                }
                                div {
                                    h5 {
                                        class: "font-semibold",
                                        "{user.user_name}"
                                    }
                                    p {
                                        class: "text-sm text-base-content/70",
                                        "Last active: {user.last_active}"
                                    }
                                }
                            }
                            div {
                                class: "text-right",
                                div {
                                    class: "grid grid-cols-3 gap-4 text-sm",
                                    div {
                                        class: "text-center",
                                        p {
                                            class: "font-bold text-primary",
                                            "{user.conversations}"
                                        }
                                        p {
                                            class: "text-xs text-base-content/60",
                                            "Chats"
                                        }
                                    }
                                    div {
                                        class: "text-center",
                                        p {
                                            class: "font-bold text-secondary",
                                            "{user.messages}"
                                        }
                                        p {
                                            class: "text-xs text-base-content/60",
                                            "Messages"
                                        }
                                    }
                                    div {
                                        class: "text-center",
                                        p {
                                            class: "font-bold text-accent",
                                            "{user.tokens_used / 1000}K"
                                        }
                                        p {
                                            class: "text-xs text-base-content/60",
                                            "Tokens"
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
}

/// 成本分析组件
#[component]
fn CostAnalysisCard(breakdown: Vec<CostBreakdown>) -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-body",
                h4 {
                    class: "text-xl font-bold mb-4",
                    "💰 Cost Breakdown"
                }
                div {
                    class: "space-y-4",
                    for item in &breakdown {
                        div {
                            class: "flex items-center justify-between p-3 bg-base-200 rounded-lg",
                            div {
                                class: "flex items-center space-x-3",
                                div {
                                    class: "text-2xl",
                                    if item.category.contains("GPT") { "🧠" }
                                    else if item.category.contains("DeepSeek") { "🔮" }
                                    else { "💾" }
                                }
                                div {
                                    h5 {
                                        class: "font-semibold",
                                        "{item.category}"
                                    }
                                    p {
                                        class: "text-sm text-base-content/70",
                                        "{item.percentage:.1}% of total cost"
                                    }
                                }
                            }
                            div {
                                class: "text-right",
                                p {
                                    class: "text-lg font-bold",
                                    "${item.amount:.2}"
                                }
                                div {
                                    class: "flex items-center space-x-1",
                                    span {
                                        class: match item.trend.as_str() {
                                            "up" => "text-error",
                                            "down" => "text-success",
                                            _ => "text-base-content/60"
                                        },
                                        match item.trend.as_str() {
                                            "up" => "↗️",
                                            "down" => "↘️",
                                            _ => "➡️"
                                        }
                                    }
                                    span {
                                        class: "text-xs text-base-content/60",
                                        match item.trend.as_str() {
                                            "up" => "Increasing",
                                            "down" => "Decreasing",
                                            _ => "Stable"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div {
                    class: "mt-6 p-4 bg-info/10 rounded-lg",
                    div {
                        class: "flex items-center space-x-2",
                        span { class: "text-info", "💡" }
                        h5 {
                            class: "font-semibold text-info",
                            "Cost Optimization Tips"
                        }
                    }
                    ul {
                        class: "mt-2 text-sm text-base-content/70 space-y-1",
                        li { "• Consider using DeepSeek for routine tasks to reduce costs" }
                        li { "• Implement response caching for frequently asked questions" }
                        li { "• Set up usage alerts to monitor spending" }
                    }
                }
            }
        }
    }
}
