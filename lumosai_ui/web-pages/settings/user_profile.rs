/*!
# User Profile Settings

用户个人设置界面，参考bionic-gpt实现，提供个人信息管理和偏好设置功能。

## 功能特性

- **个人信息**: 姓名、邮箱、头像管理
- **偏好设置**: 主题、语言、通知设置
- **安全设置**: 密码修改、两步验证
- **使用统计**: 个人使用数据展示
*/

#![allow(non_snake_case)]
use crate::app_layout::{Layout, SideBar};
use crate::types::Rbac;
use dioxus::prelude::*;

// 临时类型定义
#[derive(Clone, Debug, PartialEq)]
pub struct UserProfile {
    pub id: i32,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: String,
    pub last_login: Option<String>,
    pub timezone: String,
    pub language: String,
    pub theme: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserStats {
    pub total_conversations: i64,
    pub total_messages: i64,
    pub total_tokens_used: i64,
    pub favorite_assistants: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NotificationSettings {
    pub email_notifications: bool,
    pub push_notifications: bool,
    pub weekly_summary: bool,
    pub security_alerts: bool,
}

/// 用户设置页面
#[component]
pub fn UserProfilePage(team_id: i32, rbac: Rbac) -> Element {
    // 模拟数据
    let user_profile = UserProfile {
        id: 1,
        email: "user@lumosai.com".to_string(),
        first_name: Some("张".to_string()),
        last_name: Some("三".to_string()),
        avatar_url: None,
        created_at: "2024-01-01".to_string(),
        last_login: Some("2024-01-20 10:30".to_string()),
        timezone: "Asia/Shanghai".to_string(),
        language: "zh-CN".to_string(),
        theme: "auto".to_string(),
    };

    let user_stats = UserStats {
        total_conversations: 156,
        total_messages: 1240,
        total_tokens_used: 89500,
        favorite_assistants: vec![
            "Code Assistant".to_string(),
            "Writing Helper".to_string(),
            "Data Analyst".to_string(),
        ],
    };

    let notification_settings = NotificationSettings {
        email_notifications: true,
        push_notifications: false,
        weekly_summary: true,
        security_alerts: true,
    };

    rsx! {
        Layout {
            section_class: "p-4",
            selected_item: SideBar::Profile,
            team_id,
            rbac: rbac.clone(),
            title: "Profile Settings",
            header: rsx! {
                div {
                    class: "flex items-center justify-between",
                    h3 {
                        class: "text-2xl font-bold",
                        "👤 Profile Settings"
                    }
                }
            },

            div {
                class: "max-w-4xl mx-auto space-y-6",

                // 个人信息卡片
                ProfileInfoCard {
                    user_profile: user_profile.clone()
                }

                // 使用统计
                UserStatsCard {
                    user_stats: user_stats.clone()
                }

                // 偏好设置
                PreferencesCard {
                    user_profile: user_profile.clone()
                }

                // 通知设置
                NotificationCard {
                    notification_settings: notification_settings.clone()
                }

                // 安全设置
                SecurityCard {}
            }
        }
    }
}

/// 个人信息卡片
#[component]
fn ProfileInfoCard(user_profile: UserProfile) -> Element {
    let _display_name =
        if let (Some(first), Some(last)) = (&user_profile.first_name, &user_profile.last_name) {
            format!("{} {}", first, last)
        } else {
            "未设置姓名".to_string()
        };

    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-body",
                h4 {
                    class: "text-xl font-bold mb-6",
                    "📝 Personal Information"
                }

                div {
                    class: "flex items-start space-x-6",

                    // 头像区域
                    div {
                        class: "flex flex-col items-center space-y-4",
                        div {
                            class: "avatar placeholder",
                            div {
                                class: "bg-neutral text-neutral-content rounded-full w-24 h-24",
                                span {
                                    class: "text-2xl",
                                    if let Some(first) = &user_profile.first_name {
                                        "{first.chars().next().unwrap_or('?')}"
                                    } else {
                                        "{user_profile.email.chars().next().unwrap_or('?')}"
                                    }
                                }
                            }
                        }
                        button {
                            class: "btn btn-outline btn-sm",
                            "Change Avatar"
                        }
                    }

                    // 信息表单
                    div {
                        class: "flex-1 space-y-4",

                        div {
                            class: "grid grid-cols-1 md:grid-cols-2 gap-4",

                            div {
                                label {
                                    class: "label",
                                    span {
                                        class: "label-text",
                                        "First Name"
                                    }
                                }
                                input {
                                    r#type: "text",
                                    class: "input input-bordered w-full",
                                    value: user_profile.first_name.unwrap_or_default(),
                                    placeholder: "Enter first name"
                                }
                            }

                            div {
                                label {
                                    class: "label",
                                    span {
                                        class: "label-text",
                                        "Last Name"
                                    }
                                }
                                input {
                                    r#type: "text",
                                    class: "input input-bordered w-full",
                                    value: user_profile.last_name.unwrap_or_default(),
                                    placeholder: "Enter last name"
                                }
                            }
                        }

                        div {
                            label {
                                class: "label",
                                span {
                                    class: "label-text",
                                    "Email Address"
                                }
                            }
                            input {
                                r#type: "email",
                                class: "input input-bordered w-full",
                                value: user_profile.email,
                                disabled: true
                            }
                            div {
                                class: "label",
                                span {
                                    class: "label-text-alt text-base-content/60",
                                    "Email cannot be changed. Contact support if needed."
                                }
                            }
                        }

                        div {
                            class: "flex justify-end",
                            button {
                                class: "btn btn-primary",
                                "Save Changes"
                            }
                        }
                    }
                }

                // 账户信息
                div {
                    class: "mt-6 pt-6 border-t border-base-300",
                    div {
                        class: "grid grid-cols-1 md:grid-cols-3 gap-4 text-sm text-base-content/70",
                        div {
                            span { "📅 Member since: " }
                            span { "{user_profile.created_at}" }
                        }
                        div {
                            span { "🕐 Last login: " }
                            span { "{user_profile.last_login.as_ref().map(|s| s.as_str()).unwrap_or(\"Never\")}" }
                        }
                        div {
                            span { "🌍 Timezone: " }
                            span { "{user_profile.timezone}" }
                        }
                    }
                }
            }
        }
    }
}

