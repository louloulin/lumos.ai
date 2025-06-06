//! 工具市场API实现

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::marketplace::ToolMarketplace;
use crate::models::{ToolPackage, ToolCategory};
use crate::search::SearchQuery;
use crate::discovery::UserContext;
use crate::analytics::{UserInfo, UserType};
use crate::error::{MarketplaceError, Result};

/// API响应包装器
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// 搜索请求参数
#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: Option<String>,
    pub category: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub published_only: Option<bool>,
    pub verified_only: Option<bool>,
    pub min_rating: Option<f64>,
}

/// 分页参数
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

/// 评分请求
#[derive(Debug, Deserialize)]
pub struct RatingRequest {
    pub rating: f64,
    pub user_id: Option<String>,
}

/// 创建API路由
pub fn create_routes(marketplace: Arc<ToolMarketplace>) -> Router {
    Router::new()
        // 搜索和发现
        .route("/search", get(search_packages))
        .route("/packages/:id", get(get_package))
        .route("/packages", get(list_packages))
        .route("/trending", get(get_trending))
        .route("/recent", get(get_recent))
        .route("/recommendations", get(get_recommendations))
        .route("/packages/:id/similar", get(get_similar))
        
        // 分类
        .route("/categories", get(get_categories))
        .route("/categories/:category/packages", get(get_packages_by_category))
        
        // 评分和下载
        .route("/packages/:id/rate", post(rate_package))
        .route("/packages/:id/download", post(download_package))
        
        // 统计
        .route("/statistics", get(get_statistics))
        .route("/packages/:id/statistics", get(get_package_statistics))
        
        // 管理（需要认证）
        .route("/packages", post(create_package))
        .route("/packages/:id", put(update_package))
        .route("/packages/:id", delete(delete_package))
        
        .with_state(marketplace)
}

