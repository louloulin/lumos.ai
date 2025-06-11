/*!
# LumosAI Interactive Web Application

This example shows how to create a fully interactive Dioxus web application using LumosAI UI components.

## Running the Application

First install trunk:
```bash
cargo install trunk
```

Then run:
```bash
trunk serve --open
```

Or for a simple server:
```bash
cargo run --example web_app
```
*/

use lumosai_ui::prelude::*;
use dioxus::prelude::*;
use std::io::{prelude::*, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() {
    println!("🚀 Starting LumosAI Interactive Web Application");
    println!("===============================================");
    
    // Create a simple HTTP server to serve the application
    start_web_server();
}

fn start_web_server() {
    
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("🌐 Server running at: http://127.0.0.1:8080");
    println!("📱 Open this URL in your browser to view the application");
    
    // Try to open browser automatically
    if let Err(e) = open::that("http://127.0.0.1:8080") {
        println!("⚠️  Could not open browser automatically: {}", e);
        println!("📖 Please manually open: http://127.0.0.1:8080");
    }
    
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    
    let request = String::from_utf8_lossy(&buffer[..]);
    let path = extract_path(&request);
    
    let (status, content_type, body) = match path.as_str() {
        "/" | "/index.html" => {
            ("200 OK", "text/html", generate_interactive_html())
        },
        "/app.js" => {
            ("200 OK", "application/javascript", generate_app_js())
        },
        "/styles.css" => {
            ("200 OK", "text/css", generate_app_css())
        },
        _ => {
            ("404 NOT FOUND", "text/html", "<h1>404 - Page Not Found</h1>".to_string())
        }
    };
    
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        status,
        content_type,
        body.len(),
        body
    );
    
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn extract_path(request: &str) -> String {
    let lines: Vec<&str> = request.lines().collect();
    if let Some(first_line) = lines.first() {
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() >= 2 {
            return parts[1].to_string();
        }
    }
    "/".to_string()
}

fn generate_interactive_html() -> String {
    // Generate the main app component
    let app_html = render(rsx! { InteractiveApp {} });
    
    format!(
        r#"<!DOCTYPE html>
<html lang="en" data-theme="light">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>🌟 LumosAI Interactive Dashboard</title>
    <link href="https://cdn.jsdelivr.net/npm/daisyui@4.4.19/dist/full.min.css" rel="stylesheet" type="text/css" />
    <script src="https://cdn.tailwindcss.com"></script>
    <link rel="stylesheet" href="/styles.css">
</head>
<body>
    <div id="app">
        {}
    </div>
    <script src="/app.js"></script>
</body>
</html>"#,
        app_html
    )
}

fn generate_app_js() -> String {
    r#"
// LumosAI Interactive Dashboard JavaScript
console.log('🌟 LumosAI Dashboard loaded successfully!');

// Application state
let appState = {
    currentPage: 'dashboard',
    theme: 'light',
    sidebarCollapsed: false,
    assistants: [
        { id: 1, name: 'Customer Support Bot', status: 'Active', conversations: 156 },
        { id: 2, name: 'Sales Assistant', status: 'Active', conversations: 89 },
        { id: 3, name: 'Technical Helper', status: 'Inactive', conversations: 23 }
    ],
    stats: {
        activeAssistants: 12,
        totalConversations: 1234,
        apiCallsToday: 5678,
        successRate: '98.5%'
    }
};

// DOM manipulation functions
function updatePage(page) {
    appState.currentPage = page;
    updateNavigation();
    updateMainContent();
    console.log('📄 Switched to page:', page);
}

function toggleSidebar() {
    appState.sidebarCollapsed = !appState.sidebarCollapsed;
    const sidebar = document.querySelector('.sidebar');
    if (sidebar) {
        if (appState.sidebarCollapsed) {
            sidebar.classList.remove('w-64');
            sidebar.classList.add('w-16');
        } else {
            sidebar.classList.remove('w-16');
            sidebar.classList.add('w-64');
        }
    }
    console.log('📱 Sidebar toggled:', appState.sidebarCollapsed ? 'collapsed' : 'expanded');
}

function toggleTheme() {
    appState.theme = appState.theme === 'light' ? 'dark' : 'light';
    document.documentElement.setAttribute('data-theme', appState.theme);
    updateThemeButton();
    console.log('🎨 Theme changed to:', appState.theme);
}

function updateNavigation() {
    const navItems = document.querySelectorAll('.nav-item');
    navItems.forEach(item => {
        const page = item.getAttribute('data-page');
        if (page === appState.currentPage) {
            item.classList.add('bg-primary', 'text-primary-content');
            item.classList.remove('hover:bg-base-300');
        } else {
            item.classList.remove('bg-primary', 'text-primary-content');
            item.classList.add('hover:bg-base-300');
        }
    });
}

function updateMainContent() {
    const mainContent = document.querySelector('.main-content');
    if (!mainContent) return;
    
    let content = '';
    switch (appState.currentPage) {
        case 'dashboard':
            content = generateDashboardContent();
            break;
        case 'assistants':
            content = generateAssistantsContent();
            break;
        case 'console':
            content = generateConsoleContent();
            break;
        case 'analytics':
            content = generateAnalyticsContent();
            break;
        case 'settings':
            content = generateSettingsContent();
            break;
        default:
            content = generateDashboardContent();
    }
    
    mainContent.innerHTML = content;
    attachEventListeners();
}

function generateDashboardContent() {
    return `
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <h2 class="text-3xl font-bold">📊 Dashboard</h2>
                <div class="flex space-x-2">
                    <button class="btn btn-primary" onclick="showNotification('New Assistant feature coming soon!')">New Assistant</button>
                    <button class="btn btn-secondary" onclick="showNotification('Import Data feature coming soon!')">Import Data</button>
                </div>
            </div>
            
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <div class="card bg-base-100 shadow-xl">
                    <div class="card-body">
                        <div class="flex items-center">
                            <div class="p-3 rounded-full text-white bg-blue-500">🤖</div>
                            <div class="ml-4">
                                <h3 class="text-sm font-medium text-gray-600">Active Assistants</h3>
                                <p class="text-2xl font-bold">${appState.stats.activeAssistants}</p>
                            </div>
                        </div>
                    </div>
                </div>
                
                <div class="card bg-base-100 shadow-xl">
                    <div class="card-body">
                        <div class="flex items-center">
                            <div class="p-3 rounded-full text-white bg-green-500">💬</div>
                            <div class="ml-4">
                                <h3 class="text-sm font-medium text-gray-600">Total Conversations</h3>
                                <p class="text-2xl font-bold">${appState.stats.totalConversations.toLocaleString()}</p>
                            </div>
                        </div>
                    </div>
                </div>
                
                <div class="card bg-base-100 shadow-xl">
                    <div class="card-body">
                        <div class="flex items-center">
                            <div class="p-3 rounded-full text-white bg-purple-500">📡</div>
                            <div class="ml-4">
                                <h3 class="text-sm font-medium text-gray-600">API Calls Today</h3>
                                <p class="text-2xl font-bold">${appState.stats.apiCallsToday.toLocaleString()}</p>
                            </div>
                        </div>
                    </div>
                </div>
                
                <div class="card bg-base-100 shadow-xl">
                    <div class="card-body">
                        <div class="flex items-center">
                            <div class="p-3 rounded-full text-white bg-orange-500">✅</div>
                            <div class="ml-4">
                                <h3 class="text-sm font-medium text-gray-600">Success Rate</h3>
                                <p class="text-2xl font-bold">${appState.stats.successRate}</p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            
            <div class="card bg-base-100 shadow-xl">
                <div class="card-body">
                    <h3 class="text-xl font-semibold mb-4">Recent Activity</h3>
                    <div class="space-y-3">
                        <div class="flex items-center space-x-3 p-3 hover:bg-base-200 rounded-lg">
                            <div class="text-lg">🤖</div>
                            <div class="flex-1">
                                <h4 class="font-medium">New assistant created</h4>
                                <p class="text-sm text-gray-600">Customer Support Bot v2.0</p>
                            </div>
                            <div class="text-xs text-gray-500">2 minutes ago</div>
                        </div>
                        <div class="flex items-center space-x-3 p-3 hover:bg-base-200 rounded-lg">
                            <div class="text-lg">💬</div>
                            <div class="flex-1">
                                <h4 class="font-medium">Conversation completed</h4>
                                <p class="text-sm text-gray-600">User inquiry about pricing</p>
                            </div>
                            <div class="text-xs text-gray-500">5 minutes ago</div>
                        </div>
                        <div class="flex items-center space-x-3 p-3 hover:bg-base-200 rounded-lg">
                            <div class="text-lg">📊</div>
                            <div class="flex-1">
                                <h4 class="font-medium">Analytics report generated</h4>
                                <p class="text-sm text-gray-600">Weekly performance summary</p>
                            </div>
                            <div class="text-xs text-gray-500">1 hour ago</div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    `;
}

function generateAssistantsContent() {
    const assistantCards = appState.assistants.map(assistant => `
        <div class="card bg-base-100 shadow-xl">
            <div class="card-body">
                <div class="space-y-4">
                    <div class="flex items-center justify-between">
                        <h3 class="text-lg font-semibold">${assistant.name}</h3>
                        <span class="badge ${assistant.status === 'Active' ? 'badge-success' : 'badge-warning'}">${assistant.status}</span>
                    </div>
                    <p class="text-gray-600">AI assistant for customer support and inquiries</p>
                    <div class="flex items-center justify-between">
                        <span class="text-sm text-gray-500">${assistant.conversations} conversations</span>
                        <button class="btn btn-primary btn-sm" onclick="showNotification('Configure ${assistant.name}')">Configure</button>
                    </div>
                </div>
            </div>
        </div>
    `).join('');
    
    return `
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <h2 class="text-3xl font-bold">🤖 AI Assistants</h2>
                <button class="btn btn-primary" onclick="showNotification('Create new assistant feature coming soon!')">Create New</button>
            </div>
            
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                ${assistantCards}
            </div>
        </div>
    `;
}

function generateConsoleContent() {
    return `
        <div class="space-y-6">
            <h2 class="text-3xl font-bold">💬 AI Console</h2>
            
            <div class="card bg-base-100 shadow-xl">
                <div class="card-body">
                    <div class="space-y-4">
                        <div class="h-96 bg-base-200 rounded-lg p-4 overflow-y-auto" id="chat-container">
                            <div class="space-y-3">
                                <div class="flex justify-end">
                                    <div class="bg-primary text-primary-content p-3 rounded-lg max-w-xs">
                                        Hello! Can you help me with my project?
                                    </div>
                                </div>
                                <div class="flex justify-start">
                                    <div class="bg-base-300 p-3 rounded-lg max-w-xs">
                                        Of course! I'd be happy to help you with your project. What specific area would you like assistance with?
                                    </div>
                                </div>
                            </div>
                        </div>
                        
                        <div class="flex space-x-2">
                            <input type="text" placeholder="Type your message..." class="input input-bordered flex-1" id="message-input" onkeypress="handleMessageKeyPress(event)">
                            <button class="btn btn-primary" onclick="sendMessage()">Send</button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    `;
}

function generateAnalyticsContent() {
    return `
        <div class="space-y-6">
            <h2 class="text-3xl font-bold">📈 Analytics</h2>
            <div class="card bg-base-100 shadow-xl">
                <div class="card-body">
                    <p class="text-gray-600">Analytics dashboard with charts and metrics coming soon...</p>
                    <div class="mt-4">
                        <button class="btn btn-primary" onclick="showNotification('Analytics features will be available in the next update!')">View Reports</button>
                    </div>
                </div>
            </div>
        </div>
    `;
}

function generateSettingsContent() {
    return `
        <div class="space-y-6">
            <h2 class="text-3xl font-bold">⚙️ Settings</h2>
            
            <div class="card bg-base-100 shadow-xl">
                <div class="card-body">
                    <h3 class="text-lg font-semibold mb-4">General Settings</h3>
                    <div class="space-y-4">
                        <div class="form-control">
                            <label class="label">
                                <span class="label-text">Application Name</span>
                            </label>
                            <input type="text" value="LumosAI Dashboard" class="input input-bordered">
                        </div>
                        
                        <div class="form-control">
                            <label class="label">
                                <span class="label-text">Admin Email</span>
                            </label>
                            <input type="email" value="admin@lumosai.com" class="input input-bordered">
                        </div>
                        
                        <div class="form-control">
                            <label class="label">
                                <span class="label-text">Theme</span>
                            </label>
                            <select class="select select-bordered" onchange="handleThemeChange(this.value)">
                                <option value="light" ${appState.theme === 'light' ? 'selected' : ''}>Light</option>
                                <option value="dark" ${appState.theme === 'dark' ? 'selected' : ''}>Dark</option>
                            </select>
                        </div>
                        
                        <div class="pt-4">
                            <button class="btn btn-primary" onclick="showNotification('Settings saved successfully!')">Save Settings</button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    `;
}

function updateThemeButton() {
    const themeButton = document.querySelector('.theme-toggle');
    if (themeButton) {
        themeButton.textContent = appState.theme === 'light' ? '🌙' : '☀️';
    }
}

function showNotification(message) {
    // Create notification element
    const notification = document.createElement('div');
    notification.className = 'alert alert-info fixed top-4 right-4 w-auto z-50 shadow-lg';
    notification.innerHTML = `
        <div class="flex items-center">
            <span>ℹ️</span>
            <span>${message}</span>
        </div>
    `;
    
    document.body.appendChild(notification);
    
    // Remove after 3 seconds
    setTimeout(() => {
        notification.remove();
    }, 3000);
}

function sendMessage() {
    const input = document.getElementById('message-input');
    const message = input.value.trim();
    
    if (message) {
        const chatContainer = document.getElementById('chat-container');
        const messageDiv = document.createElement('div');
        messageDiv.className = 'flex justify-end';
        messageDiv.innerHTML = `
            <div class="bg-primary text-primary-content p-3 rounded-lg max-w-xs">
                ${message}
            </div>
        `;
        
        chatContainer.appendChild(messageDiv);
        input.value = '';
        
        // Simulate AI response
        setTimeout(() => {
            const responseDiv = document.createElement('div');
            responseDiv.className = 'flex justify-start';
            responseDiv.innerHTML = `
                <div class="bg-base-300 p-3 rounded-lg max-w-xs">
                    Thanks for your message! This is a demo response. In a real application, this would connect to an AI service.
                </div>
            `;
            chatContainer.appendChild(responseDiv);
            chatContainer.scrollTop = chatContainer.scrollHeight;
        }, 1000);
        
        chatContainer.scrollTop = chatContainer.scrollHeight;
    }
}

function handleMessageKeyPress(event) {
    if (event.key === 'Enter') {
        sendMessage();
    }
}

function handleThemeChange(theme) {
    appState.theme = theme;
    document.documentElement.setAttribute('data-theme', theme);
    updateThemeButton();
    showNotification(`Theme changed to ${theme}`);
}

function attachEventListeners() {
    // Attach event listeners for dynamically created content
    const navItems = document.querySelectorAll('.nav-item');
    navItems.forEach(item => {
        item.addEventListener('click', () => {
            const page = item.getAttribute('data-page');
            updatePage(page);
        });
    });
}

// Initialize the application
document.addEventListener('DOMContentLoaded', function() {
    console.log('🚀 Initializing LumosAI Dashboard...');
    
    // Set up navigation
    updateNavigation();
    updateMainContent();
    updateThemeButton();
    
    // Attach global event listeners
    const sidebarToggle = document.querySelector('.sidebar-toggle');
    if (sidebarToggle) {
        sidebarToggle.addEventListener('click', toggleSidebar);
    }
    
    const themeToggle = document.querySelector('.theme-toggle');
    if (themeToggle) {
        themeToggle.addEventListener('click', toggleTheme);
    }
    
    attachEventListeners();
    
    console.log('✅ LumosAI Dashboard initialized successfully!');
    showNotification('Welcome to LumosAI Dashboard! 🌟');
});
"#.to_string()
}

fn generate_app_css() -> String {
    r#"
/* LumosAI Dashboard Custom Styles */

.navbar {
    @apply sticky top-0 z-50;
}

.sidebar {
    @apply transition-all duration-300;
}

.nav-item {
    @apply transition-colors duration-200 cursor-pointer;
}

.stats-card {
    @apply transition-transform hover:scale-105;
}

.main-content {
    @apply flex-1 p-6;
}

/* Custom animations */
@keyframes fadeIn {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
}

.fade-in {
    animation: fadeIn 0.3s ease-out;
}

/* Responsive design improvements */
@media (max-width: 768px) {
    .sidebar {
        @apply w-16;
    }
    
    .main-content {
        @apply p-4;
    }
}

/* Theme-specific styles */
[data-theme="dark"] {
    --fallback-b1: #1f2937;
    --fallback-b2: #374151;
    --fallback-b3: #4b5563;
}

/* Loading states */
.loading-skeleton {
    @apply animate-pulse bg-base-300 rounded;
}

/* Interactive elements */
.interactive-hover {
    @apply transition-all duration-200 hover:shadow-lg hover:-translate-y-1;
}

/* Chat styles */
#chat-container {
    scrollbar-width: thin;
    scrollbar-color: rgba(156, 163, 175, 0.5) transparent;
}

#chat-container::-webkit-scrollbar {
    width: 6px;
}

