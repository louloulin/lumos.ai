/*!
# Enhanced Notification System

高级实时通知系统，参考bionic-gpt实现，提供企业级的通知管理和用户反馈功能。

## 功能特性

- **实时通知**: WebSocket实时通知推送
- **多类型支持**: 成功、警告、错误、信息等多种通知类型
- **持久化通知**: 重要通知的持久化存储和管理
- **通知中心**: 集中的通知管理界面
- **用户偏好**: 个性化通知设置和过滤
*/

#![allow(non_snake_case)]
use crate::app_layout::{Layout, SideBar};
use crate::types::Rbac;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

// 通知类型定义
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NotificationType {
    Success,
    Warning,
    Error,
    Info,
    System,
}

impl NotificationType {
    pub fn icon(&self) -> &'static str {
        match self {
            NotificationType::Success => "✅",
            NotificationType::Warning => "⚠️",
            NotificationType::Error => "❌",
            NotificationType::Info => "ℹ️",
            NotificationType::System => "🔔",
        }
    }

    pub fn color_class(&self) -> &'static str {
        match self {
            NotificationType::Success => "alert-success",
            NotificationType::Warning => "alert-warning",
            NotificationType::Error => "alert-error",
            NotificationType::Info => "alert-info",
            NotificationType::System => "alert-info",
        }
    }

    pub fn badge_class(&self) -> &'static str {
        match self {
            NotificationType::Success => "badge-success",
            NotificationType::Warning => "badge-warning",
            NotificationType::Error => "badge-error",
            NotificationType::Info => "badge-info",
            NotificationType::System => "badge-primary",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl NotificationPriority {
    pub fn display_text(&self) -> &'static str {
        match self {
            NotificationPriority::Low => "Low",
            NotificationPriority::Normal => "Normal",
            NotificationPriority::High => "High",
            NotificationPriority::Critical => "Critical",
        }
    }

    pub fn badge_class(&self) -> &'static str {
        match self {
            NotificationPriority::Low => "badge-ghost",
            NotificationPriority::Normal => "badge-neutral",
            NotificationPriority::High => "badge-warning",
            NotificationPriority::Critical => "badge-error",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Notification {
    pub id: u64,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub priority: NotificationPriority,
    pub timestamp: String,
    pub is_read: bool,
    pub is_persistent: bool,
    pub action_url: Option<String>,
    pub action_label: Option<String>,
    pub user_id: Option<i32>,
    pub team_id: Option<i32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NotificationStats {
    pub total: usize,
    pub unread: usize,
    pub high_priority: usize,
    pub system_notifications: usize,
}

/// 增强的通知系统页面
#[component]
pub fn NotificationSystemPage(team_id: i32, rbac: Rbac) -> Element {
    // 模拟通知数据
    let notifications = vec![
        Notification {
            id: 1,
            title: "AI Model Update".to_string(),
            message: "GPT-4 model has been updated with improved performance and new capabilities."
                .to_string(),
            notification_type: NotificationType::System,
            priority: NotificationPriority::High,
            timestamp: "2 minutes ago".to_string(),
            is_read: false,
            is_persistent: true,
            action_url: Some("/models".to_string()),
            action_label: Some("View Models".to_string()),
            user_id: Some(rbac.team_id),
            team_id: Some(team_id),
        },
        Notification {
            id: 2,
            title: "API Limit Warning".to_string(),
            message: "Your team is approaching the monthly API usage limit (85% used).".to_string(),
            notification_type: NotificationType::Warning,
            priority: NotificationPriority::High,
            timestamp: "1 hour ago".to_string(),
            is_read: false,
            is_persistent: true,
            action_url: Some("/api-keys".to_string()),
            action_label: Some("Manage API Keys".to_string()),
            user_id: Some(rbac.team_id),
            team_id: Some(team_id),
        },
        Notification {
            id: 3,
            title: "Conversation Exported".to_string(),
            message: "Your conversation export has been completed successfully.".to_string(),
            notification_type: NotificationType::Success,
            priority: NotificationPriority::Normal,
            timestamp: "3 hours ago".to_string(),
            is_read: true,
            is_persistent: false,
            action_url: Some("/history".to_string()),
            action_label: Some("Download".to_string()),
            user_id: Some(rbac.team_id),
            team_id: Some(team_id),
        },
        Notification {
            id: 4,
            title: "Security Alert".to_string(),
            message: "Unusual login activity detected from a new location.".to_string(),
            notification_type: NotificationType::Error,
            priority: NotificationPriority::Critical,
            timestamp: "1 day ago".to_string(),
            is_read: false,
            is_persistent: true,
            action_url: Some("/audit-trail".to_string()),
            action_label: Some("Review Activity".to_string()),
            user_id: Some(rbac.team_id),
            team_id: Some(team_id),
        },
    ];

    let stats = NotificationStats {
        total: notifications.len(),
        unread: notifications.iter().filter(|n| !n.is_read).count(),
        high_priority: notifications
            .iter()
            .filter(|n| {
                matches!(
                    n.priority,
                    NotificationPriority::High | NotificationPriority::Critical
                )
            })
            .count(),
        system_notifications: notifications
            .iter()
            .filter(|n| matches!(n.notification_type, NotificationType::System))
            .count(),
    };

    rsx! {
        Layout {
            section_class: "p-4",
            selected_item: SideBar::AuditTrail,
            team_id,
            rbac: rbac.clone(),
            title: "Notification Center",
            header: rsx! {
                div {
                    class: "flex items-center justify-between",
                    h3 {
                        class: "text-2xl font-bold",
                        "🔔 Notification Center"
                    }
                    div {
                        class: "flex space-x-2",
                        button {
                            class: "btn btn-outline btn-sm",
                            "⚙️ Settings"
                        }
                        button {
                            class: "btn btn-primary btn-sm",
                            "✅ Mark All Read"
                        }
                    }
                }
            },

            div {
                class: "max-w-6xl mx-auto space-y-6",

                // 通知统计概览
                NotificationStatsPanel {
                    stats: stats.clone()
                }

                // 通知过滤器
                NotificationFilters {}

                // 通知列表
                NotificationList {
                    notifications: notifications.clone()
                }
            }
        }
    }
}

/// 通知统计面板
#[component]
fn NotificationStatsPanel(stats: NotificationStats) -> Element {
    rsx! {
        div {
            class: "grid grid-cols-1 md:grid-cols-4 gap-4",

            // 总通知数
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body p-4",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-sm font-medium text-base-content/70",
                                "Total Notifications"
                            }
                            p {
                                class: "text-2xl font-bold text-primary",
                                "{stats.total}"
                            }
                        }
                        div {
                            class: "text-2xl",
                            "📬"
                        }
                    }
                }
            }

            // 未读通知数
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body p-4",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-sm font-medium text-base-content/70",
                                "Unread"
                            }
                            p {
                                class: "text-2xl font-bold text-warning",
                                "{stats.unread}"
                            }
                        }
                        div {
                            class: "text-2xl",
                            "📩"
                        }
                    }
                }
            }

            // 高优先级通知
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body p-4",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-sm font-medium text-base-content/70",
                                "High Priority"
                            }
                            p {
                                class: "text-2xl font-bold text-error",
                                "{stats.high_priority}"
                            }
                        }
                        div {
                            class: "text-2xl",
                            "🚨"
                        }
                    }
                }
            }

            // 系统通知
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body p-4",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-sm font-medium text-base-content/70",
                                "System"
                            }
                            p {
                                class: "text-2xl font-bold text-info",
                                "{stats.system_notifications}"
                            }
                        }
                        div {
                            class: "text-2xl",
                            "🔔"
                        }
                    }
                }
            }
        }
    }
}

