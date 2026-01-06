//! ENGRAM Memory System - 基于ENGRAM论文的三种记忆类型实现
//!
//! ENGRAM论文的核心洞察：简单架构可以超越复杂系统
//! 实现Episodic（事件记忆）、Semantic（事实记忆）、Procedural（过程记忆）三种记忆类型

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, Utc};

use crate::error::{Error, Result};
use crate::llm::{Message, LlmProvider};

/// ENGRAM三种记忆类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryType {
    /// Episodic Memory - 事件记忆
    /// 存储具体的事件、经历和时间序列数据
    /// 回答："What happened?"
    Episodic,

    /// Semantic Memory - 事实记忆
    /// 存储事实、概念、规则和一般知识
    /// 回答："What is known?"
    Semantic,

    /// Procedural Memory - 过程记忆
    /// 存储技能、程序和操作序列
    /// 回答："How to do?"
    Procedural,
}

/// 记忆条目 - ENGRAM系统的基础数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// 唯一标识
    pub id: String,

    /// 记忆类型
    pub memory_type: MemoryType,

    /// 记忆内容
    pub content: MemoryContent,

    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,

    /// 重要性评分 (0.0-1.0)
    pub importance: f32,

    /// 访问频率
    pub access_count: u64,

    /// 最后访问时间
    pub last_accessed: DateTime<Utc>,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 版本号
    pub version: u64,

    /// 关联的上下文ID列表
    pub context_ids: Vec<String>,

    /// 向量表示（用于语义搜索）
    pub embeddings: Option<Vec<f32>>,
}

/// 记忆内容枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryContent {
    /// 文本内容
    Text(String),

    /// 结构化数据
    Structured(serde_json::Value),

    /// 事件序列
    EventSequence(Vec<Event>),

    /// 技能/程序定义
    Skill(SkillDefinition),
}

/// 事件定义（用于Episodic Memory）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// 事件描述
    pub description: String,

    /// 事件时间戳
    pub timestamp: DateTime<Utc>,

    /// 事件参与者
    pub participants: Vec<String>,

    /// 事件结果/影响
    pub outcome: Option<String>,

    /// 情感评分 (-1.0到1.0)
    pub emotional_impact: Option<f32>,
}

/// 技能定义（用于Procedural Memory）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    /// 技能名称
    pub name: String,

    /// 技能描述
    pub description: String,

    /// 执行步骤
    pub steps: Vec<String>,

    /// 前置条件
    pub prerequisites: Vec<String>,

    /// 预期结果
    pub expected_outcome: String,

    /// 熟练度 (0.0-1.0)
    pub proficiency: f32,
}

/// ENGRAM记忆系统trait
#[async_trait]
pub trait EngramMemorySystem: Send + Sync {
    /// 存储记忆条目
    async fn store_memory(&self, entry: MemoryEntry) -> Result<String>;

    /// 检索记忆（根据类型和查询）
    async fn retrieve_memory(&self, query: &MemoryQuery) -> Result<Vec<MemoryEntry>>;

    /// 更新记忆重要性
    async fn update_importance(&self, memory_id: &str, new_importance: f32) -> Result<()>;

    /// 压缩记忆（减少存储空间）
    async fn compress_memory(&self, compression_config: &CompressionConfig) -> Result<CompressionResult>;

    /// 合并相似记忆
    async fn consolidate_memory(&self, consolidation_config: &ConsolidationConfig) -> Result<ConsolidationResult>;

    /// 获取记忆统计信息
    async fn get_memory_stats(&self) -> Result<MemoryStats>;

    /// 遗忘机制（移除低重要性记忆）
    async fn forget_memory(&self, forget_config: &ForgetConfig) -> Result<ForgetResult>;
}

/// 记忆查询
#[derive(Debug, Clone)]
pub struct MemoryQuery {
    /// 记忆类型过滤
    pub memory_types: Option<Vec<MemoryType>>,

    /// 文本查询（用于语义搜索）
    pub text_query: Option<String>,

    /// 时间范围
    pub time_range: Option<TimeRange>,

