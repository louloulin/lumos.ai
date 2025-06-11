/*!
# LumosAI Desktop Application

A desktop application for LumosAI UI using Dioxus Desktop.

## Features

- **Native Desktop App**: Cross-platform desktop application using Dioxus
- **Shared Codebase**: Same components as web version
- **Native Performance**: Compiled Rust with native webview
- **Hot Reload**: Development mode with file watching
- **System Integration**: Native window controls and system tray

## Usage

```bash
# Development mode
cargo run --bin lumosai-desktop --features desktop

# Build for production
cargo build --bin lumosai-desktop --features desktop --release
```
*/

use dioxus::prelude::*;
use lumosai_ui::prelude::*;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🖥️  Launching LumosAI Desktop Application...");

    dioxus_desktop::launch_cfg(
        App,
        dioxus_desktop::Config::new()
            .with_window(
                dioxus_desktop::WindowBuilder::new()
                    .with_title("LumosAI - AI Assistant Platform")
                    .with_inner_size(dioxus_desktop::LogicalSize::new(1200.0, 800.0))
                    .with_min_inner_size(dioxus_desktop::LogicalSize::new(800.0, 600.0))
                    .with_resizable(true)
                    .with_maximized(false)
                    .with_decorations(true)
            )
            .with_custom_head(r#"
                <link href="https://cdn.jsdelivr.net/npm/daisyui@4.4.19/dist/full.min.css" rel="stylesheet" type="text/css" />
                <script src="https://cdn.tailwindcss.com"></script>
                <style>
                    body { margin: 0; padding: 0; }
                    .desktop-app { height: 100vh; overflow: hidden; }
                </style>
            "#.to_string())
    );
}

// Main App Component (shared with web version)
#[component]
fn App() -> Element {
    rsx! {
        div {
            class: "desktop-app",
            Router::<Route> {}
        }
    }
}

// Route definitions (shared with web version)
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

// Include all the same components from main.rs
// (In a real implementation, these would be in shared modules)

#[component]
fn Home() -> Element {
    Dashboard()
}

#[component]
fn Dashboard() -> Element {
    rsx! {
        BaseLayout {
            title: "LumosAI Desktop".to_string(),
            fav_icon_src: "/favicon.svg".to_string(),
            collapse_svg_src: "/icons/collapse.svg".to_string(),
            stylesheets: vec![],
            section_class: "p-6 bg-base-100 h-full".to_string(),
            js_href: "".to_string(),

            header: rsx! {
                div {
                    class: "flex items-center justify-between w-full",
                    h1 {
                        class: "text-2xl font-bold text-base-content",
                        "🖥️ LumosAI Desktop"
                    }
                    div {
                        class: "flex items-center space-x-4",
                        Button {
                            button_scheme: ButtonScheme::Primary,
                            button_size: ButtonSize::Small,
                            "New Assistant"
                        }
                        Button {
                            button_scheme: ButtonScheme::Ghost,
                            button_size: ButtonSize::Small,
                            onclick: |_| {
                                // Desktop-specific settings
                                println!("Opening desktop settings...");
                            },
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
                    span { class: "font-semibold", "LumosAI Desktop" }
                }
            },

            sidebar_footer: rsx! {
                div {
                    class: "text-xs text-gray-500",
                    "Desktop v", env!("CARGO_PKG_VERSION")
                }
            },

            // Dashboard content
            DashboardContent {}
        }
    }
}

// Include other components (NavItem, DashboardContent, etc.)
// These would be shared between web and desktop versions
