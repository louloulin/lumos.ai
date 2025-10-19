//! Graph RAG (GraphRAG) - 知识图谱检索
//!
//! 实现基于知识图谱的检索增强生成，对标 LlamaIndex 的 GraphRAG 能力。
//!
//! 核心功能：
//! - 实体提取和关系识别
//! - 知识图谱构建
//! - 图遍历检索
//! - 社区检测和聚类

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    error::{RagError, Result},
    retriever::Retriever,
    types::{Document, RetrievalRequest, RetrievalResult, ScoredDocument},
};

/// 知识图谱中的实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// 实体 ID
    pub id: String,
    /// 实体名称
    pub name: String,
    /// 实体类型（如：Person, Organization, Location）
    pub entity_type: String,
    /// 实体属性
    pub properties: HashMap<String, String>,
    /// 关联的文档 ID 列表
    pub document_ids: Vec<String>,
}

/// 知识图谱中的关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    /// 关系 ID
    pub id: String,
    /// 源实体 ID
    pub source_id: String,
    /// 目标实体 ID
    pub target_id: String,
    /// 关系类型（如：works_for, located_in, related_to）
    pub relation_type: String,
    /// 关系权重（0.0-1.0）
    pub weight: f32,
    /// 关系属性
    pub properties: HashMap<String, String>,
}

