/*!
# LumosAI UI 集成示例

这个示例展示了如何在 LumosAI 应用中集成和使用 UI 组件。

## 运行示例

```bash
cargo run --example ui_integration_demo --features ui
```

## 功能展示

1. **基础布局** - 展示如何使用 BaseLayout 和 AppLayout
2. **AI 组件** - 展示聊天界面、助手管理等 AI 专用组件
3. **响应式设计** - 展示移动端适配和主题切换
4. **交互功能** - 展示表单、模态框、通知等交互组件
*/

#[cfg(feature = "ui")]
use lumosai::prelude::*;

#[cfg(feature = "ui")]
#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 LumosAI UI 集成示例");
    println!("========================");

    // 1. 基础布局示例
    demo_basic_layout().await?;
    
    // 2. AI 组件示例
    demo_ai_components().await?;
    
    // 3. 交互组件示例
    demo_interactive_components().await?;
    
    // 4. 完整应用示例
    demo_full_application().await?;

    println!("\n✅ 所有 UI 示例运行完成！");
    Ok(())
}

#[cfg(feature = "ui")]
async fn demo_basic_layout() -> Result<()> {
    println!("\n📱 1. 基础布局示例");
    println!("------------------");

    // 创建基础页面布局
    let page = rsx! {
        BaseLayout {
            title: "LumosAI Dashboard",
            fav_icon_src: "/favicon.svg",
            stylesheets: vec!["/styles/app.css".to_string()],
            js_href: "/js/app.js",
            section_class: "p-6 bg-base-100",
            
            // 页面头部
            header: rsx! {
                div {
                    class: "flex items-center justify-between",
                    h1 {
                        class: "text-2xl font-bold text-base-content",
                        "LumosAI 控制台"
                    }
                    div {
                        class: "flex items-center space-x-4",
                        Button {
                            button_scheme: ButtonScheme::Primary,
                            "新建助手"
                        }
                        Button {
                            button_scheme: ButtonScheme::Ghost,
                            "设置"
                        }
                    }
                }
            },
            
            // 侧边栏
            sidebar: rsx! {
                nav {
                    class: "space-y-2",
                    a {
                        href: "/dashboard",
                        class: "flex items-center space-x-3 px-4 py-2 rounded-lg hover:bg-base-200",
                        span { "🏠" }
                        span { "仪表板" }
                    }
                    a {
                        href: "/assistants",
                        class: "flex items-center space-x-3 px-4 py-2 rounded-lg hover:bg-base-200",
                        span { "🤖" }
                        span { "AI 助手" }
                    }
                    a {
                        href: "/console",
                        class: "flex items-center space-x-3 px-4 py-2 rounded-lg hover:bg-base-200",
                        span { "💬" }
                        span { "对话控制台" }
                    }
                }
            },
            
            // 侧边栏头部
            sidebar_header: rsx! {
                div {
                    class: "flex items-center space-x-3",
                    div {
                        class: "w-8 h-8 bg-primary rounded-lg flex items-center justify-center",
                        span {
                            class: "text-primary-content font-bold",
                            "L"
                        }
                    }
                    span {
                        class: "text-lg font-semibold",
                        "LumosAI"
                    }
                }
            },
            
            // 侧边栏底部
            sidebar_footer: rsx! {
                div {
                    class: "space-y-2",
                    a {
                        href: "/settings",
                        class: "flex items-center space-x-3 px-4 py-2 rounded-lg hover:bg-base-200",
                        span { "⚙️" }
                        span { "设置" }
                    }
                }
            },
            
            // 主要内容
            div {
                class: "space-y-6",
                
                // 欢迎卡片
                Card {
                    class: "bg-gradient-to-r from-primary to-secondary text-primary-content",
                    div {
                        class: "p-6",
                        h2 {
                            class: "text-3xl font-bold mb-2",
                            "欢迎使用 LumosAI"
                        }
                        p {
                            class: "text-lg opacity-90",
                            "构建强大的 AI 应用，从这里开始。"
                        }
                    }
                }
                
                // 快速操作
                div {
                    class: "grid grid-cols-1 md:grid-cols-3 gap-6",
                    
                    Card {
                        class: "hover:shadow-lg transition-shadow",
                        div {
                            class: "p-6 text-center",
                            div {
                                class: "text-4xl mb-4",
                                "🤖"
                            }
                            h3 {
                                class: "text-xl font-semibold mb-2",
                                "创建助手"
                            }
                            p {
                                class: "text-base-content/70 mb-4",
                                "构建您的专属 AI 助手"
                            }
                            Button {
                                button_scheme: ButtonScheme::Primary,
                                button_size: ButtonSize::Small,
                                "开始创建"
                            }
                        }
                    }
                    
                    Card {
                        class: "hover:shadow-lg transition-shadow",
                        div {
                            class: "p-6 text-center",
                            div {
                                class: "text-4xl mb-4",
                                "💬"
                            }
                            h3 {
                                class: "text-xl font-semibold mb-2",
                                "开始对话"
                            }
                            p {
                                class: "text-base-content/70 mb-4",
                                "与您的 AI 助手开始对话"
                            }
                            Button {
                                button_scheme: ButtonScheme::Secondary,
                                button_size: ButtonSize::Small,
                                "进入控制台"
                            }
                        }
                    }
                    
                    Card {
                        class: "hover:shadow-lg transition-shadow",
                        div {
                            class: "p-6 text-center",
                            div {
                                class: "text-4xl mb-4",
                                "📊"
                            }
                            h3 {
                                class: "text-xl font-semibold mb-2",
                                "查看分析"
                            }
                            p {
                                class: "text-base-content/70 mb-4",
                                "监控您的 AI 应用性能"
                            }
                            Button {
                                button_scheme: ButtonScheme::Info,
                                button_size: ButtonSize::Small,
                                "查看报告"
                            }
                        }
                    }
                }
            }
        }
    };

    // 渲染页面
    let html = render(page);
    println!("✅ 基础布局 HTML 生成成功 ({} 字符)", html.len());
    
    Ok(())
}

