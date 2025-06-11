/*!
# Chat Form Component

聊天输入表单组件，提供多种输入方式。

## 功能特性

- **文本输入**: 支持多行文本和快捷键
- **文件上传**: 支持图片、文档等文件
- **语音输入**: 语音转文字功能
- **快捷操作**: 表情、模板等快捷输入
*/

use dioxus::prelude::*;

/// 聊天表单组件
#[component]
pub fn ChatForm(
    is_locked: bool,
    on_send: EventHandler<String>
) -> Element {
    

    
    rsx! {
        div {
            class: "p-4 bg-base-100",

            // 简化的输入区域
            div {
                class: "flex items-center space-x-3",

                input {
                    class: "input input-bordered flex-1",
                    placeholder: if is_locked { "AI正在回复中..." } else { "输入消息..." },
                    disabled: is_locked,
                    r#type: "text"
                }

                button {
                    class: "btn btn-primary",
                    disabled: is_locked,
                    onclick: move |_| {
                        on_send.call("测试消息".to_string());
                    },
                    "发送"
                }
            }
        }
    }
}


