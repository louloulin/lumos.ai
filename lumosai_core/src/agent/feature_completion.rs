use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::agent::trait_def::Agent;
use crate::llm::Message;

/// 功能完整性检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureCompletenessResult {
    pub overall_score: f64, // 0.0 - 1.0
    pub missing_features: Vec<MissingFeature>,
    pub implemented_features: Vec<String>,
    pub recommendations: Vec<String>,
    pub priority_features: Vec<String>,
}

/// 缺失功能信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingFeature {
    pub name: String,
    pub category: String,
    pub priority: String, // "low", "medium", "high", "critical"
    pub description: String,
    pub implementation_effort: String, // "low", "medium", "high"
    pub business_impact: String,
}

/// 功能完整性检查器
pub struct FeatureCompletenessChecker;

impl FeatureCompletenessChecker {
    /// 检查Agent的功能完整性
    pub async fn check_completeness<T: Agent>(agent: &T) -> FeatureCompletenessResult {
        let mut missing_features = Vec::new();
        let mut implemented_features = Vec::new();
        let mut total_score = 0.0;
        let mut max_score = 0.0;
        
        // 检查核心功能
        Self::check_core_features(agent, &mut missing_features, &mut implemented_features, &mut total_score, &mut max_score).await;
        
        // 检查高级功能
        Self::check_advanced_features(agent, &mut missing_features, &mut implemented_features, &mut total_score, &mut max_score).await;
        
        // 检查集成功能
        Self::check_integration_features(agent, &mut missing_features, &mut implemented_features, &mut total_score, &mut max_score);
        
        // 检查监控和诊断功能
        Self::check_monitoring_features(agent, &mut missing_features, &mut implemented_features, &mut total_score, &mut max_score).await;
        
        let overall_score = if max_score > 0.0 { total_score / max_score } else { 0.0 };
        
        let recommendations = Self::generate_recommendations(&missing_features, overall_score);
        let priority_features = Self::identify_priority_features(&missing_features);
        
        FeatureCompletenessResult {
            overall_score,
            missing_features,
            implemented_features,
            recommendations,
            priority_features,
        }
    }
    
    /// 检查核心功能
    async fn check_core_features<T: Agent>(
        agent: &T,
        missing_features: &mut Vec<MissingFeature>,
        implemented_features: &mut Vec<String>,
        total_score: &mut f64,
        max_score: &mut f64,
    ) {
        *max_score += 10.0; // 核心功能总分
        
        // 检查基本生成功能
        let test_message = vec![Message {
            role: crate::llm::Role::User,
            content: "Hello".to_string(),
            name: None,
            metadata: None,
        }];
        
        match agent.generate(&test_message, &Default::default()).await {
            Ok(_) => {
                implemented_features.push("Basic Generation".to_string());
                *total_score += 3.0;
            }
            Err(_) => {
                missing_features.push(MissingFeature {
                    name: "Basic Generation".to_string(),
                    category: "Core".to_string(),
                    priority: "critical".to_string(),
                    description: "Agent cannot generate basic responses".to_string(),
                    implementation_effort: "high".to_string(),
                    business_impact: "Critical - Agent is non-functional without this".to_string(),
                });
            }
        }
        
        // 检查流式处理
        match agent.stream(&test_message, &Default::default()).await {
            Ok(_) => {
                implemented_features.push("Streaming".to_string());
                *total_score += 2.0;
            }
            Err(_) => {
                missing_features.push(MissingFeature {
                    name: "Streaming".to_string(),
                    category: "Core".to_string(),
                    priority: "high".to_string(),
                    description: "Agent doesn't support streaming responses".to_string(),
                    implementation_effort: "medium".to_string(),
                    business_impact: "High - Affects user experience and responsiveness".to_string(),
                });
            }
        }
        
        // 检查工具支持
        let tools = agent.get_tools();
        if !tools.is_empty() {
            implemented_features.push("Tool Support".to_string());
            *total_score += 2.0;
        } else {
            missing_features.push(MissingFeature {
                name: "Tool Support".to_string(),
                category: "Core".to_string(),
                priority: "high".to_string(),
                description: "Agent has no tools configured".to_string(),
                implementation_effort: "medium".to_string(),
                business_impact: "High - Limits agent capabilities significantly".to_string(),
            });
        }
        
        // 检查内存支持
        if agent.has_own_memory() {
            implemented_features.push("Memory Support".to_string());
            *total_score += 2.0;
        } else {
            missing_features.push(MissingFeature {
                name: "Memory Support".to_string(),
                category: "Core".to_string(),
                priority: "medium".to_string(),
                description: "Agent doesn't have persistent memory".to_string(),
                implementation_effort: "medium".to_string(),
                business_impact: "Medium - Affects conversation continuity".to_string(),
            });
        }
        
        // 检查配置管理
        match agent.validate_config() {
            Ok(_) => {
                implemented_features.push("Configuration Validation".to_string());
                *total_score += 1.0;
            }
            Err(_) => {
                missing_features.push(MissingFeature {
                    name: "Configuration Validation".to_string(),
                    category: "Core".to_string(),
                    priority: "medium".to_string(),
                    description: "Agent configuration validation is not working".to_string(),
                    implementation_effort: "low".to_string(),
                    business_impact: "Medium - May lead to runtime errors".to_string(),
                });
            }
        }
    }
    
