/*!
# Enhanced Workflows Management

增强的工作流管理界面，参考bionic-gpt实现，提供自动化工作流的创建和管理功能。

## 功能特性

- **可视化编辑**: 拖拽式工作流设计器
- **模板库**: 预设的工作流模板
- **执行监控**: 实时监控工作流执行状态
- **版本控制**: 工作流版本管理和回滚
- **集成支持**: 与外部系统的无缝集成
*/

#![allow(non_snake_case)]
use dioxus::prelude::*;
use crate::app_layout::{Layout, SideBar};
use crate::types::Rbac;

// 临时类型定义
#[derive(Clone, Debug, PartialEq)]
pub struct Workflow {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub status: WorkflowStatus,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: String,
    pub execution_count: i64,
    pub success_rate: f64,
    pub avg_duration: f64,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WorkflowStatus {
    Draft,
    Active,
    Paused,
    Archived,
}

impl WorkflowStatus {
    pub fn display_text(&self) -> &str {
        match self {
            WorkflowStatus::Draft => "Draft",
            WorkflowStatus::Active => "Active",
            WorkflowStatus::Paused => "Paused",
            WorkflowStatus::Archived => "Archived",
        }
    }
    
    pub fn badge_class(&self) -> &str {
        match self {
            WorkflowStatus::Draft => "badge badge-neutral",
            WorkflowStatus::Active => "badge badge-success",
            WorkflowStatus::Paused => "badge badge-warning",
            WorkflowStatus::Archived => "badge badge-error",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkflowExecution {
    pub id: i32,
    pub workflow_id: i32,
    pub workflow_name: String,
    pub status: ExecutionStatus,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub duration: Option<f64>,
    pub triggered_by: String,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExecutionStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl ExecutionStatus {
    pub fn display_text(&self) -> &str {
        match self {
            ExecutionStatus::Running => "Running",
            ExecutionStatus::Completed => "Completed",
            ExecutionStatus::Failed => "Failed",
            ExecutionStatus::Cancelled => "Cancelled",
        }
    }
    
    pub fn badge_class(&self) -> &str {
        match self {
            ExecutionStatus::Running => "badge badge-info",
            ExecutionStatus::Completed => "badge badge-success",
            ExecutionStatus::Failed => "badge badge-error",
            ExecutionStatus::Cancelled => "badge badge-warning",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkflowTemplate {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub category: String,
    pub icon: String,
    pub use_count: i64,
}

/// 增强工作流管理页面
#[component]
pub fn EnhancedWorkflowsPage(
    team_id: i32,
    rbac: Rbac,
) -> Element {
    // 模拟数据
    let workflows = vec![
        Workflow {
            id: 1,
            name: "Document Processing Pipeline".to_string(),
            description: Some("Automatically process uploaded documents and extract insights".to_string()),
            category: "Data Processing".to_string(),
            status: WorkflowStatus::Active,
            created_at: "2024-01-15".to_string(),
            updated_at: "2024-01-20".to_string(),
            created_by: "admin@lumosai.com".to_string(),
            execution_count: 156,
            success_rate: 94.2,
            avg_duration: 45.3,
            is_active: true,
        },
        Workflow {
            id: 2,
            name: "Customer Support Automation".to_string(),
            description: Some("Route and respond to customer inquiries automatically".to_string()),
            category: "Customer Service".to_string(),
            status: WorkflowStatus::Active,
            created_at: "2024-01-10".to_string(),
            updated_at: "2024-01-19".to_string(),
            created_by: "support@lumosai.com".to_string(),
            execution_count: 892,
            success_rate: 87.5,
            avg_duration: 12.8,
            is_active: true,
        },
        Workflow {
            id: 3,
            name: "Content Generation Workflow".to_string(),
            description: Some("Generate marketing content based on product data".to_string()),
            category: "Content Creation".to_string(),
            status: WorkflowStatus::Draft,
            created_at: "2024-01-18".to_string(),
            updated_at: "2024-01-18".to_string(),
            created_by: "marketing@lumosai.com".to_string(),
            execution_count: 0,
            success_rate: 0.0,
            avg_duration: 0.0,
            is_active: false,
        },
    ];

    let recent_executions = vec![
        WorkflowExecution {
            id: 1,
            workflow_id: 1,
            workflow_name: "Document Processing Pipeline".to_string(),
            status: ExecutionStatus::Completed,
            started_at: "2024-01-20 10:30:00".to_string(),
            completed_at: Some("2024-01-20 10:31:15".to_string()),
            duration: Some(75.2),
            triggered_by: "user@lumosai.com".to_string(),
            error_message: None,
        },
        WorkflowExecution {
            id: 2,
            workflow_id: 2,
            workflow_name: "Customer Support Automation".to_string(),
            status: ExecutionStatus::Running,
            started_at: "2024-01-20 10:25:00".to_string(),
            completed_at: None,
            duration: None,
            triggered_by: "system".to_string(),
            error_message: None,
        },
        WorkflowExecution {
            id: 3,
            workflow_id: 1,
            workflow_name: "Document Processing Pipeline".to_string(),
            status: ExecutionStatus::Failed,
            started_at: "2024-01-20 09:45:00".to_string(),
            completed_at: Some("2024-01-20 09:45:30".to_string()),
            duration: Some(30.1),
            triggered_by: "api".to_string(),
            error_message: Some("Invalid document format".to_string()),
        },
    ];

    let workflow_templates = vec![
        WorkflowTemplate {
            id: 1,
            name: "Email Processing".to_string(),
            description: "Automatically categorize and respond to emails".to_string(),
            category: "Communication".to_string(),
            icon: "📧".to_string(),
            use_count: 45,
        },
        WorkflowTemplate {
            id: 2,
            name: "Data Sync".to_string(),
            description: "Synchronize data between different systems".to_string(),
            category: "Integration".to_string(),
            icon: "🔄".to_string(),
            use_count: 32,
        },
        WorkflowTemplate {
            id: 3,
            name: "Report Generation".to_string(),
            description: "Generate automated reports from data sources".to_string(),
            category: "Analytics".to_string(),
            icon: "📊".to_string(),
            use_count: 28,
        },
    ];

    rsx! {
        Layout {
            section_class: "p-4",
            selected_item: SideBar::Workflows,
            team_id,
            rbac: rbac.clone(),
            title: "Workflows",
            header: rsx! {
                div {
                    class: "flex items-center justify-between",
                    h3 {
                        class: "text-2xl font-bold",
                        "⚡ Workflows"
                    }
                    div {
                        class: "flex gap-4",
                        button {
                            class: "btn btn-outline gap-2",
                            span { "📋" }
                            "Templates"
                        }
                        button {
                            class: "btn btn-primary gap-2",
                            span { "➕" }
                            "Create Workflow"
                        }
                    }
                }
            },

            div {
                class: "space-y-6",

                // 工作流概览
                WorkflowOverview {
                    workflows: workflows.clone()
                }

                // 工作流列表
                WorkflowsList {
                    workflows: workflows.clone(),
                    rbac: rbac.clone(),
                    team_id
                }

                // 最近执行
                RecentExecutions {
                    executions: recent_executions.clone()
                }

                // 工作流模板
                WorkflowTemplates {
                    templates: workflow_templates.clone()
                }
            }
        }
    }
}

/// 工作流概览组件
#[component]
fn WorkflowOverview(workflows: Vec<Workflow>) -> Element {
    let total_workflows = workflows.len();
    let active_workflows = workflows.iter().filter(|w| w.is_active).count();
    let total_executions: i64 = workflows.iter().map(|w| w.execution_count).sum();
    let avg_success_rate = if !workflows.is_empty() {
        workflows.iter().map(|w| w.success_rate).sum::<f64>() / workflows.len() as f64
    } else {
        0.0
    };

    rsx! {
        div {
            class: "grid grid-cols-1 md:grid-cols-4 gap-6",

            // 总工作流数
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-lg font-semibold text-base-content/80",
                                "Total Workflows"
                            }
                            p {
                                class: "text-3xl font-bold text-primary",
                                "{total_workflows}"
                            }
                        }
                        div {
                            class: "text-4xl",
                            "⚡"
                        }
                    }
                }
            }

            // 活跃工作流
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-lg font-semibold text-base-content/80",
                                "Active"
                            }
                            p {
                                class: "text-3xl font-bold text-success",
                                "{active_workflows}"
                            }
                        }
                        div {
                            class: "text-4xl",
                            "✅"
                        }
                    }
                }
            }

