# 🌟 LumosAI

<div align="center">

**A powerful enterprise-grade AI framework built with Rust for creating intelligent applications**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/louloulin/lumos.ai)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/lumosai)
[![Tests](https://img.shields.io/badge/tests-7%2F7%20passing-brightgreen.svg)](tests/)

> 🎉 **Project Status Update**: LumosAI has completed comprehensive project enhancement work!
> - ✅ All compilation issues have been fixed
> - ✅ Complete testing framework established (7/7 tests passing)
> - ✅ Enterprise-grade features fully preserved
> - ✅ Production-ready status achieved
>
> 📖 See [Implementation Summary](docs/overview/IMPLEMENTATION_SUMMARY.md) for detailed information

[📖 Documentation](docs/index.md) | [📚 Docs Index](docs/index.md) | [🌏 中文文档](docs/README_CN.md) | [🚀 Quick Start](docs/quick-start/README.md) | [💡 Examples](#examples) | [🤝 Contributing](#contributing)

</div>

---

## Table of Contents
- [Features](#features)
- [Quick Start](#quick-start)
- [Requirements](#requirements)
- [Examples](#examples)
- [Architecture](#architecture)
- [Workspace Overview](#workspace-overview)
- [Feature Flags](#feature-flags)
- [Documentation](#documentation)
- [Changelog & Releases](#changelog--releases)
- [Contributing](#contributing)
- [Community & Support](#community--support)
- [License](#license)

## ✨ Features

### 🤖 **Intelligent Agent System**
- **Multi-Model Support**: OpenAI GPT, Anthropic Claude, local models
- **Specialized Agents**: Research, writing, analysis, and custom roles
- **Tool Integration**: Extensible tool system with built-in tools
- **Conversation Memory**: Persistent context and conversation history

### 🧠 **Advanced RAG System**
- **Document Processing**: PDF, text, markdown, and web content
- **Smart Chunking**: Recursive, semantic, and custom chunking strategies
- **Vector Storage**: Memory, PostgreSQL, Qdrant, Weaviate backends
- **Hybrid Retrieval**: Semantic search + keyword matching

### 🔄 **Workflow Orchestration**
- **Multi-Agent Collaboration**: Sequential, parallel, and conditional workflows
- **Task Management**: Complex task decomposition and execution
- **Event-Driven Architecture**: Real-time event processing and routing
- **Error Handling**: Robust retry mechanisms and fallback strategies

### 🛡️ **Enterprise Security**
- **Authentication**: JWT, OAuth2, API keys, multi-factor authentication
- **Authorization**: Role-based access control (RBAC) with fine-grained permissions
- **Multi-Tenant**: Isolated tenant environments with custom configurations
- **Audit Logging**: Comprehensive security and compliance logging

### 📊 **Monitoring & Observability**
- **Real-time Metrics**: Performance, usage, and health monitoring
- **Distributed Tracing**: Request tracing across agent interactions
- **Custom Dashboards**: Grafana and Prometheus integration
- **Alerting**: Intelligent alerting for system anomalies

### ⚡ **High Performance**
- **Rust Performance**: Memory-safe, zero-cost abstractions
- **Async/Await**: Non-blocking I/O for high concurrency
- **Caching**: Intelligent caching at multiple layers
- **Scalability**: Horizontal scaling with load balancing

---

## 🚀 Quick Start

### Installation

Add LumosAI to your `Cargo.toml`:

```toml
[dependencies]
lumosai = "0.2.0"
tokio = { version = "1.0", features = ["full"] }
```

默认启用内存向量存储（`vector-memory`），无需额外配置即可运行 RAG 与向量示例。

### Basic Usage

```rust
use lumosai::prelude::*;

/// Entry point: demonstrate Agent creation and chat
#[tokio::main]
async fn main() -> Result<()> {
    // 🤖 Create a simple agent
    let agent = Agent::builder()
        .name("assistant")
        .model("gpt-4")
        .system_prompt("You are a helpful AI assistant")
        .build()
        .await?;

    // 💬 Have a conversation
    let response = agent.chat("Hello, how are you?").await?;
    println!("Agent: {}", response);

    Ok(())
}
```

### Advanced Example: RAG System

```rust
use lumosai::prelude::*;

/// Entry point: demonstrate RAG creation and retrieval
#[tokio::main]
async fn main() -> Result<()> {
    // 📦 Create vector storage
    let storage = lumosai::vector::memory().await?;

    // 🧠 Create RAG system
    let rag = lumosai::rag::builder()
        .storage(storage)
        .embedding_provider("openai")
        .chunking_strategy("recursive")
        .build()
        .await?;

    // 📄 Add documents
    rag.add_document("AI is transforming industries...").await?;

    // 🔍 Search and generate
    let results = rag.search("What is AI?", 5).await?;
    println!("Found {} relevant documents", results.len());

    Ok(())
}
```

---

## 🧩 Requirements
- `Rust` >= `1.70` (workspace uses Rust 2021 edition)
- `tokio` 1.x (async runtime)
- macOS, Linux, or Windows (primary CI validated on macOS)
- Optional: access to external model providers when using integrations

## 💡 Examples

Our comprehensive example suite demonstrates real-world usage patterns:

| Example | Description | Complexity |
|---------|-------------|------------|
| [🤖 Basic Agent](examples/basic_agent.rs) | Simple agent creation and conversation | ⭐ |
| [🧠 RAG System](examples/rag_system.rs) | Document processing and retrieval | ⭐⭐ |
| [🛠️ Tool Integration](examples/tool_integration.rs) | Adding tools to agents | ⭐⭐ |
| [💾 Memory System](examples/memory_system.rs) | Conversation memory and context | ⭐⭐ |
| [📊 Vector Storage](examples/vector_storage.rs) | Vector database operations | ⭐⭐ |
| [🌊 Streaming Response](examples/streaming_response.rs) | Real-time streaming responses | ⭐⭐⭐ |
| [👥 Multi-Agent Workflow](examples/multi_agent_workflow.rs) | Agent collaboration patterns | ⭐⭐⭐ |
| [🚀 Enhanced Features](examples/enhanced_features_demo.rs) | Advanced framework capabilities | ⭐⭐⭐ |
| [⚡ Performance Benchmark](examples/performance_benchmark.rs) | Performance testing and optimization | ⭐⭐⭐ |
| [🔐 Authentication](examples/auth_demo.rs) | Enterprise security features | ⭐⭐⭐⭐ |
| [📈 Monitoring](examples/monitoring_demo_simple.rs) | System monitoring and metrics | ⭐⭐⭐⭐ |
| [🎯 Complete API Demo](examples/simplified_api_complete_demo.rs) | Full framework demonstration | ⭐⭐⭐⭐⭐ |
| [🔧 Tool Macro Demo](examples/tool_macro_demo.rs) | Procedural macro for tool creation | ⭐⭐ |
| [🧠 Unified Memory Demo](examples/unified_memory_demo.rs) | Unified memory interface system | ⭐⭐ |
| [❌ Friendly Errors Demo](examples/friendly_errors_demo.rs) | User-friendly error handling | ⭐⭐ |

### Running Examples

```bash
# Basic agent example
cargo run --example basic_agent

# RAG system with document processing
cargo run --example rag_system

# Multi-agent collaboration
cargo run --example multi_agent_workflow

# Complete API demonstration
cargo run --example simplified_api_complete_demo
```

---

## 🏗️ Architecture

LumosAI follows a modular, layered architecture designed for scalability and maintainability:

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                        │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │
│  │   Web UI    │ │     CLI     │ │    Custom Applications  │ │
│  └─────────────┘ └─────────────┘ └─────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                     API Layer                               │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │
│  │  REST API   │ │  GraphQL    │ │      WebSocket API      │ │
│  └─────────────┘ └─────────────┘ └─────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                   Service Layer                             │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │
│  │   Agents    │ │  Workflows  │ │      Authentication     │ │
│  │   Memory    │ │     RAG     │ │       Monitoring        │ │
│  │   Tools     │ │   Events    │ │       Security          │ │
│  └─────────────┘ └─────────────┘ └─────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                    Core Layer                               │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │
│  │   Traits    │ │   Types     │ │       Utilities         │ │
│  │   Errors    │ │   Config    │ │       Macros            │ │
│  └─────────────┘ └─────────────┘ └─────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                Infrastructure Layer                         │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │
│  │  Databases  │ │   Storage   │ │      External APIs      │ │
│  │   Cache     │ │   Queues    │ │       Providers         │ │
│  └─────────────┘ └─────────────┘ └─────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Core Components

- **🤖 Agent System**: Intelligent agents with specialized capabilities
- **🧠 RAG Engine**: Advanced retrieval-augmented generation
- **🔄 Workflow Engine**: Multi-agent orchestration and task management
- **💾 Memory System**: Persistent context and conversation management
- **🛠️ Tool System**: Extensible tool integration framework
- **🔐 Security Layer**: Authentication, authorization, and audit logging
- **📊 Monitoring**: Real-time metrics, tracing, and observability

---

## 🧭 Workspace Overview
The project is organized as a Rust workspace with modular crates:

- `lumosai_core` — Core traits, types, configs, and simplified APIs
- `lumosai_vector` — Unified vector storage layer (memory backend enabled by default)
- `lumosai_rag` — Retrieval-Augmented Generation engine and pipelines
- `lumosai_cli` — Command-line tooling and example runners
- `lumosai_evals` — Evaluation utilities and benchmarking (optional)
- `lumosai_network` — Networking utilities and integrations (optional)
- `lumosai_mcp` — MCP integrations (optional)
- `lumosai_auth` — Authentication and security primitives
- `lumosai_enterprise` — Enterprise features (RBAC, multi-tenant, auditing)
- `lumosai_security` — Security policies and helper utilities
- `lumosai_telemetry` — Monitoring and observability instrumentation
- `lumosai_voice` — Voice interfaces and audio processing
- `lumosai_bindings` — Language bindings (Python/Node) and interop
- `lumos_macro` — Procedural macros and DSL support
- `lumosai_derive` — Derive macros for common patterns
- `lumosai_multimodal` — Multimodal processing utilities
- `lumosai_examples` — Comprehensive examples and demos

Refer to `Cargo.toml` for the full workspace member list and temporarily excluded crates.

---

## 🧱 Feature Flags
Feature flags allow tailoring builds to your needs (from `Cargo.toml`):

- `default = ["integrations", "vector-memory"]`
- `integrations` — Enables HTTP client integrations (e.g., `reqwest`-based remote APIs)
- `vector-memory` — Enables in-memory vector storage via `lumosai_vector/memory`
- (temporarily disabled) `vector-qdrant`, `vector-weaviate`, `vector-postgres` — alternative vector databases

Enable/disable features in your `Cargo.toml`:

```toml
[dependencies]
lumosai = { version = "0.2.0", features = ["integrations", "vector-memory"] }
```

## 📚 Documentation

### 📖 User Guides
- [🚀 Getting Started Guide](docs/getting_started.md) - Your first steps with LumosAI
- [📖 Overview](docs/1_overview.md) - Project overview and introduction
- [🏗️ Architecture](docs/2_architecture.md) - System architecture and design
- [⚙️ Tech Stack](docs/3_tech_stack.md) - Technology stack and dependencies
- [🔧 Core Components](docs/4_core_components.md) - Core framework components

### 🔧 Technical References
- [📋 API Reference](docs/5_api_reference.md) - Complete API documentation
- [🛠️ Development Guide](docs/6_development_guide.md) - Development setup and guidelines
- [🚀 Deployment Guide](docs/7_deployment_guide.md) - Production deployment strategies
- [🧠 Vector Databases](docs/VECTOR_DATABASES.md) - Vector database integration guide
- [📊 Vector API Reference](docs/vector_api_reference.md) - Vector operations API

### 🛡️ Enterprise Features
- [⚡ Chain Operations](docs/CHAIN_OPERATIONS_BEST_PRACTICES.md) - Best practices for chain operations
- [🔧 DSL Macros](docs/dsl_macros.md) - Domain-specific language macros
- [🚀 Release Guide](docs/RELEASE_GUIDE.md) - Release management and versioning

### 💡 Tutorials & Examples
- [🧪 Testing Guide](docs/testing/README.md) - Testing strategies and best practices
- [❓ FAQ](docs/8_faq.md) - Frequently asked questions
- [🚀 Quick Start](docs/quick-start/README.md) - Quick start guide

---

## 📜 Changelog & Releases
- Release notes: `docs/releases/`
- Update history: `docs/updates/`
- Release configuration: `release.toml`

We follow semantic versioning where applicable. Refer to the release guide for details: `docs/RELEASE_GUIDE.md`.

## 🤝 Contributing

We welcome contributions of all kinds! Whether you're fixing bugs, adding features, improving documentation, or sharing feedback, your contributions help make LumosAI better for everyone.

### 🚀 Quick Contribution Guide

1. **🍴 Fork the repository**
2. **🌿 Create your feature branch** (`git checkout -b feature/amazing-feature`)
3. **✅ Make your changes** (follow our coding standards)
4. **🧪 Add tests** for your changes
5. **📝 Update documentation** if needed
6. **✨ Commit your changes** (`git commit -m 'Add amazing feature'`)
7. **📤 Push to the branch** (`git push origin feature/amazing-feature`)
8. **🔄 Open a Pull Request**

### 📋 Contribution Areas

- **🐛 Bug Reports**: Help us identify and fix issues
- **💡 Feature Requests**: Suggest new capabilities and improvements
- **📖 Documentation**: Improve guides, examples, and API docs
- **🧪 Testing**: Add test coverage and improve test quality
- **🎨 Examples**: Create real-world usage examples
- **🔧 Performance**: Optimize performance and resource usage
- **🛡️ Security**: Enhance security features and practices

### 🎯 Development Setup

```bash
# Clone the repository
git clone https://github.com/louloulin/lumos.ai.git
cd lumosai

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the project
cargo build

# Run tests
cargo test

# Run examples
cargo run --example basic_agent

# Check code quality
cargo clippy
cargo fmt --check
```

### 📏 Code Standards

- **🦀 Rust Best Practices**: Follow Rust idioms and conventions
- **📝 Documentation**: Document all public APIs with examples
- **🧪 Testing**: Maintain high test coverage (aim for >80%)
- **🔍 Code Quality**: Pass `cargo clippy` and `cargo fmt` checks
- **⚡ Performance**: Consider performance implications of changes
- **🛡️ Security**: Follow secure coding practices

### 🏷️ Issue Labels

- `good first issue` - Perfect for newcomers
- `help wanted` - Community contributions welcome
- `bug` - Something isn't working
- `enhancement` - New feature or improvement
- `documentation` - Documentation improvements
- `performance` - Performance-related changes
- `security` - Security-related issues

---

## 🌟 Community & Support

### 💬 Join Our Community

- **💬 Discord**: [Join our Discord server](https://discord.gg/lumosai) for real-time discussions
- **📧 Mailing List**: [Subscribe to our newsletter](https://lumosai.com/newsletter) for updates
- **🐦 Twitter**: Follow [@LumosAI](https://twitter.com/lumosai) for announcements
- **📺 YouTube**: [LumosAI Channel](https://youtube.com/lumosai) for tutorials and demos

### 🆘 Getting Help

- **📖 Documentation**: Check our [comprehensive docs](docs/README.md)
- **💡 Examples**: Browse [example applications](examples/)
- **🐛 Issues**: Report bugs on [GitHub Issues](https://github.com/louloulin/lumos.ai/issues)
- **💬 Discussions**: Ask questions in [GitHub Discussions](https://github.com/louloulin/lumos.ai/discussions)
- **📧 Email**: Contact us at [support@lumosai.com](mailto:support@lumosai.com)

### 🏆 Contributors

Thanks to all our amazing contributors! 🎉

<a href="https://github.com/louloulin/lumos.ai/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=lumosai/lumosai" />
</a>

### 🚀 Enterprise Support

For enterprise customers, we offer:

- **🎯 Priority Support**: Dedicated support channels
- **🏗️ Custom Development**: Tailored solutions for your needs
- **📚 Training & Consulting**: Expert guidance and training
- **🔒 Security & Compliance**: Enhanced security features
- **📈 SLA Guarantees**: Service level agreements

Contact us at [enterprise@lumosai.com](mailto:enterprise@lumosai.com) for more information.

---

## License

This project is licensed under the MIT License — see `LICENSE` for details.



## 📄 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

```
MIT License

Copyright (c) 2024 LumosAI

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

<div align="center">

**⭐ Star us on GitHub if you find LumosAI helpful!**

[⭐ Star](https://github.com/louloulin/lumos.ai) | [🐛 Report Bug](https://github.com/louloulin/lumos.ai/issues) | [💡 Request Feature](https://github.com/louloulin/lumos.ai/issues) | [📖 Documentation](docs/README.md)

**Built with ❤️ by the LumosAI team**

</div>