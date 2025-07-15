use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::error::{Error, Result};
use crate::agent::trait_def::{Agent, AgentStatus};
use crate::llm::Message;

/// API一致性检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyCheckResult {
    pub is_consistent: bool,
    pub issues: Vec<ConsistencyIssue>,
    pub score: f64, // 0.0 - 1.0
    pub recommendations: Vec<String>,
}

/// API一致性问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyIssue {
    pub category: String,
    pub severity: String, // "low", "medium", "high", "critical"
    pub description: String,
    pub location: String,
    pub suggestion: String,
}

/// API一致性检查器
pub struct ApiConsistencyChecker;

impl ApiConsistencyChecker {
    /// 检查Agent的API一致性
    pub async fn check_agent_consistency<T: Agent>(agent: &T) -> ConsistencyCheckResult {
        let mut issues = Vec::new();
        let mut score = 1.0;
        
        // 检查基本配置一致性
        Self::check_basic_configuration(agent, &mut issues, &mut score);
        
        // 检查方法实现一致性
        Self::check_method_implementation(agent, &mut issues, &mut score).await;
        
        // 检查状态管理一致性
        Self::check_state_management(agent, &mut issues, &mut score);
        
        // 检查错误处理一致性
        Self::check_error_handling(agent, &mut issues, &mut score).await;
        
        // 生成建议
        let recommendations = Self::generate_recommendations(&issues);
        
        ConsistencyCheckResult {
            is_consistent: issues.iter().all(|i| i.severity != "critical"),
            issues,
            score: score.max(0.0),
            recommendations,
        }
    }
    
    /// 检查基本配置一致性
    fn check_basic_configuration<T: Agent>(agent: &T, issues: &mut Vec<ConsistencyIssue>, score: &mut f64) {
        // 检查名称是否为空
        if agent.get_name().trim().is_empty() {
            issues.push(ConsistencyIssue {
                category: "Configuration".to_string(),
                severity: "high".to_string(),
                description: "Agent name is empty".to_string(),
                location: "get_name()".to_string(),
                suggestion: "Provide a meaningful agent name".to_string(),
            });
            *score -= 0.2;
        }
        
        // 检查指令是否为空
        if agent.get_instructions().trim().is_empty() {
            issues.push(ConsistencyIssue {
                category: "Configuration".to_string(),
                severity: "medium".to_string(),
                description: "Agent instructions are empty".to_string(),
                location: "get_instructions()".to_string(),
                suggestion: "Provide clear instructions for the agent".to_string(),
            });
            *score -= 0.1;
        }
        
        // 检查工具配置
        let tools = agent.get_tools();
        if tools.is_empty() {
            issues.push(ConsistencyIssue {
                category: "Configuration".to_string(),
                severity: "low".to_string(),
                description: "Agent has no tools configured".to_string(),
                location: "get_tools()".to_string(),
                suggestion: "Consider adding relevant tools to enhance agent capabilities".to_string(),
            });
            *score -= 0.05;
        }
    }
    
    /// 检查方法实现一致性
    async fn check_method_implementation<T: Agent>(agent: &T, issues: &mut Vec<ConsistencyIssue>, score: &mut f64) {
        // 测试基本生成功能
        let test_messages = vec![Message {
            role: crate::llm::Role::User,
            content: "Hello".to_string(),
            name: None,
            metadata: None,
        }];
        
        // 检查generate方法
        match agent.generate(&test_messages, &Default::default()).await {
            Ok(result) => {
                if result.response.trim().is_empty() {
                    issues.push(ConsistencyIssue {
                        category: "Implementation".to_string(),
                        severity: "medium".to_string(),
                        description: "Generate method returns empty response".to_string(),
                        location: "generate()".to_string(),
                        suggestion: "Ensure generate method returns meaningful responses".to_string(),
                    });
                    *score -= 0.1;
                }
            }
            Err(_) => {
                issues.push(ConsistencyIssue {
                    category: "Implementation".to_string(),
                    severity: "high".to_string(),
                    description: "Generate method fails with basic input".to_string(),
                    location: "generate()".to_string(),
                    suggestion: "Fix generate method implementation to handle basic cases".to_string(),
                });
                *score -= 0.2;
            }
        }
        
        // 检查健康检查方法
        match agent.health_check().await {
            Ok(health) => {
                if health.is_empty() {
                    issues.push(ConsistencyIssue {
                        category: "Implementation".to_string(),
                        severity: "low".to_string(),
                        description: "Health check returns empty data".to_string(),
                        location: "health_check()".to_string(),
                        suggestion: "Provide comprehensive health information".to_string(),
                    });
                    *score -= 0.05;
                }
            }
            Err(_) => {
                issues.push(ConsistencyIssue {
                    category: "Implementation".to_string(),
                    severity: "medium".to_string(),
                    description: "Health check method fails".to_string(),
                    location: "health_check()".to_string(),
                    suggestion: "Implement proper health check functionality".to_string(),
                });
                *score -= 0.1;
            }
        }
    }
    