/// 知识图谱
#[derive(Debug, Clone)]
pub struct KnowledgeGraph {
    /// 实体映射：entity_id -> Entity
    entities: Arc<RwLock<HashMap<String, Entity>>>,
    /// 关系映射：relation_id -> Relation
    relations: Arc<RwLock<HashMap<String, Relation>>>,
    /// 实体邻接表：entity_id -> [relation_id]
    adjacency: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// 文档到实体的映射：document_id -> [entity_id]
    document_entities: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl KnowledgeGraph {
    /// 创建新的知识图谱
    pub fn new() -> Self {
        Self {
            entities: Arc::new(RwLock::new(HashMap::new())),
            relations: Arc::new(RwLock::new(HashMap::new())),
            adjacency: Arc::new(RwLock::new(HashMap::new())),
            document_entities: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 添加实体
    pub async fn add_entity(&self, entity: Entity) -> Result<()> {
        let entity_id = entity.id.clone();

        // 添加实体
        self.entities
            .write()
            .await
            .insert(entity_id.clone(), entity.clone());

        // 初始化邻接表
        self.adjacency
            .write()
            .await
            .entry(entity_id.clone())
            .or_insert_with(Vec::new);

        // 更新文档到实体的映射
        for doc_id in &entity.document_ids {
            self.document_entities
                .write()
                .await
                .entry(doc_id.clone())
                .or_insert_with(Vec::new)
                .push(entity_id.clone());
        }

        Ok(())
    }

    /// 添加关系
    pub async fn add_relation(&self, relation: Relation) -> Result<()> {
        let relation_id = relation.id.clone();
        let source_id = relation.source_id.clone();
        let target_id = relation.target_id.clone();

        // 添加关系
        self.relations
            .write()
            .await
            .insert(relation_id.clone(), relation);

        // 更新邻接表
        self.adjacency
            .write()
            .await
            .entry(source_id)
            .or_insert_with(Vec::new)
            .push(relation_id.clone());

        self.adjacency
            .write()
            .await
            .entry(target_id)
            .or_insert_with(Vec::new)
            .push(relation_id);

        Ok(())
    }

    /// 获取实体
    pub async fn get_entity(&self, entity_id: &str) -> Option<Entity> {
        self.entities.read().await.get(entity_id).cloned()
    }

    /// 获取关系
    pub async fn get_relation(&self, relation_id: &str) -> Option<Relation> {
        self.relations.read().await.get(relation_id).cloned()
    }

    /// 获取实体的邻居（通过关系连接的实体）
    pub async fn get_neighbors(&self, entity_id: &str, max_depth: usize) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut neighbors = Vec::new();

        queue.push_back((entity_id.to_string(), 0));
        visited.insert(entity_id.to_string());

        while let Some((current_id, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            // 获取当前实体的所有关系
            if let Some(relation_ids) = self.adjacency.read().await.get(&current_id) {
                for relation_id in relation_ids {
                    if let Some(relation) = self.get_relation(relation_id).await {
                        // 找到邻居实体
                        let neighbor_id = if relation.source_id == current_id {
                            &relation.target_id
                        } else {
                            &relation.source_id
                        };

                        if !visited.contains(neighbor_id) {
                            visited.insert(neighbor_id.clone());
                            neighbors.push(neighbor_id.clone());
                            queue.push_back((neighbor_id.clone(), depth + 1));
                        }
                    }
                }
            }
        }

        neighbors
    }

    /// 获取与文档相关的所有实体
    pub async fn get_document_entities(&self, document_id: &str) -> Vec<Entity> {
        let mut entities = Vec::new();

        if let Some(entity_ids) = self.document_entities.read().await.get(document_id) {
            for entity_id in entity_ids {
                if let Some(entity) = self.get_entity(entity_id).await {
                    entities.push(entity);
                }
            }
        }

        entities
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// GraphRAG 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphRagConfig {
    /// 图遍历的最大深度
    pub max_traversal_depth: usize,
    /// 是否启用社区检测
    pub enable_community_detection: bool,
    /// 实体相似度阈值
    pub entity_similarity_threshold: f32,
    /// 最大返回结果数
    pub max_results: usize,
}

impl Default for GraphRagConfig {
    fn default() -> Self {
        Self {
            max_traversal_depth: 2,
            enable_community_detection: false,
            entity_similarity_threshold: 0.7,
            max_results: 10,
        }
    }
}

/// GraphRAG 检索器
pub struct GraphRagRetriever {
    /// 知识图谱
    graph: Arc<KnowledgeGraph>,
    /// 文档存储：document_id -> Document
    documents: Arc<RwLock<HashMap<String, Document>>>,
    /// 配置
    config: GraphRagConfig,
}

impl GraphRagRetriever {
    /// 创建新的 GraphRAG 检索器
    pub fn new(config: GraphRagConfig) -> Self {
        Self {
            graph: Arc::new(KnowledgeGraph::new()),
            documents: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// 添加文档到图谱
    pub async fn add_document(&self, document: Document) -> Result<()> {
        let doc_id = document.id.clone();

        // 存储文档
        self.documents
            .write()
            .await
            .insert(doc_id.clone(), document.clone());

        // 提取实体和关系（简化实现，实际应使用 NER 模型）
        let entities = self.extract_entities(&document).await?;
        let relations = self.extract_relations(&document, &entities).await?;

        // 添加实体到图谱
        for entity in entities {
            self.graph.add_entity(entity).await?;
        }

        // 添加关系到图谱
        for relation in relations {
            self.graph.add_relation(relation).await?;
        }

        Ok(())
    }

    /// 从文档中提取实体（简化实现）
    async fn extract_entities(&self, document: &Document) -> Result<Vec<Entity>> {
        // 简化实现：基于关键词提取
        // 实际应使用 NER 模型（如 spaCy, BERT-NER）
        let mut entities = Vec::new();
        let content = &document.content;

        // 示例：提取大写开头的词作为潜在实体
        let words: Vec<&str> = content.split_whitespace().collect();
        let mut entity_names = HashSet::new();

        for word in words {
            if word.chars().next().map_or(false, |c| c.is_uppercase()) {
                let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
                if clean_word.len() > 2 {
                    entity_names.insert(clean_word.to_string());
                }
            }
        }

        // 创建实体对象
        for (idx, name) in entity_names.iter().enumerate() {
            entities.push(Entity {
                id: format!("{}_{}", document.id, idx),
                name: name.clone(),
                entity_type: "Unknown".to_string(),
                properties: HashMap::new(),
                document_ids: vec![document.id.clone()],
            });
        }

        Ok(entities)
    }

    /// 从文档中提取关系（简化实现）
    async fn extract_relations(
        &self,
        _document: &Document,
        entities: &[Entity],
    ) -> Result<Vec<Relation>> {
        // 简化实现：基于共现创建关系
        // 实际应使用关系抽取模型
        let mut relations = Vec::new();

        // 为每对实体创建 "related_to" 关系
        for (i, entity1) in entities.iter().enumerate() {
            for entity2 in entities.iter().skip(i + 1) {
                relations.push(Relation {
                    id: format!("rel_{}_{}", entity1.id, entity2.id),
                    source_id: entity1.id.clone(),
                    target_id: entity2.id.clone(),
                    relation_type: "related_to".to_string(),
                    weight: 0.5,
                    properties: HashMap::new(),
                });
            }
        }

        Ok(relations)
    }
}

#[async_trait]
impl Retriever for GraphRagRetriever {
    async fn retrieve(&self, request: &RetrievalRequest) -> Result<RetrievalResult> {
        // 1. 从查询中提取实体
        let query_entities = self.extract_query_entities(&request.query).await?;

        // 2. 在图谱中查找相关实体
        let mut relevant_entity_ids = HashSet::new();
        for query_entity in &query_entities {
            // 查找匹配的实体
            let entities = self.graph.entities.read().await;
            for (entity_id, entity) in entities.iter() {
                if entity
                    .name
                    .to_lowercase()
                    .contains(&query_entity.to_lowercase())
                {
                    relevant_entity_ids.insert(entity_id.clone());
                }
            }
        }

        // 3. 图遍历：获取相关实体的邻居
        let mut all_relevant_entities = relevant_entity_ids.clone();
        for entity_id in &relevant_entity_ids {
            let neighbors = self
                .graph
                .get_neighbors(entity_id, self.config.max_traversal_depth)
                .await;
            all_relevant_entities.extend(neighbors);
        }

        // 4. 收集相关文档
        let mut document_scores: HashMap<String, f32> = HashMap::new();
        for entity_id in &all_relevant_entities {
            if let Some(entity) = self.graph.get_entity(entity_id).await {
                for doc_id in &entity.document_ids {
                    *document_scores.entry(doc_id.clone()).or_insert(0.0) += 1.0;
                }
            }
        }

        // 5. 按分数排序并返回
        let mut scored_docs: Vec<(String, f32)> = document_scores.into_iter().collect();
        scored_docs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let limit = request.options.limit.unwrap_or(self.config.max_results);
        let documents = self.documents.read().await;

        let results: Vec<ScoredDocument> = scored_docs
            .into_iter()
            .take(limit)
            .filter_map(|(doc_id, score)| {
                documents.get(&doc_id).map(|doc| ScoredDocument {
                    document: doc.clone(),
                    score,
                })
            })
            .collect();

        Ok(RetrievalResult {
            documents: results.clone(),
            total_count: results.len(),
        })
    }
}

impl GraphRagRetriever {
    /// 从查询中提取实体（简化实现）
    async fn extract_query_entities(&self, query: &str) -> Result<Vec<String>> {
        // 简化实现：提取大写开头的词
        let words: Vec<&str> = query.split_whitespace().collect();
        let mut entities = Vec::new();

        for word in words {
            if word.chars().next().map_or(false, |c| c.is_uppercase()) {
                let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
                if clean_word.len() > 2 {
                    entities.push(clean_word.to_string());
                }
            }
        }

        // 如果没有找到实体，返回整个查询
        if entities.is_empty() {
            entities.push(query.to_string());
        }

        Ok(entities)
    }
}
