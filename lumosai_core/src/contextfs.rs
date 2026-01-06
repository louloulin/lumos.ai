//! ContextFS - Context File System Abstraction
//!
//! 基于ContextFS论文的统一文件系统抽象，将所有AI Agent资源抽象为文件系统路径。
//! 实现"Everything is File, Everything is Context"的设计理念。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{Error, Result};

/// 上下文对象 - ContextFS的核心抽象
///
/// 表示任何可以被AI Agent理解和操作的资源，包括Agent、记忆、工作流、会话等。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    /// 上下文唯一标识
    pub id: ContextId,

    /// 上下文内容 - 支持多模态数据
    pub content: ContextContent,

    /// 元数据 - 描述性信息
    pub metadata: HashMap<String, Value>,

    /// 关系网络 - 与其他上下文的关联
    pub relations: Vec<ContextRelation>,

    /// 向量表示 - 用于语义搜索
    pub embeddings: Option<Vec<f32>>,

    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// 版本号 - 用于并发控制
    pub version: u64,
}

/// 上下文标识
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContextId {
    /// 上下文类型
    pub context_type: ContextType,

    /// 实体ID
    pub entity_id: String,

    /// 可选的子路径
    pub sub_path: Option<String>,
}

/// 上下文类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContextType {
    /// Agent上下文
    Agent,

    /// 记忆上下文
    Memory,

    /// 工作流上下文
    Workflow,

    /// 会话上下文
    Session,

    /// 系统上下文
    System,
}

/// 上下文内容 - 支持多模态数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextContent {
    /// 文本内容
    Text(String),

    /// 代码内容
    Code {
        language: String,
        content: String,
    },

    /// 结构化数据
    Structured(Value),

    /// 二进制数据（如图像、音频）
    Binary(Vec<u8>),

    /// 多模态内容
    MultiModal(Vec<ContextContent>),
}

/// 上下文关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRelation {
    /// 关系类型
    pub relation_type: String,

    /// 目标上下文ID
    pub target_id: ContextId,

    /// 关系强度/权重
    pub weight: f32,

    /// 关系属性
    pub properties: HashMap<String, Value>,
}

/// ContextFS文件系统trait
#[async_trait]
pub trait ContextFileSystem: Send + Sync {
    /// 读取上下文
    async fn read_context(&self, path: &str) -> Result<Context>;

    /// 写入上下文
    async fn write_context(&self, path: &str, context: Context) -> Result<()>;

    /// 删除上下文
    async fn delete_context(&self, path: &str) -> Result<()>;

    /// 列出上下文
    async fn list_context(&self, path: &str) -> Result<Vec<ContextId>>;

    /// 语义搜索上下文
    async fn search_context(&self, query: &SemanticQuery) -> Result<Vec<Context>>;

    /// 检查上下文是否存在
    async fn exists_context(&self, path: &str) -> Result<bool>;

    /// 创建目录（如果需要）
    async fn create_directory(&self, path: &str) -> Result<()> {
        // 默认实现：目录自动创建
        Ok(())
    }
}

/// 语义查询
#[derive(Debug, Clone)]
pub struct SemanticQuery {
    /// 文本查询
    pub query: String,

    /// 查询向量嵌入（可选，用于语义搜索）
    pub query_embedding: Option<Vec<f32>>,

    /// 上下文类型过滤
    pub context_types: Option<Vec<ContextType>>,

    /// 实体ID过滤
    pub entity_ids: Option<Vec<String>>,

    /// 元数据过滤
    pub metadata_filters: Option<HashMap<String, Value>>,

    /// 相似度阈值
    pub threshold: Option<f32>,

    /// 返回数量限制
    pub limit: Option<usize>,

    /// 排序方式
    pub sort_by: Option<SortBy>,
}

/// 排序方式
#[derive(Debug, Clone)]
pub enum SortBy {
    /// 按相似度降序
    Relevance,

    /// 按创建时间降序
    CreatedAt,

    /// 按更新时间降序
    UpdatedAt,

    /// 按版本降序
    Version,
}

/// ContextFS路径解析器
pub struct ContextPathResolver;

impl ContextPathResolver {
    /// 解析路径为ContextId
    pub fn parse_path(path: &str) -> Result<ContextId> {
        let path = path.trim_start_matches('/');

        if path.starts_with("agent/") {
            Self::parse_entity_path(path, ContextType::Agent)
        } else if path.starts_with("memory/") {
            Self::parse_entity_path(path, ContextType::Memory)
        } else if path.starts_with("workflow/") {
            Self::parse_entity_path(path, ContextType::Workflow)
        } else if path.starts_with("session/") {
            Self::parse_entity_path(path, ContextType::Session)
        } else if path.starts_with("sys/agentmem/") {
            Self::parse_system_path(path)
        } else {
            Err(Error::UnsupportedOperation(format!("Invalid ContextFS path: {}", path)))
        }
    }

