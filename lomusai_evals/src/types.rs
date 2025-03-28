use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 测试信息，描述评估的背景
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TestInfo {
    /// 测试名称
    pub test_name: Option<String>,
    
    /// 测试文件路径
    pub test_path: Option<String>,
    
    /// 测试标签
    pub tags: Vec<String>,
    
    /// 测试描述
    pub description: Option<String>,
}

/// 评估结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalResult {
    /// 唯一ID
    pub id: String,
    
    /// 全局运行ID，用于关联多个相关评估
    pub global_run_id: String,
    
    /// 特定评估的运行ID
    pub run_id: String,
    
    /// 被评估的输入
    pub input: String,
    
    /// 被评估的输出
    pub output: String,
    
    /// 评估得分 (0.0-1.0)
    pub score: f64,
    
    /// 得分的细节信息
    pub score_details: HashMap<String, serde_json::Value>,
    
    /// 评估开始时间
    pub created_at: DateTime<Utc>,
    
    /// 评估器名称
    pub evaluator_name: String,
    
    /// 指标名称
    pub metric_name: String,
    
    /// 目标名称（如Agent名称）
    pub target_name: Option<String>,
    
    /// 相关的测试信息
    pub test_info: Option<TestInfo>,
    
    /// 评估过程中使用的指令
    pub instructions: Option<String>,
}

impl Default for EvalResult {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            global_run_id: Uuid::new_v4().to_string(),
            run_id: Uuid::new_v4().to_string(),
            input: String::new(),
            output: String::new(),
            score: 0.0,
            score_details: HashMap::new(),
            created_at: Utc::now(),
            evaluator_name: String::new(),
            metric_name: String::new(),
            target_name: None,
            test_info: None,
            instructions: None,
        }
    }
}

/// 评估选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalOptions {
    /// 全局运行ID，如果不提供则自动生成
    pub global_run_id: Option<String>,
    
    /// 特定评估的运行ID，如果不提供则自动生成
    pub run_id: Option<String>,
    
    /// 目标名称（如Agent名称）
    pub target_name: Option<String>,
    
    /// 相关的测试信息
    pub test_info: Option<TestInfo>,
    
    /// 评估过程中使用的指令
    pub instructions: Option<String>,
    
    /// 是否记录评估结果
    pub log_results: bool,
}

impl Default for EvalOptions {
    fn default() -> Self {
        Self {
            global_run_id: None,
            run_id: None,
            target_name: None,
            test_info: None,
            instructions: None,
            log_results: true,
        }
    }
} 