/*!
# LumosAI Application Launcher

A unified launcher for LumosAI UI applications supporting both web and desktop modes using Dioxus.

## Features

- **Web Mode**: Browser-based application with hot reload
- **Desktop Mode**: Native desktop application
- **Unified Codebase**: Same components work in both modes
- **Hot Reload**: Development mode with file watching
- **Responsive Design**: Adaptive UI for different screen sizes

## Usage

```bash
# Web mode (default)
cargo run --bin lumosai-web-server

# Desktop mode
cargo run --bin lumosai-web-server --features desktop

# Fullstack mode (SSR + hydration)
cargo run --bin lumosai-web-server --features fullstack
```
*/

use dioxus::prelude::*;
use web_pages::base_layout::BaseLayout;
use web_pages::console::chat_console::ChatConsole;

// AI功能模块
mod ai_client;
mod streaming;
mod api_server;

use ai_client::AIClient;
use streaming::AppState;

#[tokio::main]
async fn main() {
    // Initialize logging
    init_logging();

    // Determine launch mode based on features
    #[cfg(feature = "desktop")]
    {
        println!("🖥️  Launching LumosAI Desktop Application...");
        launch_desktop();
    }

    #[cfg(all(feature = "fullstack", not(feature = "desktop")))]
    {
        println!("🌐 Launching LumosAI Fullstack Application...");
        launch_fullstack();
    }

    #[cfg(all(not(feature = "desktop"), not(feature = "fullstack")))]
    {
        // 检查是否启动API服务器模式
        if std::env::args().any(|arg| arg == "--api-server") {
            println!("🚀 Launching LumosAI API Server...");
            launch_api_server().await;
        } else {
            println!("🌐 Launching LumosAI Web Application...");
            launch_web();
        }
    }
}

#[cfg(feature = "desktop")]
fn launch_desktop() {
    dioxus::launch(App);
}

#[cfg(all(not(feature = "desktop"), feature = "fullstack"))]
fn launch_fullstack() {
    println!("🌐 Starting LumosAI Fullstack Application...");
    println!("📱 Open http://localhost:8080 in your browser");

    dioxus::launch(App);
}

#[cfg(all(not(feature = "desktop"), not(feature = "fullstack")))]
fn launch_web() {
    println!("🌐 Starting LumosAI Web Application...");
    println!("📱 This will open in your default browser");

    dioxus::launch(App);
}

async fn launch_api_server() {
    if let Err(e) = api_server::start_api_server().await {
        eprintln!("❌ Failed to start API server: {}", e);
        std::process::exit(1);
    }
}

// Main App Component
#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

// Route definitions
#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/dashboard")]
    Dashboard {},
    #[route("/assistants")]
    Assistants {},
    #[route("/console")]
    Console {},
    #[route("/analytics")]
    Analytics {},
    #[route("/settings")]
    Settings {},
}

// Route components
#[component]
fn Home() -> Element {
    Dashboard()
}

#[component]
fn Dashboard() -> Element {
    rsx! {
        BaseLayout {
            title: "LumosAI Dashboard".to_string(),
            fav_icon_src: "/favicon.svg".to_string(),
            collapse_svg_src: "/icons/collapse.svg".to_string(),
            stylesheets: vec![
                "https://cdn.jsdelivr.net/npm/daisyui@4.4.19/dist/full.min.css".to_string(),
            ],
            section_class: "p-6 bg-base-100".to_string(),
            js_href: "/app.js".to_string(),

            header: rsx! {
                div {
                    class: "flex items-center justify-between w-full",
                    h1 {
                        class: "text-2xl font-bold text-base-content",
                        "🌟 LumosAI Dashboard"
                    }
                    div {
                        class: "flex items-center space-x-4",
                        button {
                            class: "btn btn-primary btn-sm",
                            "New Assistant"
                        }
                        button {
                            class: "btn btn-ghost btn-sm",
                            "Settings"
                        }
                    }
                }
            },

            sidebar: rsx! {
                nav {
                    class: "p-4",
                    ul {
                        class: "space-y-2",

                        NavItem { route: Route::Dashboard {}, icon: "📊", title: "Dashboard" }
                        NavItem { route: Route::Assistants {}, icon: "🤖", title: "Assistants" }
                        NavItem { route: Route::Console {}, icon: "💬", title: "Console" }
                        NavItem { route: Route::Analytics {}, icon: "📈", title: "Analytics" }
                        NavItem { route: Route::Settings {}, icon: "⚙️", title: "Settings" }
                    }
                }
            },

            sidebar_header: rsx! {
                div {
                    class: "flex items-center space-x-2",
                    div {
                        class: "w-8 h-8 bg-primary rounded-lg flex items-center justify-center",
                        span { class: "text-primary-content font-bold", "L" }
                    }
                    span { class: "font-semibold", "LumosAI" }
                }
            },

            sidebar_footer: rsx! {
                div {
                    class: "text-xs text-gray-500",
                    "v0.1.0"
                }
            },

            // Dashboard content
            DashboardContent {}
        }
    }
}

