/*!
# Basic Layout Example

This example demonstrates how to use the basic layout components from LumosAI UI.
*/

use lumosai_ui::prelude::*;
use web_pages::base_layout::BaseLayout;

fn main() {
    // This would typically be used in a web application context
    println!("LumosAI UI Basic Layout Example");
    
    // Example of how the layout would be structured
    let layout_html = render_basic_layout();
    println!("Generated HTML length: {} characters", layout_html.len());
}

#[component]
fn BasicApp() -> Element {
    rsx! {
        BaseLayout {
            title: "LumosAI Dashboard".to_string(),
            fav_icon_src: "/favicon.svg".to_string(),
            collapse_svg_src: "/icons/collapse.svg".to_string(),
            stylesheets: vec![
                "/styles/tailwind.css".to_string(),
                "/styles/app.css".to_string()
            ],
            section_class: "p-6 bg-base-100".to_string(),
            js_href: "/js/app.js".to_string(),
            
            // Header content
            header: rsx! {
                div {
                    class: "flex items-center justify-between w-full",
                    h1 {
                        class: "text-2xl font-bold text-base-content",
                        "LumosAI Dashboard"
                    }
                    div {
                        class: "flex items-center space-x-4",
                        Button {
                            button_scheme: ButtonScheme::Primary,
                            button_size: ButtonSize::Small,
                            "New Assistant"
                        }
                        Button {
                            button_scheme: ButtonScheme::Secondary,
                            button_size: ButtonSize::Small,
                            "Settings"
                        }
                    }
                }
            },
            
            // Sidebar navigation
            sidebar: rsx! {
                nav {
                    class: "space-y-2",
                    
                    // Dashboard
                    a {
                        href: "/dashboard",
                        class: "flex items-center space-x-3 px-4 py-2 rounded-lg hover:bg-base-200 text-base-content",
                        span { "🏠" }
                        span { "Dashboard" }
                    }
                    
                    // Assistants
                    a {
                        href: "/assistants",
                        class: "flex items-center space-x-3 px-4 py-2 rounded-lg hover:bg-base-200 text-base-content",
                        span { "🤖" }
                        span { "Assistants" }
                    }
                    
                    // Console
                    a {
                        href: "/console",
                        class: "flex items-center space-x-3 px-4 py-2 rounded-lg hover:bg-base-200 text-base-content",
                        span { "💬" }
                        span { "Console" }
                    }
                    
                    // Workflows
                    a {
                        href: "/workflows",
                        class: "flex items-center space-x-3 px-4 py-2 rounded-lg hover:bg-base-200 text-base-content",
                        span { "🔄" }
                        span { "Workflows" }
                    }
                    
                    // Datasets
                    a {
                        href: "/datasets",
                        class: "flex items-center space-x-3 px-4 py-2 rounded-lg hover:bg-base-200 text-base-content",
                        span { "📊" }
                        span { "Datasets" }
                    }
                    
                    // Models
                    a {
                        href: "/models",
                        class: "flex items-center space-x-3 px-4 py-2 rounded-lg hover:bg-base-200 text-base-content",
                        span { "🧠" }
                        span { "Models" }
                    }
                }
            },
            
            // Sidebar header
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
                        class: "text-lg font-semibold text-base-content",
                        "LumosAI"
                    }
                }
            },
            
            // Sidebar footer
            sidebar_footer: rsx! {
                div {
                    class: "space-y-2",
                    a {
                        href: "/settings",
                        class: "flex items-center space-x-3 px-4 py-2 rounded-lg hover:bg-base-200 text-base-content",
                        span { "⚙️" }
                        span { "Settings" }
                    }
                    a {
                        href: "/help",
                        class: "flex items-center space-x-3 px-4 py-2 rounded-lg hover:bg-base-200 text-base-content",
                        span { "❓" }
                        span { "Help" }
                    }
                }
            },

            // Main content area (children)
            div {
                class: "space-y-6",
                
                // Welcome section
                Card {
                    class: "bg-gradient-to-r from-primary to-secondary text-primary-content",
                    div {
                        class: "p-6",
                        h2 {
                            class: "text-3xl font-bold mb-2",
                            "Welcome to LumosAI"
                        }
                        p {
                            class: "text-lg opacity-90",
                            "Build powerful AI applications with our intuitive interface."
                        }
                        div {
                            class: "mt-4",
                            Button {
                                button_scheme: ButtonScheme::Accent,
                                "Get Started"
                            }
                        }
                    }
                }
                
                // Quick actions
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
                                "Create Assistant"
                            }
                            p {
                                class: "text-base-content/70 mb-4",
                                "Build a new AI assistant for your needs"
                            }
                            Button {
                                button_scheme: ButtonScheme::Primary,
                                button_size: ButtonSize::Small,
                                "Create"
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
                                "Start Chat"
                            }
                            p {
                                class: "text-base-content/70 mb-4",
                                "Begin a conversation with your assistant"
                            }
                            Button {
                                button_scheme: ButtonScheme::Secondary,
                                button_size: ButtonSize::Small,
                                "Chat"
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
                                "View Analytics"
                            }
                            p {
                                class: "text-base-content/70 mb-4",
                                "Monitor your AI application performance"
                            }
                            Button {
                                button_scheme: ButtonScheme::Info,
                                button_size: ButtonSize::Small,
                                "Analytics"
                            }
                        }
                    }
                }
            }
        }
    }
}

fn render_basic_layout() -> String {
    render(rsx! { BasicApp {} })
}
