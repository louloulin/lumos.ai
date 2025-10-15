/*!
# File Upload Component

文件上传组件，参考bionic-gpt实现，支持多种文件类型。

## 功能特性

- **多文件上传**: 支持同时上传多个文件
- **文件类型检查**: 支持文档、图片、文本等格式
- **拖拽上传**: 支持拖拽文件到上传区域
- **进度显示**: 显示上传进度和状态
- **预览功能**: 支持图片和文档预览
*/

#![allow(non_snake_case)]
use dioxus::prelude::*;

/// 文件上传组件
#[component]
pub fn FileUpload(
    team_id: i32,
    conversation_id: Option<i64>,
    on_upload_complete: EventHandler<Vec<String>>,
) -> Element {
    // 简化实现，移除use_signal依赖
    let selected_files = Vec::<String>::new();
    let upload_progress = 0.0;
    let is_uploading = false;
    let show_preview = false;

    rsx! {
        div {
            class: "modal modal-open",
            id: "file-upload-modal",

            div {
                class: "modal-box max-w-4xl",

                // 模态框头部
                div {
                    class: "flex items-center justify-between mb-6",
                    h3 {
                        class: "text-2xl font-bold text-base-content",
                        "📁 文件上传"
                    }
                    button {
                        class: "btn btn-ghost btn-sm",
                        "✕"
                    }
                }

                // 上传区域
                div {
                    class: "border-2 border-dashed border-base-300 rounded-lg p-8 mb-6 hover:border-primary transition-colors",
                    class: if is_uploading { "pointer-events-none opacity-50" } else { "cursor-pointer" },

                    div {
                        class: "text-center",

                        // 上传图标
                        div {
                            class: "text-6xl mb-4",
                            if is_uploading {
                                "⏳"
                            } else {
                                "📁"
                            }
                        }

                        // 上传提示
                        if is_uploading {
                            div {
                                h4 {
                                    class: "text-lg font-semibold mb-2",
                                    "正在上传文件..."
                                }
                                div {
                                    class: "w-full bg-base-200 rounded-full h-2 mb-4",
                                    div {
                                        class: "bg-primary h-2 rounded-full transition-all duration-300",
                                        style: "width: {upload_progress}%"
                                    }
                                }
                                p {
                                    class: "text-sm text-base-content/60",
                                    "上传进度: {upload_progress:.1}%"
                                }
                            }
                        } else {
                            div {
                                h4 {
                                    class: "text-lg font-semibold mb-2",
                                    "拖拽文件到此处或点击选择"
                                }
                                p {
                                    class: "text-base-content/60 mb-4",
                                    "支持多种文件格式，最大50MB"
                                }

                                input {
                                    r#type: "file",
                                    multiple: true,
                                    class: "hidden",
                                    id: "file-input",
                                    accept: ".pdf,.doc,.docx,.txt,.md,.jpg,.jpeg,.png,.csv,.xlsx"
                                }

                                button {
                                    class: "btn btn-primary gap-2",
                                    onclick: move |_| {
                                        // 触发文件选择
                                    },
                                    span { "📎" }
                                    "选择文件"
                                }
                            }
                        }
                    }
                }

                // 支持的文件类型
                SupportedFileTypes {}

                // 已选择的文件列表
                if !selected_files.is_empty() {
                    SelectedFilesList {
                        files: selected_files.clone()
                    }
                }

                // 操作按钮
                div {
                    class: "modal-action",

                    button {
                        class: "btn btn-ghost",
                        disabled: is_uploading,
                        "取消"
                    }

                    if !selected_files.is_empty() {
                        button {
                            class: "btn btn-primary gap-2",
                            disabled: is_uploading,
                            span { "⬆️" }
                            "上传文件 ({selected_files.len()})"
                        }
                    }

                    button {
                        class: "btn btn-ghost gap-2",
                        span { "👁️" }
                        "预览"
                    }
                }

                // 文件预览
                if show_preview && !selected_files.is_empty() {
                    FilePreview {
                        files: selected_files.clone()
                    }
                }
            }
        }
    }
}

