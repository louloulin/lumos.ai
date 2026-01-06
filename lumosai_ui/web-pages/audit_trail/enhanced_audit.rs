/*!
# Enhanced Audit Trail

增强的审计日志界面，参考bionic-gpt实现，提供系统活动的完整审计追踪。

## 功能特性

- **活动监控**: 实时监控用户和系统活动
- **详细日志**: 完整的操作记录和上下文信息
- **智能过滤**: 按用户、时间、操作类型过滤
- **导出功能**: 支持审计报告导出
- **安全分析**: 异常活动检测和告警
*/

#![allow(non_snake_case)]
use crate::app_layout::{Layout, SideBar};
use crate::types::Rbac;
use dioxus::prelude::*;

// 临时类型定义
#[derive(Clone, Debug, PartialEq)]
pub struct AuditTrail {
    pub id: i32,
    pub user_email: String,
    pub user_name: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<i32>,
    pub resource_name: Option<String>,
    pub details: Option<String>,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub created_at: String,
    pub severity: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuditStats {
    pub total_events: i64,
    pub today_events: i64,
    pub unique_users: i64,
    pub security_events: i64,
    pub failed_logins: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SecurityAlert {
    pub id: i32,
    pub alert_type: String,
    pub description: String,
    pub user_email: String,
    pub created_at: String,
    pub severity: String,
    pub is_resolved: bool,
}

/// 增强审计日志页面
#[component]
pub fn EnhancedAuditPage(team_id: i32, rbac: Rbac) -> Element {
    // 模拟数据
    let audit_trails = vec![
        AuditTrail {
            id: 1,
            user_email: "admin@lumosai.com".to_string(),
            user_name: Some("张三".to_string()),
            action: "CREATE_ASSISTANT".to_string(),
            resource_type: "Assistant".to_string(),
            resource_id: Some(123),
            resource_name: Some("Code Helper".to_string()),
            details: Some("Created new AI assistant for code generation".to_string()),
            ip_address: "192.168.1.100".to_string(),
            user_agent: Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64)".to_string()),
            created_at: "2024-01-20 10:30:15".to_string(),
            severity: "INFO".to_string(),
        },
        AuditTrail {
            id: 2,
            user_email: "user@lumosai.com".to_string(),
            user_name: Some("李四".to_string()),
            action: "LOGIN_FAILED".to_string(),
            resource_type: "Authentication".to_string(),
            resource_id: None,
            resource_name: None,
            details: Some("Invalid password attempt".to_string()),
            ip_address: "203.0.113.45".to_string(),
            user_agent: Some("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)".to_string()),
            created_at: "2024-01-20 09:45:22".to_string(),
            severity: "WARNING".to_string(),
        },
        AuditTrail {
            id: 3,
            user_email: "dev@lumosai.com".to_string(),
            user_name: Some("王五".to_string()),
            action: "DELETE_DATASET".to_string(),
            resource_type: "Dataset".to_string(),
            resource_id: Some(456),
            resource_name: Some("Test Data".to_string()),
            details: Some("Permanently deleted dataset and all associated documents".to_string()),
            ip_address: "192.168.1.101".to_string(),
            user_agent: Some("Mozilla/5.0 (X11; Linux x86_64)".to_string()),
            created_at: "2024-01-20 08:15:33".to_string(),
            severity: "HIGH".to_string(),
        },
    ];

    let audit_stats = AuditStats {
        total_events: 15420,
        today_events: 89,
        unique_users: 23,
        security_events: 12,
        failed_logins: 3,
    };

    let security_alerts = vec![
        SecurityAlert {
            id: 1,
            alert_type: "MULTIPLE_FAILED_LOGINS".to_string(),
            description: "Multiple failed login attempts from same IP".to_string(),
            user_email: "unknown@external.com".to_string(),
            created_at: "2024-01-20 09:45:00".to_string(),
            severity: "HIGH".to_string(),
            is_resolved: false,
        },
        SecurityAlert {
            id: 2,
            alert_type: "UNUSUAL_ACCESS_PATTERN".to_string(),
            description: "Access from unusual geographic location".to_string(),
            user_email: "admin@lumosai.com".to_string(),
            created_at: "2024-01-19 22:30:00".to_string(),
            severity: "MEDIUM".to_string(),
            is_resolved: true,
        },
    ];

    rsx! {
        Layout {
            section_class: "p-4",
            selected_item: SideBar::AuditTrail,
            team_id,
            rbac: rbac.clone(),
            title: "Audit Trail",
            header: rsx! {
                div {
                    class: "flex items-center justify-between",
                    h3 {
                        class: "text-2xl font-bold",
                        "🔍 Audit Trail"
                    }
                    div {
                        class: "flex gap-4",
                        if rbac.can_view_audit_trail() {
                            button {
                                class: "btn btn-outline gap-2",
                                span { "📊" }
                                "Export Report"
                            }
                            button {
                                class: "btn btn-primary gap-2",
                                span { "🔍" }
                                "Filter"
                            }
                        }
                    }
                }
            },

            div {
                class: "space-y-6",

                // 审计统计概览
                AuditStatsOverview {
                    stats: audit_stats.clone()
                }

                // 安全告警
                if !security_alerts.is_empty() {
                    SecurityAlertsSection {
                        alerts: security_alerts.clone(),
                        rbac: rbac.clone()
                    }
                }

                // 审计日志列表
                AuditTrailTable {
                    audit_trails: audit_trails.clone(),
                    rbac: rbac.clone()
                }

                // 审计指南
                AuditGuide {}
            }
        }
    }
}

/// 审计统计概览组件
#[component]
fn AuditStatsOverview(stats: AuditStats) -> Element {
    rsx! {
        div {
            class: "grid grid-cols-1 md:grid-cols-5 gap-6",

            // 总事件数
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-lg font-semibold text-base-content/80",
                                "Total Events"
                            }
                            p {
                                class: "text-3xl font-bold text-primary",
                                "{stats.total_events}"
                            }
                        }
                        div {
                            class: "text-4xl",
                            "📊"
                        }
                    }
                }
            }

            // 今日事件
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-lg font-semibold text-base-content/80",
                                "Today"
                            }
                            p {
                                class: "text-3xl font-bold text-secondary",
                                "{stats.today_events}"
                            }
                        }
                        div {
                            class: "text-4xl",
                            "📅"
                        }
                    }
                }
            }

            // 活跃用户
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-lg font-semibold text-base-content/80",
                                "Active Users"
                            }
                            p {
                                class: "text-3xl font-bold text-accent",
                                "{stats.unique_users}"
                            }
                        }
                        div {
                            class: "text-4xl",
                            "👥"
                        }
                    }
                }
            }

            // 安全事件
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-lg font-semibold text-base-content/80",
                                "Security Events"
                            }
                            p {
                                class: "text-3xl font-bold text-warning",
                                "{stats.security_events}"
                            }
                        }
                        div {
                            class: "text-4xl",
                            "🔒"
                        }
                    }
                }
            }

            // 失败登录
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-lg font-semibold text-base-content/80",
                                "Failed Logins"
                            }
                            p {
                                class: "text-3xl font-bold text-error",
                                "{stats.failed_logins}"
                            }
                        }
                        div {
                            class: "text-4xl",
                            "❌"
                        }
                    }
                }
            }
        }
    }
}

