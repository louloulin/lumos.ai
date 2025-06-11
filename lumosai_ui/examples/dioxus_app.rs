/*!
# LumosAI Dioxus Web Application

This example shows how to create and launch a complete Dioxus web application using LumosAI UI components.

## Running the Application

```bash
# For web (requires trunk)
cargo install trunk
trunk serve --open

# For desktop
cargo run --example dioxus_app --features desktop

# For server-side rendering
cargo run --example dioxus_app --features ssr
```
*/

use lumosai_ui::prelude::*;
use dioxus::prelude::*;

fn main() {
    println!("🚀 Starting LumosAI UI Application");
    println!("==================================");

    // For demonstration, we'll render to HTML
    let html = render_app_to_html();
    println!("📄 Generated HTML ({} chars)", html.len());

    // Save to file for viewing
    std::fs::write("lumosai_app.html", &html).expect("Failed to write HTML file");
    println!("✅ HTML saved to: lumosai_app.html");
    println!("🌐 Open this file in your browser to view the application");
}

#[component]
fn App() -> Element {
    // Application state (simplified for SSR)
    let current_page = "dashboard".to_string();
    let theme = "light".to_string();
    let sidebar_collapsed = false;

    rsx! {
        div {
            class: "min-h-screen bg-base-100",
            
            // Navigation Header
            header {
                class: "navbar bg-base-200 shadow-lg",
                div {
                    class: "navbar-start",
                    button {
                        class: "btn btn-ghost btn-circle",
                        "☰"
                    }
                    h1 {
                        class: "text-xl font-bold ml-4",
                        "🌟 LumosAI Dashboard"
                    }
                }
                div {
                    class: "navbar-end",
                    div {
                        class: "flex items-center space-x-2",
                        
                        // Theme Toggle
                        button {
                            class: "btn btn-ghost btn-sm",
                            if theme == "light" { "🌙" } else { "☀️" }
                        }
                        
                        // User Menu
                        div {
                            class: "dropdown dropdown-end",
                            button {
                                class: "btn btn-ghost btn-circle avatar",
                                "👤"
                            }
                        }
                    }
                }
            }
            
            div {
                class: "flex",
                
                // Sidebar
                aside {
                    class: if sidebar_collapsed {
                        "w-16 bg-base-200 min-h-screen transition-all duration-300"
                    } else {
                        "w-64 bg-base-200 min-h-screen transition-all duration-300"
                    },
                    
                    nav {
                        class: "p-4",
                        ul {
                            class: "space-y-2",
                            
                            NavItem {
                                icon: "📊",
                                label: "Dashboard",
                                active: current_page == "dashboard",
                                collapsed: sidebar_collapsed,
                            }

                            NavItem {
                                icon: "🤖",
                                label: "Assistants",
                                active: current_page == "assistants",
                                collapsed: sidebar_collapsed,
                            }

                            NavItem {
                                icon: "💬",
                                label: "Console",
                                active: current_page == "console",
                                collapsed: sidebar_collapsed,
                            }

                            NavItem {
                                icon: "📈",
                                label: "Analytics",
                                active: current_page == "analytics",
                                collapsed: sidebar_collapsed,
                            }

                            NavItem {
                                icon: "⚙️",
                                label: "Settings",
                                active: current_page == "settings",
                                collapsed: sidebar_collapsed,
                            }
                        }
                    }
                }
                
                // Main Content
                main {
                    class: "flex-1 p-6",
                    
                    match current_page.as_str() {
                        "dashboard" => rsx! { DashboardPage {} },
                        "assistants" => rsx! { AssistantsPage {} },
                        "console" => rsx! { ConsolePage {} },
                        "analytics" => rsx! { AnalyticsPage {} },
                        "settings" => rsx! { SettingsPage {} },
                        _ => rsx! { DashboardPage {} }
                    }
                }
            }
        }
    }
}

#[component]
fn NavItem(
    icon: String,
    label: String,
    active: bool,
    collapsed: bool,
) -> Element {
    rsx! {
        li {
            button {
                class: if active {
                    "w-full flex items-center p-3 text-left bg-primary text-primary-content rounded-lg"
                } else {
                    "w-full flex items-center p-3 text-left hover:bg-base-300 rounded-lg"
                },

                
                span { class: "text-lg", "{icon}" }
                
                if !collapsed {
                    span { class: "ml-3", "{label}" }
                }
            }
        }
    }
}