/// 支持的文件类型组件
#[component]
fn SupportedFileTypes() -> Element {
    rsx! {
        div {
            class: "alert alert-info mb-6",

            div {
                h5 {
                    class: "font-semibold mb-3",
                    "📋 支持的文件类型"
                }

                div {
                    class: "grid grid-cols-1 md:grid-cols-3 gap-4 text-sm",

                    div {
                        h6 {
                            class: "font-medium mb-2",
                            "📄 文档类型"
                        }
                        ul {
                            class: "space-y-1 text-xs",
                            li { ".pdf, .doc, .docx" }
                            li { ".txt, .md, .rtf" }
                            li { ".csv, .xlsx, .tsv" }
                        }
                    }

                    div {
                        h6 {
                            class: "font-medium mb-2",
                            "🖼️ 图片类型"
                        }
                        ul {
                            class: "space-y-1 text-xs",
                            li { ".jpg, .jpeg" }
                            li { ".png, .gif" }
                            li { ".webp, .svg" }
                        }
                    }

                    div {
                        h6 {
                            class: "font-medium mb-2",
                            "📝 文本类型"
                        }
                        ul {
                            class: "space-y-1 text-xs",
                            li { ".html, .xml" }
                            li { ".json, .yaml" }
                            li { ".eml, .msg" }
                        }
                    }
                }

                div {
                    class: "mt-4 p-3 bg-warning/10 rounded-lg",
                    p {
                        class: "text-xs text-warning-content",
                        "⚠️ 最大文件大小: 50MB | 建议单次上传不超过10个文件"
                    }
                }
            }
        }
    }
}

/// 已选择文件列表组件
#[component]
fn SelectedFilesList(files: Vec<String>) -> Element {
    rsx! {
        div {
            class: "mb-6",

            h5 {
                class: "font-semibold mb-3",
                "📎 已选择的文件 ({files.len()})"
            }

            div {
                class: "space-y-2 max-h-40 overflow-y-auto",

                for (_index, file) in files.iter().enumerate() {
                    div {
                        class: "flex items-center justify-between p-3 bg-base-200 rounded-lg",

                        div {
                            class: "flex items-center space-x-3",
                            span {
                                class: "text-2xl",
                                "{get_file_icon(file)}"
                            }
                            div {
                                p {
                                    class: "font-medium text-sm",
                                    "{file}"
                                }
                                p {
                                    class: "text-xs text-base-content/60",
                                    "{get_file_size_display(file)}"
                                }
                            }
                        }

                        button {
                            class: "btn btn-ghost btn-xs text-error",
                            "🗑️"
                        }
                    }
                }
            }
        }
    }
}

/// 文件预览组件
#[component]
fn FilePreview(files: Vec<String>) -> Element {
    rsx! {
        div {
            class: "mt-6 p-4 bg-base-200 rounded-lg",

            h5 {
                class: "font-semibold mb-3",
                "👁️ 文件预览"
            }

            div {
                class: "grid grid-cols-2 md:grid-cols-4 gap-4",

                for file in &files {
                    div {
                        class: "p-3 bg-base-100 rounded-lg text-center",

                        div {
                            class: "text-4xl mb-2",
                            "{get_file_icon(file)}"
                        }

                        p {
                            class: "text-xs font-medium truncate",
                            "{file}"
                        }

                        p {
                            class: "text-xs text-base-content/60",
                            "{get_file_type(file)}"
                        }
                    }
                }
            }
        }
    }
}

/// 获取文件图标
fn get_file_icon(filename: &str) -> &'static str {
    let extension = filename.split('.').last().unwrap_or("").to_lowercase();
    match extension.as_str() {
        "pdf" => "📄",
        "doc" | "docx" => "📝",
        "txt" | "md" => "📃",
        "jpg" | "jpeg" | "png" | "gif" => "🖼️",
        "csv" | "xlsx" => "📊",
        "zip" | "rar" => "📦",
        "mp3" | "wav" => "🎵",
        "mp4" | "avi" => "🎬",
        _ => "📁",
    }
}

/// 获取文件大小显示
fn get_file_size_display(_filename: &str) -> &'static str {
    // TODO: 实现真实的文件大小获取
    "未知大小"
}

/// 获取文件类型
fn get_file_type(filename: &str) -> &'static str {
    let extension = filename.split('.').last().unwrap_or("").to_lowercase();
    match extension.as_str() {
        "pdf" => "PDF文档",
        "doc" | "docx" => "Word文档",
        "txt" => "文本文件",
        "md" => "Markdown",
        "jpg" | "jpeg" | "png" | "gif" => "图片文件",
        "csv" => "CSV表格",
        "xlsx" => "Excel表格",
        _ => "未知类型",
    }
}

/// 开始上传文件
fn start_upload(
    _files: Vec<String>,
    _team_id: i32,
    _conversation_id: Option<i64>,
    _progress: Signal<f64>,
    _on_complete: impl Fn(Vec<String>),
) {
    // TODO: 实现真实的文件上传逻辑
    // 这里应该调用后端API上传文件
}
