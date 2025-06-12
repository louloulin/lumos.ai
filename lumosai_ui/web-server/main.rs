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
use web_pages::console::enhanced_console::{EnhancedAssistantConsole, SinglePrompt, Capability};
use web_pages::types::{Rbac, Visibility};
use web_pages::app_layout::SideBar;
use web_pages::console::PendingChatState;

#[cfg(feature = "fullstack")]
use dioxus_fullstack::prelude::*;
#[cfg(feature = "fullstack")]
use std::net::Ipv4Addr;

// AI功能模块
mod ai_client;
#[cfg(any(feature = "server", feature = "fullstack"))]
mod streaming;
#[cfg(any(feature = "server", feature = "fullstack"))]
mod api_server;
mod database;
mod tools;
#[cfg(any(feature = "server", feature = "fullstack"))]
mod file_handler;

#[cfg(any(feature = "server", feature = "fullstack"))]
use ai_client::AIClient;
#[cfg(any(feature = "server", feature = "fullstack"))]
use streaming::AppState;

fn main() {
    // Initialize logging
    init_logging();

    // Determine launch mode based on features
    #[cfg(feature = "server")]
    {
        println!("🌐 Launching LumosAI Server...");
        launch_server();
    }

    #[cfg(all(feature = "desktop", not(feature = "server")))]
    {
        println!("🖥️  Launching LumosAI Desktop Application...");
        launch_desktop();
    }

    #[cfg(all(feature = "fullstack", not(feature = "server")))]
    {
        println!("🌐 Launching LumosAI Fullstack Application...");
        launch_fullstack();
    }

    #[cfg(all(not(feature = "desktop"), not(feature = "fullstack"), not(feature = "server")))]
    {
        println!("🌐 Launching LumosAI Web Application...");
        launch_web();
    }
}

#[cfg(feature = "desktop")]
fn launch_desktop() {
    dioxus::launch(App);
}

#[cfg(all(feature = "fullstack", not(feature = "server")))]
fn launch_fullstack() {
    println!("🌐 Starting LumosAI Fullstack Application...");
    println!("📱 Open http://localhost:8080 in your browser");

    // For fullstack mode, we use dioxus LaunchBuilder
    dioxus::LaunchBuilder::new()
        .launch(App);
}

#[cfg(all(not(feature = "desktop"), not(feature = "fullstack")))]
fn launch_web() {
    println!("🌐 Starting LumosAI Web Application...");
    println!("📱 This will open in your default browser");

    // For web mode, we use the simple launch
    dioxus::launch(App);
}

#[cfg(any(feature = "server", feature = "fullstack"))]
async fn launch_api_server() {
    if let Err(e) = api_server::start_api_server().await {
        eprintln!("❌ Failed to start API server: {}", e);
        std::process::exit(1);
    }
}

#[cfg(feature = "server")]
fn launch_server() {
    println!("🌐 Starting LumosAI Fullstack Server...");
    println!("📱 Open http://localhost:8080 in your browser");

    // For server mode, dioxus::launch automatically sets up the server
    dioxus::launch(App);
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
    rsx! {
        Dashboard {}
    }
}

