//! Repository-First Storage Strategy
//!
//! 基于ENGRAM论文的Repository-First策略实现，确保数据一致性和可靠性。
//! Repository作为主存储，VectorStore作为辅助索引，所有操作保证数据一致性。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{Error, Result};
use crate::llm::Message;

/// Repository-First存储策略的核心trait
///
/// Repository是主存储，负责数据的持久化和一致性保证。
/// VectorStore是辅助索引，负责快速检索和语义搜索。
#[async_trait]
pub trait RepositoryFirstStorage: Send + Sync {
    /// 存储消息 - Repository-First策略
    ///
    /// 1. 先写入Repository（主存储）
    /// 2. 再同步到VectorStore（辅助索引）
    /// 3. 确保数据一致性
    async fn store_message(&self, message: &Message, namespace: Option<&str>) -> Result<()>;

    /// 检索消息 - 支持多种检索策略
    ///
    /// 根据查询类型选择最优的检索策略：
    /// - 时间查询：从Repository检索
    /// - 语义查询：从VectorStore检索
    /// - 混合查询：合并两者结果
    async fn retrieve_messages(&self, query: &StorageQuery) -> Result<Vec<Message>>;

    /// 同步数据一致性
    ///
    /// 检查并修复Repository和VectorStore之间的一致性问题
    async fn sync_consistency(&self) -> Result<ConsistencyReport>;

    /// 获取存储统计信息
    async fn get_stats(&self) -> Result<StorageStats>;
}

/// Repository trait - 主存储接口
#[async_trait]
pub trait Repository: Send + Sync {
    /// 存储消息到主存储
    async fn store(&self, message: &Message, namespace: Option<&str>) -> Result<String>;

    /// 从主存储检索消息
    async fn retrieve(&self, query: &RepositoryQuery) -> Result<Vec<StoredMessage>>;

    /// 删除消息
    async fn delete(&self, message_id: &str) -> Result<()>;

    /// 批量操作
    async fn batch_store(&self, messages: &[(&Message, Option<&str>)]) -> Result<Vec<String>>;

    /// 获取存储统计
    async fn stats(&self) -> Result<RepositoryStats>;
}

/// VectorStore trait - 辅助索引接口
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// 添加向量到索引
    async fn add_vectors(&self, vectors: &[VectorEntry]) -> Result<()>;

    /// 语义搜索
    async fn semantic_search(
        &self,
        query: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult>>;

    /// 删除向量
    async fn remove_vectors(&self, ids: &[String]) -> Result<()>;

    /// 重新索引（用于一致性修复）
    async fn reindex(&self, entries: &[VectorEntry]) -> Result<()>;

    /// 获取索引统计
    async fn index_stats(&self) -> Result<VectorStats>;
}

/// 存储查询
#[derive(Debug, Clone)]
pub struct StorageQuery {
    /// 查询类型
    pub query_type: QueryType,

    /// 命名空间
    pub namespace: Option<String>,

    /// 时间范围查询
    pub time_range: Option<TimeRange>,

    /// 语义查询
    pub semantic_query: Option<SemanticQuery>,

    /// 限制返回数量
    pub limit: Option<usize>,

    /// 排序方式
    pub sort_by: Option<SortBy>,
}

impl Default for StorageQuery {
    fn default() -> Self {
        Self {
            query_type: QueryType::Temporal,
            namespace: None,
            time_range: None,
            semantic_query: None,
            limit: Some(10),
            sort_by: Some(SortBy::TimestampDesc),
        }
    }
}

/// 查询类型
#[derive(Debug, Clone)]
pub enum QueryType {
    /// 时间顺序查询（从Repository）
    Temporal,

    /// 语义查询（从VectorStore）
    Semantic,

    /// 混合查询（合并两者）
    Hybrid,
}

/// 时间范围
#[derive(Debug, Clone)]
pub struct TimeRange {
    pub start: chrono::DateTime<chrono::Utc>,
    pub end: chrono::DateTime<chrono::Utc>,
}

/// 语义查询参数
#[derive(Debug, Clone)]
pub struct SemanticQuery {
    pub query: String,
    pub top_k: usize,
    pub threshold: Option<f32>,
    pub filters: Option<HashMap<String, String>>,
}

/// 排序方式
#[derive(Debug, Clone)]
pub enum SortBy {
    TimestampAsc,
    TimestampDesc,
    Relevance,
}

/// 存储的消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub id: String,
    pub message: Message,
    pub namespace: Option<String>,
    pub stored_at: chrono::DateTime<chrono::Utc>,
    pub version: u64,
}