/// 安全告警区块组件
#[component]
fn SecurityAlertsSection(alerts: Vec<SecurityAlert>, rbac: Rbac) -> Element {
    let unresolved_alerts: Vec<_> = alerts.iter().filter(|a| !a.is_resolved).collect();

    rsx! {
        div {
            class: "card bg-base-100 shadow-lg border-l-4 border-warning",
            div {
                class: "card-header p-6 border-b border-base-300",
                div {
                    class: "flex items-center justify-between",
                    h4 {
                        class: "text-xl font-bold text-warning",
                        "🚨 Security Alerts ({unresolved_alerts.len()})"
                    }
                    if rbac.can_view_audit_trail() {
                        button {
                            class: "btn btn-outline btn-sm",
                            "View All"
                        }
                    }
                }
            }
            div {
                class: "card-body",
                div {
                    class: "space-y-3",
                    for alert in &unresolved_alerts {
                        SecurityAlertItem {
                            alert: (*alert).clone()
                        }
                    }
                    if unresolved_alerts.is_empty() {
                        div {
                            class: "text-center py-4 text-base-content/60",
                            "✅ No active security alerts"
                        }
                    }
                }
            }
        }
    }
}

/// 安全告警项组件
#[component]
fn SecurityAlertItem(alert: SecurityAlert) -> Element {
    rsx! {
            div {
                class: "flex items-start justify-between p-3 bg-base-200 rounded-lg",
                div {
                    class: "flex-1",
                    div {
                        class: "flex items-center gap-2 mb-1",
                        div {
                            class: match alert.severity.as_str() {
                                "HIGH" => "badge badge-error badge-sm",
                                "MEDIUM" => "badge badge-warning badge-sm",
                                _ => "badge badge-info badge-sm"
                            },
                            "{alert.severity}"
                        }
                        span {
                            class: "text-sm font-medium",
    {alert.alert_type.replace("_", " ")}
                        }
                    }
                    p {
                        class: "text-sm text-base-content/70 mb-1",
                        "{alert.description}"
                    }
                    div {
                        class: "flex items-center gap-4 text-xs text-base-content/60",
                        span { "User: {alert.user_email}" }
                        span { "Time: {alert.created_at}" }
                    }
                }
                div {
                    class: "flex gap-2",
                    button {
                        class: "btn btn-ghost btn-xs",
                        "View"
                    }
                    button {
                        class: "btn btn-success btn-xs",
                        "Resolve"
                    }
                }
            }
        }
}

