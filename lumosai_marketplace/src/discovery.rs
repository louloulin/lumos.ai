//! 工具发现引擎实现

use async_trait::async_trait;
use std::collections::HashMap;

use crate::models::{ToolPackage, ToolCategory};
use crate::search::{SearchEngine, SearchQuery, SearchResult as SearchEngineResult};
use crate::storage::Storage;
use crate::error::{MarketplaceError, Result};

/// 搜索结果
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// 工具包
    pub package: ToolPackage,
    
    /// 相关性分数
    pub relevance_score: f32,
    
    /// 匹配原因
    pub match_reason: String,
    
    /// 推荐理由
    pub recommendation_reason: Option<String>,
}

/// 工具发现引擎trait
#[async_trait]
pub trait ToolDiscoveryEngine: Send + Sync {
    /// 搜索工具
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>>;
    
    /// 推荐工具
    async fn recommend(&self, user_context: &UserContext) -> Result<Vec<SearchResult>>;
    
    /// 获取热门工具
    async fn get_trending(&self, category: Option<ToolCategory>, limit: usize) -> Result<Vec<SearchResult>>;
    
    /// 获取相似工具
    async fn get_similar(&self, package_id: uuid::Uuid, limit: usize) -> Result<Vec<SearchResult>>;
    
    /// 获取新发布的工具
    async fn get_recent(&self, limit: usize) -> Result<Vec<SearchResult>>;
}

/// 用户上下文
#[derive(Debug, Clone)]
pub struct UserContext {
    /// 用户ID
    pub user_id: Option<String>,
    
    /// 用户偏好的分类
    pub preferred_categories: Vec<ToolCategory>,
    
    /// 用户使用过的工具
    pub used_tools: Vec<String>,
    
    /// 用户搜索历史
    pub search_history: Vec<String>,
    
    /// 用户评分历史
    pub rating_history: HashMap<String, f64>,
}

/// 默认工具发现引擎
pub struct DefaultToolDiscoveryEngine {
    storage: std::sync::Arc<dyn Storage>,
    search_engine: std::sync::Arc<dyn SearchEngine>,
}

impl DefaultToolDiscoveryEngine {
    /// 创建新的默认发现引擎
    pub fn new(
        storage: std::sync::Arc<dyn Storage>,
        search_engine: std::sync::Arc<dyn SearchEngine>,
    ) -> Self {
        Self {
            storage,
            search_engine,
        }
    }
    
    /// 计算推荐分数
    fn calculate_recommendation_score(&self, package: &ToolPackage, user_context: &UserContext) -> f32 {
        let mut score = 0.0f32;
        
        // 基础分数：评分和下载量
        score += package.rating as f32 * 0.3;
        score += (package.download_count as f32).log10() * 0.2;
        
        // 分类匹配
        for category in &package.categories {
            if user_context.preferred_categories.contains(category) {
                score += 2.0;
            }
        }
        
        // 关键词匹配
        for keyword in &package.keywords {
            if user_context.search_history.iter().any(|h| h.contains(keyword)) {
                score += 1.0;
            }
        }
        
        // 避免推荐已使用的工具
        if user_context.used_tools.contains(&package.name) {
            score *= 0.5;
        }
        
        score.max(0.0).min(10.0)
    }
    
    /// 生成匹配原因
    fn generate_match_reason(&self, package: &ToolPackage, query: &SearchQuery) -> String {
        let mut reasons = Vec::new();
        
        if !query.text.is_empty() {
            if package.name.to_lowercase().contains(&query.text.to_lowercase()) {
                reasons.push("名称匹配".to_string());
            }
            if package.description.to_lowercase().contains(&query.text.to_lowercase()) {
                reasons.push("描述匹配".to_string());
            }
            for keyword in &package.keywords {
                if keyword.to_lowercase().contains(&query.text.to_lowercase()) {
                    reasons.push("关键词匹配".to_string());
                    break;
                }
            }
        }
        
        for category in &query.categories {
            if package.categories.contains(category) {
                reasons.push(format!("分类匹配: {}", category.display_name()));
            }
        }
        
        if reasons.is_empty() {
            "通用匹配".to_string()
        } else {
            reasons.join(", ")
        }
    }
    
    /// 生成推荐理由
    fn generate_recommendation_reason(&self, package: &ToolPackage, user_context: &UserContext) -> Option<String> {
        let mut reasons = Vec::new();
        
        // 高评分
        if package.rating >= 4.5 {
            reasons.push("高评分工具".to_string());
        }
        
        // 热门工具
        if package.download_count > 1000 {
            reasons.push("热门工具".to_string());
        }
        
        // 分类匹配
        for category in &package.categories {
            if user_context.preferred_categories.contains(category) {
                reasons.push(format!("符合您的{}偏好", category.display_name()));
                break;
            }
        }
        
        // 新工具
        let days_since_creation = (chrono::Utc::now() - package.created_at).num_days();
        if days_since_creation <= 30 {
            reasons.push("新发布工具".to_string());
        }
        
        if reasons.is_empty() {
            None
        } else {
            Some(reasons.join(", "))
        }
    }
}

#[async_trait]
impl ToolDiscoveryEngine for DefaultToolDiscoveryEngine {
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let search_results = self.search_engine.search(query).await?;
        
        let mut results = Vec::new();
        for search_result in search_results {
            if let Some(package) = self.storage.get_package(search_result.package_id).await? {
                let match_reason = self.generate_match_reason(&package, query);
                
                results.push(SearchResult {
                    package,
                    relevance_score: search_result.score,
                    match_reason,
                    recommendation_reason: None,
                });
            }
        }
        