    /// 重要性阈值
    pub min_importance: Option<f32>,

    /// 上下文ID过滤
    pub context_ids: Vec<String>,

    /// 排序方式
    pub sort_by: MemorySortBy,

    /// 限制返回数量
    pub limit: Option<usize>,
}

/// 时间范围
#[derive(Debug, Clone)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// 记忆排序方式
#[derive(Debug, Clone)]
pub enum MemorySortBy {
    /// 按重要性降序
    Importance,

    /// 按访问时间降序
    LastAccessed,

    /// 按创建时间降序
    CreatedAt,

    /// 按访问频率降序
    AccessFrequency,
}

/// 压缩配置
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// 压缩阈值（重要性低于此值的内容可能被压缩）
    pub compression_threshold: f32,

    /// 是否保留原始内容
    pub preserve_original: bool,

    /// 压缩算法
    pub algorithm: CompressionAlgorithm,
}

/// 压缩算法
#[derive(Debug, Clone)]
pub enum CompressionAlgorithm {
    /// 摘要压缩（生成摘要）
    Summarization,

    /// 合并相似内容
    Deduplication,

    /// 时间聚合
    TemporalAggregation,
}

/// 压缩结果
#[derive(Debug, Clone)]
pub struct CompressionResult {
    /// 压缩的记忆条目数量
    pub compressed_count: usize,

    /// 节省的空间（字节）
    pub space_saved: usize,

    /// 压缩耗时
    pub duration_ms: u64,
}

/// 巩固配置
#[derive(Debug, Clone)]
pub struct ConsolidationConfig {
    /// 相似度阈值
    pub similarity_threshold: f32,

    /// 是否合并Episodic记忆
    pub consolidate_episodic: bool,

    /// 是否合并Semantic记忆
    pub consolidate_semantic: bool,

    /// 是否合并Procedural记忆
    pub consolidate_procedural: bool,
}

/// 巩固结果
#[derive(Debug, Clone)]
pub struct ConsolidationResult {
    /// 合并的记忆组数量
    pub consolidated_groups: usize,

    /// 减少的记忆条目数量
    pub entries_reduced: usize,

    /// 新创建的巩固记忆数量
    pub new_consolidated_memories: usize,
}

/// 遗忘配置
#[derive(Debug, Clone)]
pub struct ForgetConfig {
    /// 重要性阈值（低于此值的记忆可能被遗忘）
    pub importance_threshold: f32,

    /// 时间阈值（超过此时间的低重要性记忆可能被遗忘）
    pub time_threshold_days: u64,

    /// 是否只遗忘Episodic记忆
    pub only_episodic: bool,

    /// 是否预览模式（不实际删除）
    pub preview_only: bool,
}

/// 遗忘结果
#[derive(Debug, Clone)]
pub struct ForgetResult {
    /// 被遗忘的记忆条目数量
    pub forgotten_count: usize,

    /// 释放的空间（字节）
    pub space_freed: usize,

    /// 预览模式下的候选遗忘数量
    pub preview_candidates: Option<usize>,
}

/// 记忆统计
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// 总记忆条目数
    pub total_entries: usize,

    /// 按类型统计
    pub by_type: HashMap<MemoryType, usize>,

    /// 平均重要性
    pub avg_importance: f32,

    /// 总存储大小（字节）
    pub total_size_bytes: usize,

    /// 最后压缩时间
    pub last_compression: Option<DateTime<Utc>>,

    /// 健康评分 (0.0-1.0)
    pub health_score: f32,
}

/// ENGRAM记忆管理器 - 协调三种记忆类型的管理
pub struct EngramMemoryManager {
    episodic_memory: Arc<dyn EngramMemorySystem>,
    semantic_memory: Arc<dyn EngramMemorySystem>,
    procedural_memory: Arc<dyn EngramMemorySystem>,
    llm_provider: Arc<dyn LlmProvider>,
    config: EngramConfig,
}

/// ENGRAM配置
#[derive(Debug, Clone)]
pub struct EngramConfig {
    /// 是否启用自动压缩
    pub auto_compression: bool,

