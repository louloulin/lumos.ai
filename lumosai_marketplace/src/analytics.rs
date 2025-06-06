//! 使用分析模块实现

use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::ToolCategory;
use crate::error::{MarketplaceError, Result};

/// 使用统计
#[derive(Debug, Clone)]
pub struct UsageStatistics {
    /// 工具包ID
    pub package_id: Uuid,
    
    /// 总下载次数
    pub total_downloads: u64,
    
    /// 日下载次数
    pub daily_downloads: u64,
    
    /// 周下载次数
    pub weekly_downloads: u64,
    
    /// 月下载次数
    pub monthly_downloads: u64,
    
    /// 平均评分
    pub average_rating: f64,
    
    /// 评分数量
    pub rating_count: u32,
    
    /// 使用时长统计
    pub usage_duration_stats: UsageDurationStats,
    
    /// 地理分布
    pub geographic_distribution: HashMap<String, u64>,
    
    /// 用户类型分布
    pub user_type_distribution: HashMap<UserType, u64>,
}

/// 使用时长统计
#[derive(Debug, Clone)]
pub struct UsageDurationStats {
    /// 平均使用时长（秒）
    pub average_duration_seconds: f64,
    
    /// 中位数使用时长（秒）
    pub median_duration_seconds: f64,
    
    /// 最大使用时长（秒）
    pub max_duration_seconds: u64,
    
    /// 最小使用时长（秒）
    pub min_duration_seconds: u64,
}

/// 用户类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UserType {
    /// 个人开发者
    Individual,
    /// 小团队
    SmallTeam,
    /// 企业
    Enterprise,
    /// 学术机构
    Academic,
    /// 开源项目
    OpenSource,
}

/// 分析报告
#[derive(Debug, Clone)]
pub struct AnalyticsReport {
    /// 报告生成时间
    pub generated_at: DateTime<Utc>,
    
    /// 报告时间范围
    pub time_range: TimeRange,
    
    /// 总体统计
    pub overall_stats: OverallStats,
    
    /// 热门工具
    pub top_tools: Vec<ToolStats>,
    
    /// 分类统计
    pub category_stats: HashMap<ToolCategory, CategoryStats>,
    
    /// 趋势数据
    pub trends: TrendData,
}

/// 时间范围
#[derive(Debug, Clone)]
pub struct TimeRange {
    /// 开始时间
    pub start: DateTime<Utc>,
    
    /// 结束时间
    pub end: DateTime<Utc>,
}

/// 总体统计
#[derive(Debug, Clone)]
pub struct OverallStats {
    /// 总工具数
    pub total_tools: u64,
    
    /// 总下载次数
    pub total_downloads: u64,
    
    /// 活跃用户数
    pub active_users: u64,
    
    /// 新用户数
    pub new_users: u64,
    
    /// 平均评分
    pub average_rating: f64,
}

/// 工具统计
#[derive(Debug, Clone)]
pub struct ToolStats {
    /// 工具包ID
    pub package_id: Uuid,
    
    /// 工具名称
    pub name: String,
    
    /// 下载次数
    pub downloads: u64,
    
    /// 评分
    pub rating: f64,
    
    /// 增长率
    pub growth_rate: f64,
}

/// 分类统计
#[derive(Debug, Clone)]
pub struct CategoryStats {
    /// 工具数量
    pub tool_count: u64,
    
    /// 总下载次数
    pub total_downloads: u64,
    
    /// 平均评分
    pub average_rating: f64,
    
    /// 增长率
    pub growth_rate: f64,
}

/// 趋势数据
#[derive(Debug, Clone)]
pub struct TrendData {
    /// 下载趋势
    pub download_trend: Vec<TrendPoint>,
    
    /// 评分趋势
    pub rating_trend: Vec<TrendPoint>,
    
    /// 用户增长趋势
    pub user_growth_trend: Vec<TrendPoint>,
}

/// 趋势点
#[derive(Debug, Clone)]
pub struct TrendPoint {
    /// 时间点
    pub timestamp: DateTime<Utc>,
    
    /// 数值
    pub value: f64,
}

/// 使用分析trait
#[async_trait]
pub trait UsageAnalytics: Send + Sync {
    /// 记录下载事件
    async fn record_download(&self, package_id: Uuid, user_info: &UserInfo) -> Result<()>;
    
    /// 记录使用事件
    async fn record_usage(&self, package_id: Uuid, usage_info: &UsageInfo) -> Result<()>;
    
    /// 记录评分事件
    async fn record_rating(&self, package_id: Uuid, rating: f64, user_info: &UserInfo) -> Result<()>;
    
    /// 获取工具使用统计
    async fn get_tool_statistics(&self, package_id: Uuid) -> Result<UsageStatistics>;
    
    /// 生成分析报告
    async fn generate_report(&self, time_range: TimeRange) -> Result<AnalyticsReport>;
    