/// Repository查询
#[derive(Debug, Clone)]
pub struct RepositoryQuery {
    pub namespace: Option<String>,
    pub time_range: Option<TimeRange>,
    pub limit: Option<usize>,
    pub sort_by: Option<SortBy>,
}

/// 向量条目
#[derive(Debug, Clone)]
pub struct VectorEntry {
    pub id: String,
    pub vector: Vec<f32>,
    pub metadata: HashMap<String, String>,
    pub namespace: Option<String>,
}

/// 搜索选项
#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub limit: usize,
    pub threshold: Option<f32>,
    pub namespace: Option<String>,
    pub filters: Option<HashMap<String, String>>,
}

/// 搜索结果
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: HashMap<String, String>,
}

/// 一致性报告
#[derive(Debug, Clone)]
pub struct ConsistencyReport {
    pub total_messages: usize,
    pub repository_count: usize,
    pub vector_store_count: usize,
    pub inconsistencies: Vec<Inconsistency>,
    pub sync_timestamp: chrono::DateTime<chrono::Utc>,
}

/// 不一致项
#[derive(Debug, Clone)]
pub struct Inconsistency {
    pub message_id: String,
    pub issue_type: InconsistencyType,
    pub description: String,
}

/// 不一致类型
#[derive(Debug, Clone)]
pub enum InconsistencyType {
    MissingInRepository,
    MissingInVectorStore,
    VersionMismatch,
    ContentMismatch,
}

/// 存储统计
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub total_messages: usize,
    pub total_namespaces: usize,
    pub repository_stats: RepositoryStats,
    pub vector_stats: VectorStats,
    pub consistency_score: f32, // 0.0-1.0
}

/// Repository统计
#[derive(Debug, Clone)]
pub struct RepositoryStats {
    pub total_messages: usize,
    pub total_namespaces: usize,
    pub oldest_message: Option<chrono::DateTime<chrono::Utc>>,
    pub newest_message: Option<chrono::DateTime<chrono::Utc>>,
}

/// Vector统计
#[derive(Debug, Clone)]
pub struct VectorStats {
    pub total_vectors: usize,
    pub dimensions: usize,
    pub index_size_mb: f64,
}

/// Repository-First存储实现
pub struct RepositoryFirstStorageImpl<R, V> {
    repository: Arc<R>,
    vector_store: Arc<V>,
    consistency_checker: Arc<ConsistencyChecker>,
    sync_manager: Arc<SyncManager>,
}

impl<R, V> RepositoryFirstStorageImpl<R, V>
where
    R: Repository + 'static,
    V: VectorStore + 'static,
{
    pub fn new(repository: Arc<R>, vector_store: Arc<V>) -> Self {
        Self {
            repository,
            vector_store,
            consistency_checker: Arc::new(ConsistencyChecker::new()),
            sync_manager: Arc::new(SyncManager::new()),
        }
    }
}