/// 使用统计卡片
#[component]
fn UserStatsCard(user_stats: UserStats) -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-body",
                h4 {
                    class: "text-xl font-bold mb-6",
                    "📊 Usage Statistics"
                }

                div {
                    class: "grid grid-cols-1 md:grid-cols-3 gap-6",

                    div {
                        class: "stat",
                        div {
                            class: "stat-figure text-primary",
                            "💬"
                        }
                        div {
                            class: "stat-title",
                            "Conversations"
                        }
                        div {
                            class: "stat-value text-primary",
                            "{user_stats.total_conversations}"
                        }
                    }

                    div {
                        class: "stat",
                        div {
                            class: "stat-figure text-secondary",
                            "📝"
                        }
                        div {
                            class: "stat-title",
                            "Messages"
                        }
                        div {
                            class: "stat-value text-secondary",
                            "{user_stats.total_messages}"
                        }
                    }

                    div {
                        class: "stat",
                        div {
                            class: "stat-figure text-accent",
                            "🪙"
                        }
                        div {
                            class: "stat-title",
                            "Tokens Used"
                        }
                        div {
                            class: "stat-value text-accent",
                            "{user_stats.total_tokens_used}"
                        }
                    }
                }

                // 常用助手
                div {
                    class: "mt-6 pt-6 border-t border-base-300",
                    h5 {
                        class: "font-semibold mb-3",
                        "⭐ Favorite Assistants"
                    }
                    div {
                        class: "flex flex-wrap gap-2",
                        for assistant in &user_stats.favorite_assistants {
                            div {
                                class: "badge badge-outline",
                                "{assistant}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 偏好设置卡片
#[component]
fn PreferencesCard(user_profile: UserProfile) -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-body",
                h4 {
                    class: "text-xl font-bold mb-6",
                    "⚙️ Preferences"
                }

                div {
                    class: "space-y-6",

                    div {
                        class: "grid grid-cols-1 md:grid-cols-2 gap-6",

                        div {
                            label {
                                class: "label",
                                span {
                                    class: "label-text",
                                    "Language"
                                }
                            }
                            select {
                                class: "select select-bordered w-full",
                                value: user_profile.language,

                                option { value: "zh-CN", "中文 (简体)" }
                                option { value: "zh-TW", "中文 (繁体)" }
                                option { value: "en-US", "English (US)" }
                                option { value: "ja-JP", "日本語" }
                                option { value: "ko-KR", "한국어" }
                            }
                        }

                        div {
                            label {
                                class: "label",
                                span {
                                    class: "label-text",
                                    "Theme"
                                }
                            }
                            select {
                                class: "select select-bordered w-full",
                                value: user_profile.theme,

                                option { value: "auto", "🌓 Auto (System)" }
                                option { value: "light", "☀️ Light" }
                                option { value: "dark", "🌙 Dark" }
                            }
                        }
                    }

                    div {
                        label {
                            class: "label",
                            span {
                                class: "label-text",
                                "Timezone"
                            }
                        }
                        select {
                            class: "select select-bordered w-full",
                            value: user_profile.timezone,

                            option { value: "Asia/Shanghai", "Asia/Shanghai (UTC+8)" }
                            option { value: "Asia/Tokyo", "Asia/Tokyo (UTC+9)" }
                            option { value: "America/New_York", "America/New_York (UTC-5)" }
                            option { value: "Europe/London", "Europe/London (UTC+0)" }
                        }
                    }

                    div {
                        class: "flex justify-end",
                        button {
                            class: "btn btn-primary",
                            "Save Preferences"
                        }
                    }
                }
            }
        }
    }
}