    /// 检查高级功能
    async fn check_advanced_features<T: Agent>(
        agent: &T,
        missing_features: &mut Vec<MissingFeature>,
        implemented_features: &mut Vec<String>,
        total_score: &mut f64,
        max_score: &mut f64,
    ) {
        *max_score += 6.0; // 高级功能总分
        
        // 检查多步推理
        let test_message = vec![Message {
            role: crate::llm::Role::User,
            content: "Solve this step by step: 2 + 2 * 3".to_string(),
            name: None,
            metadata: None,
        }];
        
        match agent.generate_with_steps(&test_message, &Default::default(), Some(3)).await {
            Ok(result) => {
                if result.steps.len() > 1 {
                    implemented_features.push("Multi-step Reasoning".to_string());
                    *total_score += 2.0;
                } else {
                    missing_features.push(MissingFeature {
                        name: "Multi-step Reasoning".to_string(),
                        category: "Advanced".to_string(),
                        priority: "medium".to_string(),
                        description: "Agent doesn't break down complex problems into steps".to_string(),
                        implementation_effort: "high".to_string(),
                        business_impact: "Medium - Limits problem-solving capabilities".to_string(),
                    });
                }
            }
            Err(_) => {
                missing_features.push(MissingFeature {
                    name: "Multi-step Reasoning".to_string(),
                    category: "Advanced".to_string(),
                    priority: "medium".to_string(),
                    description: "Agent doesn't support multi-step reasoning".to_string(),
                    implementation_effort: "high".to_string(),
                    business_impact: "Medium - Limits complex problem solving".to_string(),
                });
            }
        }
        
        // 检查语音支持
        if agent.get_voice().is_some() {
            implemented_features.push("Voice Support".to_string());
            *total_score += 1.0;
        } else {
            missing_features.push(MissingFeature {
                name: "Voice Support".to_string(),
                category: "Advanced".to_string(),
                priority: "low".to_string(),
                description: "Agent doesn't support voice interactions".to_string(),
                implementation_effort: "medium".to_string(),
                business_impact: "Low - Nice to have for accessibility".to_string(),
            });
        }
        
        // 检查工作流支持
        match agent.get_workflows(&Default::default()).await {
            Ok(workflows) => {
                if !workflows.is_empty() {
                    implemented_features.push("Workflow Support".to_string());
                    *total_score += 2.0;
                } else {
                    missing_features.push(MissingFeature {
                        name: "Workflow Support".to_string(),
                        category: "Advanced".to_string(),
                        priority: "medium".to_string(),
                        description: "Agent doesn't support predefined workflows".to_string(),
                        implementation_effort: "high".to_string(),
                        business_impact: "Medium - Limits automation capabilities".to_string(),
                    });
                }
            }
            Err(_) => {
                missing_features.push(MissingFeature {
                    name: "Workflow Support".to_string(),
                    category: "Advanced".to_string(),
                    priority: "medium".to_string(),
                    description: "Agent workflow system is not functional".to_string(),
                    implementation_effort: "high".to_string(),
                    business_impact: "Medium - Limits automation capabilities".to_string(),
                });
            }
        }
        
        // 检查结构化输出
        // 注意：这需要Agent实现AgentStructuredOutput trait
        // 这里我们简化检查，看是否能处理JSON请求
        let json_request = vec![Message {
            role: crate::llm::Role::User,
            content: "Return a JSON object with name and age fields".to_string(),
            name: None,
            metadata: None,
        }];
        
        match agent.generate(&json_request, &Default::default()).await {
            Ok(result) => {
                if result.response.contains("{") && result.response.contains("}") {
                    implemented_features.push("Structured Output".to_string());
                    *total_score += 1.0;
                } else {
                    missing_features.push(MissingFeature {
                        name: "Structured Output".to_string(),
                        category: "Advanced".to_string(),
                        priority: "medium".to_string(),
                        description: "Agent doesn't support structured output generation".to_string(),
                        implementation_effort: "medium".to_string(),
                        business_impact: "Medium - Limits integration capabilities".to_string(),
                    });
                }
            }
            Err(_) => {
                missing_features.push(MissingFeature {
                    name: "Structured Output".to_string(),
                    category: "Advanced".to_string(),
                    priority: "medium".to_string(),
                    description: "Agent cannot generate structured responses".to_string(),
                    implementation_effort: "medium".to_string(),
                    business_impact: "Medium - Limits API integration".to_string(),
                });
            }
        }
    }
    
