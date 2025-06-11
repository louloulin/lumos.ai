use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use web_pages::*;

pub fn routes() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/dashboard", get(dashboard))
        .route("/assistants", get(assistants_page))
        .route("/console", get(console_page))
        .route("/analytics", get(analytics_page))
        .route("/settings", get(settings_page))
}

async fn index() -> impl IntoResponse {
    dashboard().await
}

async fn dashboard() -> impl IntoResponse {
    let page = rsx! {
        base_layout::BaseLayout {
            title: "LumosAI Dashboard".to_string(),
            fav_icon_src: "/static/favicon.svg".to_string(),
            collapse_svg_src: "/static/icons/collapse.svg".to_string(),
            stylesheets: vec![
                "/static/index.css".to_string(),
                "/static/output.css".to_string()
            ],
            section_class: "p-6 bg-base-100".to_string(),
            js_href: "/static/index.js".to_string(),
            
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
                menu::Menu {
                    items: vec![
                        menu::MenuItem {
                            id: "dashboard".to_string(),
                            title: "Dashboard".to_string(),
                            icon: "📊".to_string(),
                            href: "/dashboard".to_string(),
                            active: true,
                        },
                        menu::MenuItem {
                            id: "assistants".to_string(),
                            title: "Assistants".to_string(),
                            icon: "🤖".to_string(),
                            href: "/assistants".to_string(),
                            active: false,
                        },
                        menu::MenuItem {
                            id: "console".to_string(),
                            title: "Console".to_string(),
                            icon: "💬".to_string(),
                            href: "/console".to_string(),
                            active: false,
                        },
                        menu::MenuItem {
                            id: "analytics".to_string(),
                            title: "Analytics".to_string(),
                            icon: "📈".to_string(),
                            href: "/analytics".to_string(),
                            active: false,
                        },
                        menu::MenuItem {
                            id: "settings".to_string(),
                            title: "Settings".to_string(),
                            icon: "⚙️".to_string(),
                            href: "/settings".to_string(),
                            active: false,
                        },
                    ]
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
                    "v", env!("CARGO_PKG_VERSION")
                }
            },
            
            // Dashboard content
            div {
                class: "space-y-6",
                
                // Stats cards
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6",
                    
                    charts::StatsCard {
                        title: "Active Assistants".to_string(),
                        value: "12".to_string(),
                        icon: "🤖".to_string(),
                        color: "blue".to_string(),
                    }
                    
                    charts::StatsCard {
                        title: "Total Conversations".to_string(),
                        value: "1,234".to_string(),
                        icon: "💬".to_string(),
                        color: "green".to_string(),
                    }
                    
                    charts::StatsCard {
                        title: "API Calls Today".to_string(),
                        value: "5,678".to_string(),
                        icon: "📡".to_string(),
                        color: "purple".to_string(),
                    }
                    
                    charts::StatsCard {
                        title: "Success Rate".to_string(),
                        value: "98.5%".to_string(),
                        icon: "✅".to_string(),
                        color: "orange".to_string(),
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
                            div {
                                class: "flex items-center space-x-3 p-3 hover:bg-base-200 rounded-lg",
                                div { class: "text-lg", "🤖" }
                                div {
                                    class: "flex-1",
                                    h4 { class: "font-medium", "New assistant created" }
                                    p { class: "text-sm text-gray-600", "Customer Support Bot v2.0" }
                                }
                                div { class: "text-xs text-gray-500", "2 minutes ago" }
                            }
                        }
                    }
                }
            }
        }
    };
    
    Html(render(page))
}

async fn assistants_page() -> impl IntoResponse {
    Html("<h1>Assistants Page - Coming Soon</h1>".to_string())
}

async fn console_page() -> impl IntoResponse {
    Html("<h1>Console Page - Coming Soon</h1>".to_string())
}

async fn analytics_page() -> impl IntoResponse {
    Html("<h1>Analytics Page - Coming Soon</h1>".to_string())
}

async fn settings_page() -> impl IntoResponse {
    Html("<h1>Settings Page - Coming Soon</h1>".to_string())
}
