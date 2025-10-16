#!/usr/bin/env cargo
//! # 智能客服系统示例
//! 
//! 本示例展示如何使用 LumosAI 构建智能客服系统，
//! 提供多轮对话、问题分类、知识库查询等功能。
//! 
//! ## 功能特性
//! - 智能意图识别和分类
//! - 多轮对话管理
//! - 知识库查询和推荐
//! - 情感分析和个性化响应

use lumosai_core::prelude::*;
use lumosai_core::llm::types::user_message;
use lumosai_core::agent::types::AgentGenerateOptions;
use lumosai_core::agent::BasicAgent;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// 客户查询类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum QueryType {
    ProductInquiry,
    TechnicalSupport,
    Billing,
    Complaint,
    General,
    Urgent,
}

/// 客户情感状态
#[derive(Debug, Clone, PartialEq)]
enum CustomerMood {
    Happy,
    Neutral,
    Frustrated,
    Angry,
    Confused,
}

/// 客户信息
#[derive(Debug, Clone)]
struct CustomerProfile {
    id: String,
    name: String,
    tier: String, // VIP, Premium, Standard
    history: Vec<String>,
    preferences: HashMap<String, String>,
}

/// 对话会话
#[derive(Debug, Clone)]
struct ConversationSession {
    session_id: String,
    customer: CustomerProfile,
    messages: Vec<String>,
    query_type: Option<QueryType>,
    mood: CustomerMood,
    satisfaction_score: Option<f32>,
    resolved: bool,
    start_time: u64,
}

/// 知识库条目
#[derive(Debug, Clone)]
struct KnowledgeItem {
    id: String,
    title: String,
    content: String,
    category: String,
    tags: Vec<String>,
    relevance_score: f32,
}

/// 智能客服系统
struct IntelligentCustomerService {
    classifier: BasicAgent,
    responder: BasicAgent,
    analyzer: BasicAgent,
    knowledge_base: Vec<KnowledgeItem>,
    active_sessions: HashMap<String, ConversationSession>,
    response_templates: HashMap<QueryType, Vec<String>>,
}

impl IntelligentCustomerService {
    /// 创建智能客服系统
    async fn new() -> Result<Self> {
        println!("🚀 正在初始化智能客服系统...");

        // 创建意图分类器 Agent
        let classifier = quick_agent(
            "意图分类器",
            "你是一个专业的客服意图分类专家，负责：
            1. 分析客户查询的意图和类型
            2. 识别查询的紧急程度
            3. 分析客户的情感状态
            4. 提供分类建议和处理优先级"
        ).build()?;

        // 创建智能回复器 Agent
        let responder = quick_agent(
            "智能回复器",
            "你是一个专业的客服代表，负责：
            1. 提供准确、友好的客户服务
            2. 根据客户情感调整回复风格
            3. 查询知识库提供准确信息
            4. 引导客户解决问题"
        ).build()?;

        // 创建对话分析器 Agent
        let analyzer = quick_agent(
            "对话分析器",
            "你是一个客服质量分析专家，负责：
            1. 分析对话质量和客户满意度
            2. 识别服务改进机会
            3. 评估问题解决效果
            4. 提供服务优化建议"
        ).build()?;

        // 初始化知识库
        let knowledge_base = Self::initialize_knowledge_base();

        // 初始化回复模板
        let mut response_templates = HashMap::new();
        response_templates.insert(QueryType::ProductInquiry, vec![
            "感谢您对我们产品的关注！我很乐意为您介绍相关信息。".to_string(),
            "让我为您查找最新的产品信息...".to_string(),
        ]);
        response_templates.insert(QueryType::TechnicalSupport, vec![
            "我理解您遇到的技术问题，让我来帮助您解决。".to_string(),
            "请详细描述您遇到的问题，我会为您提供解决方案。".to_string(),
        ]);
        response_templates.insert(QueryType::Billing, vec![
            "关于账单问题，我会仔细为您核查。".to_string(),
            "让我查看您的账户信息，为您解答账单相关问题。".to_string(),
        ]);

        println!("✅ 智能客服系统初始化完成！");
        println!("   🎯 意图分类器 - 智能意图识别");
        println!("   💬 智能回复器 - 个性化回复");
        println!("   📊 对话分析器 - 质量分析");
        println!("   📚 知识库条目: {} 个", knowledge_base.len());

        Ok(Self {
            classifier,
            responder,
            analyzer,
            knowledge_base,
            active_sessions: HashMap::new(),
            response_templates,
        })
    }

