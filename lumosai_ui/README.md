# LumosAI UI

A modern UI component library for LumosAI applications, based on bionic-gpt's proven design patterns.

## 🚀 Features

- **Modern Web Components** - Built with Dioxus and Rust
- **Responsive Design** - Mobile-first responsive layouts
- **DaisyUI Integration** - Beautiful, accessible components
- **AI-Focused Patterns** - Specialized components for AI applications
- **Dark/Light Themes** - Built-in theme switching
- **TypeScript Support** - Enhanced interactivity with TypeScript

## 📦 What's Included

### Core Components
- **Layout System** - Base layouts, app layouts, navigation
- **UI Components** - Buttons, forms, modals, cards
- **AI Components** - Chat interfaces, assistant management
- **Interactive Features** - File upload, voice input, real-time chat

### Modules
- 🤖 **Assistants** - AI assistant management interfaces
- 💬 **Console** - Chat and conversation interfaces  
- 🔄 **Workflows** - Workflow editing and management
- 📊 **Datasets** - Data management interfaces
- 🔧 **Models** - Model configuration interfaces
- 🔑 **API Keys** - API key management
- 👥 **Teams** - Team collaboration features
- 📈 **Analytics** - Charts and metrics display

## 🛠️ Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
lumosai_ui = { path = "path/to/lumosai_ui" }
```

## 📖 Usage

### Basic Setup

```rust
use lumosai_ui::prelude::*;

#[component]
fn App() -> Element {
    rsx! {
        BaseLayout {
            title: "LumosAI Dashboard",
            fav_icon_src: "/favicon.svg",
            stylesheets: vec!["/styles.css".to_string()],
            js_href: "/app.js",
            section_class: "p-4",
            header: rsx! {
                h1 { "Welcome to LumosAI" }
            },
            sidebar: rsx! {
                Menu { /* navigation items */ }
            },
            sidebar_header: rsx! {
                div { "LumosAI" }
            },
            sidebar_footer: rsx! {
                div { "v1.0.0" }
            },
            // Main content
            div {
                class: "container mx-auto",
                h2 { "Dashboard" }
                // Your app content here
            }
        }
    }
}
```

### Using Components

```rust
use lumosai_ui::prelude::*;

#[component]
fn ChatInterface() -> Element {
    rsx! {
        console::Console {
            team_id: 1,
            // Chat configuration
        }
    }
}

#[component]
fn AssistantManager() -> Element {
    rsx! {
        my_assistants::AssistantCard {
            assistant_name: "My Assistant",
            description: "A helpful AI assistant",
            // Assistant configuration
        }
    }
}
```

## 🎨 Styling

The library uses Tailwind CSS and DaisyUI for styling. Make sure to include the CSS in your project:

```html
<!-- Include Tailwind CSS and DaisyUI -->
<link href="https://cdn.jsdelivr.net/npm/daisyui@4.4.24/dist/full.min.css" rel="stylesheet" type="text/css" />
<script src="https://cdn.tailwindcss.com"></script>
```

## 🏗️ Architecture

```
lumosai_ui/
├── src/lib.rs          # Main library exports
├── web-pages/          # UI Components
│   ├── types.rs        # Type definitions
│   ├── lib.rs          # Component exports
│   ├── console/        # Chat interfaces
│   ├── assistants/     # Assistant management
│   ├── workflows/      # Workflow components
│   └── ...             # Other modules
└── web-assets/         # Frontend Assets
    ├── typescript/     # Interactive features
    ├── scss/           # Styling
    └── images/         # Icons and images
```

## 🔧 Development

### Prerequisites
- Rust 1.70+
- Node.js 18+
- npm or yarn

### Building

```bash
# Build the Rust components
cargo build

# Build the frontend assets
cd web-assets
npm install
npm run build
```

### Running Examples

```bash
# Run with cargo
cargo run --example basic_layout

# Or with a web server
cargo run --example web_server
```

## 📚 Components Reference

### Layout Components
- `BaseLayout` - Main page layout with sidebar
- `AppLayout` - Application-specific layout
- `Menu` - Navigation menu component

### UI Components  
- `Button` - Various button styles and states
- `Card` - Content cards with headers/footers
- `Modal` - Modal dialogs and overlays
- `Input` - Form input components
- `Select` - Dropdown selection components

### AI-Specific Components
- `Console` - Chat interface with streaming
- `AssistantCard` - Assistant management card
- `WorkflowEditor` - Visual workflow editor
- `ModelSelector` - Model configuration interface

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

Based on the excellent design patterns from [bionic-gpt](https://github.com/bionic-gpt/bionic-gpt).
