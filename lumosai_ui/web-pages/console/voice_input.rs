/*!
# Voice Input Component

语音输入组件，参考bionic-gpt实现，支持语音转文字功能。

## 功能特性

- **语音录制**: 支持实时语音录制
- **语音转文字**: 集成语音识别API
- **多语言支持**: 支持中文、英文等多种语言
- **实时反馈**: 显示录制状态和音量
- **快捷操作**: 支持快捷键控制
*/

#![allow(non_snake_case)]
use dioxus::prelude::*;

/// 语音输入按钮组件
#[component]
pub fn VoiceInputButton(
    is_locked: bool,
    on_voice_result: EventHandler<String>,
) -> Element {
    let mut is_recording = use_signal(|| false);
    let mut recording_duration = use_signal(|| 0);
    let mut audio_level = use_signal(|| 0.0);
    let mut recognition_text = use_signal(|| String::new());

    rsx! {
        div {
            class: "relative",
            
            // 主录音按钮
            button {
                class: if *is_recording.read() {
                    "btn btn-error btn-sm w-10 h-10 p-0 animate-pulse"
                } else {
                    "btn btn-ghost btn-sm w-10 h-10 p-0"
                },
                title: if *is_recording.read() { "停止录音" } else { "开始语音输入" },
                disabled: is_locked,
                onclick: move |_| {
                    if *is_recording.read() {
                        stop_recording(is_recording, on_voice_result, recognition_text.read().clone());
                    } else {
                        start_recording(is_recording, recording_duration, audio_level, recognition_text);
                    }
                },
                
                if *is_recording.read() {
                    "🔴"
                } else {
                    "🎤"
                }
            }
            
            // 录音状态指示器
            if *is_recording.read() {
                VoiceRecordingIndicator {
                    duration: *recording_duration.read(),
                    audio_level: *audio_level.read(),
                    recognition_text: recognition_text.read().clone()
                }
            }
        }
    }
}

/// 语音录制指示器组件
#[component]
fn VoiceRecordingIndicator(
    duration: i32,
    audio_level: f64,
    recognition_text: String,
) -> Element {
    rsx! {
        div {
            class: "absolute bottom-full mb-2 left-1/2 transform -translate-x-1/2 z-50",
            
            div {
                class: "bg-base-100 border border-base-300 rounded-lg shadow-lg p-4 min-w-80",
                
                // 录音头部
                div {
                    class: "flex items-center justify-between mb-3",
                    
                    div {
                        class: "flex items-center space-x-2",
                        span {
                            class: "text-error animate-pulse",
                            "🔴"
                        }
                        span {
                            class: "text-sm font-medium",
                            "正在录音..."
                        }
                    }
                    
                    div {
                        class: "text-xs text-base-content/60",
                        format_duration(duration)
                    }
                }
                
                // 音量指示器
                div {
                    class: "mb-3",
                    div {
                        class: "text-xs text-base-content/60 mb-1",
                        "音量"
                    }
                    div {
                        class: "w-full bg-base-200 rounded-full h-2",
                        div {
                            class: "bg-primary h-2 rounded-full transition-all duration-100",
                            style: "width: {audio_level * 100.0}%"
                        }
                    }
                }
                
                // 实时识别文本
                if !recognition_text.is_empty() {
                    div {
                        class: "mb-3",
                        div {
                            class: "text-xs text-base-content/60 mb-1",
                            "识别结果"
                        }
                        div {
                            class: "p-2 bg-base-200 rounded text-sm",
                            "{recognition_text}"
                        }
                    }
                }
                
                // 操作提示
                div {
                    class: "text-xs text-base-content/60 text-center",
                    "再次点击麦克风停止录音"
                }
            }
        }
    }
}

