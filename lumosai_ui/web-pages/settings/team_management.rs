/*!
# Team Management

团队管理界面，参考bionic-gpt实现，提供团队成员管理和权限控制功能。

## 功能特性

- **成员管理**: 邀请、移除团队成员
- **权限控制**: 基于角色的权限管理
- **邀请管理**: 管理待处理的邀请
- **团队设置**: 团队基本信息配置
*/

#![allow(non_snake_case)]
use crate::app_layout::{Layout, SideBar};
use crate::types::Rbac;
use dioxus::prelude::*;

// 临时类型定义
#[derive(Clone, Debug, PartialEq)]
pub struct TeamMember {
    pub id: i32,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: String,
    pub joined_at: String,
    pub last_active: Option<String>,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TeamInvitation {
    pub id: i32,
    pub email: String,
    pub role: String,
    pub invited_by: String,
    pub invited_at: String,
    pub expires_at: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TeamInfo {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub member_count: usize,
}

/// 团队管理页面
#[component]
pub fn TeamManagementPage(team_id: i32, rbac: Rbac) -> Element {
    // 模拟数据
    let team_info = TeamInfo {
        id: team_id,
        name: "Lumos AI Team".to_string(),
        description: Some("AI开发团队，专注于智能助手和自动化解决方案".to_string()),
        created_at: "2024-01-01".to_string(),
        member_count: 5,
    };

    let members = vec![
        TeamMember {
            id: 1,
            email: "admin@lumosai.com".to_string(),
            first_name: Some("张".to_string()),
            last_name: Some("三".to_string()),
            role: "Owner".to_string(),
            joined_at: "2024-01-01".to_string(),
            last_active: Some("2024-01-20 10:30".to_string()),
            is_active: true,
        },
        TeamMember {
            id: 2,
            email: "dev@lumosai.com".to_string(),
            first_name: Some("李".to_string()),
            last_name: Some("四".to_string()),
            role: "Developer".to_string(),
            joined_at: "2024-01-05".to_string(),
            last_active: Some("2024-01-20 09:15".to_string()),
            is_active: true,
        },
        TeamMember {
            id: 3,
            email: "designer@lumosai.com".to_string(),
            first_name: Some("王".to_string()),
            last_name: Some("五".to_string()),
            role: "Designer".to_string(),
            joined_at: "2024-01-10".to_string(),
            last_active: Some("2024-01-19 16:45".to_string()),
            is_active: true,
        },
    ];

    let invitations = vec![TeamInvitation {
        id: 1,
        email: "newdev@lumosai.com".to_string(),
        role: "Developer".to_string(),
        invited_by: "admin@lumosai.com".to_string(),
        invited_at: "2024-01-18".to_string(),
        expires_at: "2024-01-25".to_string(),
    }];

    rsx! {
        Layout {
            section_class: "p-4",
            selected_item: SideBar::Team,
            team_id,
            rbac: rbac.clone(),
            title: "Team Management",
            header: rsx! {
                div {
                    class: "flex items-center justify-between",
                    h3 {
                        class: "text-2xl font-bold",
                        "👥 Team Management"
                    }
                    div {
                        class: "flex gap-4",
                        if rbac.can_make_invitations() {
                            button {
                                class: "btn btn-primary gap-2",
                                onclick: move |_| {
                                    // TODO: 打开邀请成员模态框
                                },
                                span { "✉️" }
                                "Invite Member"
                            }
                        }
                    }
                }
            },

            div {
                class: "space-y-6",

                // 团队信息概览
                TeamOverview {
                    team_info: team_info.clone()
                }

                // 团队成员列表
                TeamMembersList {
                    members: members.clone(),
                    rbac: rbac.clone()
                }

                // 待处理邀请
                if !invitations.is_empty() {
                    PendingInvitations {
                        invitations: invitations.clone(),
                        rbac: rbac.clone()
                    }
                }

                // 角色权限说明
                RolePermissions {}
            }
        }
    }
}

/// 团队概览组件
#[component]
fn TeamOverview(team_info: TeamInfo) -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-body",
                div {
                    class: "flex items-start justify-between",
                    div {
                        h4 {
                            class: "text-2xl font-bold mb-2",
                            "{team_info.name}"
                        }
                        if let Some(description) = &team_info.description {
                            p {
                                class: "text-base-content/70 mb-4",
                                "{description}"
                            }
                        }
                        div {
                            class: "flex items-center space-x-6 text-sm text-base-content/60",
                            div {
                                span { "📅 Created: " }
                                span { "{team_info.created_at}" }
                            }
                            div {
                                span { "👥 Members: " }
                                span { "{team_info.member_count}" }
                            }
                        }
                    }
                    div {
                        class: "text-6xl",
                        "🏢"
                    }
                }
            }
        }
    }
}