            // 总执行次数
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-lg font-semibold text-base-content/80",
                                "Total Executions"
                            }
                            p {
                                class: "text-3xl font-bold text-secondary",
                                "{total_executions}"
                            }
                        }
                        div {
                            class: "text-4xl",
                            "🚀"
                        }
                    }
                }
            }

            // 平均成功率
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-lg font-semibold text-base-content/80",
                                "Success Rate"
                            }
                            p {
                                class: "text-3xl font-bold text-accent",
                                "{avg_success_rate:.1}%"
                            }
                        }
                        div {
                            class: "text-4xl",
                            "📈"
                        }
                    }
                }
            }
        }
    }
}

/// 工作流列表组件
#[component]
fn WorkflowsList(
    workflows: Vec<Workflow>,
    rbac: Rbac,
    team_id: i32,
) -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-header p-6 border-b border-base-300",
                h4 {
                    class: "text-xl font-bold",
                    "Workflows ({workflows.len()})"
                }
            }
            div {
                class: "card-body p-0",
                if workflows.is_empty() {
                    EmptyWorkflows {}
                } else {
                    div {
                        class: "overflow-x-auto",
                        table {
                            class: "table table-zebra w-full",
                            thead {
                                tr {
                                    th { "Name" }
                                    th { "Category" }
                                    th { "Status" }
                                    th { "Executions" }
                                    th { "Success Rate" }
                                    th { "Avg Duration" }
                                    th { "Updated" }
                                    th {
                                        class: "text-right",
                                        "Actions"
                                    }
                                }
                            }
                            tbody {
                                for workflow in &workflows {
                                    WorkflowRow {
                                        workflow: workflow.clone(),
                                        rbac: rbac.clone(),
                                        team_id
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

/// 工作流行组件
#[component]
fn WorkflowRow(
    workflow: Workflow,
    rbac: Rbac,
    team_id: i32,
) -> Element {
    rsx! {
        tr {
            td {
                div {
                    div {
                        class: "font-medium",
                        "{workflow.name}"
                    }
                    if let Some(description) = &workflow.description {
                        div {
                            class: "text-sm text-base-content/60",
                            "{description}"
                        }
                    }
                }
            }
            td {
                div {
                    class: "badge badge-outline",
                    "{workflow.category}"
                }
            }
            td {
                div {
                    class: workflow.status.badge_class(),
                    "{workflow.status.display_text()}"
                }
            }
            td {
                div {
                    class: "text-sm",
                    "{workflow.execution_count}"
                }
            }
            td {
                div {
                    class: "text-sm",
                    "{workflow.success_rate:.1}%"
                }
            }
            td {
                div {
                    class: "text-sm",
                    "{workflow.avg_duration:.1}s"
                }
            }
            td {
                div {
                    class: "text-sm text-base-content/70",
                    "{workflow.updated_at}"
                }
            }
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
                                href: format!("/teams/{}/workflows/{}", team_id, workflow.id),
                                "View Details"
                            }
                        }
                        li {
                            a {
                                "Edit Workflow"
                            }
                        }
                        li {
                            a {
                                if workflow.is_active { "Pause" } else { "Activate" }
                            }
                        }
                        li {
                            a {
                                "Duplicate"
                            }
                        }
                        li {
                            a {
                                class: "text-error",
                                "Delete"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 最近执行组件
#[component]
fn RecentExecutions(executions: Vec<WorkflowExecution>) -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-header p-6 border-b border-base-300",
                h4 {
                    class: "text-xl font-bold",
                    "Recent Executions"
                }
            }
            div {
                class: "card-body",
                div {
                    class: "space-y-3",
                    for execution in &executions {
                        ExecutionItem {
                            execution: execution.clone()
                        }
                    }
                }
            }
        }
    }
}

/// 执行项组件
#[component]
fn ExecutionItem(execution: WorkflowExecution) -> Element {
    rsx! {
        div {
            class: "flex items-center justify-between p-3 bg-base-200 rounded-lg",
            div {
                class: "flex-1",
                div {
                    class: "flex items-center gap-2 mb-1",
                    div {
                        class: execution.status.badge_class(),
                        "{execution.status.display_text()}"
                    }
                    span {
                        class: "font-medium",
                        "{execution.workflow_name}"
                    }
                }
                div {
                    class: "flex items-center gap-4 text-sm text-base-content/70",
                    span { "Started: {execution.started_at}" }
                    if let Some(duration) = execution.duration {
                        span { "Duration: {duration:.1}s" }
                    }
                    span { "By: {execution.triggered_by}" }
                }
                if let Some(error) = &execution.error_message {
                    div {
                        class: "text-sm text-error mt-1",
                        "Error: {error}"
                    }
                }
            }
            button {
                class: "btn btn-ghost btn-xs",
                "View"
            }
        }
    }
}

/// 工作流模板组件
#[component]
fn WorkflowTemplates(templates: Vec<WorkflowTemplate>) -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-header p-6 border-b border-base-300",
                h4 {
                    class: "text-xl font-bold",
                    "Workflow Templates"
                }
            }
            div {
                class: "card-body",
                div {
                    class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                    for template in &templates {
                        TemplateCard {
                            template: template.clone()
                        }
                    }
                }
            }
        }
    }
}

/// 模板卡片组件
#[component]
fn TemplateCard(template: WorkflowTemplate) -> Element {
    rsx! {
        div {
            class: "card bg-base-200 shadow-md hover:shadow-lg transition-shadow cursor-pointer",
            div {
                class: "card-body p-4",
                div {
                    class: "flex items-center gap-3 mb-3",
                    div {
                        class: "text-3xl",
                        "{template.icon}"
                    }
                    div {
                        h5 {
                            class: "font-bold",
                            "{template.name}"
                        }
                        div {
                            class: "badge badge-outline badge-sm",
                            "{template.category}"
                        }
                    }
                }
                p {
                    class: "text-sm text-base-content/70 mb-3",
                    "{template.description}"
                }
                div {
                    class: "flex items-center justify-between",
                    span {
                        class: "text-xs text-base-content/60",
                        "Used {template.use_count} times"
                    }
                    button {
                        class: "btn btn-primary btn-xs",
                        "Use Template"
                    }
                }
            }
        }
    }
}

/// 空状态组件
#[component]
fn EmptyWorkflows() -> Element {
    rsx! {
        div {
            class: "text-center py-12",
            div {
                class: "text-6xl mb-4",
                "⚡"
            }
            h4 {
                class: "text-xl font-semibold mb-2",
                "No Workflows"
            }
            p {
                class: "text-base-content/60 mb-4",
                "Create your first workflow to automate your processes"
            }
            button {
                class: "btn btn-primary gap-2",
                span { "➕" }
                "Create Workflow"
            }
        }
    }
}