#chat-container::-webkit-scrollbar-track {
    background: transparent;
}

#chat-container::-webkit-scrollbar-thumb {
    background-color: rgba(156, 163, 175, 0.5);
    border-radius: 3px;
}

#chat-container::-webkit-scrollbar-thumb:hover {
    background-color: rgba(156, 163, 175, 0.7);
}
"#.to_string()
}

#[component]
fn InteractiveApp() -> Element {
    rsx! {
        div {
            class: "min-h-screen bg-base-100",
            
            // Navigation Header
            header {
                class: "navbar bg-base-200 shadow-lg",
                div {
                    class: "navbar-start",
                    button {
                        class: "btn btn-ghost btn-circle sidebar-toggle",
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
                            class: "btn btn-ghost btn-sm theme-toggle",
                            "🌙"
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
                    class: "w-64 bg-base-200 min-h-screen transition-all duration-300 sidebar",
                    
                    nav {
                        class: "p-4",
                        ul {
                            class: "space-y-2",
                            
                            li {
                                button {
                                    class: "w-full flex items-center p-3 text-left bg-primary text-primary-content rounded-lg nav-item",
                                    "data-page": "dashboard",
                                    
                                    span { class: "text-lg", "📊" }
                                    span { class: "ml-3", "Dashboard" }
                                }
                            }
                            
                            li {
                                button {
                                    class: "w-full flex items-center p-3 text-left hover:bg-base-300 rounded-lg nav-item",
                                    "data-page": "assistants",
                                    
                                    span { class: "text-lg", "🤖" }
                                    span { class: "ml-3", "Assistants" }
                                }
                            }
                            
                            li {
                                button {
                                    class: "w-full flex items-center p-3 text-left hover:bg-base-300 rounded-lg nav-item",
                                    "data-page": "console",
                                    
                                    span { class: "text-lg", "💬" }
                                    span { class: "ml-3", "Console" }
                                }
                            }
                            
                            li {
                                button {
                                    class: "w-full flex items-center p-3 text-left hover:bg-base-300 rounded-lg nav-item",
                                    "data-page": "analytics",
                                    
                                    span { class: "text-lg", "📈" }
                                    span { class: "ml-3", "Analytics" }
                                }
                            }
                            
                            li {
                                button {
                                    class: "w-full flex items-center p-3 text-left hover:bg-base-300 rounded-lg nav-item",
                                    "data-page": "settings",
                                    
                                    span { class: "text-lg", "⚙️" }
                                    span { class: "ml-3", "Settings" }
                                }
                            }
                        }
                    }
                }
                
                // Main Content
                main {
                    class: "main-content",
                    
                    // Content will be dynamically updated by JavaScript
                    div {
                        class: "space-y-6",
                        h2 {
                            class: "text-3xl font-bold",
                            "📊 Dashboard"
                        }
                        p {
                            class: "text-gray-600",
                            "Loading dashboard content..."
                        }
                    }
                }
            }
        }
    }
}