/// 通知过滤器
#[component]
fn NotificationFilters() -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-body",
                h4 {
                    class: "text-lg font-bold mb-4",
                    "🔍 Filter Notifications"
                }
                div {
                    class: "flex flex-wrap gap-4",

                    // 类型过滤
                    div {
                        class: "form-control",
                        label {
                            class: "label",
                            span {
                                class: "label-text",
                                "Type"
                            }
                        }
                        select {
                            class: "select select-bordered select-sm",
                            option { value: "all", "All Types" }
                            option { value: "success", "Success" }
                            option { value: "warning", "Warning" }
                            option { value: "error", "Error" }
                            option { value: "info", "Info" }
                            option { value: "system", "System" }
                        }
                    }

                    // 优先级过滤
                    div {
                        class: "form-control",
                        label {
                            class: "label",
                            span {
                                class: "label-text",
                                "Priority"
                            }
                        }
                        select {
                            class: "select select-bordered select-sm",
                            option { value: "all", "All Priorities" }
                            option { value: "critical", "Critical" }
                            option { value: "high", "High" }
                            option { value: "normal", "Normal" }
                            option { value: "low", "Low" }
                        }
                    }

                    // 状态过滤
                    div {
                        class: "form-control",
                        label {
                            class: "label",
                            span {
                                class: "label-text",
                                "Status"
                            }
                        }
                        select {
                            class: "select select-bordered select-sm",
                            option { value: "all", "All" }
                            option { value: "unread", "Unread" }
                            option { value: "read", "Read" }
                        }
                    }

                    // 搜索框
                    div {
                        class: "form-control flex-1",
                        label {
                            class: "label",
                            span {
                                class: "label-text",
                                "Search"
                            }
                        }
                        input {
                            class: "input input-bordered input-sm",
                            "type": "text",
                            placeholder: "Search notifications..."
                        }
                    }
                }
            }
        }
    }
}

