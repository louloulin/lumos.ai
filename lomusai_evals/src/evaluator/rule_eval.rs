//! 基于规则的评估器实现

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use async_trait;

use crate::error::{Error, Result};
use crate::types::{EvalOptions, EvalResult};
use crate::evaluator::Evaluator;

/// 评估规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// 规则名称
    pub name: String,
    
    /// 规则描述
    pub description: String,
    
    /// 规则权重，用于计算最终得分
    pub weight: f64,
    
    /// 规则类型
    pub rule_type: RuleType,
}

/// 规则类型
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RuleType {
    /// 正则表达式规则，检查输出是否匹配正则
    #[serde(rename = "regex")]
    Regex(String),
    
    /// 关键词规则，检查输出是否包含关键词
    #[serde(rename = "keywords")]
    Keywords(Vec<String>),
    
    /// 长度规则，检查输出长度是否在指定范围内
    #[serde(rename = "length")]
    Length { min: Option<usize>, max: Option<usize> },
    
    /// 自定义规则，使用闭包评估
    #[serde(skip)]
    Custom(#[serde(skip)] Box<dyn Fn(&str, &str) -> (bool, Option<String>) + Send + Sync>),
}

// 手动实现Debug特性
impl std::fmt::Debug for RuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleType::Regex(pattern) => write!(f, "Regex({:?})", pattern),
            RuleType::Keywords(keywords) => write!(f, "Keywords({:?})", keywords),
            RuleType::Length { min, max } => write!(f, "Length {{ min: {:?}, max: {:?} }}", min, max),
            RuleType::Custom(_) => write!(f, "Custom(function)"),
        }
    }
}

// 手动实现Clone特性
impl Clone for RuleType {
    fn clone(&self) -> Self {
        match self {
            RuleType::Regex(pattern) => RuleType::Regex(pattern.clone()),
            RuleType::Keywords(keywords) => RuleType::Keywords(keywords.clone()),
            RuleType::Length { min, max } => RuleType::Length { 
                min: min.clone(), 
                max: max.clone() 
            },
            // 自定义规则不能克隆，返回默认正则
            RuleType::Custom(_) => RuleType::Regex(".*".to_string()),
        }
    }
}

/// 规则评估结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEvalResult {
    /// 规则名称
    pub rule_name: String,
    
    /// 是否通过
    pub passed: bool,
    
    /// 规则得分，0-1
    pub score: f64,
    
    /// 规则权重
    pub weight: f64,
    
    /// 评估消息
    pub message: Option<String>,
}

/// 基于规则的评估器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEvaluatorConfig {
    /// 评估规则
    #[serde(skip)]
    pub rules: Vec<Rule>,
    
    /// 不符合规则时的默认得分
    pub default_fail_score: f64,
}

impl Default for RuleEvaluatorConfig {
    fn default() -> Self {
        Self {
            rules: Vec::new(),
            default_fail_score: 0.0,
        }
    }
}

/// 基于规则的评估器
pub struct RuleEvaluator {
    /// 评估器名称
    name: String,
    
    /// 评估器配置
    config: RuleEvaluatorConfig,
}

impl RuleEvaluator {
    /// 创建一个新的规则评估器
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            config: RuleEvaluatorConfig::default(),
        }
    }
    
    /// 设置评估器配置
    pub fn with_config(mut self, config: RuleEvaluatorConfig) -> Self {
        self.config = config;
        self
    }
    
    /// 添加规则
    pub fn add_rule(mut self, rule: Rule) -> Self {
        self.config.rules.push(rule);
        self
    }
    
    /// 评估单个规则
    fn evaluate_rule(&self, rule: &Rule, input: &str, output: &str) -> RuleEvalResult {
        match &rule.rule_type {
            RuleType::Regex(pattern) => {
                let re = match regex::Regex::new(pattern) {
                    Ok(re) => re,
                    Err(e) => {
                        return RuleEvalResult {
                            rule_name: rule.name.clone(),
                            passed: false,
                            score: self.config.default_fail_score,
                            weight: rule.weight,
                            message: Some(format!("正则表达式无效: {}", e)),
                        };
                    }
                };
                
                let is_match = re.is_match(output);
                RuleEvalResult {
                    rule_name: rule.name.clone(),
                    passed: is_match,
                    score: if is_match { 1.0 } else { self.config.default_fail_score },
                    weight: rule.weight,
                    message: Some(if is_match {
                        format!("输出匹配正则表达式")
                    } else {
                        format!("输出不匹配正则表达式")
                    }),
                }
            },
            RuleType::Keywords(keywords) => {
                let found_keywords: Vec<&String> = keywords.iter()
                    .filter(|&keyword| output.contains(keyword))
                    .collect();
                    
                let passed = !found_keywords.is_empty();
                let score = if passed {
                    found_keywords.len() as f64 / keywords.len() as f64
                } else {
                    self.config.default_fail_score
                };
                
                RuleEvalResult {
                    rule_name: rule.name.clone(),
                    passed,
                    score,
                    weight: rule.weight,
                    message: Some(if passed {
                        format!("找到关键词: {}", found_keywords.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "))
                    } else {
                        format!("未找到任何关键词")
                    }),
                }
            },
            RuleType::Length { min, max } => {
                let length = output.len();
                let min_ok = min.map_or(true, |min_len| length >= min_len);
                let max_ok = max.map_or(true, |max_len| length <= max_len);
                let passed = min_ok && max_ok;
                
                RuleEvalResult {
                    rule_name: rule.name.clone(),
                    passed,
                    score: if passed { 1.0 } else { self.config.default_fail_score },
                    weight: rule.weight,
                    message: Some(if !min_ok {
                        format!("输出长度({})小于最小长度({})", length, min.unwrap())
                    } else if !max_ok {
                        format!("输出长度({})大于最大长度({})", length, max.unwrap())
                    } else {
                        format!("输出长度({})在允许范围内", length)
                    }),
                }
            },
            RuleType::Custom(evaluator) => {
                let (passed, message) = evaluator(input, output);
                
                RuleEvalResult {
                    rule_name: rule.name.clone(),
                    passed,
                    score: if passed { 1.0 } else { self.config.default_fail_score },
                    weight: rule.weight,
                    message,
                }
            },
        }
    }
}

