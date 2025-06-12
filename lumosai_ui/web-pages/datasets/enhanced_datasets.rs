/*!
# Enhanced Datasets Management

增强的数据集管理界面，参考bionic-gpt实现，提供数据集的创建、管理和监控功能。

## 功能特性

- **数据集管理**: 创建、编辑、删除数据集
- **文档上传**: 支持多种格式的文档上传
- **处理监控**: 实时监控数据处理状态
- **权限控制**: 基于RBAC的访问控制
- **搜索过滤**: 智能搜索和分类过滤
*/

#![allow(non_snake_case)]
use dioxus::prelude::*;
use crate::app_layout::{Layout, SideBar};
use crate::types::Rbac;

// 临时类型定义
#[derive(Clone, Debug, PartialEq)]
pub struct Dataset {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub visibility: String,
    pub created_at: String,
    pub updated_at: String,
    pub document_count: i64,
    pub total_size: i64,
    pub processing_status: String,
    pub model_id: Option<i32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Document {
    pub id: i32,
    pub name: String,
    pub file_type: String,
    pub file_size: i64,
    pub uploaded_at: String,
    pub processing_status: String,
    pub chunk_count: Option<i32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProcessingStats {
    pub total_documents: i64,
    pub processed_documents: i64,
    pub failed_documents: i64,
    pub total_chunks: i64,
    pub processing_time_avg: f64,
}

/// 增强数据集管理页面
#[component]
pub fn EnhancedDatasetsPage(
    team_id: i32,
    rbac: Rbac,
) -> Element {
    // 模拟数据
    let datasets = vec![
        Dataset {
            id: 1,
            name: "Product Documentation".to_string(),
            description: Some("Complete product documentation and user guides".to_string()),
            visibility: "Team".to_string(),
            created_at: "2024-01-15".to_string(),
            updated_at: "2024-01-20".to_string(),
            document_count: 45,
            total_size: 12500000, // 12.5MB
            processing_status: "Completed".to_string(),
            model_id: Some(1),
        },
        Dataset {
            id: 2,
            name: "Customer Support FAQ".to_string(),
            description: Some("Frequently asked questions and support articles".to_string()),
            visibility: "Private".to_string(),
            created_at: "2024-01-10".to_string(),
            updated_at: "2024-01-19".to_string(),
            document_count: 128,
            total_size: 8900000, // 8.9MB
            processing_status: "Processing".to_string(),
            model_id: Some(2),
        },
        Dataset {
            id: 3,
            name: "Technical Specifications".to_string(),
            description: None,
            visibility: "Company".to_string(),
            created_at: "2024-01-05".to_string(),
            updated_at: "2024-01-18".to_string(),
            document_count: 23,
            total_size: 15600000, // 15.6MB
            processing_status: "Failed".to_string(),
            model_id: None,
        },
    ];

    let processing_stats = ProcessingStats {
        total_documents: 196,
        processed_documents: 168,
        failed_documents: 5,
        total_chunks: 2847,
        processing_time_avg: 2.3,
    };

    rsx! {
        Layout {
            section_class: "p-4",
            selected_item: SideBar::Datasets,
            team_id,
            rbac: rbac.clone(),
            title: "Datasets",
            header: rsx! {
                div {
                    class: "flex items-center justify-between",
                    h3 {
                        class: "text-2xl font-bold",
                        "📊 Datasets"
                    }
                    div {
                        class: "flex gap-4",
                        if rbac.can_view_datasets() {
                            button {
                                class: "btn btn-primary gap-2",
                                onclick: move |_| {
                                    // TODO: 打开创建数据集模态框
                                },
                                span { "➕" }
                                "Create Dataset"
                            }
                        }
                    }
                }
            },

            div {
                class: "space-y-6",

                // 处理统计概览
                ProcessingOverview {
                    stats: processing_stats.clone()
                }

                // 数据集列表
                DatasetsList {
                    datasets: datasets.clone(),
                    rbac: rbac.clone(),
                    team_id
                }

                // 快速操作指南
                QuickActionsGuide {}
            }
        }
    }
}

/// 处理统计概览组件
#[component]
fn ProcessingOverview(stats: ProcessingStats) -> Element {
    let processing_rate = if stats.total_documents > 0 {
        (stats.processed_documents as f64 / stats.total_documents as f64) * 100.0
    } else {
        0.0
    };

    rsx! {
        div {
            class: "grid grid-cols-1 md:grid-cols-4 gap-6",

            // 总文档数
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-lg font-semibold text-base-content/80",
                                "Total Documents"
                            }
                            p {
                                class: "text-3xl font-bold text-primary",
                                "{stats.total_documents}"
                            }
                        }
                        div {
                            class: "text-4xl",
                            "📄"
                        }
                    }
                }
            }

            // 处理完成数
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-lg font-semibold text-base-content/80",
                                "Processed"
                            }
                            p {
                                class: "text-3xl font-bold text-success",
                                "{stats.processed_documents}"
                            }
                        }
                        div {
                            class: "text-4xl",
                            "✅"
                        }
                    }
                    div {
                        class: "mt-2",
                        div {
                            class: "w-full bg-base-200 rounded-full h-2",
                            div {
                                class: "bg-success h-2 rounded-full",
                                style: "width: {processing_rate}%"
                            }
                        }
                        p {
                            class: "text-xs text-base-content/60 mt-1",
                            "{processing_rate:.1}% complete"
                        }
                    }
                }
            }

            // 失败数
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-lg font-semibold text-base-content/80",
                                "Failed"
                            }
                            p {
                                class: "text-3xl font-bold text-error",
                                "{stats.failed_documents}"
                            }
                        }
                        div {
                            class: "text-4xl",
                            "❌"
                        }
                    }
                }
            }

            // 总块数
            div {
                class: "card bg-base-100 shadow-lg",
                div {
                    class: "card-body",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 {
                                class: "text-lg font-semibold text-base-content/80",
                                "Total Chunks"
                            }
                            p {
                                class: "text-3xl font-bold text-secondary",
                                "{stats.total_chunks}"
                            }
                        }
                        div {
                            class: "text-4xl",
                            "🧩"
                        }
                    }
                    p {
                        class: "text-sm text-base-content/60",
                        "Avg: {stats.processing_time_avg:.1}s per doc"
                    }
                }
            }
        }
    }
}

