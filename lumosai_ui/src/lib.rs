/*!
# LumosAI UI

A modern UI component library for LumosAI applications, based on bionic-gpt's proven design patterns.

This library provides:
- Modern web components built with Dioxus
- Responsive layouts with Tailwind CSS
- DaisyUI component integration
- AI-focused UI patterns
- Accessibility-first design

## Features

- **Web Pages**: Complete page layouts and components
- **Web Assets**: Styling, scripts, and static resources

## Usage

```rust
use lumosai_ui::prelude::*;

#[component]
fn App() -> Element {
    rsx! {
        BaseLayout {
            title: "LumosAI Dashboard",
            AppLayout {
                // Your app content here
            }
        }
    }
}
```
*/

// Re-export workspace crates
pub use web_assets as assets;
pub use web_pages as pages;

// Re-export commonly used types and traits
pub mod prelude {
    pub use dioxus::prelude::*;
    pub use daisy_rsx::*;
    
    // Re-export types
    pub use web_pages::types::*;
    
    // Core layout components from web-pages
    pub use web_pages::{
        app_layout,
        base_layout,
        menu,
        confirm_modal::ConfirmModal,
        snackbar,
        hero,
        profile,
        profile_popup,
        logout_form,
        charts,
    };
    
    // All page modules
    pub use web_pages::{
        api_keys,
        assistants,
        audit_trail,
        console,
        datasets,
        documents,
        history,
        integrations,
        models,
        my_assistants,
        pipelines,
        rate_limits,
        team,
        teams,
        workflows,
    };
    
    // Routes
    pub use web_pages::routes;
    
    // Utility functions
    pub use web_pages::{render, visibility_to_string, string_to_visibility};
}
