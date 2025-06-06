//! 工具市场搜索引擎实现

use async_trait::async_trait;
use tantivy::{
    Index, IndexWriter, IndexReader, Document, Term, TantivyDocument,
    schema::{Schema, Field, TEXT, STORED, STRING, FAST, U64},
    query::{QueryParser, FuzzyTermQuery, BooleanQuery, Occur, TermQuery},
    collector::TopDocs,
    ReloadPolicy,
};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::models::{ToolPackage, ToolCategory};
use crate::error::{MarketplaceError, Result};

/// 搜索查询
#[derive(Debug, Clone)]
pub struct SearchQuery {
    /// 查询文本
    pub text: String,
    
    /// 分类过滤
    pub categories: Vec<ToolCategory>,
    
    /// 关键词过滤
    pub keywords: Vec<String>,
    
    /// 是否只搜索已发布的工具
    pub published_only: bool,
    
    /// 是否只搜索已验证的工具
    pub verified_only: bool,
    
    /// 最小评分
    pub min_rating: Option<f64>,
    
    /// 排序方式
    pub sort_by: SortBy,
    
    /// 结果数量限制
    pub limit: usize,
    
    /// 偏移量
    pub offset: usize,
}

/// 排序方式
#[derive(Debug, Clone)]
pub enum SortBy {
    /// 相关性
    Relevance,
    /// 下载次数
    Downloads,
    /// 评分
    Rating,
    /// 创建时间
    CreatedAt,
    /// 更新时间
    UpdatedAt,
}

/// 搜索结果
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// 工具包ID
    pub package_id: Uuid,
    
    /// 相关性分数
    pub score: f32,
    
    /// 匹配的字段
    pub matched_fields: Vec<String>,
    
    /// 高亮片段
    pub highlights: Vec<String>,
}

/// 搜索引擎trait
#[async_trait]
pub trait SearchEngine: Send + Sync {
    /// 初始化搜索引擎
    async fn initialize(&self) -> Result<()>;
    
    /// 索引工具包
    async fn index_package(&self, package: &ToolPackage) -> Result<()>;
    
    /// 更新工具包索引
    async fn update_package_index(&self, package: &ToolPackage) -> Result<()>;
    
    /// 删除工具包索引
    async fn delete_package_index(&self, package_id: Uuid) -> Result<()>;
    
    /// 搜索工具包
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>>;
    
    /// 建议搜索词
    async fn suggest(&self, prefix: &str, limit: usize) -> Result<Vec<String>>;
    
    /// 重建索引
    async fn rebuild_index(&self, packages: &[ToolPackage]) -> Result<()>;
    
    /// 获取索引统计信息
    async fn get_index_stats(&self) -> Result<IndexStats>;
}

/// 索引统计信息
#[derive(Debug, Clone)]
pub struct IndexStats {
    /// 索引的文档数量
    pub document_count: u64,
    
    /// 索引大小（字节）
    pub index_size_bytes: u64,
    
    /// 最后更新时间
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Tantivy搜索引擎实现
pub struct TantivySearchEngine {
    index: Index,
    schema: Schema,
    fields: SearchFields,
    writer: Arc<RwLock<IndexWriter>>,
    reader: IndexReader,
    fuzzy_matcher: SkimMatcherV2,
}

/// 搜索字段定义
#[derive(Debug, Clone)]
struct SearchFields {
    id: Field,
    name: Field,
    description: Field,
    author: Field,
    keywords: Field,
    categories: Field,
    license: Field,
    download_count: Field,
    rating: Field,
    published: Field,
    verified: Field,
    created_at: Field,
    updated_at: Field,
}

impl TantivySearchEngine {
    /// 创建新的Tantivy搜索引擎
    pub async fn new(index_path: impl AsRef<Path>) -> Result<Self> {
        let schema = Self::build_schema();
        let fields = Self::extract_fields(&schema);
        
        // 创建或打开索引
        let index = if index_path.as_ref().exists() {
            Index::open_in_dir(index_path)?
        } else {
            std::fs::create_dir_all(&index_path)?;
            Index::create_in_dir(index_path, schema.clone())?
        };
        
        // 创建写入器和读取器
        let writer = index.writer(50_000_000)?; // 50MB heap
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?;
        
        let fuzzy_matcher = SkimMatcherV2::default();
        
        Ok(Self {
            index,
            schema,
            fields,
            writer: Arc::new(RwLock::new(writer)),
            reader,
            fuzzy_matcher,
        })
    }
    