    /// 获取热门工具
    async fn get_trending_tools(&self, limit: usize, time_range: TimeRange) -> Result<Vec<ToolStats>>;
    
    /// 获取分类统计
    async fn get_category_statistics(&self, time_range: TimeRange) -> Result<HashMap<ToolCategory, CategoryStats>>;
}

/// 用户信息
#[derive(Debug, Clone)]
pub struct UserInfo {
    /// 用户ID
    pub user_id: Option<String>,
    
    /// 用户类型
    pub user_type: UserType,
    
    /// 地理位置
    pub location: Option<String>,
    
    /// IP地址
    pub ip_address: Option<String>,
    
    /// 用户代理
    pub user_agent: Option<String>,
}

/// 使用信息
#[derive(Debug, Clone)]
pub struct UsageInfo {
    /// 用户信息
    pub user_info: UserInfo,
    
    /// 使用时长（秒）
    pub duration_seconds: u64,
    
    /// 使用开始时间
    pub start_time: DateTime<Utc>,
    
    /// 使用结束时间
    pub end_time: DateTime<Utc>,
    
    /// 使用上下文
    pub context: HashMap<String, String>,
}

/// 默认使用分析实现
pub struct DefaultUsageAnalytics {
    // 简化实现，实际应该使用数据库存储
    download_events: std::sync::Arc<std::sync::Mutex<Vec<DownloadEvent>>>,
    usage_events: std::sync::Arc<std::sync::Mutex<Vec<UsageEvent>>>,
    rating_events: std::sync::Arc<std::sync::Mutex<Vec<RatingEvent>>>,
}