    /// 初始化知识库
    fn initialize_knowledge_base() -> Vec<KnowledgeItem> {
        vec![
            KnowledgeItem {
                id: "kb001".to_string(),
                title: "产品功能介绍".to_string(),
                content: "我们的产品提供AI驱动的智能分析功能，支持实时数据处理和自动化工作流。".to_string(),
                category: "产品信息".to_string(),
                tags: vec!["功能".to_string(), "AI".to_string(), "分析".to_string()],
                relevance_score: 0.0,
            },
            KnowledgeItem {
                id: "kb002".to_string(),
                title: "账单查询方法".to_string(),
                content: "您可以登录账户后台查看详细账单，或联系客服获取账单明细。".to_string(),
                category: "账单服务".to_string(),
                tags: vec!["账单".to_string(), "查询".to_string(), "方法".to_string()],
                relevance_score: 0.0,
            },
            KnowledgeItem {
                id: "kb003".to_string(),
                title: "技术支持流程".to_string(),
                content: "遇到技术问题时，请先检查网络连接，然后重启应用。如问题持续，请联系技术支持。".to_string(),
                category: "技术支持".to_string(),
                tags: vec!["技术".to_string(), "支持".to_string(), "流程".to_string()],
                relevance_score: 0.0,
            },
            KnowledgeItem {
                id: "kb004".to_string(),
                title: "退款政策".to_string(),
                content: "我们提供30天无理由退款服务。请在购买后30天内联系客服申请退款。".to_string(),
                category: "售后服务".to_string(),
                tags: vec!["退款".to_string(), "政策".to_string(), "售后".to_string()],
                relevance_score: 0.0,
            },
            KnowledgeItem {
                id: "kb005".to_string(),
                title: "VIP服务特权".to_string(),
                content: "VIP客户享有优先技术支持、专属客服、免费升级等特权服务。".to_string(),
                category: "VIP服务".to_string(),
                tags: vec!["VIP".to_string(), "特权".to_string(), "服务".to_string()],
                relevance_score: 0.0,
            },
        ]
    }

    /// 开始客服对话演示
    async fn start_customer_service_demo(&mut self) -> Result<()> {
        println!("\n🎭 开始智能客服对话演示...");

        // 创建示例客户
        let customer = CustomerProfile {
            id: "cust_001".to_string(),
            name: "张先生".to_string(),
            tier: "VIP".to_string(),
            history: vec![
                "上月购买了企业版服务".to_string(),
                "曾咨询过技术集成问题".to_string(),
            ],
            preferences: {
                let mut prefs = HashMap::new();
                prefs.insert("communication_style".to_string(), "professional".to_string());
                prefs.insert("preferred_language".to_string(), "chinese".to_string());
                prefs
            },
        };

        // 模拟多轮对话
        let conversations = vec![
            "你好，我想了解一下你们最新的AI功能有哪些？",
            "听起来不错，但是我担心数据安全问题，你们是如何保护客户数据的？",
            "我现在的账单好像有问题，能帮我查看一下吗？",
            "谢谢你的帮助，我对服务很满意！",
        ];

        for (i, message) in conversations.iter().enumerate() {
            println!("\n--- 对话轮次 {} ---", i + 1);
            self.handle_customer_query(&customer, message).await?;
        }

        // 生成对话分析报告
        self.generate_conversation_analysis().await?;

        println!("✅ 客服对话演示完成！");
        Ok(())
    }

    /// 处理客户查询
    async fn handle_customer_query(&mut self, customer: &CustomerProfile, query: &str) -> Result<()> {
        println!("👤 客户 ({}): {}", customer.name, query);

        // 步骤 1: 意图分类和情感分析
        let (query_type, mood) = self.classify_query_and_mood(query).await?;
        
        // 步骤 2: 知识库查询
        let relevant_knowledge = self.search_knowledge_base(query, &query_type).await?;
        
        // 步骤 3: 生成个性化回复
        let response = self.generate_personalized_response(
            customer, query, &query_type, &mood, &relevant_knowledge
        ).await?;
        
        // 步骤 4: 记录对话
        self.record_conversation(customer, query, &response, &query_type, &mood).await?;

        println!("🤖 客服: {}", response);
        
        Ok(())
    }