#[async_trait::async_trait]
impl Evaluator for RuleEvaluator {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn evaluate(&self, input: &str, output: &str, options: &EvalOptions) -> Result<EvalResult> {
        // 评估所有规则
        let mut rule_results = Vec::new();
        let mut total_weight = 0.0;
        let mut weighted_score = 0.0;
        
        for rule in &self.config.rules {
            let result = self.evaluate_rule(rule, input, output);
            weighted_score += result.score * result.weight;
            total_weight += result.weight;
            rule_results.push(result);
        }
        
        // 计算加权平均分
        let final_score = if total_weight > 0.0 {
            weighted_score / total_weight
        } else {
            // 如果没有规则或所有规则权重为0，返回默认满分
            1.0
        };
        
        // 创建结果详情
        let mut score_details = HashMap::new();
        score_details.insert("rule_results".to_string(), 
            serde_json::to_value(&rule_results).map_err(|e| Error::Serialization(e))?);
        
        // 创建评估结果
        let global_run_id = options.global_run_id.clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());
            
        let run_id = options.run_id.clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());
            
        let result = EvalResult {
            id: Uuid::new_v4().to_string(),
            global_run_id,
            run_id,
            input: input.to_string(),
            output: output.to_string(),
            score: final_score,
            score_details,
            created_at: Utc::now(),
            evaluator_name: self.name.clone(),
            metric_name: "rule_based".to_string(),
            target_name: options.target_name.clone(),
            test_info: options.test_info.clone(),
            instructions: options.instructions.clone(),
        };
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_regex_rule() {
        // 创建规则评估器
        let evaluator = RuleEvaluator::new("regex_evaluator")
            .add_rule(Rule {
                name: "contains_number".to_string(),
                description: "回答中应包含数字".to_string(),
                weight: 1.0,
                rule_type: RuleType::Regex(r"\d+".to_string()),
            });
            
        // 测试匹配的情况
        let options = EvalOptions::default();
        let result1 = evaluator.evaluate("", "回答中包含数字123", &options).await.unwrap();
        assert_eq!(result1.score, 1.0);
        
        // 测试不匹配的情况
        let result2 = evaluator.evaluate("", "回答中不包含数字", &options).await.unwrap();
        assert_eq!(result2.score, 0.0);
    }
    
    #[tokio::test]
    async fn test_keywords_rule() {
        // 创建规则评估器
        let evaluator = RuleEvaluator::new("keywords_evaluator")
            .add_rule(Rule {
                name: "rust_features".to_string(),
                description: "回答中应包含Rust的关键特性".to_string(),
                weight: 1.0,
                rule_type: RuleType::Keywords(vec![
                    "所有权".to_string(), 
                    "安全".to_string(), 
                    "并发".to_string()
                ]),
            });
            
        // 测试包含部分关键词的情况
        let options = EvalOptions::default();
        let result = evaluator.evaluate(
            "", 
            "Rust的所有权系统保证了内存安全", 
            &options
        ).await.unwrap();
        
        // 包含了2/3的关键词，得分应该是2/3
        assert_eq!(result.score, 2.0/3.0);
    }
    
    #[tokio::test]
    async fn test_length_rule() {
        // 创建规则评估器
        let evaluator = RuleEvaluator::new("length_evaluator")
            .add_rule(Rule {
                name: "length_check".to_string(),
                description: "回答长度应在指定范围内".to_string(),
                weight: 1.0,
                rule_type: RuleType::Length { min: Some(10), max: Some(100) },
            });
            
        // 测试长度在范围内的情况
        let options = EvalOptions::default();
        let result1 = evaluator.evaluate("", "这个回答长度适中，在允许范围内", &options).await.unwrap();
        assert_eq!(result1.score, 1.0);
        
        // 测试长度太短的情况
        let result2 = evaluator.evaluate("", "太短了", &options).await.unwrap();
        assert_eq!(result2.score, 0.0);
    }
    
    #[tokio::test]
    async fn test_custom_rule() {
        // 创建规则评估器
        let evaluator = RuleEvaluator::new("custom_evaluator")
            .add_rule(Rule {
                name: "question_answer".to_string(),
                description: "回答应该针对问题".to_string(),
                weight: 1.0,
                rule_type: RuleType::Custom(Box::new(|input, output| {
                    // 修改为单独提取关键词，不再使用简单分割
                    let keywords = vec!["Rust", "特点", "语言"];
                    println!("Keywords to search: {:?}", keywords);
                    let mut found = false;
                    
                    for word in &keywords {
                        println!("Checking keyword: '{}'", word);
                        if output.contains(word) {
                            println!("Found matching word: '{}'", word);
                            found = true;
                            break;
                        }
                    }
                    
                    println!("Custom rule result: {}", found);
                    (found, Some(if found { 
                        "回答包含问题中的关键词".to_string() 
                    } else { 
                        "回答未包含问题中的关键词".to_string() 
                    }))
                })),
            });
            
        // 测试回答相关的情况
        let options = EvalOptions::default();
        let input = "Rust语言的特点是什么？";
        let output = "Rust语言的特点包括内存安全、并发安全和零成本抽象";
        println!("Input: '{}'", input);
        println!("Output: '{}'", output);
        
        // 先测试直接调用评估规则函数
        let rule = &evaluator.config.rules[0];
        let rule_result = evaluator.evaluate_rule(rule, input, output);
        println!("Direct rule evaluation result: {:?}", rule_result);
        
        // 使用评估器评估
        let result = evaluator.evaluate(input, output, &options).await.unwrap();
        println!("Final evaluation result: {}", result.score);
        
        // 检查结果中的详细信息
        if let Some(details) = result.score_details.get("rule_results") {
            println!("Rule results: {}", details);
        }
        
        assert_eq!(result.score, 1.0);
    }
    
    #[tokio::test]
    async fn test_multiple_rules() {
        // 创建有多个规则的评估器
        let evaluator = RuleEvaluator::new("multi_rule_evaluator")
            .add_rule(Rule {
                name: "contains_number".to_string(),
                description: "回答中应包含数字".to_string(),
                weight: 1.0,
                rule_type: RuleType::Regex(r"\b\d+\b".to_string()),
            })
            .add_rule(Rule {
                name: "length_check".to_string(),
                description: "回答长度应在指定范围内".to_string(),
                weight: 2.0,
                rule_type: RuleType::Length { min: Some(20), max: None },
            });
            
        // 测试多规则评估
        let options = EvalOptions::default();
        let input = "";
        let output = "这是一个超过20个字符的回答，但不包含数字";
        println!("Output: '{}'", output);
        
        // 先测试各个规则的单独结果
        let rules = &evaluator.config.rules;
        for (i, rule) in rules.iter().enumerate() {
            let rule_result = evaluator.evaluate_rule(rule, input, output);
            println!("Rule #{} evaluation result: {:?}", i+1, rule_result);
        }
        
        // 使用评估器评估
        let result = evaluator.evaluate(input, output, &options).await.unwrap();
        println!("Final evaluation result: {}", result.score);
        
        // 检查结果中的详细信息
        if let Some(details) = result.score_details.get("rule_results") {
            println!("Rule results: {}", details);
        }
        
        // 计算加权平均分的详细过程
        let mut total_weighted_score = 0.0;
        let mut total_weight = 0.0;
        
        for rule in rules {
            let rule_result = evaluator.evaluate_rule(rule, input, output);
            total_weighted_score += rule_result.score * rule_result.weight;
            total_weight += rule_result.weight;
            println!("Rule '{}': score={}, weight={}, weighted={}", 
                rule_result.rule_name, rule_result.score, rule_result.weight, 
                rule_result.score * rule_result.weight);
        }
        
        let expected_score = if total_weight > 0.0 { 
            total_weighted_score / total_weight 
        } else { 
            1.0 
        };
        println!("Expected score: {}", expected_score);
        
        // 规则1未通过(0分)，规则2通过(1分)，权重比为1:2，所以加权平均分为(0*1 + 1*2)/3 = 2/3
        assert_eq!(result.score, 2.0/3.0);
    }
} 