    /// 构建搜索模式
    fn build_schema() -> Schema {
        let mut schema_builder = Schema::builder();
        
        // 添加字段
        schema_builder.add_text_field("id", STRING | STORED);
        schema_builder.add_text_field("name", TEXT | STORED);
        schema_builder.add_text_field("description", TEXT | STORED);
        schema_builder.add_text_field("author", TEXT | STORED);
        schema_builder.add_text_field("keywords", TEXT);
        schema_builder.add_text_field("categories", TEXT);
        schema_builder.add_text_field("license", STRING | STORED);
        schema_builder.add_u64_field("download_count", FAST | STORED);
        schema_builder.add_u64_field("rating", FAST | STORED);
        schema_builder.add_text_field("published", STRING | FAST);
        schema_builder.add_text_field("verified", STRING | FAST);
        schema_builder.add_u64_field("created_at", FAST | STORED);
        schema_builder.add_u64_field("updated_at", FAST | STORED);
        
        schema_builder.build()
    }
    
    /// 提取字段引用
    fn extract_fields(schema: &Schema) -> SearchFields {
        SearchFields {
            id: schema.get_field("id").unwrap(),
            name: schema.get_field("name").unwrap(),
            description: schema.get_field("description").unwrap(),
            author: schema.get_field("author").unwrap(),
            keywords: schema.get_field("keywords").unwrap(),
            categories: schema.get_field("categories").unwrap(),
            license: schema.get_field("license").unwrap(),
            download_count: schema.get_field("download_count").unwrap(),
            rating: schema.get_field("rating").unwrap(),
            published: schema.get_field("published").unwrap(),
            verified: schema.get_field("verified").unwrap(),
            created_at: schema.get_field("created_at").unwrap(),
            updated_at: schema.get_field("updated_at").unwrap(),
        }
    }
    
    /// 将工具包转换为文档
    fn package_to_document(&self, package: &ToolPackage) -> Document {
        let mut doc = TantivyDocument::default();
        
        doc.add_text(self.fields.id, &package.id.to_string());
        doc.add_text(self.fields.name, &package.name);
        doc.add_text(self.fields.description, &package.description);
        doc.add_text(self.fields.author, &package.author);
        doc.add_text(self.fields.keywords, &package.keywords.join(" "));
        
        let categories_text: Vec<String> = package.categories.iter()
            .map(|c| c.display_name().to_string())
            .collect();
        doc.add_text(self.fields.categories, &categories_text.join(" "));
        
        doc.add_text(self.fields.license, &package.license);
        doc.add_u64(self.fields.download_count, package.download_count);
        doc.add_u64(self.fields.rating, (package.rating * 100.0) as u64); // 存储为整数
        doc.add_text(self.fields.published, if package.published { "true" } else { "false" });
        doc.add_text(self.fields.verified, if package.verified { "true" } else { "false" });
        doc.add_u64(self.fields.created_at, package.created_at.timestamp() as u64);
        doc.add_u64(self.fields.updated_at, package.updated_at.timestamp() as u64);
        
        doc
    }
}

#[async_trait]
impl SearchEngine for TantivySearchEngine {
    async fn initialize(&self) -> Result<()> {
        // Tantivy索引在创建时已经初始化
        Ok(())
    }
    
    async fn index_package(&self, package: &ToolPackage) -> Result<()> {
        let doc = self.package_to_document(package);
        
        let mut writer = self.writer.write().await;
        writer.add_document(doc)?;
        writer.commit()?;
        
        Ok(())
    }
    
    async fn update_package_index(&self, package: &ToolPackage) -> Result<()> {
        // 先删除旧文档
        self.delete_package_index(package.id).await?;
        
        // 再添加新文档
        self.index_package(package).await?;
        
        Ok(())
    }
    
    async fn delete_package_index(&self, package_id: Uuid) -> Result<()> {
        let term = Term::from_field_text(self.fields.id, &package_id.to_string());
        
        let mut writer = self.writer.write().await;
        writer.delete_term(term);
        writer.commit()?;
        
        Ok(())
    }
    
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let searcher = self.reader.searcher();
        
        // 构建查询
        let mut boolean_query = BooleanQuery::new(vec![]);
        
        // 文本查询
        if !query.text.is_empty() {
            let query_parser = QueryParser::for_index(
                &self.index,
                vec![self.fields.name, self.fields.description, self.fields.keywords],
            );
            
            if let Ok(parsed_query) = query_parser.parse_query(&query.text) {
                boolean_query.add_clause((Occur::Must, parsed_query));
            } else {
                // 如果解析失败，使用模糊查询
                let fuzzy_query = FuzzyTermQuery::new(
                    Term::from_field_text(self.fields.name, &query.text),
                    2, // 最大编辑距离
                    true,
                );
                boolean_query.add_clause((Occur::Should, Box::new(fuzzy_query)));
            }
        }
        
        // 分类过滤
        for category in &query.categories {
            let term = Term::from_field_text(self.fields.categories, category.display_name());
            let term_query = TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic);
            boolean_query.add_clause((Occur::Must, Box::new(term_query)));
        }
        