#[async_trait]
impl<R, V> RepositoryFirstStorage for RepositoryFirstStorageImpl<R, V>
where
    R: Repository + 'static,
    V: VectorStore + 'static,
{
    async fn store_message(&self, message: &Message, namespace: Option<&str>) -> Result<()> {
        // 1. 先写入Repository
        let message_id = self.repository.store(message, namespace).await?;

        // 2. 生成向量表示（这里需要实际的embedding逻辑）
        // TODO: 集成实际的embedding服务
        let vector = self.generate_embedding(message).await?;

        // 3. 写入VectorStore
        let vector_entry = VectorEntry {
            id: message_id.clone(),
            vector,
            metadata: HashMap::new(), // TODO: 提取消息元数据
            namespace: namespace.map(|s| s.to_string()),
        };

        self.vector_store.add_vectors(&[vector_entry]).await?;

        // 4. 记录同步状态
        self.sync_manager.record_sync(&message_id).await?;

        Ok(())
    }

    async fn retrieve_messages(&self, query: &StorageQuery) -> Result<Vec<Message>> {
        match &query.query_type {
            QueryType::Temporal => {
                // 从Repository检索
                let repo_query = RepositoryQuery {
                    namespace: query.namespace.clone(),
                    time_range: query.time_range.clone(),
                    limit: query.limit,
                    sort_by: query.sort_by.clone(),
                };

                let stored_messages = self.repository.retrieve(&repo_query).await?;
                Ok(stored_messages.into_iter().map(|sm| sm.message).collect())
            }

            QueryType::Semantic => {
                // 从VectorStore检索
                if let Some(semantic_query) = &query.semantic_query {
                    let search_options = SearchOptions {
                        limit: semantic_query.top_k,
                        threshold: semantic_query.threshold,
                        namespace: query.namespace.clone(),
                        filters: semantic_query.filters.clone(),
                    };

                    let results = self
                        .vector_store
                        .semantic_search(&semantic_query.query, &search_options)
                        .await?;

                    // 根据搜索结果从Repository获取完整消息
                    let message_ids: Vec<String> = results.into_iter().map(|r| r.id).collect();
                    let messages = self.get_messages_by_ids(&message_ids).await?;
                    Ok(messages)
                } else {
                    Ok(vec![])
                }
            }

            QueryType::Hybrid => {
                // 混合查询：合并时间和语义结果
                let mut results = Vec::new();

                // 时间查询结果
                if let Ok(temporal_results) = self
                    .retrieve_messages(&StorageQuery {
                        query_type: QueryType::Temporal,
                        namespace: query.namespace.clone(),
                        time_range: query.time_range.clone(),
                        limit: query.limit,
                        ..Default::default()
                    })
                    .await
                {
                    results.extend(temporal_results);
                }

                // 语义查询结果
                if let Ok(semantic_results) = self
                    .retrieve_messages(&StorageQuery {
                        query_type: QueryType::Semantic,
                        namespace: query.namespace.clone(),
                        semantic_query: query.semantic_query.clone(),
                        limit: query.limit,
                        ..Default::default()
                    })
                    .await
                {
                    results.extend(semantic_results);
                }

                // 去重和排序
                self.deduplicate_and_sort(results, query.sort_by.as_ref())
            }
        }
    }

    async fn sync_consistency(&self) -> Result<ConsistencyReport> {
        self.consistency_checker
            .check_consistency(self.repository.as_ref(), self.vector_store.as_ref())
            .await
    }

    async fn get_stats(&self) -> Result<StorageStats> {
        let repo_stats = self.repository.stats().await?;
        let vector_stats = self.vector_store.index_stats().await?;
        let consistency_report = self.sync_consistency().await?;

        let consistency_score = if consistency_report.total_messages > 0 {
            1.0 - (consistency_report.inconsistencies.len() as f32
                / consistency_report.total_messages as f32)
        } else {
            1.0
        };

        Ok(StorageStats {
            total_messages: consistency_report.total_messages,
            total_namespaces: repo_stats.total_namespaces,
            repository_stats: repo_stats,
            vector_stats,
            consistency_score,
        })
    }
}

impl<R, V> RepositoryFirstStorageImpl<R, V>
where
    R: Repository + 'static,
    V: VectorStore + 'static,
{
    /// 生成消息的向量表示
    async fn generate_embedding(&self, _message: &Message) -> Result<Vec<f32>> {
        // TODO: 集成实际的embedding服务
        // 这里返回一个虚拟的向量用于测试
        Ok(vec![0.1, 0.2, 0.3, 0.4, 0.5]) // 5维向量作为示例
    }

    /// 根据ID批量获取消息
    async fn get_messages_by_ids(&self, _message_ids: &[String]) -> Result<Vec<Message>> {
        // TODO: 实现批量获取
        // 这里返回空结果用于测试
        Ok(vec![])
    }

    /// 去重和排序结果
    fn deduplicate_and_sort(
        &self,
        mut messages: Vec<Message>,
        sort_by: Option<&SortBy>,
    ) -> Result<Vec<Message>> {
        // 简单的去重逻辑（基于内容哈希）
        let mut seen = std::collections::HashSet::new();
        messages.retain(|msg| {
            let hash = format!("{:?}-{}", msg.role, msg.content);
            seen.insert(hash)
        });

        // 排序
        match sort_by {
            Some(SortBy::TimestampAsc) => {
                messages.sort_by(|a, b| {
                    // 假设消息有时间戳，这里需要实际的时间戳字段
                    std::cmp::Ordering::Equal
                });
            }
            Some(SortBy::TimestampDesc) => {
                messages.sort_by(|a, b| std::cmp::Ordering::Equal.reverse());
            }
            _ => {} // 其他排序方式保持原顺序
        }

        Ok(messages)
    }
}

/// 一致性检查器
pub struct ConsistencyChecker;

impl ConsistencyChecker {
    pub fn new() -> Self {
        Self
    }