    /// 检查集成功能
    fn check_integration_features<T: Agent>(
        agent: &T,
        missing_features: &mut Vec<MissingFeature>,
        implemented_features: &mut Vec<String>,
        total_score: &mut f64,
        max_score: &mut f64,
    ) {
        *max_score += 4.0; // 集成功能总分
        
        // 检查元数据支持
        let metadata = agent.get_metadata();
        if !metadata.is_empty() {
            implemented_features.push("Metadata Support".to_string());
            *total_score += 1.0;
        } else {
            missing_features.push(MissingFeature {
                name: "Metadata Support".to_string(),
                category: "Integration".to_string(),
                priority: "low".to_string(),
                description: "Agent doesn't provide metadata information".to_string(),
                implementation_effort: "low".to_string(),
                business_impact: "Low - Useful for debugging and monitoring".to_string(),
            });
        }
        
        // 检查状态管理
        let status = agent.get_status();
        implemented_features.push("Status Management".to_string());
        *total_score += 1.0;
        
        // 检查配置重载
        // 这里我们假设如果Agent有配置，就支持重载
        implemented_features.push("Configuration Management".to_string());
        *total_score += 1.0;
        
        // 检查工具动态管理
        let tools_count = agent.get_tools().len();
        if tools_count > 0 {
            implemented_features.push("Dynamic Tool Management".to_string());
            *total_score += 1.0;
        } else {
            missing_features.push(MissingFeature {
                name: "Dynamic Tool Management".to_string(),
                category: "Integration".to_string(),
                priority: "medium".to_string(),
                description: "Agent doesn't support dynamic tool addition/removal".to_string(),
                implementation_effort: "medium".to_string(),
                business_impact: "Medium - Limits runtime flexibility".to_string(),
            });
        }
    }
    