#[component]
fn NavItem(route: Route, icon: &'static str, title: &'static str) -> Element {
    let nav = use_navigator();
    let current_route = use_route::<Route>();
    let is_active = std::mem::discriminant(&current_route) == std::mem::discriminant(&route);

    rsx! {
        li {
            button {
                class: if is_active {
                    "w-full flex items-center p-3 text-left bg-primary text-primary-content rounded-lg"
                } else {
                    "w-full flex items-center p-3 text-left hover:bg-base-300 rounded-lg"
                },
                onclick: move |_| { nav.push(route.clone()); },

                span { class: "text-lg", "{icon}" }
                span { class: "ml-3", "{title}" }
            }
        }
    }
}

#[component]
fn DashboardContent() -> Element {
    rsx! {
        div {
            class: "space-y-6",

            // Stats cards
            div {
                class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6",

                StatsCard {
                    title: "Active Assistants",
                    value: "12",
                    icon: "🤖",
                    color: "blue",
                }

                StatsCard {
                    title: "Total Conversations",
                    value: "1,234",
                    icon: "💬",
                    color: "green",
                }

                StatsCard {
                    title: "API Calls Today",
                    value: "5,678",
                    icon: "📡",
                    color: "purple",
                }

                StatsCard {
                    title: "Success Rate",
                    value: "98.5%",
                    icon: "✅",
                    color: "orange",
                }
            }

            // Recent activity
            div {
                class: "card bg-base-100 shadow-xl",
                div {
                    class: "card-body",
                    h3 { class: "text-xl font-semibold mb-4", "Recent Activity" }
                    div {
                        class: "space-y-3",
                        ActivityItem {
                            icon: "🤖",
                            title: "New assistant created",
                            description: "Customer Support Bot v2.0",
                            time: "2 minutes ago"
                        }
                        ActivityItem {
                            icon: "💬",
                            title: "Conversation completed",
                            description: "User inquiry about pricing",
                            time: "5 minutes ago"
                        }
                        ActivityItem {
                            icon: "📊",
                            title: "Analytics report generated",
                            description: "Weekly performance summary",
                            time: "1 hour ago"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn StatsCard(title: &'static str, value: &'static str, icon: &'static str, color: &'static str) -> Element {
    let bg_color = match color {
        "blue" => "bg-blue-500",
        "green" => "bg-green-500",
        "purple" => "bg-purple-500",
        "orange" => "bg-orange-500",
        _ => "bg-gray-500",
    };

    rsx! {
        div {
            class: "card bg-base-100 shadow-xl hover:shadow-2xl transition-shadow",
            div {
                class: "card-body",
                div {
                    class: "flex items-center",
                    div {
                        class: "p-3 rounded-full text-white {bg_color}",
                        "{icon}"
                    }
                    div {
                        class: "ml-4",
                        h3 { class: "text-sm font-medium text-gray-600", "{title}" }
                        p { class: "text-2xl font-bold", "{value}" }
                    }
                }
            }
        }
    }
}

#[component]
fn ActivityItem(icon: &'static str, title: &'static str, description: &'static str, time: &'static str) -> Element {
    rsx! {
        div {
            class: "flex items-center space-x-3 p-3 hover:bg-base-200 rounded-lg transition-colors",
            div { class: "text-lg", "{icon}" }
            div {
                class: "flex-1",
                h4 { class: "font-medium", "{title}" }
                p { class: "text-sm text-gray-600", "{description}" }
            }
            div { class: "text-xs text-gray-500", "{time}" }
        }
    }
}

#[component]
fn Assistants() -> Element {
    rsx! {
        div {
            class: "p-6",
            h1 { class: "text-3xl font-bold mb-6", "🤖 AI Assistants" }
            p { class: "text-gray-600", "Manage your AI assistants and their configurations." }

            div {
                class: "mt-6",
                button {
                    class: "btn btn-primary",
                    "Create New Assistant"
                }
            }
        }
    }
}

#[component]
fn Console() -> Element {
    rsx! {
        div {
            class: "h-full flex flex-col",

            // 使用新的聊天控制台组件
            ChatConsole {}
        }
    }
}

#[component]
fn Analytics() -> Element {
    rsx! {
        div {
            class: "p-6",
            h1 { class: "text-3xl font-bold mb-6", "📈 Analytics" }
            p { class: "text-gray-600", "View performance metrics and usage analytics." }

            div {
                class: "mt-6 card bg-base-100 shadow-xl",
                div {
                    class: "card-body",
                    p { class: "text-gray-500", "Analytics dashboard will be implemented here..." }
                }
            }
        }
    }
}

#[component]
fn Settings() -> Element {
    rsx! {
        div {
            class: "p-6",
            h1 { class: "text-3xl font-bold mb-6", "⚙️ Settings" }
            p { class: "text-gray-600", "Configure your application preferences." }

            div {
                class: "mt-6 card bg-base-100 shadow-xl",
                div {
                    class: "card-body",
                    p { class: "text-gray-500", "Settings panel will be implemented here..." }
                }
            }
        }
    }
}

// Utility functions
fn init_logging() {
    tracing_subscriber::fmt::init();
}

fn open_browser(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", url])
            .spawn()?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()?;
    }

    Ok(())
}