    /// 生成路径字符串
    pub fn to_path(id: &ContextId) -> String {
        let base = match id.context_type {
            ContextType::Agent => "agent",
            ContextType::Memory => "memory",
            ContextType::Workflow => "workflow",
            ContextType::Session => "session",
            ContextType::System => "sys/agentmem",
        };

        if let Some(sub_path) = &id.sub_path {
            format!("/{}/{}/{}", base, id.entity_id, sub_path)
        } else {
            format!("/{}/{}", base, id.entity_id)
        }
    }

    fn parse_entity_path(path: &str, context_type: ContextType) -> Result<ContextId> {
        let parts: Vec<&str> = path.split('/').collect();

        if parts.len() < 2 {
            return Err(Error::UnsupportedOperation(format!("Invalid entity path: {}", path)));
        }

        let entity_id = parts[1].to_string();
        let sub_path = if parts.len() > 2 {
            Some(parts[2..].join("/"))
        } else {
            None
        };

        Ok(ContextId {
            context_type,
            entity_id,
            sub_path,
        })
    }

    fn parse_system_path(path: &str) -> Result<ContextId> {
        let parts: Vec<&str> = path.split('/').collect();

        if parts.len() < 3 {
            return Err(Error::UnsupportedOperation(format!("Invalid system path: {}", path)));
        }

        let entity_id = parts[2].to_string();
        let sub_path = if parts.len() > 3 {
            Some(parts[3..].join("/"))
        } else {
            None
        };

        Ok(ContextId {
            context_type: ContextType::System,
            entity_id,
            sub_path,
        })
    }
}

/// 内存实现的基础ContextFS
pub struct InMemoryContextFS {
    contexts: Arc<RwLock<HashMap<String, Context>>>,
}

impl InMemoryContextFS {
    pub fn new() -> Self {
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ContextFileSystem for InMemoryContextFS {
    async fn read_context(&self, path: &str) -> Result<Context> {
        let contexts = self.contexts.read().await;
        contexts.get(path).cloned().ok_or_else(|| {
            Error::NotFound(format!("Context not found: {}", path))
        })
    }

    async fn write_context(&self, path: &str, context: Context) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        contexts.insert(path.to_string(), context);
        Ok(())
    }

    async fn delete_context(&self, path: &str) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        if contexts.remove(path).is_none() {
            return Err(Error::NotFound(format!("Context not found: {}", path)));
        }
        Ok(())
    }

    async fn list_context(&self, path: &str) -> Result<Vec<ContextId>> {
        let contexts = self.contexts.read().await;
        let matching_ids: Vec<ContextId> = contexts
            .keys()
            .filter(|key| key.starts_with(path))
            .filter_map(|key| ContextPathResolver::parse_path(key).ok())
            .collect();

        Ok(matching_ids)
    }