/// 通知列表组件
#[component]
fn NotificationList(notifications: Vec<Notification>) -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-body",
                h4 {
                    class: "text-lg font-bold mb-4",
                    "📋 Recent Notifications"
                }
                div {
                    class: "space-y-3",
                    for notification in &notifications {
                        NotificationItem {
                            notification: (*notification).clone()
                        }
                    }
                    if notifications.is_empty() {
                        div {
                            class: "text-center py-8 text-base-content/60",
                            div {
                                class: "text-4xl mb-2",
                                "📭"
                            }
                            p { "No notifications to display" }
                        }
                    }
                }
            }
        }
    }
}

/// 单个通知项组件
#[component]
fn NotificationItem(notification: Notification) -> Element {
    rsx! {
        div {
            class: format!(
                "flex items-start gap-4 p-4 rounded-lg border-l-4 {} {}",
                if notification.is_read { "bg-base-200/50" } else { "bg-base-100" },
                match notification.priority {
                    NotificationPriority::Critical => "border-error",
                    NotificationPriority::High => "border-warning",
                    NotificationPriority::Normal => "border-info",
                    NotificationPriority::Low => "border-base-300",
                }
            ),

            // 通知图标
            div {
                class: "flex-shrink-0",
                div {
                    class: format!(
                        "w-10 h-10 rounded-full flex items-center justify-center text-lg {}",
                        if notification.is_read { "bg-base-300" } else { "bg-primary text-primary-content" }
                    ),
                    "{notification.notification_type.icon()}"
                }
            }

            // 通知内容
            div {
                class: "flex-1 min-w-0",
                div {
                    class: "flex items-start justify-between mb-2",
                    div {
                        class: "flex items-center gap-2",
                        h5 {
                            class: format!(
                                "font-semibold {}",
                                if notification.is_read { "text-base-content/70" } else { "text-base-content" }
                            ),
                            "{notification.title}"
                        }
                        span {
                            class: format!("badge badge-sm {}", notification.notification_type.badge_class()),
                            "{notification.notification_type.icon()}"
                        }
                        span {
                            class: format!("badge badge-sm {}", notification.priority.badge_class()),
                            "{notification.priority.display_text()}"
                        }
                        if !notification.is_read {
                            span {
                                class: "badge badge-primary badge-sm",
                                "NEW"
                            }
                        }
                    }
                    div {
                        class: "flex items-center gap-2",
                        span {
                            class: "text-xs text-base-content/60",
                            "{notification.timestamp}"
                        }
                        if notification.is_persistent {
                            span {
                                class: "text-xs",
                                "📌"
                            }
                        }
                    }
                }

                p {
                    class: format!(
                        "text-sm mb-3 {}",
                        if notification.is_read { "text-base-content/60" } else { "text-base-content/80" }
                    ),
                    "{notification.message}"
                }

                // 操作按钮
                div {
                    class: "flex items-center gap-2",
                    if let Some(action_url) = &notification.action_url {
                        if let Some(action_label) = &notification.action_label {
                            a {
                                href: "{action_url}",
                                class: "btn btn-primary btn-xs",
                                "{action_label}"
                            }
                        }
                    }
                    if !notification.is_read {
                        button {
                            class: "btn btn-ghost btn-xs",
                            "Mark as Read"
                        }
                    }
                    button {
                        class: "btn btn-ghost btn-xs",
                        "🗑️ Delete"
                    }
                }
            }
        }
    }
}