    /// 检查状态管理一致性
    fn check_state_management<T: Agent>(agent: &T, issues: &mut Vec<ConsistencyIssue>, score: &mut f64) {
        let status = agent.get_status();
        
        // 检查状态是否合理
        match status {
            AgentStatus::Error(ref msg) => {
                if msg.trim().is_empty() {
                    issues.push(ConsistencyIssue {
                        category: "State Management".to_string(),
                        severity: "medium".to_string(),
                        description: "Agent is in error state but error message is empty".to_string(),
                        location: "get_status()".to_string(),
                        suggestion: "Provide meaningful error messages when agent is in error state".to_string(),
                    });
                    *score -= 0.1;
                }
            }
            AgentStatus::Stopped => {
                issues.push(ConsistencyIssue {
                    category: "State Management".to_string(),
                    severity: "high".to_string(),
                    description: "Agent is in stopped state but still responding".to_string(),
                    location: "get_status()".to_string(),
                    suggestion: "Ensure agent state accurately reflects its operational status".to_string(),
                });
                *score -= 0.2;
            }
            _ => {} // 其他状态正常
        }
        
        // 检查元数据一致性
        let metadata = agent.get_metadata();
        if metadata.contains_key("version") {
            if let Some(version) = metadata.get("version") {
                if version.trim().is_empty() {
                    issues.push(ConsistencyIssue {
                        category: "State Management".to_string(),
                        severity: "low".to_string(),
                        description: "Version metadata is empty".to_string(),
                        location: "get_metadata()".to_string(),
                        suggestion: "Provide valid version information".to_string(),
                    });
                    *score -= 0.05;
                }
            }
        }
    }
    
    /// 检查错误处理一致性
    async fn check_error_handling<T: Agent>(agent: &T, issues: &mut Vec<ConsistencyIssue>, score: &mut f64) {
        // 测试无效输入处理
        let invalid_messages = vec![Message {
            role: crate::llm::Role::Custom("invalid_role".to_string()),
            content: "".to_string(),
            name: None,
            metadata: None,
        }];
        
        match agent.generate(&invalid_messages, &Default::default()).await {
            Ok(result) => {
                if result.response.trim().is_empty() {
                    // 这可能是正确的行为，不算错误
                } else {
                    // 检查是否有适当的错误指示
                    if !result.response.to_lowercase().contains("error") && 
                       !result.response.to_lowercase().contains("invalid") {
                        issues.push(ConsistencyIssue {
                            category: "Error Handling".to_string(),
                            severity: "low".to_string(),
                            description: "Agent doesn't clearly indicate error for invalid input".to_string(),
                            location: "generate()".to_string(),
                            suggestion: "Provide clear error messages for invalid inputs".to_string(),
                        });
                        *score -= 0.05;
                    }
                }
            }
            Err(_) => {
                // 返回错误是正确的行为
            }
        }
        
        // 测试内存操作错误处理
        if agent.has_own_memory() {
            match agent.get_memory_value("non_existent_key").await {
                Ok(None) => {
                    // 正确行为：返回None而不是错误
                }
                Ok(Some(_)) => {
                    issues.push(ConsistencyIssue {
                        category: "Error Handling".to_string(),
                        severity: "medium".to_string(),
                        description: "Memory returns value for non-existent key".to_string(),
                        location: "get_memory_value()".to_string(),
                        suggestion: "Ensure memory operations return None for non-existent keys".to_string(),
                    });
                    *score -= 0.1;
                }
                Err(_) => {
                    // 也是可接受的行为，但建议返回None
                    issues.push(ConsistencyIssue {
                        category: "Error Handling".to_string(),
                        severity: "low".to_string(),
                        description: "Memory operations return errors instead of None for missing keys".to_string(),
                        location: "get_memory_value()".to_string(),
                        suggestion: "Consider returning None instead of errors for missing keys".to_string(),
                    });
                    *score -= 0.05;
                }
            }
        }
    }
    
    /// 生成改进建议
    fn generate_recommendations(issues: &[ConsistencyIssue]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        let critical_count = issues.iter().filter(|i| i.severity == "critical").count();
        let high_count = issues.iter().filter(|i| i.severity == "high").count();
        let medium_count = issues.iter().filter(|i| i.severity == "medium").count();
        
        if critical_count > 0 {
            recommendations.push(format!("Address {} critical issues immediately", critical_count));
        }
        
        if high_count > 0 {
            recommendations.push(format!("Fix {} high-priority issues", high_count));
        }
        
        if medium_count > 0 {
            recommendations.push(format!("Consider addressing {} medium-priority issues", medium_count));
        }
        
        // 按类别分组建议
        let mut categories: HashMap<String, usize> = HashMap::new();
        for issue in issues {
            *categories.entry(issue.category.clone()).or_insert(0) += 1;
        }
        
        for (category, count) in categories {
            if count > 2 {
                recommendations.push(format!("Focus on improving {} (has {} issues)", category, count));
            }
        }
        
        if recommendations.is_empty() {
            recommendations.push("API consistency is good! Consider regular consistency checks.".to_string());
        }
        
        recommendations
    }
}

/// API标准化工具
pub struct ApiStandardizer;

impl ApiStandardizer {
    /// 标准化Agent响应格式
    pub fn standardize_response(response: &str) -> String {
        // 移除多余的空白字符
        let cleaned = response.trim();
        
        // 确保响应不为空
        if cleaned.is_empty() {
            return "I apologize, but I couldn't generate a response. Please try again.".to_string();
        }
        
        // 确保响应以适当的标点符号结尾
        let mut standardized = cleaned.to_string();
        if !standardized.ends_with('.') && !standardized.ends_with('!') && !standardized.ends_with('?') {
            standardized.push('.');
        }
        
        standardized
    }
    
    /// 标准化错误消息格式
    pub fn standardize_error_message(error: &str) -> String {
        let cleaned = error.trim();
        
        if cleaned.is_empty() {
            return "An unknown error occurred".to_string();
        }
        
        // 确保错误消息以"Error:"开头
        if !cleaned.to_lowercase().starts_with("error:") {
            format!("Error: {}", cleaned)
        } else {
            cleaned.to_string()
        }
    }
    
    /// 标准化Agent名称格式
    pub fn standardize_agent_name(name: &str) -> String {
        let cleaned = name.trim();
        
        if cleaned.is_empty() {
            return "Unnamed Agent".to_string();
        }
        
        // 转换为标题格式
        cleaned.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}