#[component]
fn Dashboard() -> Element {
    rsx! {
        BaseLayout {
            title: "LumosAI Dashboard".to_string(),
            fav_icon_src: "/favicon.svg".to_string(),
            collapse_svg_src: "/icons/collapse.svg".to_string(),
            stylesheets: vec![
                "https://cdn.jsdelivr.net/npm/tailwindcss@3.4.0/dist/tailwind.min.css".to_string(),
                "https://cdn.jsdelivr.net/npm/daisyui@4.4.19/dist/full.min.css".to_string(),
                "data:text/css,.sidebar{background-color:hsl(var(--b2));border-right:1px solid hsl(var(--b3))}.main-content{background-color:hsl(var(--b1));flex:1}.card{background-color:hsl(var(--b1));border:1px solid hsl(var(--b3));border-radius:0.5rem;box-shadow:0 1px 3px 0 rgba(0,0,0,0.1)}.card-header{border-bottom:1px solid hsl(var(--b3));padding:1rem;font-weight:600}.card-body{padding:1rem}.btn{padding:0.5rem 1rem;border-radius:0.375rem;font-weight:500;transition:all 0.2s;border:none;cursor:pointer;display:inline-flex;align-items:center;gap:0.5rem}.btn:hover{transform:translateY(-1px);box-shadow:0 4px 8px rgba(0,0,0,0.1)}.btn-primary{background-color:hsl(var(--p));color:hsl(var(--pc))}.btn-primary:hover{background-color:hsl(var(--p)/0.9)}.btn-secondary{background-color:hsl(var(--s));color:hsl(var(--sc))}.btn-ghost{background-color:transparent;color:hsl(var(--bc))}.btn-ghost:hover{background-color:hsl(var(--b3))}.menu{padding:0.5rem}.menu li{margin-bottom:0.25rem}.menu li a{display:flex;align-items:center;padding:0.75rem 1rem;color:hsl(var(--bc));text-decoration:none;border-radius:0.375rem;transition:all 0.2s}.menu li a:hover{background-color:hsl(var(--b3));transform:translateX(4px)}.menu li a.active{background-color:hsl(var(--p));color:hsl(var(--pc));font-weight:600}.input,.textarea,.select{width:100%;padding:0.75rem;border:1px solid hsl(var(--b3));border-radius:0.375rem;background-color:hsl(var(--b1));color:hsl(var(--bc));transition:all 0.2s}.input:focus,.textarea:focus,.select:focus{outline:none;border-color:hsl(var(--p));box-shadow:0 0 0 3px hsl(var(--p)/0.1)}.textarea{resize:vertical;min-height:4rem}.chat-container{display:flex;flex-direction:column;height:100%}.chat-messages{flex:1;overflow-y:auto;padding:1rem;display:flex;flex-direction:column;gap:1rem}.chat-input{border-top:1px solid hsl(var(--b3));padding:1rem;background-color:hsl(var(--b1))}.message{display:flex;gap:0.75rem;max-width:80%}.message.user{flex-direction:row-reverse;margin-left:auto}.message-content{padding:0.75rem 1rem;border-radius:1rem;word-wrap:break-word}.message.user .message-content{background-color:hsl(var(--p));color:hsl(var(--pc));border-bottom-right-radius:0.25rem}.message.assistant .message-content{background-color:hsl(var(--b2));color:hsl(var(--bc));border-bottom-left-radius:0.25rem}.dashboard-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(300px,1fr));gap:1.5rem;padding:1rem}.stat-card{background-color:hsl(var(--b1));border:1px solid hsl(var(--b3));border-radius:0.5rem;padding:1.5rem;transition:all 0.2s}.stat-card:hover{transform:translateY(-2px);box-shadow:0 8px 16px rgba(0,0,0,0.1)}.stat-value{font-size:2rem;font-weight:bold;color:hsl(var(--bc));margin-bottom:0.5rem}.stat-title{font-size:0.875rem;color:hsl(var(--bc)/0.7);text-transform:uppercase;letter-spacing:0.05em}.loading{display:inline-block;width:1rem;height:1rem;border:2px solid hsl(var(--b3));border-radius:50%;border-top-color:hsl(var(--p));animation:spin 1s ease-in-out infinite}@keyframes spin{to{transform:rotate(360deg)}}@media (max-width:768px){.sidebar{position:fixed;top:0;left:0;bottom:0;width:16rem;transform:translateX(-100%);transition:transform 0.3s ease;z-index:50}.sidebar.open{transform:translateX(0)}.dashboard-grid{grid-template-columns:1fr}.message{max-width:95%}}".to_string(),
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
    #[cfg(not(feature = "server"))]
    {
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

    #[cfg(feature = "server")]
    {
        rsx! {
            li {
                button {
                    class: "w-full flex items-center p-3 text-left hover:bg-base-300 rounded-lg",
                    span { class: "text-lg", "{icon}" }
                    span { class: "ml-3", "{title}" }
                }
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
    // 创建模拟数据
    let rbac = Rbac {
        email: "user@example.com".to_string(),
        first_name: Some("Demo".to_string()),
        last_name: Some("User".to_string()),
        team_id: 1,
        role: "Admin".to_string(),
    };

    let prompt = SinglePrompt {
        name: "AI Assistant".to_string(),
        model_name: Some("gpt-4".to_string()),
    };

    let capabilities = vec![
        Capability {
            name: "Text Generation".to_string(),
        }
    ];

    rsx! {
        div {
            class: "h-full flex flex-col",

            // 使用增强的聊天控制台组件
            EnhancedAssistantConsole {
                team_id: 1,
                conversation_id: None,
                rbac: rbac,
                chat_history: vec![],
                pending_chat_state: PendingChatState::None,
                prompt: prompt,
                selected_item: SideBar::Console,
                title: "AI Console".to_string(),
                header: rsx! {
                    h1 { class: "text-xl font-bold", "AI Assistant Console" }
                },
                is_tts_disabled: false,
                capabilities: capabilities,
                enabled_tools: vec![],
                available_tools: vec![],
            }
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