    /// 压缩间隔（小时）
    pub compression_interval_hours: u64,

    /// 是否启用自动巩固
    pub auto_consolidation: bool,

    /// 巩固间隔（小时）
    pub consolidation_interval_hours: u64,

    /// 是否启用遗忘机制
    pub enable_forgetting: bool,

    /// 遗忘检查间隔（小时）
    pub forgetting_check_hours: u64,

    /// LLM辅助记忆处理
    pub enable_llm_assistance: bool,
}

impl EngramMemoryManager {
    pub fn new(
        episodic: Arc<dyn EngramMemorySystem>,
        semantic: Arc<dyn EngramMemorySystem>,
        procedural: Arc<dyn EngramMemorySystem>,
        llm_provider: Arc<dyn LlmProvider>,
        config: EngramConfig,
    ) -> Self {
        Self {
            episodic_memory: episodic,
            semantic_memory: semantic,
            procedural_memory: procedural,
            llm_provider,
            config,
        }
    }

    /// 智能存储 - 根据内容自动分类到合适的记忆类型
    pub async fn intelligent_store(&self, content: &str, context: &MemoryContext) -> Result<String> {
        // 使用LLM分析内容并分类
        let memory_type = self.classify_content(content, context).await?;

        // 创建记忆条目
        let entry = self.create_memory_entry(content, memory_type.clone(), context).await?;

        // 存储到对应类型的记忆系统
        match memory_type {
            MemoryType::Episodic => self.episodic_memory.store_memory(entry).await,
            MemoryType::Semantic => self.semantic_memory.store_memory(entry).await,
            MemoryType::Procedural => self.procedural_memory.store_memory(entry).await,
        }
    }