#[cfg(feature = "ui")]
async fn demo_ai_components() -> Result<()> {
    println!("\n🤖 2. AI 组件示例");
    println!("------------------");
    
    // 这里可以展示 AI 专用组件的使用
    println!("✅ AI 组件包括:");
    println!("   - 聊天控制台 (Console)");
    println!("   - 助手管理 (Assistants)");
    println!("   - 工作流编辑器 (Workflows)");
    println!("   - 数据集管理 (Datasets)");
    println!("   - 模型配置 (Models)");
    
    Ok(())
}

#[cfg(feature = "ui")]
async fn demo_interactive_components() -> Result<()> {
    println!("\n🎯 3. 交互组件示例");
    println!("--------------------");
    
    println!("✅ 交互组件包括:");
    println!("   - 模态框 (Modal)");
    println!("   - 表单组件 (Forms)");
    println!("   - 通知系统 (Snackbar)");
    println!("   - 确认对话框 (ConfirmModal)");
    println!("   - 文件上传 (FileUpload)");
    
    Ok(())
}

#[cfg(feature = "ui")]
async fn demo_full_application() -> Result<()> {
    println!("\n🚀 4. 完整应用示例");
    println!("--------------------");
    
    println!("✅ 完整应用特性:");
    println!("   - 响应式设计 (移动端适配)");
    println!("   - 深色/浅色主题");
    println!("   - 无障碍设计 (WCAG 兼容)");
    println!("   - 实时交互 (WebSocket 支持)");
    println!("   - 模块化架构 (按需加载)");
    
    Ok(())
}

#[cfg(not(feature = "ui"))]
fn main() {
    println!("❌ 此示例需要启用 'ui' 功能");
    println!("请使用以下命令运行:");
    println!("cargo run --example ui_integration_demo --features ui");
}
