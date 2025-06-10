# 🤝 Contributing to LumosAI

Thank you for your interest in contributing to LumosAI! We welcome contributions of all kinds and appreciate your help in making this project better.

## 🚀 Quick Start for Contributors

### 1. 🍴 Fork and Clone

```bash
# Fork the repository on GitHub, then clone your fork
git clone https://github.com/YOUR_USERNAME/lumosai.git
cd lumosai

# Add the original repository as upstream
git remote add upstream https://github.com/lumosai/lumosai.git
```

### 2. 🛠️ Development Setup

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install required tools
rustup component add clippy rustfmt

# Build the project
cargo build

# Run tests
cargo test

# Check code quality
cargo clippy
cargo fmt --check
```

### 3. 🌿 Create a Branch

```bash
# Create and switch to a new branch
git checkout -b feature/your-feature-name

# Or for bug fixes
git checkout -b fix/issue-description
```

## 📋 Types of Contributions

### 🐛 Bug Reports
- Use the [bug report template](.github/ISSUE_TEMPLATE/bug_report.md)
- Include clear reproduction steps
- Provide system information and error messages
- Add relevant logs or screenshots

### 💡 Feature Requests
- Use the [feature request template](.github/ISSUE_TEMPLATE/feature_request.md)
- Describe the problem you're trying to solve
- Explain your proposed solution
- Consider alternative approaches

### 🔧 Code Contributions
- **Bug Fixes**: Fix existing issues
- **New Features**: Add new capabilities
- **Performance**: Optimize existing code
- **Documentation**: Improve code documentation
- **Tests**: Add or improve test coverage

### 📖 Documentation
- **User Guides**: Help users understand features
- **API Documentation**: Document public APIs
- **Examples**: Create practical examples
- **Tutorials**: Step-by-step learning materials

## 🎯 Development Guidelines

### 📏 Code Standards

#### Rust Code Style
```rust
// ✅ Good: Clear, documented function
/// Calculates the similarity between two vectors using cosine similarity.
/// 
/// # Arguments
/// * `a` - First vector
/// * `b` - Second vector
/// 
/// # Returns
/// Similarity score between 0.0 and 1.0
/// 
/// # Example
/// ```
/// let similarity = cosine_similarity(&vec1, &vec2)?;
/// assert!(similarity >= 0.0 && similarity <= 1.0);
/// ```
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> Result<f32> {
    if a.len() != b.len() {
        return Err(Error::DimensionMismatch);
    }
    
    // Implementation...
    Ok(0.95)
}
```

#### Error Handling
```rust
// ✅ Good: Proper error handling
pub async fn load_document(path: &Path) -> Result<Document> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| Error::FileRead {
            path: path.to_path_buf(),
            source: e,
        })?;
    
    Document::parse(&content)
}

// ❌ Bad: Unwrapping without context
pub async fn load_document(path: &Path) -> Document {
    let content = tokio::fs::read_to_string(path).await.unwrap();
    Document::parse(&content).unwrap()
}
```

### 🧪 Testing Requirements

#### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_agent_creation() {
        let agent = Agent::builder()
            .name("test-agent")
            .model("gpt-4")
            .build()
            .await
            .expect("Failed to create agent");
        
        assert_eq!(agent.name(), "test-agent");
        assert_eq!(agent.model(), "gpt-4");
    }
    
    #[tokio::test]
    async fn test_agent_generation() {
        let agent = create_test_agent().await;
        let response = agent.generate("Hello").await;
        
        assert!(response.is_ok());
        assert!(!response.unwrap().is_empty());
    }
}
```

#### Integration Tests
```rust
// tests/integration_test.rs
use lumosai::prelude::*;

#[tokio::test]
async fn test_full_rag_pipeline() {
    let storage = VectorStorage::memory().await.unwrap();
    let rag = RagSystem::new(storage).await.unwrap();
    
    // Add documents
    rag.add_document("Test content").await.unwrap();
    
    // Search
    let results = rag.search("test", 5).await.unwrap();
    assert!(!results.is_empty());
}
```

### 📊 Performance Considerations

- **Async/Await**: Use async for I/O operations
- **Memory Management**: Avoid unnecessary allocations
- **Caching**: Implement caching where appropriate
- **Benchmarking**: Add benchmarks for performance-critical code

```rust
// Example benchmark
#[cfg(test)]
mod benches {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn benchmark_vector_similarity(c: &mut Criterion) {
        let vec1 = vec![1.0; 1000];
        let vec2 = vec![0.5; 1000];
        
        c.bench_function("cosine_similarity", |b| {
            b.iter(|| cosine_similarity(black_box(&vec1), black_box(&vec2)))
        });
    }
    
    criterion_group!(benches, benchmark_vector_similarity);
    criterion_main!(benches);
}
```

## 🔄 Pull Request Process

### 1. 📝 Before Submitting

- [ ] Code follows style guidelines
- [ ] Tests pass locally (`cargo test`)
- [ ] Code is properly formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation is updated
- [ ] Examples work correctly

### 2. 📤 Submitting

1. **Push your changes**
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create Pull Request**
   - Use the [PR template](.github/PULL_REQUEST_TEMPLATE.md)
   - Link related issues
   - Describe changes clearly
   - Add screenshots if applicable

3. **Review Process**
   - Automated checks must pass
   - At least one maintainer review required
   - Address feedback promptly
   - Keep PR updated with main branch

### 3. ✅ After Approval

- PR will be merged by maintainers
- Your branch can be deleted
- Thank you for your contribution! 🎉

## 🏷️ Issue Labels

Understanding our label system:

- `good first issue` - Perfect for newcomers
- `help wanted` - Community contributions welcome
- `bug` - Something isn't working
- `enhancement` - New feature or improvement
- `documentation` - Documentation improvements
- `performance` - Performance-related changes
- `security` - Security-related issues
- `breaking-change` - Changes that break backward compatibility

## 🎯 Areas for Contribution

### High Priority
- 🧪 **Test Coverage**: Increase test coverage
- 📖 **Documentation**: Improve user guides
- 🐛 **Bug Fixes**: Fix reported issues
- ⚡ **Performance**: Optimize critical paths

### Medium Priority
- 🔌 **Integrations**: Add new LLM providers
- 🛠️ **Tools**: Create useful agent tools
- 📊 **Examples**: Real-world use cases
- 🔍 **Monitoring**: Enhance observability

### Future Opportunities
- 🌐 **Web Interface**: Browser-based management
- 📱 **Mobile Support**: Mobile-friendly APIs
- 🔄 **Streaming**: Enhanced streaming capabilities
- 🤖 **AI Features**: Advanced AI capabilities

## 💬 Communication

- **💬 Discord**: [Join our Discord](https://discord.gg/lumosai) for real-time chat
- **📧 Email**: [contributors@lumosai.com](mailto:contributors@lumosai.com)
- **🐛 Issues**: Use GitHub Issues for bug reports and feature requests
- **💡 Discussions**: Use GitHub Discussions for questions and ideas

## 📜 Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## 🙏 Recognition

Contributors are recognized in:
- 📋 **README**: Listed in contributors section
- 📝 **Changelog**: Mentioned in release notes
- 🏆 **Hall of Fame**: Special recognition for significant contributions
- 🎁 **Swag**: Exclusive contributor merchandise

---

**Thank you for contributing to LumosAI! Together, we're building the future of AI development.** 🚀