    async fn search_context(&self, query: &SemanticQuery) -> Result<Vec<Context>> {
        let contexts = self.contexts.read().await;

        // ✅ 实现真正的向量相似度搜索
        let query_embedding = &query.query_embedding;

        let mut scored_contexts: Vec<(f64, Context)> = contexts
            .values()
            .filter(|ctx| {
                // 类型过滤
                if let Some(types) = &query.context_types {
                    if !types.contains(&ctx.id.context_type) {
                        return false;
                    }
                }

                // 实体ID过滤
                if let Some(ids) = &query.entity_ids {
                    if !ids.contains(&ctx.id.entity_id) {
                        return false;
                    }
                }

                // 必须有向量嵌入才能进行语义搜索
                ctx.embeddings.is_some()
            })
            .filter_map(|ctx| {
                // 计算余弦相似度
                if let (Some(query_emb), Some(ctx_emb)) = (query_embedding, &ctx.embeddings) {
                    if query_emb.len() != ctx_emb.len() {
                        return None;  // 维度不匹配，跳过
                    }

                    let similarity = Self::cosine_similarity(query_emb, ctx_emb);
                    Some((similarity, ctx.clone()))
                } else if query_embedding.is_none() {
                    // 回退到文本匹配
                    if Self::matches_query(&ctx.content, &query.query) {
                        Some((0.5, ctx.clone()))  // 默认中等相似度
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        // 按相似度排序（降序）
        scored_contexts.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // 取前 N 个结果
        let limit = query.limit.unwrap_or(10);
        let results: Vec<Context> = scored_contexts
            .into_iter()
            .take(limit)
            .map(|(_, ctx)| ctx)
            .collect();

        Ok(results)
    }

    async fn exists_context(&self, path: &str) -> Result<bool> {
        let contexts = self.contexts.read().await;
        Ok(contexts.contains_key(path))
    }
}

impl InMemoryContextFS {
    /// 计算两个向量的余弦相似度
    ///
    /// 返回值范围: [-1, 1]
    /// - 1.0: 完全相同方向
    /// - 0.0: 正交（无关）
    /// - -1.0: 完全相反方向
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
        if a.len() != b.len() {
            return 0.0;  // 维度不匹配
        }

        let dot_product: f64 = a.iter()
            .zip(b.iter())
            .map(|(x, y)| (*x as f64) * (*y as f64))
            .sum();

        let norm_a: f64 = a.iter()
            .map(|x| (*x as f64) * (*x as f64))
            .sum::<f64>()
            .sqrt();

        let norm_b: f64 = b.iter()
            .map(|x| (*x as f64) * (*x as f64))
            .sum::<f64>()
            .sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }

    fn matches_query(content: &ContextContent, query: &str) -> bool {
        match content {
            ContextContent::Text(text) => text.to_lowercase().contains(&query.to_lowercase()),
            ContextContent::Code { content, .. } => content.to_lowercase().contains(&query.to_lowercase()),
            ContextContent::Structured(value) => {
                if let Some(text) = value.as_str() {
                    text.to_lowercase().contains(&query.to_lowercase())
                } else {
                    false
                }
            },
            ContextContent::Binary(_) => false, // 二进制内容不支持文本搜索
            ContextContent::MultiModal(contents) => {
                contents.iter().any(|c| Self::matches_query(c, query))
            }
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            id: ContextId {
                context_type: ContextType::Agent,
                entity_id: "default".to_string(),
                sub_path: None,
            },
            content: ContextContent::Text("".to_string()),
            metadata: HashMap::new(),
            relations: Vec::new(),
            embeddings: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            version: 1,
        }
    }
}

impl Context {
    /// 创建新的上下文
    pub fn new(id: ContextId, content: ContextContent) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            content,
            metadata: HashMap::new(),
            relations: Vec::new(),
            embeddings: None,
            created_at: now,
            updated_at: now,
            version: 1,
        }
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// 添加关系
    pub fn with_relation(mut self, relation: ContextRelation) -> Self {
        self.relations.push(relation);
        self
    }

    /// 设置向量表示
    pub fn with_embeddings(mut self, embeddings: Vec<f32>) -> Self {
        self.embeddings = Some(embeddings);
        self
    }

    /// 更新内容
    pub fn update_content(&mut self, content: ContextContent) {
        self.content = content;
        self.updated_at = chrono::Utc::now();
        self.version += 1;
    }

    /// 添加元数据（可变版本）
    pub fn add_metadata(&mut self, key: impl Into<String>, value: Value) {
        self.metadata.insert(key.into(), value);
        self.updated_at = chrono::Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_creation() {
        let id = ContextId {
            context_type: ContextType::Agent,
            entity_id: "test-agent".to_string(),
            sub_path: None,
        };

        let content = ContextContent::Text("Hello, World!".to_string());
        let context = Context::new(id.clone(), content);

        assert_eq!(context.id, id);
        assert_eq!(context.version, 1);
        assert!(context.metadata.is_empty());
        assert!(context.relations.is_empty());
        assert!(context.embeddings.is_none());
    }

    #[tokio::test]
    async fn test_context_path_resolver() {
        // 测试Agent路径
        let agent_path = "/agent/test-agent";
        let agent_id = ContextPathResolver::parse_path(agent_path).unwrap();
        assert_eq!(agent_id.context_type, ContextType::Agent);
        assert_eq!(agent_id.entity_id, "test-agent");
        assert_eq!(agent_id.sub_path, None);

        let reconstructed_path = ContextPathResolver::to_path(&agent_id);
        assert_eq!(reconstructed_path, agent_path);

        // 测试Memory路径
        let memory_path = "/memory/test-memory/config";
        let memory_id = ContextPathResolver::parse_path(memory_path).unwrap();
        assert_eq!(memory_id.context_type, ContextType::Memory);
        assert_eq!(memory_id.entity_id, "test-memory");
        assert_eq!(memory_id.sub_path, Some("config".to_string()));

        // 测试系统路径
        let sys_path = "/sys/agentmem/config/database_url";
        let sys_id = ContextPathResolver::parse_path(sys_path).unwrap();
        assert_eq!(sys_id.context_type, ContextType::System);
        assert_eq!(sys_id.entity_id, "config");
        assert_eq!(sys_id.sub_path, Some("database_url".to_string()));
    }

    #[tokio::test]
    async fn test_in_memory_context_fs() {
        let fs = InMemoryContextFS::new();

        // 创建Agent上下文
        let agent_id = ContextId {
            context_type: ContextType::Agent,
            entity_id: "test-agent".to_string(),
            sub_path: Some("config".to_string()),
        };

        let agent_context = Context::new(
            agent_id.clone(),
            ContextContent::Structured(serde_json::json!({
                "name": "Test Agent",
                "instructions": "You are a helpful assistant."
            }))
        );

        let path = "/agent/test-agent/config";

        // 写入上下文
        fs.write_context(path, agent_context.clone()).await.unwrap();

        // 读取上下文
        let read_context = fs.read_context(path).await.unwrap();
        assert_eq!(read_context.id, agent_id);

        // 检查是否存在
        assert!(fs.exists_context(path).await.unwrap());

        // 列出上下文
        let contexts = fs.list_context("/agent/").await.unwrap();
        assert_eq!(contexts.len(), 1);
        assert_eq!(contexts[0], agent_id);

        // 搜索上下文
        let query = SemanticQuery {
            query: "helpful".to_string(),
            context_types: Some(vec![ContextType::Agent]),
            entity_ids: None,
            metadata_filters: None,
            threshold: None,
            limit: Some(5),
            sort_by: None,
        };

        let results = fs.search_context(&query).await.unwrap();
        assert_eq!(results.len(), 1);

        // 删除上下文
        fs.delete_context(path).await.unwrap();
        assert!(!fs.exists_context(path).await.unwrap());
    }

    #[test]
    fn test_context_content_matching() {
        // 测试文本内容匹配
        let text_content = ContextContent::Text("Hello world, this is a test.".to_string());
        assert!(InMemoryContextFS::matches_query(&text_content, "world"));
        assert!(InMemoryContextFS::matches_query(&text_content, "WORLD")); // 大小写不敏感
        assert!(!InMemoryContextFS::matches_query(&text_content, "nonexistent"));

        // 测试代码内容匹配
        let code_content = ContextContent::Code {
            language: "rust".to_string(),
            content: "fn main() { println!(\"Hello\"); }".to_string(),
        };
        assert!(InMemoryContextFS::matches_query(&code_content, "println"));
        assert!(!InMemoryContextFS::matches_query(&code_content, "python"));

        // 测试结构化内容匹配
        let structured_content = ContextContent::Structured(serde_json::json!("Hello world"));
        assert!(InMemoryContextFS::matches_query(&structured_content, "world"));
        assert!(!InMemoryContextFS::matches_query(&structured_content, "nonexistent"));
    }

    #[tokio::test]
    async fn test_contextfs_integration() {
        // 集成测试：验证ContextFS的完整功能
        let fs = InMemoryContextFS::new();

        // 创建并存储Agent上下文
        let agent_id = ContextId {
            context_type: ContextType::Agent,
            entity_id: "integration-test-agent".to_string(),
            sub_path: Some("profile".to_string()),
        };

        let agent_context = Context::new(
            agent_id.clone(),
            ContextContent::Structured(serde_json::json!({
                "name": "Integration Test Agent",
                "purpose": "Testing ContextFS functionality",
                "capabilities": ["search", "reasoning", "execution"]
            }))
        );

        let path = "/agent/integration-test-agent/profile";

        // 测试完整CRUD操作
        fs.write_context(path, agent_context).await.unwrap();
        assert!(fs.exists_context(path).await.unwrap());

        let retrieved = fs.read_context(path).await.unwrap();
        assert_eq!(retrieved.id, agent_id);

        // 测试语义搜索
        let query = SemanticQuery {
            query: "testing".to_string(),
            context_types: Some(vec![ContextType::Agent]),
            limit: Some(5),
            ..Default::default()
        };

        let results = fs.search_context(&query).await.unwrap();
        assert!(!results.is_empty());

        // 测试路径解析
        let parsed_id = ContextPathResolver::parse_path(path).unwrap();
        assert_eq!(parsed_id, agent_id);

        let reconstructed_path = ContextPathResolver::to_path(&parsed_id);
        assert_eq!(reconstructed_path, path);

        // 清理
        fs.delete_context(path).await.unwrap();
        assert!(!fs.exists_context(path).await.unwrap());
    }
}
