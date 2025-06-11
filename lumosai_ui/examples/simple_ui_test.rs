/*!
# Simple UI Test

This example tests basic UI functionality without complex layouts.
*/

use lumosai_ui::prelude::*;

fn main() {
    println!("🎨 LumosAI UI Simple Test");
    
    // Test basic component rendering
    test_basic_components();
    
    // Test HTML generation
    test_html_generation();
    
    println!("✅ All UI tests completed successfully!");
}

fn test_basic_components() {
    println!("\n📦 Testing basic components...");
    
    // Test Button component
    let button_html = render(rsx! {
        Button {
            button_scheme: ButtonScheme::Primary,
            button_size: ButtonSize::Medium,
            "Test Button"
        }
    });
    
    println!("✅ Button component rendered: {} chars", button_html.len());
    
    // Test Card component
    let card_html = render(rsx! {
        Card {
            class: "test-card",
            div {
                class: "p-4",
                h3 { "Test Card" }
                p { "This is a test card component." }
            }
        }
    });
    
    println!("✅ Card component rendered: {} chars", card_html.len());
    
    // Test Input component
    let input_html = render(rsx! {
        Input {
            input_type: InputType::Text,
            name: "test_input",
            label: "Test Input",
            placeholder: "Enter test value",
            value: "".to_string(),
        }
    });
    
    println!("✅ Input component rendered: {} chars", input_html.len());
}

fn test_html_generation() {
    println!("\n🌐 Testing HTML generation...");
    
    // Test a simple page structure
    let page_html = render(rsx! {
        div {
            class: "container mx-auto p-4",
            header {
                class: "mb-6",
                h1 {
                    class: "text-3xl font-bold text-gray-800",
                    "LumosAI UI Test Page"
                }
                p {
                    class: "text-gray-600",
                    "Testing UI component functionality"
                }
            }
            main {
                class: "space-y-4",
                Card {
                    div {
                        class: "p-6",
                        h2 {
                            class: "text-xl font-semibold mb-4",
                            "Welcome to LumosAI"
                        }
                        p {
                            class: "mb-4",
                            "This is a test of the LumosAI UI components."
                        }
                        div {
                            class: "flex space-x-2",
                            Button {
                                button_scheme: ButtonScheme::Primary,
                                "Primary Action"
                            }
                            Button {
                                button_scheme: ButtonScheme::Secondary,
                                "Secondary Action"
                            }
                        }
                    }
                }
                Card {
                    div {
                        class: "p-6",
                        h3 {
                            class: "text-lg font-medium mb-3",
                            "Form Example"
                        }
                        div {
                            class: "space-y-3",
                            Input {
                                input_type: InputType::Text,
                                name: "username",
                                label: "Username",
                                placeholder: "Enter your username",
                                value: "".to_string(),
                            }
                            Input {
                                input_type: InputType::Email,
                                name: "email",
                                label: "Email",
                                placeholder: "Enter your email",
                                value: "".to_string(),
                            }
                            Button {
                                button_scheme: ButtonScheme::Success,
                                "Submit"
                            }
                        }
                    }
                }
            }
        }
    });
    
    println!("✅ Complete page rendered: {} chars", page_html.len());
    
    // Verify HTML contains expected elements
    assert!(page_html.contains("LumosAI UI Test Page"));
    assert!(page_html.contains("Primary Action"));
    assert!(page_html.contains("input"));
    assert!(page_html.contains("button"));
    
    println!("✅ HTML validation passed");
}

#[component]
fn TestApp() -> Element {
    rsx! {
        div {
            class: "min-h-screen bg-gray-100 py-8",
            div {
                class: "max-w-4xl mx-auto px-4",
                h1 {
                    class: "text-4xl font-bold text-center mb-8 text-gray-800",
                    "🌟 LumosAI UI Component Test"
                }
                
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                    
                    // Button showcase
                    Card {
                        class: "p-6",
                        h2 {
                            class: "text-2xl font-semibold mb-4",
                            "Buttons"
                        }
                        div {
                            class: "space-y-3",
                            div {
                                class: "flex flex-wrap gap-2",
                                Button {
                                    button_scheme: ButtonScheme::Primary,
                                    "Primary"
                                }
                                Button {
                                    button_scheme: ButtonScheme::Secondary,
                                    "Secondary"
                                }
                                Button {
                                    button_scheme: ButtonScheme::Success,
                                    "Success"
                                }
                                Button {
                                    button_scheme: ButtonScheme::Warning,
                                    "Warning"
                                }
                                Button {
                                    button_scheme: ButtonScheme::Error,
                                    "Error"
                                }
                            }
                            div {
                                class: "flex flex-wrap gap-2",
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
                    
                    // Form showcase
                    Card {
                        class: "p-6",
                        h2 {
                            class: "text-2xl font-semibold mb-4",
                            "Form Elements"
                        }
                        div {
                            class: "space-y-4",
                            Input {
                                input_type: InputType::Text,
                                name: "sample_text",
                                label: "Text Input",
                                placeholder: "Enter some text",
                                value: "".to_string(),
                            }
                            Input {
                                input_type: InputType::Email,
                                name: "sample_email",
                                label: "Email Input",
                                placeholder: "user@example.com",
                                value: "".to_string(),
                            }
                            Input {
                                input_type: InputType::Password,
                                name: "sample_password",
                                label: "Password Input",
                                placeholder: "Enter password",
                                value: "".to_string(),
                            }
                        }
                    }
                }
                
                // Status section
                div {
                    class: "mt-8 text-center",
                    Card {
                        class: "p-6 bg-green-50 border-green-200",
                        div {
                            class: "text-green-800",
                            h3 {
                                class: "text-xl font-semibold mb-2",
                                "✅ UI Components Working"
                            }
                            p {
                                "All LumosAI UI components are rendering correctly!"
                            }
                        }
                    }
                }
            }
        }
    }
}