#[derive(Debug, Clone)]
struct DownloadEvent {
    package_id: Uuid,
    user_info: UserInfo,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct UsageEvent {
    package_id: Uuid,
    usage_info: UsageInfo,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct RatingEvent {
    package_id: Uuid,
    rating: f64,
    user_info: UserInfo,
    timestamp: DateTime<Utc>,
}

impl DefaultUsageAnalytics {
    /// 创建新的默认使用分析
    pub fn new() -> Self {
        Self {
            download_events: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            usage_events: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            rating_events: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl UsageAnalytics for DefaultUsageAnalytics {
    async fn record_download(&self, package_id: Uuid, user_info: &UserInfo) -> Result<()> {
        let event = DownloadEvent {
            package_id,
            user_info: user_info.clone(),
            timestamp: Utc::now(),
        };
        
        let mut events = self.download_events.lock()
            .map_err(|_| MarketplaceError::Internal("Failed to acquire lock".to_string()))?;
        events.push(event);
        
        Ok(())
    }
    
    async fn record_usage(&self, package_id: Uuid, usage_info: &UsageInfo) -> Result<()> {
        let event = UsageEvent {
            package_id,
            usage_info: usage_info.clone(),
            timestamp: Utc::now(),
        };
        
        let mut events = self.usage_events.lock()
            .map_err(|_| MarketplaceError::Internal("Failed to acquire lock".to_string()))?;
        events.push(event);
        
        Ok(())
    }
    
    async fn record_rating(&self, package_id: Uuid, rating: f64, user_info: &UserInfo) -> Result<()> {
        let event = RatingEvent {
            package_id,
            rating,
            user_info: user_info.clone(),
            timestamp: Utc::now(),
        };
        
        let mut events = self.rating_events.lock()
            .map_err(|_| MarketplaceError::Internal("Failed to acquire lock".to_string()))?;
        events.push(event);
        
        Ok(())
    }
    
    async fn get_tool_statistics(&self, package_id: Uuid) -> Result<UsageStatistics> {
        let download_events = self.download_events.lock()
            .map_err(|_| MarketplaceError::Internal("Failed to acquire lock".to_string()))?;
        let usage_events = self.usage_events.lock()
            .map_err(|_| MarketplaceError::Internal("Failed to acquire lock".to_string()))?;
        let rating_events = self.rating_events.lock()
            .map_err(|_| MarketplaceError::Internal("Failed to acquire lock".to_string()))?;
        
        // 计算下载统计
        let total_downloads = download_events.iter()
            .filter(|e| e.package_id == package_id)
            .count() as u64;
        
        let now = Utc::now();
        let daily_downloads = download_events.iter()
            .filter(|e| e.package_id == package_id && (now - e.timestamp) <= Duration::days(1))
            .count() as u64;
        
        let weekly_downloads = download_events.iter()
            .filter(|e| e.package_id == package_id && (now - e.timestamp) <= Duration::weeks(1))
            .count() as u64;
        
        let monthly_downloads = download_events.iter()
            .filter(|e| e.package_id == package_id && (now - e.timestamp) <= Duration::days(30))
            .count() as u64;
        
        // 计算评分统计
        let package_ratings: Vec<f64> = rating_events.iter()
            .filter(|e| e.package_id == package_id)
            .map(|e| e.rating)
            .collect();
        
        let average_rating = if package_ratings.is_empty() {
            0.0
        } else {
            package_ratings.iter().sum::<f64>() / package_ratings.len() as f64
        };
        
        let rating_count = package_ratings.len() as u32;
        
        // 计算使用时长统计
        let durations: Vec<u64> = usage_events.iter()
            .filter(|e| e.package_id == package_id)
            .map(|e| e.usage_info.duration_seconds)
            .collect();
        
        let usage_duration_stats = if durations.is_empty() {
            UsageDurationStats {
                average_duration_seconds: 0.0,
                median_duration_seconds: 0.0,
                max_duration_seconds: 0,
                min_duration_seconds: 0,
            }
        } else {
            let average = durations.iter().sum::<u64>() as f64 / durations.len() as f64;
            let mut sorted_durations = durations.clone();
            sorted_durations.sort();
            let median = sorted_durations[sorted_durations.len() / 2] as f64;
            let max = *durations.iter().max().unwrap();
            let min = *durations.iter().min().unwrap();
            
            UsageDurationStats {
                average_duration_seconds: average,
                median_duration_seconds: median,
                max_duration_seconds: max,
                min_duration_seconds: min,
            }
        };
        
        Ok(UsageStatistics {
            package_id,
            total_downloads,
            daily_downloads,
            weekly_downloads,
            monthly_downloads,
            average_rating,
            rating_count,
            usage_duration_stats,
            geographic_distribution: HashMap::new(), // 简化实现
            user_type_distribution: HashMap::new(), // 简化实现
        })
    }
    
    async fn generate_report(&self, time_range: TimeRange) -> Result<AnalyticsReport> {
        // 简化的报告生成实现
        Ok(AnalyticsReport {
            generated_at: Utc::now(),
            time_range,
            overall_stats: OverallStats {
                total_tools: 0,
                total_downloads: 0,
                active_users: 0,
                new_users: 0,
                average_rating: 0.0,
            },
            top_tools: Vec::new(),
            category_stats: HashMap::new(),
            trends: TrendData {
                download_trend: Vec::new(),
                rating_trend: Vec::new(),
                user_growth_trend: Vec::new(),
            },
        })
    }
    
    async fn get_trending_tools(&self, _limit: usize, _time_range: TimeRange) -> Result<Vec<ToolStats>> {
        // 简化实现
        Ok(Vec::new())
    }
    
    async fn get_category_statistics(&self, _time_range: TimeRange) -> Result<HashMap<ToolCategory, CategoryStats>> {
        // 简化实现
        Ok(HashMap::new())
    }
}

impl Default for DefaultUsageAnalytics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_analytics_creation() {
        let analytics = DefaultUsageAnalytics::new();
        
        let user_info = UserInfo {
            user_id: Some("test_user".to_string()),
            user_type: UserType::Individual,
            location: Some("US".to_string()),
            ip_address: None,
            user_agent: None,
        };
        
        let package_id = Uuid::new_v4();
        
        // 记录下载事件
        analytics.record_download(package_id, &user_info).await.unwrap();
        
        // 获取统计信息
        let stats = analytics.get_tool_statistics(package_id).await.unwrap();
        assert_eq!(stats.total_downloads, 1);
    }
    
    #[tokio::test]
    async fn test_usage_recording() {
        let analytics = DefaultUsageAnalytics::new();
        let package_id = Uuid::new_v4();
        
        let user_info = UserInfo {
            user_id: Some("test_user".to_string()),
            user_type: UserType::Individual,
            location: Some("US".to_string()),
            ip_address: None,
            user_agent: None,
        };
        
        let usage_info = UsageInfo {
            user_info: user_info.clone(),
            duration_seconds: 300,
            start_time: Utc::now() - Duration::minutes(5),
            end_time: Utc::now(),
            context: HashMap::new(),
        };
        
        analytics.record_usage(package_id, &usage_info).await.unwrap();
        
        let stats = analytics.get_tool_statistics(package_id).await.unwrap();
        assert_eq!(stats.usage_duration_stats.average_duration_seconds, 300.0);
    }
    
    #[tokio::test]
    async fn test_rating_recording() {
        let analytics = DefaultUsageAnalytics::new();
        let package_id = Uuid::new_v4();
        
        let user_info = UserInfo {
            user_id: Some("test_user".to_string()),
            user_type: UserType::Individual,
            location: Some("US".to_string()),
            ip_address: None,
            user_agent: None,
        };
        
        analytics.record_rating(package_id, 4.5, &user_info).await.unwrap();
        analytics.record_rating(package_id, 5.0, &user_info).await.unwrap();
        
        let stats = analytics.get_tool_statistics(package_id).await.unwrap();
        assert_eq!(stats.rating_count, 2);
        assert_eq!(stats.average_rating, 4.75);
    }
}