        Ok(results)
    }
    
    async fn recommend(&self, user_context: &UserContext) -> Result<Vec<SearchResult>> {
        // 获取所有已发布的工具包
        let packages = self.storage.list_packages(0, 100).await?;
        
        let mut recommendations = Vec::new();
        for package in packages {
            if !package.published {
                continue;
            }
            
            let score = self.calculate_recommendation_score(&package, user_context);
            if score > 3.0 { // 只推荐分数较高的工具
                let recommendation_reason = self.generate_recommendation_reason(&package, user_context);
                
                recommendations.push(SearchResult {
                    package,
                    relevance_score: score,
                    match_reason: "个性化推荐".to_string(),
                    recommendation_reason,
                });
            }
        }
        
        // 按分数排序
        recommendations.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        recommendations.truncate(20); // 限制推荐数量
        
        Ok(recommendations)
    }
    
    async fn get_trending(&self, category: Option<ToolCategory>, limit: usize) -> Result<Vec<SearchResult>> {
        let packages = if let Some(cat) = category {
            self.storage.search_by_category(&cat).await?
        } else {
            self.storage.get_popular_packages(limit as u32).await?
        };
        
        let mut results = Vec::new();
        for package in packages.into_iter().take(limit) {
            if package.published {
                results.push(SearchResult {
                    package,
                    relevance_score: 0.0, // 趋势不需要相关性分数
                    match_reason: "热门趋势".to_string(),
                    recommendation_reason: Some("当前热门工具".to_string()),
                });
            }
        }
        
        Ok(results)
    }
    
    async fn get_similar(&self, package_id: uuid::Uuid, limit: usize) -> Result<Vec<SearchResult>> {
        let target_package = self.storage.get_package(package_id).await?
            .ok_or_else(|| MarketplaceError::ToolNotFound(package_id.to_string()))?;
        
        // 基于分类和关键词查找相似工具
        let mut similar_packages = Vec::new();
        
        // 按分类查找
        for category in &target_package.categories {
            let category_packages = self.storage.search_by_category(category).await?;
            similar_packages.extend(category_packages);
        }
        
        // 按关键词查找
        if !target_package.keywords.is_empty() {
            let keyword_packages = self.storage.search_by_keywords(&target_package.keywords).await?;
            similar_packages.extend(keyword_packages);
        }
        
        // 去重并排除目标工具本身
        similar_packages.retain(|p| p.id != package_id && p.published);
        similar_packages.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap());
        similar_packages.dedup_by(|a, b| a.id == b.id);
        
        let mut results = Vec::new();
        for package in similar_packages.into_iter().take(limit) {
            results.push(SearchResult {
                package,
                relevance_score: 0.0,
                match_reason: "相似工具".to_string(),
                recommendation_reason: Some("与您查看的工具相似".to_string()),
            });
        }
        
        Ok(results)
    }
    
    async fn get_recent(&self, limit: usize) -> Result<Vec<SearchResult>> {
        let packages = self.storage.list_packages(0, limit as u32).await?;
        
        let mut results = Vec::new();
        for package in packages {
            if package.published {
                results.push(SearchResult {
                    package,
                    relevance_score: 0.0,
                    match_reason: "最新发布".to_string(),
                    recommendation_reason: Some("最新发布的工具".to_string()),
                });
            }
        }
        
        Ok(results)
    }
}

impl Default for UserContext {
    fn default() -> Self {
        Self {
            user_id: None,
            preferred_categories: Vec::new(),
            used_tools: Vec::new(),
            search_history: Vec::new(),
            rating_history: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::SqliteStorage;
    use crate::search::TantivySearchEngine;
    use tempfile::{TempDir, NamedTempFile};
    use std::sync::Arc;
    
    async fn create_test_discovery_engine() -> DefaultToolDiscoveryEngine {
        let temp_file = NamedTempFile::new().unwrap();
        let database_url = format!("sqlite://{}", temp_file.path().display());
        let storage = Arc::new(SqliteStorage::new(&database_url).await.unwrap());
        storage.initialize().await.unwrap();
        
        let temp_dir = TempDir::new().unwrap();
        let search_engine = Arc::new(TantivySearchEngine::new(temp_dir.path()).await.unwrap());
        
        DefaultToolDiscoveryEngine::new(storage, search_engine)
    }
    
    #[tokio::test]
    async fn test_discovery_engine_creation() {
        let engine = create_test_discovery_engine().await;
        
        let query = SearchQuery::default();
        let results = engine.search(&query).await.unwrap();
        assert!(results.is_empty()); // 空数据库应该返回空结果
    }
    
    #[tokio::test]
    async fn test_recommendation_score_calculation() {
        let engine = create_test_discovery_engine().await;
        let package = create_test_package();
        let user_context = UserContext {
            preferred_categories: vec![ToolCategory::Utility],
            ..Default::default()
        };
        
        let score = engine.calculate_recommendation_score(&package, &user_context);
        assert!(score > 0.0);
    }
    
    #[tokio::test]
    async fn test_match_reason_generation() {
        let engine = create_test_discovery_engine().await;
        let package = create_test_package();
        let query = SearchQuery {
            text: "test".to_string(),
            categories: vec![ToolCategory::Utility],
            ..Default::default()
        };
        
        let reason = engine.generate_match_reason(&package, &query);
        assert!(reason.contains("匹配"));
    }
    
    fn create_test_package() -> ToolPackage {
        use chrono::Utc;
        use semver::Version;
        use std::collections::HashMap;
        use uuid::Uuid;
        
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
            download_count: 100,
            rating: 4.5,
            rating_count: 10,
            published: true,
            verified: false,
            security_audit: None,
            performance_benchmark: None,
        }
    }
}