/// 实时通知Toast组件
#[component]
pub fn NotificationToast(
    notification: Option<Notification>,
    on_dismiss: EventHandler<()>,
) -> Element {
    if let Some(notif) = notification {
        rsx! {
            div {
                class: "toast toast-top toast-end z-50",
                div {
                    class: format!("alert {} shadow-lg max-w-sm", notif.notification_type.color_class()),
                    div {
                        class: "flex items-start gap-3",
                        span {
                            class: "text-lg",
                            "{notif.notification_type.icon()}"
                        }
                        div {
                            class: "flex-1",
                            h4 {
                                class: "font-bold text-sm",
                                "{notif.title}"
                            }
                            p {
                                class: "text-xs opacity-80",
                                "{notif.message}"
                            }
                        }
                        button {
                            class: "btn btn-ghost btn-xs",
                            onclick: move |_| on_dismiss.call(()),
                            "✕"
                        }
                    }
                }
            }
        }
    } else {
        rsx! { div {} }
    }
}

/// 通知偏好设置组件
#[component]
pub fn NotificationPreferences(team_id: i32, rbac: Rbac) -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-body",
                h4 {
                    class: "text-lg font-bold mb-4",
                    "⚙️ Notification Preferences"
                }
                div {
                    class: "space-y-4",

                    // 通知类型设置
                    div {
                        h5 {
                            class: "font-semibold mb-2",
                            "Notification Types"
                        }
                        div {
                            class: "space-y-2",
                            div {
                                class: "form-control",
                                label {
                                    class: "label cursor-pointer",
                                    span {
                                        class: "label-text",
                                        "🔔 System Notifications"
                                    }
                                    input {
                                        "type": "checkbox",
                                        class: "toggle toggle-primary",
                                        checked: true
                                    }
                                }
                            }
                            div {
                                class: "form-control",
                                label {
                                    class: "label cursor-pointer",
                                    span {
                                        class: "label-text",
                                        "⚠️ Security Alerts"
                                    }
                                    input {
                                        "type": "checkbox",
                                        class: "toggle toggle-warning",
                                        checked: true
                                    }
                                }
                            }
                            div {
                                class: "form-control",
                                label {
                                    class: "label cursor-pointer",
                                    span {
                                        class: "label-text",
                                        "📊 Usage Reports"
                                    }
                                    input {
                                        "type": "checkbox",
                                        class: "toggle toggle-info",
                                        checked: false
                                    }
                                }
                            }
                        }
                    }

                    // 通知方式设置
                    div {
                        h5 {
                            class: "font-semibold mb-2",
                            "Delivery Methods"
                        }
                        div {
                            class: "space-y-2",
                            div {
                                class: "form-control",
                                label {
                                    class: "label cursor-pointer",
                                    span {
                                        class: "label-text",
                                        "🌐 In-App Notifications"
                                    }
                                    input {
                                        "type": "checkbox",
                                        class: "toggle toggle-primary",
                                        checked: true
                                    }
                                }
                            }
                            div {
                                class: "form-control",
                                label {
                                    class: "label cursor-pointer",
                                    span {
                                        class: "label-text",
                                        "📧 Email Notifications"
                                    }
                                    input {
                                        "type": "checkbox",
                                        class: "toggle toggle-secondary",
                                        checked: false
                                    }
                                }
                            }
                        }
                    }

                    // 保存按钮
                    div {
                        class: "pt-4",
                        button {
                            class: "btn btn-primary",
                            "💾 Save Preferences"
                        }
                    }
                }
            }
        }
    }
}