#[component]
fn DashboardPage() -> Element {
    rsx! {
        div {
            class: "space-y-6",
            
            // Page Header
            div {
                class: "flex items-center justify-between",
                h2 {
                    class: "text-3xl font-bold",
                    "📊 Dashboard"
                }
                div {
                    class: "flex space-x-2",
                    Button {
                        button_scheme: ButtonScheme::Primary,
                        "New Assistant"
                    }
                    Button {
                        button_scheme: ButtonScheme::Secondary,
                        "Import Data"
                    }
                }
            }
            
            // Stats Cards
            div {
                class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6",
                
                StatsCard {
                    title: "Active Assistants",
                    value: "12",
                    icon: "🤖",
                    color: "bg-blue-500"
                }
                
                StatsCard {
                    title: "Total Conversations",
                    value: "1,234",
                    icon: "💬",
                    color: "bg-green-500"
                }
                
                StatsCard {
                    title: "API Calls Today",
                    value: "5,678",
                    icon: "📡",
                    color: "bg-purple-500"
                }
                
                StatsCard {
                    title: "Success Rate",
                    value: "98.5%",
                    icon: "✅",
                    color: "bg-orange-500"
                }
            }
            
            // Recent Activity
            Card {
                class: "p-6",
                h3 {
                    class: "text-xl font-semibold mb-4",
                    "Recent Activity"
                }
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

#[component]
fn AssistantsPage() -> Element {
    rsx! {
        div {
            class: "space-y-6",
            
            h2 {
                class: "text-3xl font-bold",
                "🤖 AI Assistants"
            }
            
            div {
                class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                
                AssistantCard {
                    name: "Customer Support Bot",
                    description: "Handles customer inquiries and support tickets",
                    status: "Active",
                    conversations: 156
                }
                
                AssistantCard {
                    name: "Sales Assistant",
                    description: "Helps with product recommendations and sales",
                    status: "Active", 
                    conversations: 89
                }
                
                AssistantCard {
                    name: "Technical Helper",
                    description: "Provides technical documentation and guidance",
                    status: "Inactive",
                    conversations: 23
                }
            }
        }
    }
}

#[component]
fn ConsolePage() -> Element {
    rsx! {
        div {
            class: "space-y-6",
            
            h2 {
                class: "text-3xl font-bold",
                "💬 AI Console"
            }
            
            Card {
                class: "p-6",
                div {
                    class: "space-y-4",
                    
                    // Chat Interface
                    div {
                        class: "h-96 bg-base-200 rounded-lg p-4 overflow-y-auto",
                        div {
                            class: "space-y-3",
                            ChatMessage {
                                role: "user",
                                content: "Hello! Can you help me with my project?"
                            }
                            ChatMessage {
                                role: "assistant", 
                                content: "Of course! I'd be happy to help you with your project. What specific area would you like assistance with?"
                            }
                        }
                    }
                    
                    // Input Area
                    div {
                        class: "flex space-x-2",
                        Input {
                            input_type: InputType::Text,
                            name: "message",
                            placeholder: "Type your message...",
                            value: "".to_string(),
                        }
                        Button {
                            button_scheme: ButtonScheme::Primary,
                            "Send"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AnalyticsPage() -> Element {
    rsx! {
        div {
            class: "space-y-6",
            
            h2 {
                class: "text-3xl font-bold",
                "📈 Analytics"
            }
            
            p {
                class: "text-gray-600",
                "Analytics dashboard coming soon..."
            }
        }
    }
}

#[component]
fn SettingsPage() -> Element {
    rsx! {
        div {
            class: "space-y-6",
            
            h2 {
                class: "text-3xl font-bold",
                "⚙️ Settings"
            }
            
            Card {
                class: "p-6",
                div {
                    class: "space-y-4",
                    
                    h3 {
                        class: "text-lg font-semibold",
                        "General Settings"
                    }
                    
                    div {
                        class: "space-y-3",
                        Input {
                            input_type: InputType::Text,
                            name: "app_name",
                            label: "Application Name",
                            value: "LumosAI Dashboard".to_string(),
                        }
                        
                        Input {
                            input_type: InputType::Email,
                            name: "admin_email",
                            label: "Admin Email",
                            value: "admin@lumosai.com".to_string(),
                        }
                        
                        Select {
                            name: "theme",
                            label: "Theme",
                            value: "light".to_string(),
                            SelectOption {
                                value: "light",
                                selected_value: "light".to_string(),
                                "Light"
                            }
                            SelectOption {
                                value: "dark",
                                selected_value: "light".to_string(),
                                "Dark"
                            }
                        }
                    }
                    
                    div {
                        class: "pt-4",
                        Button {
                            button_scheme: ButtonScheme::Primary,
                            "Save Settings"
                        }
                    }
                }
            }
        }
    }
}

// Helper Components

#[component]
fn StatsCard(title: String, value: String, icon: String, color: String) -> Element {
    rsx! {
        Card {
            class: "p-6",
            div {
                class: "flex items-center",
                div {
                    class: format!("p-3 rounded-full text-white {}", color),
                    "{icon}"
                }
                div {
                    class: "ml-4",
                    h3 {
                        class: "text-sm font-medium text-gray-600",
                        "{title}"
                    }
                    p {
                        class: "text-2xl font-bold",
                        "{value}"
                    }
                }
            }
        }
    }
}

#[component]
fn ActivityItem(icon: String, title: String, description: String, time: String) -> Element {
    rsx! {
        div {
            class: "flex items-center space-x-3 p-3 hover:bg-base-200 rounded-lg",
            div {
                class: "text-lg",
                "{icon}"
            }
            div {
                class: "flex-1",
                h4 {
                    class: "font-medium",
                    "{title}"
                }
                p {
                    class: "text-sm text-gray-600",
                    "{description}"
                }
            }
            div {
                class: "text-xs text-gray-500",
                "{time}"
            }
        }
    }
}

#[component]
fn AssistantCard(name: String, description: String, status: String, conversations: i32) -> Element {
    rsx! {
        Card {
            class: "p-6",
            div {
                class: "space-y-4",
                div {
                    class: "flex items-center justify-between",
                    h3 {
                        class: "text-lg font-semibold",
                        "{name}"
                    }
                    Label {
                        label_role: if status == "Active" { LabelRole::Success } else { LabelRole::Warning },
                        "{status}"
                    }
                }
                p {
                    class: "text-gray-600",
                    "{description}"
                }
                div {
                    class: "flex items-center justify-between",
                    span {
                        class: "text-sm text-gray-500",
                        "{conversations} conversations"
                    }
                    Button {
                        button_scheme: ButtonScheme::Primary,
                        button_size: ButtonSize::Small,
                        "Configure"
                    }
                }
            }
        }
    }
}

#[component]
fn ChatMessage(role: String, content: String) -> Element {
    rsx! {
        div {
            class: if role == "user" {
                "flex justify-end"
            } else {
                "flex justify-start"
            },
            div {
                class: if role == "user" {
                    "bg-primary text-primary-content p-3 rounded-lg max-w-xs"
                } else {
                    "bg-base-300 p-3 rounded-lg max-w-xs"
                },
                "{content}"
            }
        }
    }
}

// Function to render the app to HTML (for non-interactive demo)
fn render_app_to_html() -> String {
    let app_html = render(rsx! { App {} });
    
    format!(
        r#"<!DOCTYPE html>
<html lang="en" data-theme="light">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>LumosAI Dashboard</title>
    <link href="https://cdn.jsdelivr.net/npm/daisyui@4.4.19/dist/full.min.css" rel="stylesheet" type="text/css" />
    <script src="https://cdn.tailwindcss.com"></script>
    <style>
        /* Custom styles for better appearance */
        .navbar {{ @apply sticky top-0 z-50; }}
        .stats-card {{ @apply transition-transform hover:scale-105; }}
        .nav-item {{ @apply transition-colors duration-200; }}
    </style>
</head>
<body>
    {}
    <script>
        console.log('🌟 LumosAI Dashboard loaded successfully!');
        
        // Add some basic interactivity
        document.addEventListener('DOMContentLoaded', function() {{
            // Add click handlers for buttons
            document.querySelectorAll('button').forEach(button => {{
                button.addEventListener('click', function(e) {{
                    console.log('Button clicked:', this.textContent);
                }});
            }});
        }});
    </script>
</body>
</html>"#,
        app_html
    )
}