/// 数据集列表组件
#[component]
fn DatasetsList(
    datasets: Vec<Dataset>,
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
                    "Datasets ({datasets.len()})"
                }
            }
            div {
                class: "card-body p-0",
                if datasets.is_empty() {
                    EmptyDatasets {}
                } else {
                    div {
                        class: "overflow-x-auto",
                        table {
                            class: "table table-zebra w-full",
                            thead {
                                tr {
                                    th { "Name" }
                                    th { "Documents" }
                                    th { "Size" }
                                    th { "Status" }
                                    th { "Visibility" }
                                    th { "Updated" }
                                    if rbac.can_view_datasets() {
                                        th {
                                            class: "text-right",
                                            "Actions"
                                        }
                                    }
                                }
                            }
                            tbody {
                                for dataset in &datasets {
                                    DatasetRow {
                                        dataset: dataset.clone(),
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

/// 数据集行组件
#[component]
fn DatasetRow(
    dataset: Dataset,
    rbac: Rbac,
    team_id: i32,
) -> Element {
    let size_display = format_file_size(dataset.total_size);
    
    rsx! {
        tr {
            td {
                div {
                    div {
                        class: "font-medium",
                        "{dataset.name}"
                    }
                    if let Some(description) = &dataset.description {
                        div {
                            class: "text-sm text-base-content/60",
                            "{description}"
                        }
                    }
                }
            }
            td {
                div {
                    class: "text-sm",
                    "{dataset.document_count} docs"
                }
            }
            td {
                div {
                    class: "text-sm",
                    "{size_display}"
                }
            }
            td {
                div {
                    class: match dataset.processing_status.as_str() {
                        "Completed" => "badge badge-success",
                        "Processing" => "badge badge-warning",
                        "Failed" => "badge badge-error",
                        _ => "badge badge-neutral"
                    },
                    "{dataset.processing_status}"
                }
            }
            td {
                div {
                    class: match dataset.visibility.as_str() {
                        "Private" => "badge badge-outline",
                        "Team" => "badge badge-primary",
                        "Company" => "badge badge-secondary",
                        _ => "badge badge-neutral"
                    },
                    "{dataset.visibility}"
                }
            }
            td {
                div {
                    class: "text-sm text-base-content/70",
                    "{dataset.updated_at}"
                }
            }
            if rbac.can_view_datasets() {
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
                                    href: format!("/teams/{}/datasets/{}", team_id, dataset.id),
                                    "View Documents"
                                }
                            }
                            li {
                                a {
                                    "Edit Dataset"
                                }
                            }
                            li {
                                a {
                                    "Download"
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
}

/// 空状态组件
#[component]
fn EmptyDatasets() -> Element {
    rsx! {
        div {
            class: "text-center py-12",
            div {
                class: "text-6xl mb-4",
                "📊"
            }
            h4 {
                class: "text-xl font-semibold mb-2",
                "No Datasets"
            }
            p {
                class: "text-base-content/60 mb-4",
                "Create your first dataset to start organizing your documents"
            }
            button {
                class: "btn btn-primary gap-2",
                span { "➕" }
                "Create Dataset"
            }
        }
    }
}

/// 快速操作指南组件
#[component]
fn QuickActionsGuide() -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-lg",
            div {
                class: "card-body",
                h4 {
                    class: "text-xl font-bold mb-4",
                    "📚 Quick Actions"
                }
                div {
                    class: "grid grid-cols-1 md:grid-cols-3 gap-6",
                    
                    div {
                        h5 {
                            class: "font-semibold mb-2",
                            "📄 Upload Documents"
                        }
                        ul {
                            class: "space-y-1 text-sm text-base-content/70",
                            li { "• Drag & drop files" }
                            li { "• Bulk upload via API" }
                            li { "• Supported: PDF, DOCX, TXT" }
                        }
                    }
                    
                    div {
                        h5 {
                            class: "font-semibold mb-2",
                            "⚙️ Processing"
                        }
                        ul {
                            class: "space-y-1 text-sm text-base-content/70",
                            li { "• Automatic text extraction" }
                            li { "• Smart chunking" }
                            li { "• Vector embeddings" }
                        }
                    }
                    
                    div {
                        h5 {
                            class: "font-semibold mb-2",
                            "🔍 Search & Query"
                        }
                        ul {
                            class: "space-y-1 text-sm text-base-content/70",
                            li { "• Semantic search" }
                            li { "• AI-powered Q&A" }
                            li { "• Context retrieval" }
                        }
                    }
                }
            }
        }
    }
}

/// 格式化文件大小
fn format_file_size(bytes: i64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}