/// 团队成员列表组件
#[component]
fn TeamMembersList(members: Vec<TeamMember>, rbac: Rbac) -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-header p-6 border-b border-base-300",
                h4 {
                    class: "text-xl font-bold",
                    "Team Members ({members.len()})"
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
                                th { "Member" }
                                th { "Role" }
                                th { "Joined" }
                                th { "Last Active" }
                                th { "Status" }
                                if rbac.can_make_invitations() {
                                    th {
                                        class: "text-right",
                                        "Actions"
                                    }
                                }
                            }
                        }
                        tbody {
                            for member in &members {
                                TeamMemberRow {
                                    member: member.clone(),
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

/// 团队成员行组件
#[component]
fn TeamMemberRow(member: TeamMember, rbac: Rbac) -> Element {
    let display_name = if let (Some(first), Some(last)) = (&member.first_name, &member.last_name) {
        format!("{} {}", first, last)
    } else {
        member.email.clone()
    };

    rsx! {
        tr {
            td {
                div {
                    class: "flex items-center space-x-3",
                    div {
                        class: "avatar placeholder",
                        div {
                            class: "bg-neutral text-neutral-content rounded-full w-8 h-8",
                            span {
                                class: "text-xs",
                                if let Some(first) = &member.first_name {
                                    "{first.chars().next().unwrap_or('?')}"
                                } else {
                                    "{member.email.chars().next().unwrap_or('?')}"
                                }
                            }
                        }
                    }
                    div {
                        div {
                            class: "font-medium",
                            "{display_name}"
                        }
                        div {
                            class: "text-sm text-base-content/60",
                            "{member.email}"
                        }
                    }
                }
            }
            td {
                div {
                    class: match member.role.as_str() {
                        "Owner" => "badge badge-primary",
                        "Admin" => "badge badge-secondary",
                        "Developer" => "badge badge-accent",
                        _ => "badge badge-neutral"
                    },
                    "{member.role}"
                }
            }
            td {
                div {
                    class: "text-sm text-base-content/70",
                    "{member.joined_at}"
                }
            }
            td {
                if let Some(last_active) = &member.last_active {
                    div {
                        class: "text-sm text-base-content/70",
                        "{last_active}"
                    }
                } else {
                    div {
                        class: "text-sm text-base-content/50",
                        "Never"
                    }
                }
            }
            td {
                if member.is_active {
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
            if rbac.can_make_invitations() && member.role != "Owner" {
                td {
                    class: "text-right",
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
                                    "Change Role"
                                }
                            }
                            li {
                                a {
                                    class: "text-error",
                                    "Remove Member"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 待处理邀请组件
#[component]
fn PendingInvitations(invitations: Vec<TeamInvitation>, rbac: Rbac) -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-header p-6 border-b border-base-300",
                h4 {
                    class: "text-xl font-bold",
                    "Pending Invitations ({invitations.len()})"
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
                                th { "Email" }
                                th { "Role" }
                                th { "Invited By" }
                                th { "Invited At" }
                                th { "Expires" }
                                if rbac.can_make_invitations() {
                                    th {
                                        class: "text-right",
                                        "Actions"
                                    }
                                }
                            }
                        }
                        tbody {
                            for invitation in &invitations {
                                tr {
                                    td {
                                        div {
                                            class: "font-medium",
                                            "{invitation.email}"
                                        }
                                    }
                                    td {
                                        div {
                                            class: "badge badge-outline",
                                            "{invitation.role}"
                                        }
                                    }
                                    td {
                                        div {
                                            class: "text-sm text-base-content/70",
                                            "{invitation.invited_by}"
                                        }
                                    }
                                    td {
                                        div {
                                            class: "text-sm text-base-content/70",
                                            "{invitation.invited_at}"
                                        }
                                    }
                                    td {
                                        div {
                                            class: "text-sm text-warning",
                                            "{invitation.expires_at}"
                                        }
                                    }
                                    if rbac.can_make_invitations() {
                                        td {
                                            class: "text-right",
                                            div {
                                                class: "flex justify-end gap-2",
                                                button {
                                                    class: "btn btn-ghost btn-xs",
                                                    title: "Resend Invitation",
                                                    "📧"
                                                }
                                                button {
                                                    class: "btn btn-ghost btn-xs text-error",
                                                    title: "Cancel Invitation",
                                                    "❌"
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
    }
}

/// 角色权限说明组件
#[component]
fn RolePermissions() -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-body",
                h4 {
                    class: "text-xl font-bold mb-4",
                    "🔐 Role Permissions"
                }
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4",

                    div {
                        class: "p-4 bg-primary/10 rounded-lg",
                        h5 {
                            class: "font-semibold text-primary mb-2",
                            "👑 Owner"
                        }
                        ul {
                            class: "space-y-1 text-sm text-base-content/70",
                            li { "• Full team management" }
                            li { "• Billing and settings" }
                            li { "• Delete team" }
                        }
                    }

                    div {
                        class: "p-4 bg-secondary/10 rounded-lg",
                        h5 {
                            class: "font-semibold text-secondary mb-2",
                            "⚡ Admin"
                        }
                        ul {
                            class: "space-y-1 text-sm text-base-content/70",
                            li { "• Manage members" }
                            li { "• Manage API keys" }
                            li { "• View all data" }
                        }
                    }

                    div {
                        class: "p-4 bg-accent/10 rounded-lg",
                        h5 {
                            class: "font-semibold text-accent mb-2",
                            "💻 Developer"
                        }
                        ul {
                            class: "space-y-1 text-sm text-base-content/70",
                            li { "• Create assistants" }
                            li { "• Use API" }
                            li { "• View team data" }
                        }
                    }

                    div {
                        class: "p-4 bg-neutral/10 rounded-lg",
                        h5 {
                            class: "font-semibold text-neutral mb-2",
                            "👁️ Viewer"
                        }
                        ul {
                            class: "space-y-1 text-sm text-base-content/70",
                            li { "• View assistants" }
                            li { "• Basic chat access" }
                            li { "• Read-only access" }
                        }
                    }
                }
            }
        }
    }
}