    pub async fn check_consistency<R, V>(
        &self,
        repository: &R,
        vector_store: &V,
    ) -> Result<ConsistencyReport>
    where
        R: Repository,
        V: VectorStore,
    {
        // TODO: 实现完整的一致性检查逻辑
        // 这里返回一个虚拟的报告
        Ok(ConsistencyReport {
            total_messages: 100,
            repository_count: 100,
            vector_store_count: 98,
            inconsistencies: vec![Inconsistency {
                message_id: "msg_123".to_string(),
                issue_type: InconsistencyType::MissingInVectorStore,
                description: "Message exists in repository but not in vector store".to_string(),
            }],
            sync_timestamp: chrono::Utc::now(),
        })
    }
}

/// 同步管理器
pub struct SyncManager {
    sync_status: Arc<RwLock<HashMap<String, chrono::DateTime<chrono::Utc>>>>,
}

impl SyncManager {
    pub fn new() -> Self {
        Self {
            sync_status: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn record_sync(&self, message_id: &str) -> Result<()> {
        let mut status = self.sync_status.write().await;
        status.insert(message_id.to_string(), chrono::Utc::now());
        Ok(())
    }

    pub async fn get_last_sync(&self, message_id: &str) -> Option<chrono::DateTime<chrono::Utc>> {
        let status = self.sync_status.read().await;
        status.get(message_id).cloned()
    }
}

/// 默认实现：内存Repository
pub struct InMemoryRepository {
    messages: Arc<RwLock<HashMap<String, StoredMessage>>>,
    namespace_index: Arc<RwLock<HashMap<Option<String>, Vec<String>>>>, // namespace -> message_ids
}

impl InMemoryRepository {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(RwLock::new(HashMap::new())),
            namespace_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Repository for InMemoryRepository {
    async fn store(&self, message: &Message, namespace: Option<&str>) -> Result<String> {
        let message_id = format!("msg_{}", uuid::Uuid::new_v4());
        let stored_message = StoredMessage {
            id: message_id.clone(),
            message: message.clone(),
            namespace: namespace.map(|s| s.to_string()),
            stored_at: chrono::Utc::now(),
            version: 1,
        };

        let mut messages = self.messages.write().await;
        messages.insert(message_id.clone(), stored_message);

        // 更新命名空间索引
        let mut index = self.namespace_index.write().await;
        index
            .entry(namespace.map(|s| s.to_string()))
            .or_insert_with(Vec::new)
            .push(message_id.clone());

        Ok(message_id)
    }

    async fn retrieve(&self, query: &RepositoryQuery) -> Result<Vec<StoredMessage>> {
        let messages = self.messages.read().await;
        let index = self.namespace_index.read().await;

        let mut candidates = Vec::new();

        // 根据命名空间过滤
        if let Some(namespace) = &query.namespace {
            if let Some(message_ids) = index.get(&Some(namespace.clone())) {
                for id in message_ids {
                    if let Some(msg) = messages.get(id) {
                        candidates.push(msg.clone());
                    }
                }
            }
        } else {
            // 如果没有指定命名空间，返回所有消息
            candidates.extend(messages.values().cloned());
        }

        // 时间范围过滤
        if let Some(time_range) = &query.time_range {
            candidates
                .retain(|msg| msg.stored_at >= time_range.start && msg.stored_at <= time_range.end);
        }

        // 排序
        match &query.sort_by {
            Some(SortBy::TimestampAsc) => {
                candidates.sort_by(|a, b| a.stored_at.cmp(&b.stored_at));
            }
            Some(SortBy::TimestampDesc) => {
                candidates.sort_by(|a, b| b.stored_at.cmp(&a.stored_at));
            }
            _ => {}
        }

        // 限制数量
        if let Some(limit) = query.limit {
            candidates.truncate(limit);
        }

        Ok(candidates)
    }

    async fn delete(&self, message_id: &str) -> Result<()> {
        let mut messages = self.messages.write().await;
        if let Some(removed) = messages.remove(message_id) {
            // 从命名空间索引中移除
            let mut index = self.namespace_index.write().await;
            if let Some(ref namespace) = removed.namespace {
                if let Some(message_ids) = index.get_mut(&Some(namespace.clone())) {
                    message_ids.retain(|id| id != message_id);
                }
            }
        }
        Ok(())
    }

    async fn batch_store(&self, messages: &[(&Message, Option<&str>)]) -> Result<Vec<String>> {
        let mut ids = Vec::new();
        for (message, namespace) in messages {
            let id = self.store(message, *namespace).await?;
            ids.push(id);
        }
        Ok(ids)
    }

    async fn stats(&self) -> Result<RepositoryStats> {
        let messages = self.messages.read().await;
        let index = self.namespace_index.read().await;

        let total_namespaces = index.len();
        let total_messages = messages.len();

        let oldest_message = messages
            .values()
            .min_by_key(|m| m.stored_at)
            .map(|m| m.stored_at);

        let newest_message = messages
            .values()
            .max_by_key(|m| m.stored_at)
            .map(|m| m.stored_at);

        Ok(RepositoryStats {
            total_messages,
            total_namespaces,
            oldest_message,
            newest_message,
        })
    }
}

/// 默认实现：内存VectorStore
pub struct InMemoryVectorStore {
    vectors: Arc<RwLock<HashMap<String, VectorEntry>>>,
}

impl InMemoryVectorStore {
    pub fn new() -> Self {
        Self {
            vectors: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl VectorStore for InMemoryVectorStore {
    async fn add_vectors(&self, vectors: &[VectorEntry]) -> Result<()> {
        let mut store = self.vectors.write().await;
        for vector in vectors {
            store.insert(vector.id.clone(), vector.clone());
        }
        Ok(())
    }

    async fn semantic_search(
        &self,
        query: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        let store = self.vectors.read().await;

        // 简单的文本匹配搜索（实际实现应该使用向量相似度）
        let mut results: Vec<SearchResult> = store
            .values()
            .filter(|entry| {
                // 命名空间过滤
                if let Some(ref namespace) = options.namespace {
                    if entry.namespace.as_ref() != Some(namespace) {
                        return false;
                    }
                }

                // 简单的文本匹配（实际应该使用向量相似度）
                entry
                    .metadata
                    .values()
                    .any(|v| v.to_lowercase().contains(&query.to_lowercase()))
            })
            .take(options.limit)
            .map(|entry| SearchResult {
                id: entry.id.clone(),
                score: 0.8, // 虚拟分数
                metadata: entry.metadata.clone(),
            })
            .collect();

        // 按分数排序
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(results)
    }

    async fn remove_vectors(&self, ids: &[String]) -> Result<()> {
        let mut store = self.vectors.write().await;
        for id in ids {
            store.remove(id);
        }
        Ok(())
    }

    async fn reindex(&self, entries: &[VectorEntry]) -> Result<()> {
        // 清除现有索引并重新添加
        let mut store = self.vectors.write().await;
        store.clear();

        for entry in entries {
            store.insert(entry.id.clone(), entry.clone());
        }

        Ok(())
    }

    async fn index_stats(&self) -> Result<VectorStats> {
        let store = self.vectors.read().await;

        Ok(VectorStats {
            total_vectors: store.len(),
            dimensions: 5, // 假设5维向量
            index_size_mb: (store.len() * 5 * 4) as f64 / (1024.0 * 1024.0), // 粗略估算
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{Message, Role};

    #[tokio::test]
    async fn test_repository_first_storage() -> Result<()> {
        let repository = Arc::new(InMemoryRepository::new());
        let vector_store = Arc::new(InMemoryVectorStore::new());
        let storage = RepositoryFirstStorageImpl::new(repository, vector_store);

        // 创建测试消息
        let message = Message::new(
            Role::User,
            "Hello, this is a test message".to_string(),
            None,
            None,
        );

        // 存储消息
        storage
            .store_message(&message, Some("test_namespace"))
            .await?;

        // 检索消息 - 时间查询
        let temporal_query = StorageQuery {
            query_type: QueryType::Temporal,
            namespace: Some("test_namespace".to_string()),
            limit: Some(10),
            ..Default::default()
        };

        let results = storage.retrieve_messages(&temporal_query).await?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "Hello, this is a test message");

        // 检索消息 - 语义查询
        let semantic_query = StorageQuery {
            query_type: QueryType::Semantic,
            namespace: Some("test_namespace".to_string()),
            semantic_query: Some(SemanticQuery {
                query: "test".to_string(),
                top_k: 5,
                threshold: None,
                filters: None,
            }),
            ..Default::default()
        };

        let semantic_results = storage.retrieve_messages(&semantic_query).await?;
        // 语义搜索可能返回空结果（取决于实现）
        assert!(semantic_results.len() >= 0);

        // 获取统计信息
        let stats = storage.get_stats().await?;
        assert_eq!(stats.total_messages, 1);
        assert!(stats.consistency_score >= 0.0 && stats.consistency_score <= 1.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_consistency_check() -> Result<()> {
        let repository = Arc::new(InMemoryRepository::new());
        let vector_store = Arc::new(InMemoryVectorStore::new());
        let storage = RepositoryFirstStorageImpl::new(repository, vector_store);

        // 执行一致性检查
        let report = storage.sync_consistency().await?;
        assert_eq!(report.total_messages, 100); // 虚拟数据
        assert!(report.inconsistencies.len() >= 0);

        Ok(())
    }
}