/// 搜索工具包
async fn search_packages(
    State(marketplace): State<Arc<ToolMarketplace>>,
    Query(params): Query<SearchParams>,
) -> Result<Json<ApiResponse<Vec<crate::discovery::SearchResult>>>, StatusCode> {
    let query = SearchQuery {
        text: params.q.unwrap_or_default(),
        categories: params.category
            .and_then(|c| parse_category(&c))
            .map(|c| vec![c])
            .unwrap_or_default(),
        published_only: params.published_only.unwrap_or(true),
        verified_only: params.verified_only.unwrap_or(false),
        min_rating: params.min_rating,
        limit: params.limit.unwrap_or(20),
        offset: params.offset.unwrap_or(0),
        ..Default::default()
    };
    
    match marketplace.advanced_search(&query).await {
        Ok(results) => Ok(Json(ApiResponse::success(results))),
        Err(e) => {
            tracing::error!("搜索失败: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取工具包详情
async fn get_package(
    State(marketplace): State<Arc<ToolMarketplace>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ToolPackage>>, StatusCode> {
    match marketplace.get_package(id).await {
        Ok(Some(package)) => Ok(Json(ApiResponse::success(package))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("获取工具包失败: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 列出工具包
async fn list_packages(
    State(marketplace): State<Arc<ToolMarketplace>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ApiResponse<Vec<ToolPackage>>>, StatusCode> {
    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(20).min(100); // 限制最大值
    
    match marketplace.list_packages(offset, limit).await {
        Ok(packages) => Ok(Json(ApiResponse::success(packages))),
        Err(e) => {
            tracing::error!("列出工具包失败: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取热门工具
async fn get_trending(
    State(marketplace): State<Arc<ToolMarketplace>>,
    Query(params): Query<SearchParams>,
) -> Result<Json<ApiResponse<Vec<crate::discovery::SearchResult>>>, StatusCode> {
    let category = params.category.and_then(|c| parse_category(&c));
    let limit = params.limit.unwrap_or(20);
    
    match marketplace.get_trending(category, limit).await {
        Ok(results) => Ok(Json(ApiResponse::success(results))),
        Err(e) => {
            tracing::error!("获取热门工具失败: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取最新工具
async fn get_recent(
    State(marketplace): State<Arc<ToolMarketplace>>,
    Query(params): Query<SearchParams>,
) -> Result<Json<ApiResponse<Vec<crate::discovery::SearchResult>>>, StatusCode> {
    let limit = params.limit.unwrap_or(20);
    
    match marketplace.get_recent(limit).await {
        Ok(results) => Ok(Json(ApiResponse::success(results))),
        Err(e) => {
            tracing::error!("获取最新工具失败: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取推荐工具
async fn get_recommendations(
    State(marketplace): State<Arc<ToolMarketplace>>,
) -> Result<Json<ApiResponse<Vec<crate::discovery::SearchResult>>>, StatusCode> {
    // 简化实现，使用默认用户上下文
    let user_context = UserContext::default();
    
    match marketplace.get_recommendations(&user_context).await {
        Ok(results) => Ok(Json(ApiResponse::success(results))),
        Err(e) => {
            tracing::error!("获取推荐工具失败: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取相似工具
async fn get_similar(
    State(marketplace): State<Arc<ToolMarketplace>>,
    Path(id): Path<Uuid>,
    Query(params): Query<SearchParams>,
) -> Result<Json<ApiResponse<Vec<crate::discovery::SearchResult>>>, StatusCode> {
    let limit = params.limit.unwrap_or(10);
    
    match marketplace.get_similar(id, limit).await {
        Ok(results) => Ok(Json(ApiResponse::success(results))),
        Err(e) => {
            tracing::error!("获取相似工具失败: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取所有分类
async fn get_categories() -> Json<ApiResponse<Vec<CategoryInfo>>> {
    let categories = vec![
        CategoryInfo { name: "Web".to_string(), display_name: "网络工具".to_string(), emoji: "🌐".to_string() },
        CategoryInfo { name: "File".to_string(), display_name: "文件操作".to_string(), emoji: "📁".to_string() },
        CategoryInfo { name: "Data".to_string(), display_name: "数据处理".to_string(), emoji: "📊".to_string() },
        CategoryInfo { name: "AI".to_string(), display_name: "AI相关".to_string(), emoji: "🤖".to_string() },
        CategoryInfo { name: "System".to_string(), display_name: "系统工具".to_string(), emoji: "⚙️".to_string() },
        CategoryInfo { name: "Math".to_string(), display_name: "数学计算".to_string(), emoji: "🔢".to_string() },
        CategoryInfo { name: "Crypto".to_string(), display_name: "加密工具".to_string(), emoji: "🔐".to_string() },
        CategoryInfo { name: "Database".to_string(), display_name: "数据库".to_string(), emoji: "🗄️".to_string() },
        CategoryInfo { name: "API".to_string(), display_name: "API工具".to_string(), emoji: "🔌".to_string() },
        CategoryInfo { name: "Utility".to_string(), display_name: "实用工具".to_string(), emoji: "🛠️".to_string() },
    ];
    
    Json(ApiResponse::success(categories))
}

#[derive(Debug, Serialize)]
struct CategoryInfo {
    name: String,
    display_name: String,
    emoji: String,
}

/// 按分类获取工具包
async fn get_packages_by_category(
    State(marketplace): State<Arc<ToolMarketplace>>,
    Path(category_name): Path<String>,
) -> Result<Json<ApiResponse<Vec<ToolPackage>>>, StatusCode> {
    let category = match parse_category(&category_name) {
        Some(cat) => cat,
        None => return Err(StatusCode::BAD_REQUEST),
    };
    
    match marketplace.get_by_category(&category).await {
        Ok(packages) => Ok(Json(ApiResponse::success(packages))),
        Err(e) => {
            tracing::error!("按分类获取工具包失败: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 评分工具包
async fn rate_package(
    State(marketplace): State<Arc<ToolMarketplace>>,
    Path(id): Path<Uuid>,
    Json(request): Json<RatingRequest>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let user_info = UserInfo {
        user_id: request.user_id,
        user_type: UserType::Individual,
        location: None,
        ip_address: None,
        user_agent: None,
    };
    
    match marketplace.rate_package(id, request.rating, &user_info).await {
        Ok(()) => Ok(Json(ApiResponse::success(()))),
        Err(e) => {
            tracing::error!("评分失败: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// 下载工具包
async fn download_package(
    State(marketplace): State<Arc<ToolMarketplace>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let user_info = UserInfo {
        user_id: None,
        user_type: UserType::Individual,
        location: None,
        ip_address: None,
        user_agent: None,
    };
    
    match marketplace.download_package(id, &user_info).await {
        Ok(download_url) => Ok(Json(ApiResponse::success(download_url))),
        Err(e) => {
            tracing::error!("下载失败: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// 获取市场统计
async fn get_statistics(
    State(marketplace): State<Arc<ToolMarketplace>>,
) -> Result<Json<ApiResponse<crate::marketplace::MarketplaceStatistics>>, StatusCode> {
    match marketplace.get_marketplace_statistics().await {
        Ok(stats) => Ok(Json(ApiResponse::success(stats))),
        Err(e) => {
            tracing::error!("获取统计信息失败: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取工具包统计
async fn get_package_statistics(
    State(marketplace): State<Arc<ToolMarketplace>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<crate::analytics::UsageStatistics>>, StatusCode> {
    match marketplace.get_tool_statistics(id).await {
        Ok(stats) => Ok(Json(ApiResponse::success(stats))),
        Err(e) => {
            tracing::error!("获取工具包统计失败: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 创建工具包（管理功能）
async fn create_package(
    State(_marketplace): State<Arc<ToolMarketplace>>,
    Json(_package): Json<ToolPackage>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    // TODO: 实现工具包创建逻辑
    // 需要认证和权限验证
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// 更新工具包（管理功能）
async fn update_package(
    State(_marketplace): State<Arc<ToolMarketplace>>,
    Path(_id): Path<Uuid>,
    Json(_package): Json<ToolPackage>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    // TODO: 实现工具包更新逻辑
    // 需要认证和权限验证
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// 删除工具包（管理功能）
async fn delete_package(
    State(_marketplace): State<Arc<ToolMarketplace>>,
    Path(_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    // TODO: 实现工具包删除逻辑
    // 需要认证和权限验证
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// 解析分类字符串
fn parse_category(category_str: &str) -> Option<ToolCategory> {
    match category_str.to_lowercase().as_str() {
        "web" => Some(ToolCategory::Web),
        "file" => Some(ToolCategory::File),
        "data" => Some(ToolCategory::Data),
        "ai" => Some(ToolCategory::AI),
        "system" => Some(ToolCategory::System),
        "math" => Some(ToolCategory::Math),
        "crypto" => Some(ToolCategory::Crypto),
        "database" => Some(ToolCategory::Database),
        "api" => Some(ToolCategory::API),
        "utility" => Some(ToolCategory::Utility),
        "custom" => Some(ToolCategory::Custom),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_category() {
        assert_eq!(parse_category("web"), Some(ToolCategory::Web));
        assert_eq!(parse_category("Web"), Some(ToolCategory::Web));
        assert_eq!(parse_category("WEB"), Some(ToolCategory::Web));
        assert_eq!(parse_category("invalid"), None);
    }
    
    #[test]
    fn test_api_response() {
        let success_response = ApiResponse::success("test data");
        assert!(success_response.success);
        assert_eq!(success_response.data, Some("test data"));
        assert!(success_response.error.is_none());
        
        let error_response: ApiResponse<String> = ApiResponse::error("test error".to_string());
        assert!(!error_response.success);
        assert!(error_response.data.is_none());
        assert_eq!(error_response.error, Some("test error".to_string()));
    }
}