    /// 分类查询意图和情感
    async fn classify_query_and_mood(&self, query: &str) -> Result<(QueryType, CustomerMood)> {
        let classification_prompt = format!(
            "请分析以下客户查询的意图和情感：

            客户查询: \"{}\"

            请判断：
            1. 查询类型（产品咨询/技术支持/账单问题/投诉/一般咨询/紧急）
            2. 客户情感（开心/中性/沮丧/愤怒/困惑）
            3. 紧急程度（1-5级）
            4. 建议的处理方式

            请以结构化方式回答。",
            query
        );

        let messages = vec![user_message(&classification_prompt)];
        let options = AgentGenerateOptions::default();
        let response = self.classifier.generate(&messages, &options).await?;

        // 简化的分类逻辑（实际应用中会更复杂）
        let query_type = if query.contains("功能") || query.contains("产品") {
            QueryType::ProductInquiry
        } else if query.contains("账单") || query.contains("费用") {
            QueryType::Billing
        } else if query.contains("问题") || query.contains("错误") {
            QueryType::TechnicalSupport
        } else if query.contains("投诉") || query.contains("不满") {
            QueryType::Complaint
        } else {
            QueryType::General
        };

        let mood = if query.contains("谢谢") || query.contains("满意") {
            CustomerMood::Happy
        } else if query.contains("问题") || query.contains("担心") {
            CustomerMood::Confused
        } else {
            CustomerMood::Neutral
        };

        println!("   🎯 意图分类: {:?}", query_type);
        println!("   😊 情感分析: {:?}", mood);

        Ok((query_type, mood))
    }

    /// 搜索知识库
    async fn search_knowledge_base(&mut self, query: &str, query_type: &QueryType) -> Result<Vec<KnowledgeItem>> {
        // 简单的关键词匹配（实际应用中会使用向量搜索）
        let mut relevant_items = Vec::new();

        for item in &mut self.knowledge_base {
            let mut score = 0.0;

            // 基于查询类型匹配
            match query_type {
                QueryType::ProductInquiry if item.category.contains("产品") => score += 0.5,
                QueryType::Billing if item.category.contains("账单") => score += 0.5,
                QueryType::TechnicalSupport if item.category.contains("技术") => score += 0.5,
                _ => {}
            }

            // 基于关键词匹配
            for tag in &item.tags {
                if query.contains(tag) {
                    score += 0.3;
                }
            }

            if query.contains(&item.title) {
                score += 0.4;
            }

            item.relevance_score = score;

            if score > 0.3 {
                relevant_items.push(item.clone());
            }
        }

        // 按相关性排序
        relevant_items.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        relevant_items.truncate(3); // 只取前3个最相关的

        println!("   📚 找到相关知识: {} 条", relevant_items.len());
        for item in &relevant_items {
            println!("     - {} (相关性: {:.2})", item.title, item.relevance_score);
        }

        Ok(relevant_items)
    }

