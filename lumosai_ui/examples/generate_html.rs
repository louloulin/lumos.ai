/*!
# Generate HTML Page

This example generates a complete HTML page to test in the browser.
*/

use lumosai_ui::prelude::*;
use std::fs;

fn main() {
    println!("🌐 Generating HTML page for browser testing...");
    
    let html = generate_complete_page();
    
    // Write to file
    fs::write("lumosai_ui_test.html", html).expect("Failed to write HTML file");
    
    println!("✅ HTML page generated: lumosai_ui_test.html");
    println!("🌐 Open this file in your browser to test the UI!");
}

fn generate_complete_page() -> String {
    let body_content = render(rsx! {
        div {
            class: "min-h-screen bg-gray-50 py-8",
            div {
                class: "max-w-6xl mx-auto px-4",
                
                // Header
                header {
                    class: "text-center mb-12",
                    h1 {
                        class: "text-4xl font-bold text-gray-800 mb-4",
                        "🌟 LumosAI UI Component Showcase"
                    }
                    p {
                        class: "text-xl text-gray-600 mb-6",
                        "A comprehensive demonstration of all UI components"
                    }
                    div {
                        class: "flex justify-center space-x-4",
                        Button {
                            button_scheme: ButtonScheme::Primary,
                            "Get Started"
                        }
                        Button {
                            button_scheme: ButtonScheme::Secondary,
                            "Learn More"
                        }
                    }
                }
                
                // Component showcase grid
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8 mb-12",
                    
                    // Buttons showcase
                    Card {
                        class: "p-6",
                        h3 {
                            class: "text-xl font-semibold mb-4 text-gray-800",
                            "🔘 Buttons"
                        }
                        div {
                            class: "space-y-4",
                            div {
                                class: "space-y-2",
                                p { class: "text-sm font-medium text-gray-600", "Button Schemes:" }
                                div {
                                    class: "flex flex-wrap gap-2",
                                    Button {
                                        button_scheme: ButtonScheme::Primary,
                                        button_size: ButtonSize::Small,
                                        "Primary"
                                    }
                                    Button {
                                        button_scheme: ButtonScheme::Secondary,
                                        button_size: ButtonSize::Small,
                                        "Secondary"
                                    }
                                    Button {
                                        button_scheme: ButtonScheme::Success,
                                        button_size: ButtonSize::Small,
                                        "Success"
                                    }
                                    Button {
                                        button_scheme: ButtonScheme::Warning,
                                        button_size: ButtonSize::Small,
                                        "Warning"
                                    }
                                    Button {
                                        button_scheme: ButtonScheme::Error,
                                        button_size: ButtonSize::Small,
                                        "Error"
                                    }
                                }
                            }
                            div {
                                class: "space-y-2",
                                p { class: "text-sm font-medium text-gray-600", "Button Sizes:" }
                                div {
                                    class: "flex flex-wrap gap-2 items-center",
                                    Button {
                                        button_scheme: ButtonScheme::Primary,
                                        button_size: ButtonSize::Small,
                                        "Small"
                                    }
                                    Button {
                                        button_scheme: ButtonScheme::Primary,
                                        button_size: ButtonSize::Medium,
                                        "Medium"
                                    }
                                    Button {
                                        button_scheme: ButtonScheme::Primary,
                                        button_size: ButtonSize::Large,
                                        "Large"
                                    }
                                }
                            }
                        }
                    }
                    
                    // Forms showcase
                    Card {
                        class: "p-6",
                        h3 {
                            class: "text-xl font-semibold mb-4 text-gray-800",
                            "📝 Form Elements"
                        }
                        div {
                            class: "space-y-4",
                            Input {
                                input_type: InputType::Text,
                                name: "demo_text",
                                label: "Text Input",
                                placeholder: "Enter some text",
                                value: "".to_string(),
                            }
                            Input {
                                input_type: InputType::Email,
                                name: "demo_email",
                                label: "Email Input",
                                placeholder: "user@example.com",
                                value: "".to_string(),
                            }
                            Input {
                                input_type: InputType::Password,
                                name: "demo_password",
                                label: "Password Input",
                                placeholder: "Enter password",
                                value: "".to_string(),
                            }
                            Select {
                                name: "demo_select",
                                label: "Select Option",
                                value: "option1".to_string(),
                                SelectOption {
                                    value: "option1",
                                    selected_value: "option1".to_string(),
                                    "Option 1"
                                }
                                SelectOption {
                                    value: "option2",
                                    selected_value: "option1".to_string(),
                                    "Option 2"
                                }
                                SelectOption {
                                    value: "option3",
                                    selected_value: "option1".to_string(),
                                    "Option 3"
                                }
                            }
                        }
                    }
                    
                    // Navigation showcase
                    Card {
                        class: "p-6",
                        h3 {
                            class: "text-xl font-semibold mb-4 text-gray-800",
                            "🧭 Navigation"
                        }
                        div {
                            class: "space-y-4",
                            div {
                                p { class: "text-sm font-medium text-gray-600 mb-2", "Breadcrumb:" }
                                Breadcrumb {
                                    items: vec![
                                        BreadcrumbItem {
                                            text: "Home".into(),
                                            href: Some("/".into()),
                                        },
                                        BreadcrumbItem {
                                            text: "Components".into(),
                                            href: Some("/components".into()),
                                        },
                                        BreadcrumbItem {
                                            text: "Navigation".into(),
                                            href: None,
                                        }
                                    ]
                                }
                            }
                            div {
                                p { class: "text-sm font-medium text-gray-600 mb-2", "Menu:" }
                                nav {
                                    class: "bg-gray-100 rounded-lg p-3",
                                    ul {
                                        class: "space-y-1",
                                        li { 
                                            a { 
                                                href: "#", 
                                                class: "block px-3 py-2 text-sm text-gray-700 hover:bg-gray-200 rounded",
                                                "Dashboard" 
                                            } 
                                        }
                                        li { 
                                            a { 
                                                href: "#", 
                                                class: "block px-3 py-2 text-sm text-gray-700 hover:bg-gray-200 rounded",
                                                "Assistants" 
                                            } 
                                        }
                                        li { 
                                            a { 
                                                href: "#", 
                                                class: "block px-3 py-2 text-sm text-gray-700 hover:bg-gray-200 rounded",
                                                "Settings" 
                                            } 
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Labels showcase
                    Card {
                        class: "p-6",
                        h3 {
                            class: "text-xl font-semibold mb-4 text-gray-800",
                            "🏷️ Labels"
                        }
                        div {
                            class: "space-y-3",
                            div {
                                class: "flex flex-wrap gap-2",
                                Label {
                                    label_role: LabelRole::Success,
                                    "Success"
                                }
                                Label {
                                    label_role: LabelRole::Warning,
                                    "Warning"
                                }
                                Label {
                                    label_role: LabelRole::Danger,
                                    "Danger"
                                }
                                Label {
                                    label_role: LabelRole::Info,
                                    "Info"
                                }
                            }
                        }
                    }
                    
                    // Data display
                    Card {
                        class: "p-6",
                        h3 {
                            class: "text-xl font-semibold mb-4 text-gray-800",
                            "📊 Data Display"
                        }
                        div {
                            class: "space-y-4",
                            div {
                                p { class: "text-sm font-medium text-gray-600 mb-2", "Sample Table:" }
                                div {
                                    class: "overflow-x-auto",
                                    table {
                                        class: "min-w-full bg-white border border-gray-200 rounded-lg",
                                        thead {
                                            class: "bg-gray-50",
                                            tr {
                                                th { 
                                                    class: "px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase tracking-wider border-b",
                                                    "Name" 
                                                }
                                                th { 
                                                    class: "px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase tracking-wider border-b",
                                                    "Status" 
                                                }
                                            }
                                        }
                                        tbody {
                                            tr {
                                                class: "border-b",
                                                td { 
                                                    class: "px-4 py-2 text-sm text-gray-900",
                                                    "John Doe" 
                                                }
                                                td { 
                                                    class: "px-4 py-2",
                                                    Label {
                                                        label_role: LabelRole::Success,
                                                        "Active"
                                                    }
                                                }
                                            }
                                            tr {
                                                td { 
                                                    class: "px-4 py-2 text-sm text-gray-900",
                                                    "Jane Smith" 
                                                }
                                                td { 
                                                    class: "px-4 py-2",
                                                    Label {
                                                        label_role: LabelRole::Warning,
                                                        "Pending"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Cards showcase
                    Card {
                        class: "p-6",
                        h3 {
                            class: "text-xl font-semibold mb-4 text-gray-800",
                            "🃏 Cards"
                        }
                        div {
                            class: "space-y-4",
                            Card {
                                class: "p-4 bg-blue-50 border-blue-200",
                                div {
                                    h4 { class: "font-medium text-blue-900 mb-2", "Info Card" }
                                    p { class: "text-blue-700 text-sm", "This is an informational card with custom styling." }
                                }
                            }
                            Card {
                                class: "p-4 bg-green-50 border-green-200",
                                div {
                                    h4 { class: "font-medium text-green-900 mb-2", "Success Card" }
                                    p { class: "text-green-700 text-sm", "This indicates a successful operation." }
                                }
                            }
                        }
                    }
                }
                
                // Status section
                div {
                    class: "text-center",
                    Card {
                        class: "p-8 bg-green-50 border-green-200",
                        div {
                            class: "text-green-800",
                            h2 {
                                class: "text-2xl font-bold mb-4",
                                "✅ All Components Working"
                            }
                            p {
                                class: "text-lg mb-6",
                                "LumosAI UI components are fully functional and ready for production use."
                            }
                            div {
                                class: "flex justify-center space-x-4",
                                Button {
                                    button_scheme: ButtonScheme::Success,
                                    "View Documentation"
                                }
                                Button {
                                    button_scheme: ButtonScheme::Primary,
                                    "Start Building"
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>LumosAI UI Component Showcase</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <style>
        /* Custom styles for better component appearance */
        .btn {{ 
            @apply inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-offset-2;
        }}
        .btn-primary {{ @apply text-white bg-blue-600 hover:bg-blue-700 focus:ring-blue-500; }}
        .btn-secondary {{ @apply text-gray-700 bg-white hover:bg-gray-50 border-gray-300 focus:ring-blue-500; }}
        .btn-success {{ @apply text-white bg-green-600 hover:bg-green-700 focus:ring-green-500; }}
        .btn-warning {{ @apply text-white bg-yellow-600 hover:bg-yellow-700 focus:ring-yellow-500; }}
        .btn-error {{ @apply text-white bg-red-600 hover:bg-red-700 focus:ring-red-500; }}
        
        .label {{ @apply inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium; }}
        .label-success {{ @apply bg-green-100 text-green-800; }}
        .label-warning {{ @apply bg-yellow-100 text-yellow-800; }}
        .label-danger {{ @apply bg-red-100 text-red-800; }}
        .label-info {{ @apply bg-blue-100 text-blue-800; }}
        
        .card {{ @apply bg-white overflow-hidden shadow rounded-lg border; }}
        
        .form-input {{ 
            @apply mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-blue-500 focus:border-blue-500;
        }}
        
        .form-label {{ @apply block text-sm font-medium text-gray-700 mb-1; }}
        
        .breadcrumb {{ @apply flex items-center space-x-2 text-sm text-gray-500; }}
        .breadcrumb a {{ @apply text-blue-600 hover:text-blue-800; }}
        .breadcrumb-separator {{ @apply text-gray-400; }}
    </style>
</head>
<body>
    {}
    <script>
        // Add some interactivity
        document.addEventListener('DOMContentLoaded', function() {{
            console.log('🌟 LumosAI UI Component Showcase loaded successfully!');
            
            // Add click handlers to buttons
            document.querySelectorAll('button').forEach(button => {{
                button.addEventListener('click', function(e) {{
                    e.preventDefault();
                    console.log('Button clicked:', this.textContent);
                    
                    // Add a visual feedback
                    this.style.transform = 'scale(0.95)';
                    setTimeout(() => {{
                        this.style.transform = 'scale(1)';
                    }}, 100);
                }});
            }});
            
            // Add form interaction
            document.querySelectorAll('input, select').forEach(input => {{
                input.addEventListener('focus', function() {{
                    console.log('Input focused:', this.name || this.type);
                }});
            }});
        }});
    </script>
</body>
</html>"#,
        body_content
    )
}