        // 发布状态过滤
        if query.published_only {
            let term = Term::from_field_text(self.fields.published, "true");
            let term_query = TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic);
            boolean_query.add_clause((Occur::Must, Box::new(term_query)));
        }
        
        // 验证状态过滤
        if query.verified_only {
            let term = Term::from_field_text(self.fields.verified, "true");
            let term_query = TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic);
            boolean_query.add_clause((Occur::Must, Box::new(term_query)));
        }
        
        // 执行搜索
        let top_docs = searcher.search(
            &boolean_query,
            &TopDocs::with_limit(query.limit).and_offset(query.offset),
        )?;
        
        // 转换结果
        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let doc = searcher.doc(doc_address)?;
            
            if let Some(id_value) = doc.get_first(self.fields.id) {
                if let Some(id_str) = id_value.as_text() {
                    if let Ok(package_id) = Uuid::parse_str(id_str) {
                        results.push(SearchResult {
                            package_id,
                            score,
                            matched_fields: vec![], // TODO: 实现字段匹配检测
                            highlights: vec![], // TODO: 实现高亮
                        });
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    async fn suggest(&self, prefix: &str, limit: usize) -> Result<Vec<String>> {
        // 简单的建议实现，基于模糊匹配
        let searcher = self.reader.searcher();
        
        // 获取所有工具名称
        let mut suggestions = Vec::new();
        
        // 这里应该实现更高效的建议算法
        // 目前使用简化版本
        let query_parser = QueryParser::for_index(&self.index, vec![self.fields.name]);
        
        if let Ok(query) = query_parser.parse_query(&format!("{}*", prefix)) {
            let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;
            
            for (_score, doc_address) in top_docs {
                let doc = searcher.doc(doc_address)?;
                if let Some(name_value) = doc.get_first(self.fields.name) {
                    if let Some(name) = name_value.as_text() {
                        suggestions.push(name.to_string());
                    }
                }
            }
        }
        
        Ok(suggestions)
    }
    
    async fn rebuild_index(&self, packages: &[ToolPackage]) -> Result<()> {
        let mut writer = self.writer.write().await;
        
        // 清空索引
        writer.delete_all_documents()?;
        
        // 重新索引所有工具包
        for package in packages {
            let doc = self.package_to_document(package);
            writer.add_document(doc)?;
        }
        
        writer.commit()?;
        
        Ok(())
    }
    
    async fn get_index_stats(&self) -> Result<IndexStats> {
        let searcher = self.reader.searcher();
        let document_count = searcher.num_docs() as u64;
        
        // 获取索引大小（简化实现）
        let index_size_bytes = 0; // TODO: 实现实际的大小计算
        
        Ok(IndexStats {
            document_count,
            index_size_bytes,
            last_updated: chrono::Utc::now(),
        })
    }
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            text: String::new(),
            categories: Vec::new(),
            keywords: Vec::new(),
            published_only: true,
            verified_only: false,
            min_rating: None,
            sort_by: SortBy::Relevance,
            limit: 20,
            offset: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_search_engine_creation() {
        let temp_dir = TempDir::new().unwrap();
        let engine = TantivySearchEngine::new(temp_dir.path()).await.unwrap();
        
        let stats = engine.get_index_stats().await.unwrap();
        assert_eq!(stats.document_count, 0);
    }
    
    #[tokio::test]
    async fn test_package_indexing() {
        let temp_dir = TempDir::new().unwrap();
        let engine = TantivySearchEngine::new(temp_dir.path()).await.unwrap();
        
        let package = create_test_package();
        engine.index_package(&package).await.unwrap();
        
        let stats = engine.get_index_stats().await.unwrap();
        assert_eq!(stats.document_count, 1);
    }
    
    #[tokio::test]
    async fn test_search_functionality() {
        let temp_dir = TempDir::new().unwrap();
        let engine = TantivySearchEngine::new(temp_dir.path()).await.unwrap();
        
        let package = create_test_package();
        engine.index_package(&package).await.unwrap();
        
        let query = SearchQuery {
            text: "test".to_string(),
            ..Default::default()
        };
        
        let results = engine.search(&query).await.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].package_id, package.id);
    }
    
    fn create_test_package() -> ToolPackage {
        use chrono::Utc;
        use semver::Version;
        use std::collections::HashMap;
        
        ToolPackage {
            id: Uuid::new_v4(),
            name: "test_tool".to_string(),
            version: Version::new(1, 0, 0),
            description: "Test tool description".to_string(),
            author: "Test Author".to_string(),
            author_email: Some("test@example.com".to_string()),
            license: "MIT".to_string(),
            homepage: None,
            repository: None,
            keywords: vec!["test".to_string()],
            categories: vec![ToolCategory::Utility],
            dependencies: HashMap::new(),
            lumos_version: "0.1.0".to_string(),
            manifest: crate::models::ToolManifest {
                tools: vec![],
                entry_point: "main.rs".to_string(),
                exports: vec![],
                permissions: vec![],
                config_schema: None,
                rust_version: None,
                build_script: None,
            },
            metadata: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            published_at: None,
            download_count: 0,
            rating: 0.0,
            rating_count: 0,
            published: true,
            verified: false,
            security_audit: None,
            performance_benchmark: None,
        }
    }
}