    /// 生成个性化回复
    async fn generate_personalized_response(
        &self,
        customer: &CustomerProfile,
        query: &str,
        query_type: &QueryType,
        mood: &CustomerMood,
        knowledge: &[KnowledgeItem],
    ) -> Result<String> {
        let knowledge_context = if knowledge.is_empty() {
            "暂无相关知识库信息".to_string()
        } else {
            knowledge.iter()
                .map(|item| format!("- {}: {}", item.title, item.content))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let response_prompt = format!(
            "请为以下客户生成个性化回复：

            客户信息:
            - 姓名: {}
            - 等级: {}
            - 历史: {:?}

            查询内容: \"{}\"
            查询类型: {:?}
            客户情感: {:?}

            相关知识:
            {}

            请生成：
            1. 友好、专业的回复
            2. 根据客户等级调整服务态度
            3. 根据情感状态调整语气
            4. 提供准确的信息和建议
            5. 如需要，引导下一步操作

            回复应该简洁明了，不超过200字。",
            customer.name,
            customer.tier,
            customer.history,
            query,
            query_type,
            mood,
            knowledge_context
        );

        let messages = vec![user_message(&response_prompt)];
        let options = AgentGenerateOptions::default();
        let response = self.responder.generate(&messages, &options).await?;

        // 根据客户等级添加个性化元素
        let personalized_response = if customer.tier == "VIP" {
            format!("尊敬的VIP客户{}，{}", customer.name, response.response)
        } else {
            format!("{}，{}", customer.name, response.response)
        };

        Ok(personalized_response)
    }

    /// 记录对话
    async fn record_conversation(
        &mut self,
        customer: &CustomerProfile,
        query: &str,
        response: &str,
        query_type: &QueryType,
        mood: &CustomerMood,
    ) -> Result<()> {
        let session_id = format!("session_{}", customer.id);
        
        let session = self.active_sessions.entry(session_id.clone()).or_insert_with(|| {
            ConversationSession {
                session_id: session_id.clone(),
                customer: customer.clone(),
                messages: Vec::new(),
                query_type: None,
                mood: CustomerMood::Neutral,
                satisfaction_score: None,
                resolved: false,
                start_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            }
        });

        session.messages.push(format!("客户: {}", query));
        session.messages.push(format!("客服: {}", response));
        session.query_type = Some(query_type.clone());
        session.mood = mood.clone();

        // 如果客户表示满意，标记为已解决
        if query.contains("满意") || query.contains("谢谢") {
            session.resolved = true;
            session.satisfaction_score = Some(4.5);
        }

        Ok(())
    }

    /// 生成对话分析报告
    async fn generate_conversation_analysis(&self) -> Result<()> {
        println!("\n📊 生成对话分析报告...");

        let total_sessions = self.active_sessions.len();
        let resolved_sessions = self.active_sessions.values().filter(|s| s.resolved).count();
        let avg_satisfaction = self.active_sessions.values()
            .filter_map(|s| s.satisfaction_score)
            .sum::<f32>() / self.active_sessions.len() as f32;

        let analysis_prompt = format!(
            "请分析以下客服对话数据：

            总对话会话: {}
            已解决会话: {}
            解决率: {:.1}%
            平均满意度: {:.2}/5.0

            会话详情:
            {}

            请提供：
            1. 服务质量评估
            2. 客户满意度分析
            3. 常见问题识别
            4. 服务改进建议
            5. 培训需求建议",
            total_sessions,
            resolved_sessions,
            (resolved_sessions as f32 / total_sessions as f32) * 100.0,
            avg_satisfaction,
            self.active_sessions.values()
                .map(|s| format!("会话{}: {:?} - {:?} - 已解决: {}", 
                    s.session_id, s.query_type, s.mood, s.resolved))
                .collect::<Vec<_>>()
                .join("\n")
        );

        let messages = vec![user_message(&analysis_prompt)];
        let options = AgentGenerateOptions::default();
        let response = self.analyzer.generate(&messages, &options).await?;

        println!("✅ 对话分析完成！");
        Ok(())
    }

    /// 显示服务统计
    fn show_service_stats(&self) {
        println!("\n📊 客服系统统计:");
        println!("   💬 活跃会话: {}", self.active_sessions.len());
        println!("   📚 知识库条目: {}", self.knowledge_base.len());
        
        let resolved_count = self.active_sessions.values().filter(|s| s.resolved).count();
        let resolution_rate = if !self.active_sessions.is_empty() {
            (resolved_count as f32 / self.active_sessions.len() as f32) * 100.0
        } else {
            0.0
        };
        println!("   ✅ 问题解决率: {:.1}%", resolution_rate);

        let avg_satisfaction = if !self.active_sessions.is_empty() {
            self.active_sessions.values()
                .filter_map(|s| s.satisfaction_score)
                .sum::<f32>() / self.active_sessions.len() as f32
        } else {
            0.0
        };
        println!("   ⭐ 平均满意度: {:.2}/5.0", avg_satisfaction);

        // 查询类型分布
        let mut type_counts = HashMap::new();
        for session in self.active_sessions.values() {
            if let Some(ref query_type) = session.query_type {
                *type_counts.entry(query_type).or_insert(0) += 1;
            }
        }

        if !type_counts.is_empty() {
            println!("\n📋 查询类型分布:");
            for (query_type, count) in type_counts {
                println!("   {:?}: {} 次", query_type, count);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🤖 LumosAI 智能客服系统示例");
    println!("================================");
    
    // 创建智能客服系统
    let mut customer_service = IntelligentCustomerService::new().await?;
    
    // 开始客服对话演示
    customer_service.start_customer_service_demo().await?;
    
    // 显示服务统计
    customer_service.show_service_stats();
    
    println!("\n🎉 智能客服系统示例运行完成！");
    println!("💡 这个示例展示了如何使用 LumosAI 构建智能客服系统，");
    println!("   实现意图识别、情感分析、知识库查询和个性化回复。");
    
    Ok(())
}