/// 审计日志表格组件
#[component]
fn AuditTrailTable(audit_trails: Vec<AuditTrail>, rbac: Rbac) -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-header p-6 border-b border-base-300",
                h4 {
                    class: "text-xl font-bold",
                    "Recent Activity ({audit_trails.len()})"
                }
            }
            div {
                class: "card-body p-0",
                div {
                    class: "overflow-x-auto",
                    table {
                        class: "table table-zebra w-full",
                        thead {
                            tr {
                                th { "Time" }
                                th { "User" }
                                th { "Action" }
                                th { "Resource" }
                                th { "IP Address" }
                                th { "Severity" }
                                if rbac.can_view_audit_trail() {
                                    th {
                                        class: "text-right",
                                        "Details"
                                    }
                                }
                            }
                        }
                        tbody {
                            for audit in &audit_trails {
                                AuditTrailRow {
                                    audit: audit.clone(),
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

/// 审计日志行组件
#[component]
fn AuditTrailRow(audit: AuditTrail, rbac: Rbac) -> Element {
    rsx! {
            tr {
                td {
                    div {
                        class: "text-sm",
                        "{audit.created_at}"
                    }
                }
                td {
                    div {
                        if let Some(name) = &audit.user_name {
                            div {
                                class: "font-medium",
                                "{name}"
                            }
                            div {
                                class: "text-xs text-base-content/60",
                                "{audit.user_email}"
                            }
                        } else {
                            div {
                                class: "font-medium",
                                "{audit.user_email}"
                            }
                        }
                    }
                }
                td {
                    div {
                        class: "font-mono text-sm",
    {audit.action.replace("_", " ")}
                    }
                }
                td {
                    div {
                        if let Some(resource_name) = &audit.resource_name {
                            div {
                                class: "font-medium",
                                "{resource_name}"
                            }
                            div {
                                class: "text-xs text-base-content/60",
                                "{audit.resource_type}"
                            }
                        } else {
                            div {
                                class: "text-sm",
                                "{audit.resource_type}"
                            }
                        }
                    }
                }
                td {
                    div {
                        class: "font-mono text-sm",
                        "{audit.ip_address}"
                    }
                }
                td {
                    div {
                        class: match audit.severity.as_str() {
                            "HIGH" => "badge badge-error",
                            "WARNING" => "badge badge-warning",
                            "INFO" => "badge badge-info",
                            _ => "badge badge-neutral"
                        },
                        "{audit.severity}"
                    }
                }
                if rbac.can_view_audit_trail() {
                    td {
                        class: "text-right",
                        button {
                            class: "btn btn-ghost btn-xs",
                            title: "View Details",
                            "👁️"
                        }
                    }
                }
            }
        }
}

/// 审计指南组件
#[component]
fn AuditGuide() -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-body",
                h4 {
                    class: "text-xl font-bold mb-4",
                    "📚 Audit Trail Guide"
                }
                div {
                    class: "grid grid-cols-1 md:grid-cols-3 gap-6",

                    div {
                        h5 {
                            class: "font-semibold mb-2",
                            "🔍 What We Track"
                        }
                        ul {
                            class: "space-y-1 text-sm text-base-content/70",
                            li { "• User authentication events" }
                            li { "• Resource creation/modification" }
                            li { "• Permission changes" }
                            li { "• Data access and exports" }
                        }
                    }

                    div {
                        h5 {
                            class: "font-semibold mb-2",
                            "🚨 Security Monitoring"
                        }
                        ul {
                            class: "space-y-1 text-sm text-base-content/70",
                            li { "• Failed login attempts" }
                            li { "• Unusual access patterns" }
                            li { "• Privilege escalations" }
                            li { "• Suspicious activities" }
                        }
                    }

                    div {
                        h5 {
                            class: "font-semibold mb-2",
                            "📊 Compliance"
                        }
                        ul {
                            class: "space-y-1 text-sm text-base-content/70",
                            li { "• GDPR compliance tracking" }
                            li { "• Data retention policies" }
                            li { "• Audit report generation" }
                            li { "• Regulatory requirements" }
                        }
                    }
                }
            }
        }
    }
}