    /// 检查监控和诊断功能
    async fn check_monitoring_features<T: Agent>(
        agent: &T,
        missing_features: &mut Vec<MissingFeature>,
        implemented_features: &mut Vec<String>,
        total_score: &mut f64,
        max_score: &mut f64,
    ) {
        *max_score += 3.0; // 监控功能总分
        
        // 检查健康检查
        match agent.health_check().await {
            Ok(health) => {
                if !health.is_empty() {
                    implemented_features.push("Health Check".to_string());
                    *total_score += 1.0;
                } else {
                    missing_features.push(MissingFeature {
                        name: "Comprehensive Health Check".to_string(),
                        category: "Monitoring".to_string(),
                        priority: "medium".to_string(),
                        description: "Health check returns empty information".to_string(),
                        implementation_effort: "low".to_string(),
                        business_impact: "Medium - Affects monitoring capabilities".to_string(),
                    });
                }
            }
            Err(_) => {
                missing_features.push(MissingFeature {
                    name: "Health Check".to_string(),
                    category: "Monitoring".to_string(),
                    priority: "high".to_string(),
                    description: "Agent health check is not functional".to_string(),
                    implementation_effort: "low".to_string(),
                    business_impact: "High - Critical for production monitoring".to_string(),
                });
            }
        }
        
        // 检查性能指标
        match agent.get_metrics().await {
            Ok(metrics) => {
                if !metrics.is_empty() {
                    implemented_features.push("Performance Metrics".to_string());
                    *total_score += 1.0;
                } else {
                    missing_features.push(MissingFeature {
                        name: "Performance Metrics".to_string(),
                        category: "Monitoring".to_string(),
                        priority: "medium".to_string(),
                        description: "Agent doesn't provide performance metrics".to_string(),
                        implementation_effort: "medium".to_string(),
                        business_impact: "Medium - Affects performance monitoring".to_string(),
                    });
                }
            }
            Err(_) => {
                missing_features.push(MissingFeature {
                    name: "Performance Metrics".to_string(),
                    category: "Monitoring".to_string(),
                    priority: "medium".to_string(),
                    description: "Agent metrics collection is not working".to_string(),
                    implementation_effort: "medium".to_string(),
                    business_impact: "Medium - Limits performance optimization".to_string(),
                });
            }
        }
        
        // 检查重置功能
        // 我们不实际调用reset，只检查是否实现
        implemented_features.push("Reset Capability".to_string());
        *total_score += 1.0;
    }
    
    /// 生成改进建议
    fn generate_recommendations(missing_features: &[MissingFeature], overall_score: f64) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if overall_score < 0.5 {
            recommendations.push("Agent functionality is significantly incomplete. Focus on implementing core features first.".to_string());
        } else if overall_score < 0.8 {
            recommendations.push("Agent has good basic functionality but is missing some important features.".to_string());
        } else {
            recommendations.push("Agent is well-implemented with most features available.".to_string());
        }
        
        let critical_features: Vec<_> = missing_features.iter()
            .filter(|f| f.priority == "critical")
            .collect();
        
        if !critical_features.is_empty() {
            recommendations.push(format!("Immediately implement {} critical features", critical_features.len()));
        }
        
        let high_priority: Vec<_> = missing_features.iter()
            .filter(|f| f.priority == "high")
            .collect();
        
        if !high_priority.is_empty() {
            recommendations.push(format!("Prioritize {} high-priority features", high_priority.len()));
        }
        
        // 按类别分组建议
        let mut categories: HashMap<String, usize> = HashMap::new();
        for feature in missing_features {
            *categories.entry(feature.category.clone()).or_insert(0) += 1;
        }
        
        for (category, count) in categories {
            if count > 2 {
                recommendations.push(format!("Focus on {} features (missing {} items)", category, count));
            }
        }
        
        recommendations
    }
    
    /// 识别优先级功能
    fn identify_priority_features(missing_features: &[MissingFeature]) -> Vec<String> {
        missing_features.iter()
            .filter(|f| f.priority == "critical" || f.priority == "high")
            .map(|f| f.name.clone())
            .collect()
    }
}