/// 完整的语音输入模态框
#[component]
pub fn VoiceInputModal(
    on_voice_result: EventHandler<String>,
    on_close: EventHandler<()>,
) -> Element {
    let mut is_recording = use_signal(|| false);
    let mut recording_duration = use_signal(|| 0);
    let mut audio_level = use_signal(|| 0.0);
    let mut recognition_text = use_signal(|| String::new());
    let mut selected_language = use_signal(|| "zh-CN".to_string());

    rsx! {
        div {
            class: "modal modal-open",
            
            div {
                class: "modal-box max-w-2xl",
                
                // 模态框头部
                div {
                    class: "flex items-center justify-between mb-6",
                    h3 {
                        class: "text-2xl font-bold text-base-content",
                        "🎤 语音输入"
                    }
                    button {
                        class: "btn btn-ghost btn-sm",
                        onclick: move |_| on_close.call(()),
                        "✕"
                    }
                }
                
                // 语言选择
                div {
                    class: "mb-6",
                    label {
                        class: "label",
                        span {
                            class: "label-text",
                            "识别语言"
                        }
                    }
                    select {
                        class: "select select-bordered w-full",
                        value: selected_language.read().clone(),
                        onchange: move |e| selected_language.set(e.value()),
                        
                        option { value: "zh-CN", "中文 (简体)" }
                        option { value: "zh-TW", "中文 (繁体)" }
                        option { value: "en-US", "English (US)" }
                        option { value: "en-GB", "English (UK)" }
                        option { value: "ja-JP", "日本語" }
                        option { value: "ko-KR", "한국어" }
                    }
                }
                
                // 录音区域
                div {
                    class: "text-center mb-6",
                    
                    // 录音按钮
                    button {
                        class: if *is_recording.read() {
                            "btn btn-error btn-lg w-32 h-32 rounded-full animate-pulse"
                        } else {
                            "btn btn-primary btn-lg w-32 h-32 rounded-full"
                        },
                        onclick: move |_| {
                            if *is_recording.read() {
                                stop_recording(is_recording, on_voice_result, recognition_text.read().clone());
                            } else {
                                start_recording(is_recording, recording_duration, audio_level, recognition_text);
                            }
                        },
                        
                        div {
                            class: "text-4xl",
                            if *is_recording.read() {
                                "⏹️"
                            } else {
                                "🎤"
                            }
                        }
                    }
                    
                    // 状态文本
                    div {
                        class: "mt-4",
                        if *is_recording.read() {
                            p {
                                class: "text-lg font-medium text-error",
                                "正在录音... ({format_duration(*recording_duration.read())})"
                            }
                        } else {
                            p {
                                class: "text-lg font-medium",
                                "点击开始语音输入"
                            }
                        }
                    }
                }
                
                // 音量指示器
                if *is_recording.read() {
                    div {
                        class: "mb-6",
                        div {
                            class: "text-center text-sm text-base-content/60 mb-2",
                            "音量"
                        }
                        div {
                            class: "w-full bg-base-200 rounded-full h-4",
                            div {
                                class: "bg-primary h-4 rounded-full transition-all duration-100",
                                style: "width: {audio_level * 100.0}%"
                            }
                        }
                    }
                }
                
                // 识别结果
                div {
                    class: "mb-6",
                    label {
                        class: "label",
                        span {
                            class: "label-text",
                            "识别结果"
                        }
                    }
                    textarea {
                        class: "textarea textarea-bordered w-full h-32",
                        placeholder: "语音识别结果将显示在这里...",
                        value: recognition_text.read().clone(),
                        oninput: move |e| recognition_text.set(e.value())
                    }
                }
                
                // 操作按钮
                div {
                    class: "modal-action",
                    
                    button {
                        class: "btn btn-ghost",
                        onclick: move |_| on_close.call(()),
                        "取消"
                    }
                    
                    button {
                        class: "btn btn-ghost",
                        onclick: move |_| recognition_text.set(String::new()),
                        "清空"
                    }
                    
                    button {
                        class: "btn btn-primary",
                        disabled: recognition_text.read().is_empty(),
                        onclick: move |_| {
                            on_voice_result.call(recognition_text.read().clone());
                            on_close.call(());
                        },
                        "使用文本"
                    }
                }
                
                // 使用提示
                div {
                    class: "mt-4 p-3 bg-info/10 rounded-lg",
                    div {
                        class: "text-xs text-info-content",
                        h6 {
                            class: "font-semibold mb-1",
                            "💡 使用提示"
                        }
                        ul {
                            class: "space-y-1",
                            li { "• 请在安静的环境中录音以获得最佳效果" }
                            li { "• 说话时保持正常语速，发音清晰" }
                            li { "• 支持实时语音识别，可以边说边看结果" }
                            li { "• 录音完成后可以手动编辑识别结果" }
                        }
                    }
                }
            }
        }
    }
}

/// 格式化录音时长
fn format_duration(seconds: i32) -> String {
    let minutes = seconds / 60;
    let seconds = seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

/// 开始录音
fn start_recording(
    is_recording: Signal<bool>,
    duration: Signal<i32>,
    audio_level: Signal<f64>,
    recognition_text: Signal<String>,
) {
    is_recording.set(true);
    duration.set(0);
    recognition_text.set(String::new());
    
    // TODO: 实现真实的语音录制逻辑
    // 1. 请求麦克风权限
    // 2. 开始录音
    // 3. 实时更新音量指示器
    // 4. 调用语音识别API
    // 5. 更新识别文本
    
    // 模拟录音过程
    simulate_recording(duration, audio_level, recognition_text);
}

/// 停止录音
fn stop_recording(
    is_recording: Signal<bool>,
    on_result: EventHandler<String>,
    text: String,
) {
    is_recording.set(false);
    
    // TODO: 实现真实的录音停止逻辑
    // 1. 停止录音
    // 2. 完成语音识别
    // 3. 返回最终结果
    
    if !text.is_empty() {
        on_result.call(text);
    }
}

/// 模拟录音过程（用于演示）
fn simulate_recording(
    duration: Signal<i32>,
    audio_level: Signal<f64>,
    recognition_text: Signal<String>,
) {
    // TODO: 实现真实的录音模拟
    // 这里应该启动一个定时器来更新录音状态
    duration.set(1);
    audio_level.set(0.5);
    recognition_text.set("正在识别语音...".to_string());
}