    /// 智能检索 - 从所有记忆类型中检索相关信息
    pub async fn intelligent_retrieve(&self, query: &str, context: &MemoryContext) -> Result<Vec<MemoryEntry>> {
        // 构建查询
        let memory_query = MemoryQuery {
            memory_types: None, // 从所有类型检索
            text_query: Some(query.to_string()),
            time_range: context.time_range.clone(),
            min_importance: Some(0.3), // 只返回重要性足够的内容
            context_ids: context.context_ids.clone().unwrap_or_default(),
            sort_by: MemorySortBy::Importance,
            limit: Some(20),
        };

        // 并行从三种记忆系统检索
        let (episodic_results, semantic_results, procedural_results) = tokio::try_join!(
            self.episodic_memory.retrieve_memory(&memory_query),
            self.semantic_memory.retrieve_memory(&memory_query),
            self.procedural_memory.retrieve_memory(&memory_query)
        )?;

        // 合并和排序结果
        let mut all_results = Vec::new();
        all_results.extend(episodic_results);
        all_results.extend(semantic_results);
        all_results.extend(procedural_results);

        // 按重要性排序并限制数量
        all_results.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal));
        all_results.truncate(memory_query.limit.unwrap_or(20));

        Ok(all_results)
    }

    /// 记忆巩固 - 合并相似记忆
    pub async fn consolidate_memories(&self) -> Result<ConsolidationResult> {
        let config = ConsolidationConfig {
            similarity_threshold: 0.8,
            consolidate_episodic: true,
            consolidate_semantic: true,
            consolidate_procedural: false, // 过程记忆通常不需要合并
        };

        let (episodic_result, semantic_result, _) = tokio::try_join!(
            self.episodic_memory.consolidate_memory(&config),
            self.semantic_memory.consolidate_memory(&config),
            async { Ok(ConsolidationResult {
                consolidated_groups: 0,
                entries_reduced: 0,
                new_consolidated_memories: 0,
            }) }
        )?;

        Ok(ConsolidationResult {
            consolidated_groups: episodic_result.consolidated_groups + semantic_result.consolidated_groups,
            entries_reduced: episodic_result.entries_reduced + semantic_result.entries_reduced,
            new_consolidated_memories: episodic_result.new_consolidated_memories + semantic_result.new_consolidated_memories,
        })
    }

    /// 使用LLM分析内容并分类
    async fn classify_content(&self, content: &str, context: &MemoryContext) -> Result<MemoryType> {
        if !self.config.enable_llm_assistance {
            // 简单启发式分类
            return self.heuristic_classification(content, context);
        }

        // 使用LLM进行智能分类
        let prompt = format!(
            "Analyze the following content and classify it into one of three memory types:\n\
             - Episodic: Personal experiences, events, specific occurrences ('What happened?')\n\
             - Semantic: Facts, concepts, general knowledge ('What is known?')\n\
             - Procedural: Skills, procedures, how-to knowledge ('How to do?')\n\n\
             Content: {}\n\n\
             Context: {:?}\n\n\
             Respond with only the memory type name.",
            content, context
        );

        let response = self.llm_provider.generate(&prompt, &Default::default()).await?;

        match response.to_lowercase().as_str() {
            "episodic" => Ok(MemoryType::Episodic),
            "semantic" => Ok(MemoryType::Semantic),
            "procedural" => Ok(MemoryType::Procedural),
            _ => self.heuristic_classification(content, context),
        }
    }

    /// 启发式内容分类
    fn heuristic_classification(&self, content: &str, _context: &MemoryContext) -> Result<MemoryType> {
        let content_lower = content.to_lowercase();

        // 检查是否包含时间相关词汇（可能为Episodic）
        let episodic_keywords = ["yesterday", "today", "tomorrow", "last week", "next month", "happened", "occurred", "experienced"];
        if episodic_keywords.iter().any(|&keyword| content_lower.contains(keyword)) {
            return Ok(MemoryType::Episodic);
        }

        // 检查是否包含程序/步骤相关词汇（可能为Procedural）
        let procedural_keywords = ["how to", "steps", "procedure", "method", "process", "algorithm", "function"];
        if procedural_keywords.iter().any(|&keyword| content_lower.contains(keyword)) {
            return Ok(MemoryType::Procedural);
        }

        // 检查是否包含事实/概念相关词汇（可能为Semantic）
        let semantic_keywords = ["is a", "are", "means", "definition", "concept", "fact", "knowledge"];
        if semantic_keywords.iter().any(|&keyword| content_lower.contains(keyword)) {
            return Ok(MemoryType::Semantic);
        }

        // 默认分类为Episodic
        Ok(MemoryType::Episodic)
    }

    /// 创建记忆条目
    async fn create_memory_entry(&self, content: &str, memory_type: MemoryType, context: &MemoryContext) -> Result<MemoryEntry> {
        let importance = self.calculate_importance(content, &memory_type, context).await?;

        let memory_content = match memory_type {
            MemoryType::Episodic => MemoryContent::Text(content.to_string()),
            MemoryType::Semantic => MemoryContent::Text(content.to_string()),
            MemoryType::Procedural => {
                // 尝试解析为技能定义
                if self.is_skill_definition(content) {
                    MemoryContent::Skill(SkillDefinition {
                        name: "Unknown Skill".to_string(),
                        description: content.to_string(),
                        steps: vec![],
                        prerequisites: vec![],
                        expected_outcome: "Skill execution".to_string(),
                        proficiency: 0.5,
                    })
                } else {
                    MemoryContent::Text(content.to_string())
                }
            }
        };

        Ok(MemoryEntry {
            id: format!("mem_{}", uuid::Uuid::new_v4()),
            memory_type,
            content: memory_content,
            metadata: context.metadata.clone(),
            importance,
            access_count: 0,
            last_accessed: Utc::now(),
            created_at: Utc::now(),
            version: 1,
            context_ids: context.context_ids.clone().unwrap_or_default(),
            embeddings: None, // TODO: 生成向量表示
        })
    }

    /// 计算记忆重要性
    async fn calculate_importance(&self, content: &str, memory_type: &MemoryType, context: &MemoryContext) -> Result<f32> {
        // 基础重要性计算
        let mut importance = 0.5;

        // 根据记忆类型调整
        match memory_type {
            MemoryType::Episodic => {
                // Episodic记忆通常较重要，因为包含个人经历
                importance += 0.2;
            }
            MemoryType::Semantic => {
                // Semantic记忆很重要，但可能更通用
                importance += 0.1;
            }
            MemoryType::Procedural => {
                // Procedural记忆非常重要，因为包含可操作的技能
                importance += 0.3;
            }
        }

        // 根据内容长度调整（更长的内容通常更重要）
        let length_bonus = (content.len() as f32 / 1000.0).min(0.2);
        importance += length_bonus;

        // 限制在0.0-1.0范围内
        Ok(importance.min(1.0).max(0.0))
    }

    /// 检查是否为技能定义
    fn is_skill_definition(&self, content: &str) -> bool {
        let content_lower = content.to_lowercase();
        let skill_indicators = ["step", "procedure", "method", "how to", "algorithm", "function", "process"];

        skill_indicators.iter().any(|&indicator| content_lower.contains(indicator))
    }
}