/// 通知设置卡片
#[component]
fn NotificationCard(notification_settings: NotificationSettings) -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-body",
                h4 {
                    class: "text-xl font-bold mb-6",
                    "🔔 Notification Settings"
                }

                div {
                    class: "space-y-4",

                    div {
                        class: "form-control",
                        label {
                            class: "label cursor-pointer",
                            span {
                                class: "label-text",
                                "📧 Email Notifications"
                            }
                            input {
                                r#type: "checkbox",
                                class: "toggle toggle-primary",
                                checked: notification_settings.email_notifications
                            }
                        }
                        div {
                            class: "label",
                            span {
                                class: "label-text-alt text-base-content/60",
                                "Receive notifications about important updates via email"
                            }
                        }
                    }

                    div {
                        class: "form-control",
                        label {
                            class: "label cursor-pointer",
                            span {
                                class: "label-text",
                                "📱 Push Notifications"
                            }
                            input {
                                r#type: "checkbox",
                                class: "toggle toggle-primary",
                                checked: notification_settings.push_notifications
                            }
                        }
                        div {
                            class: "label",
                            span {
                                class: "label-text-alt text-base-content/60",
                                "Receive real-time notifications in your browser"
                            }
                        }
                    }

                    div {
                        class: "form-control",
                        label {
                            class: "label cursor-pointer",
                            span {
                                class: "label-text",
                                "📈 Weekly Summary"
                            }
                            input {
                                r#type: "checkbox",
                                class: "toggle toggle-primary",
                                checked: notification_settings.weekly_summary
                            }
                        }
                        div {
                            class: "label",
                            span {
                                class: "label-text-alt text-base-content/60",
                                "Get a weekly summary of your AI assistant usage"
                            }
                        }
                    }

                    div {
                        class: "form-control",
                        label {
                            class: "label cursor-pointer",
                            span {
                                class: "label-text",
                                "🔒 Security Alerts"
                            }
                            input {
                                r#type: "checkbox",
                                class: "toggle toggle-primary",
                                checked: notification_settings.security_alerts
                            }
                        }
                        div {
                            class: "label",
                            span {
                                class: "label-text-alt text-base-content/60",
                                "Important security notifications (recommended)"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 安全设置卡片
#[component]
fn SecurityCard() -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-body",
                h4 {
                    class: "text-xl font-bold mb-6",
                    "🔒 Security Settings"
                }

                div {
                    class: "space-y-6",

                    div {
                        class: "flex items-center justify-between p-4 bg-base-200 rounded-lg",
                        div {
                            h5 {
                                class: "font-semibold",
                                "🔑 Change Password"
                            }
                            p {
                                class: "text-sm text-base-content/60",
                                "Update your account password"
                            }
                        }
                        button {
                            class: "btn btn-outline btn-sm",
                            "Change"
                        }
                    }

                    div {
                        class: "flex items-center justify-between p-4 bg-base-200 rounded-lg",
                        div {
                            h5 {
                                class: "font-semibold",
                                "📱 Two-Factor Authentication"
                            }
                            p {
                                class: "text-sm text-base-content/60",
                                "Add an extra layer of security to your account"
                            }
                        }
                        button {
                            class: "btn btn-primary btn-sm",
                            "Enable"
                        }
                    }

                    div {
                        class: "flex items-center justify-between p-4 bg-base-200 rounded-lg",
                        div {
                            h5 {
                                class: "font-semibold",
                                "📋 Active Sessions"
                            }
                            p {
                                class: "text-sm text-base-content/60",
                                "Manage your active login sessions"
                            }
                        }
                        button {
                            class: "btn btn-outline btn-sm",
                            "View"
                        }
                    }
                }
            }
        }
    }
}
