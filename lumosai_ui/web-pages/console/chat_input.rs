/*!
# Chat Input Component

聊天输入组件，提供用户与AI助手交互的输入界面。

## 功能特性

- **文本输入**: 支持多行文本输入
- **文件上传**: 支持文件拖拽和选择上传
- **语音输入**: 支持语音录制和转文字
- **快捷操作**: 支持快捷键发送和格式化
- **实时预览**: 支持Markdown实时预览
*/

#![allow(non_snake_case)]
use dioxus::prelude::*;
use web_assets::files::*;
use crate::types::Rbac;

/// 聊天输入组件
#[component]
pub fn ChatInput(
    team_id: i32,
    conversation_id: Option<i64>,
    rbac: Rbac,
    disabled: bool,
) -> Element {
    // 状态管理
    let mut message = use_signal(|| String::new());
    let mut is_recording = use_signal(|| false);
    let mut show_file_upload = use_signal(|| false);
    let mut is_sending = use_signal(|| false);

    // 发送消息处理
    let mut send_message = move |_| {
        let msg = message.read().clone();
        if !msg.trim().is_empty() && !is_sending() {
            is_sending.set(true);
            // TODO: 实际发送消息到后端
            println!("发送消息: {}", msg);
            message.set(String::new());
            // 模拟发送完成
            spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                is_sending.set(false);
            });
        }
    };

    rsx! {
        div {
            class: "border-t border-base-300 bg-base-100 p-4",

            // 文件上传区域
            if show_file_upload() {
                FileUploadArea {
                    team_id,
                    conversation_id,
                    on_close: move |_| show_file_upload.set(false)
                }
            }

            // 主输入区域
            div {
                class: "flex flex-col space-y-3",

                // 输入框和按钮
                div {
                    class: "flex items-end space-x-3",

                    // 主输入框
                    div {
                        class: "flex-1 relative",
                        textarea {
                            class: "textarea textarea-bordered w-full min-h-[60px] max-h-[200px] resize-none pr-12",
                            placeholder: if disabled { "AI正在处理中..." } else { "输入您的消息... (Shift+Enter换行，Enter发送)" },
                            value: "{message}",
                            disabled,
                            oninput: move |evt| {
                                message.set(evt.value());
                            },
                            onkeydown: move |evt| {
                                if evt.key() == Key::Enter && !evt.modifiers().shift() {
                                    evt.prevent_default();
                                    send_message(());
                                }
                            }
                        }

                        // 输入框内的操作按钮
                        div {
                            class: "absolute right-2 bottom-2 flex space-x-1",

                            // 语音输入按钮
                            button {
                                class: "btn btn-ghost btn-xs",
                                class: if is_recording() { "text-error" } else { "text-base-content/60" },
                                title: if is_recording() { "停止录音" } else { "语音输入" },
                                disabled,
                                onclick: move |_| {
                                    is_recording.set(!is_recording());
                                    // TODO: 实现语音录制
                                },
                                img {
                                    src: if is_recording() { stop_recording_svg.name } else { microphone_svg.name },
                                    class: "w-4 h-4"
                                }
                            }

                            // 文件上传按钮
                            button {
                                class: "btn btn-ghost btn-xs text-base-content/60",
                                title: "上传文件",
                                disabled,
                                onclick: move |_| {
                                    show_file_upload.set(!show_file_upload());
                                },
                                img {
                                    src: attach_svg.name,
                                    class: "w-4 h-4"
                                }
                            }
                        }
                    }

                    // 发送按钮
                    button {
                        class: "btn btn-primary",
                        class: if message.read().trim().is_empty() || disabled { "btn-disabled" } else { "" },
                        disabled: message.read().trim().is_empty() || disabled || is_sending(),
                        onclick: move |_| send_message(()),
                        if is_sending() {
                            span {
                                class: "loading loading-spinner loading-sm"
                            }
                        } else {
                            img {
                                src: submit_button_svg.name,
                                class: "w-5 h-5"
                            }
                        }
                    }
                }

                // 底部工具栏
                div {
                    class: "flex items-center justify-between text-xs text-base-content/60",

                    // 左侧：快捷提示
                    div {
                        class: "flex items-center space-x-4",
                        span {
                            "💡 提示: Shift+Enter换行，Enter发送"
                        }
                        if let Some(conv_id) = conversation_id {
                            span {
                                "对话ID: {conv_id}"
                            }
                        }
                    }

                    // 右侧：字符计数和状态
                    div {
                        class: "flex items-center space-x-2",
                        span {
                            class: if message.read().len() > 1000 { "text-warning" } else { "" },
                            "{message.read().len()}/2000"
                        }
                        if is_recording() {
                            span {
                                class: "text-error animate-pulse",
                                "🔴 录音中"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 文件上传区域组件
#[component]
fn FileUploadArea(
    team_id: i32,
    conversation_id: Option<i64>,
    on_close: EventHandler<()>,
) -> Element {
    let mut drag_over = use_signal(|| false);

    rsx! {
        div {
            class: "border border-dashed border-base-300 rounded-lg p-4 mb-3 bg-base-50",
            class: if drag_over() { "border-primary bg-primary/5" } else { "" },
            ondragover: move |evt| {
                evt.prevent_default();
                drag_over.set(true);
            },
            ondragleave: move |_| {
                drag_over.set(false);
            },
            ondrop: move |evt| {
                evt.prevent_default();
                drag_over.set(false);
                // TODO: 处理文件拖拽
            },

            div {
                class: "flex items-center justify-between mb-3",
                h4 {
                    class: "font-medium text-base-content",
                    "📎 文件上传"
                }
                button {
                    class: "btn btn-ghost btn-sm",
                    onclick: move |_| on_close.call(()),
                    "✕"
                }
            }

            div {
                class: "text-center py-6",
                div {
                    class: "text-4xl mb-2",
                    "📁"
                }
                p {
                    class: "text-base-content/70 mb-2",
                    "拖拽文件到此处或点击选择文件"
                }
                p {
                    class: "text-xs text-base-content/50",
                    "支持图片、文档、代码等多种格式，最大10MB"
                }
            }

            div {
                class: "flex justify-center space-x-2 mt-4",
                input {
                    r#type: "file",
                    class: "file-input file-input-bordered file-input-sm",
                    multiple: true,
                    accept: ".txt,.md,.pdf,.doc,.docx,.jpg,.jpeg,.png,.gif,.zip"
                }
                button {
                    class: "btn btn-primary btn-sm",
                    "上传文件"
                }
            }
        }
    }
}

/// 语音录制组件
#[component]
fn VoiceRecorder(
    is_recording: bool,
    on_stop: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            class: "flex items-center space-x-3 p-3 bg-error/10 border border-error/20 rounded-lg",
            
            div {
                class: "animate-pulse rounded-full h-3 w-3 bg-error"
            }
            
            div {
                class: "flex-1",
                p {
                    class: "text-sm font-medium text-error",
                    "🎤 正在录音..."
                }
                p {
                    class: "text-xs text-base-content/60",
                    "点击停止按钮结束录音"
                }
            }
            
            button {
                class: "btn btn-error btn-sm",
                onclick: move |_| {
                    // TODO: 停止录音并转换为文字
                    on_stop.call("录音转换的文字内容".to_string());
                },
                "停止录音"
            }
        }
    }
}