/// 记忆上下文
#[derive(Debug, Clone, Default)]
pub struct MemoryContext {
    pub context_ids: Option<Vec<String>>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub time_range: Option<TimeRange>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
}

#[cfg(test)]
mod engram_unit_tests {
    use super::*;

    #[test]
    fn test_engram_memory_types() {
        // 测试三种记忆类型的定义
        let episodic = EngramMemoryType::Episodic;
        let semantic = EngramMemoryType::Semantic;
        let procedural = EngramMemoryType::Procedural;

        // 测试序列化
        let serialized = serde_json::to_string(&episodic).unwrap();
        assert_eq!(serialized, "\"Episodic\"");

        let deserialized: EngramMemoryType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, EngramMemoryType::Episodic);
    }

    #[test]
    fn test_memory_entry_creation() {
        let entry = EngramMemoryEntry {
            id: "test_id".to_string(),
            memory_type: EngramMemoryType::Semantic,
            content: MemoryContent::Text("Test content".to_string()),
            metadata: HashMap::new(),
            importance: 0.8,
            access_count: 0,
            last_accessed: Utc::now(),
            created_at: Utc::now(),
            version: 1,
            context_ids: vec!["ctx1".to_string()],
            embeddings: Some(vec![0.1, 0.2, 0.3]),
        };

        assert_eq!(entry.memory_type, EngramMemoryType::Semantic);
        assert_eq!(entry.importance, 0.8);
        assert_eq!(entry.content, MemoryContent::Text("Test content".to_string()));
    }

    #[test]
    fn test_skill_definition() {
        let skill = SkillDefinition {
            name: "Bake Cake".to_string(),
            description: "How to bake a delicious chocolate cake".to_string(),
            steps: vec![
                "Preheat oven to 350°F".to_string(),
                "Mix dry ingredients".to_string(),
                "Add wet ingredients".to_string(),
                "Bake for 30 minutes".to_string(),
            ],
            prerequisites: vec!["Oven".to_string(), "Baking pan".to_string()],
            expected_outcome: "A delicious chocolate cake".to_string(),
            proficiency: 0.7,
        };

        assert_eq!(skill.name, "Bake Cake");
        assert_eq!(skill.steps.len(), 4);
        assert_eq!(skill.proficiency, 0.7);
    }

    #[test]
    fn test_event_definition() {
        let event = Event {
            description: "Attended AI conference".to_string(),
            timestamp: Utc::now(),
            participants: vec!["Alice".to_string(), "Bob".to_string()],
            outcome: Some("Learned about latest AI trends".to_string()),
            emotional_impact: Some(0.8),
        };

        assert_eq!(event.description, "Attended AI conference");
        assert_eq!(event.participants.len(), 2);
        assert_eq!(event.emotional_impact, Some(0.8));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_type_serialization() {
        let memory_type = MemoryType::Episodic;
        let serialized = serde_json::to_string(&memory_type).unwrap();
        let deserialized: MemoryType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(memory_type, deserialized);
    }

    #[test]
    fn test_memory_entry_creation() {
        let entry = MemoryEntry {
            id: "test_id".to_string(),
            memory_type: MemoryType::Semantic,
            content: MemoryContent::Text("Test content".to_string()),
            metadata: HashMap::new(),
            importance: 0.8,
            access_count: 0,
            last_accessed: Utc::now(),
            created_at: Utc::now(),
            version: 1,
            context_ids: vec!["ctx1".to_string()],
            embeddings: None,
        };

        assert_eq!(entry.memory_type, MemoryType::Semantic);
        assert_eq!(entry.importance, 0.8);
    }

    #[test]
    fn test_heuristic_classification() {
        let manager = EngramMemoryManager::new(
            // Mock implementations for testing
            Arc::new(MockEngramMemory::new()),
            Arc::new(MockEngramMemory::new()),
            Arc::new(MockEngramMemory::new()),
            Arc::new(MockLlmProvider::new(vec![])),
            EngramConfig {
                auto_compression: false,
                compression_interval_hours: 24,
                auto_consolidation: false,
                consolidation_interval_hours: 24,
                enable_forgetting: false,
                forgetting_check_hours: 24,
                enable_llm_assistance: false,
            },
        );

        let context = MemoryContext::default();

        // 测试Episodic分类
        let episodic_result = manager.heuristic_classification("Yesterday I went to the store", &context).unwrap();
        assert_eq!(episodic_result, MemoryType::Episodic);

        // 测试Procedural分类
        let procedural_result = manager.heuristic_classification("How to bake a cake: step 1, step 2", &context).unwrap();
        assert_eq!(procedural_result, MemoryType::Procedural);

        // 测试Semantic分类
        let semantic_result = manager.heuristic_classification("Rust is a programming language", &context).unwrap();
        assert_eq!(semantic_result, MemoryType::Semantic);
    }

    // Mock implementations for testing
    struct MockEngramMemory;

    impl MockEngramMemory {
        fn new() -> Self {
            Self
        }
    }

    #[async_trait]
    impl EngramMemorySystem for MockEngramMemory {
        async fn store_memory(&self, _entry: MemoryEntry) -> Result<String> {
            Ok("mock_id".to_string())
        }

        async fn retrieve_memory(&self, _query: &MemoryQuery) -> Result<Vec<MemoryEntry>> {
            Ok(vec![])
        }

        async fn update_importance(&self, _memory_id: &str, _new_importance: f32) -> Result<()> {
            Ok(())
        }

        async fn compress_memory(&self, _config: &CompressionConfig) -> Result<CompressionResult> {
            Ok(CompressionResult {
                compressed_count: 0,
                space_saved: 0,
                duration_ms: 0,
            })
        }

        async fn consolidate_memory(&self, _config: &ConsolidationConfig) -> Result<ConsolidationResult> {
            Ok(ConsolidationResult {
                consolidated_groups: 0,
                entries_reduced: 0,
                new_consolidated_memories: 0,
            })
        }

        async fn get_memory_stats(&self) -> Result<MemoryStats> {
            Ok(MemoryStats {
                total_entries: 0,
                by_type: HashMap::new(),
                avg_importance: 0.0,
                total_size_bytes: 0,
                last_compression: None,
                health_score: 1.0,
            })
        }

        async fn forget_memory(&self, _config: &ForgetConfig) -> Result<ForgetResult> {
            Ok(ForgetResult {
                forgotten_count: 0,
                space_freed: 0,
                preview_candidates: None,
            })
        }
    }

    // Mock LLM provider for testing
    struct MockLlmProvider {
        responses: Vec<String>,
    }

    impl MockLlmProvider {
        fn new(responses: Vec<String>) -> Self {
            Self { responses }
        }
    }

    #[async_trait]
    impl LlmProvider for MockLlmProvider {
        fn name(&self) -> &str { "mock" }
        async fn generate(&self, _prompt: &str, _options: &crate::llm::LlmOptions) -> Result<String> {
            Ok(self.responses.get(0).cloned().unwrap_or_else(|| "mock response".to_string()))
        }
        async fn generate_with_messages(&self, _messages: &[crate::llm::Message], _options: &crate::llm::LlmOptions) -> Result<String> {
            Ok(self.responses.get(0).cloned().unwrap_or_else(|| "mock response".to_string()))
        }
    }
}
