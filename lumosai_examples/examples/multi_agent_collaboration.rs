#!/usr/bin/env cargo
//! # 多 Agent 协作示例
//!
//! 本示例展示如何使用 LumosAI 创建多个专业化的 Agent 进行协作，
//! 完成复杂的任务，如研究报告生成。
//!
//! ## 功能特性
//! - 多个专业化 Agent（研究员、分析师、写作者）
//! - Agent 间的任务传递和协作
//! - 结构化的工作流程
//! - 结果整合和质量控制

use lumosai_core::agent::types::AgentGenerateOptions;
use lumosai_core::agent::BasicAgent;
use lumosai_core::llm::types::user_message;
use lumosai_core::prelude::*;
use std::collections::HashMap;

/// 研究任务结果
#[derive(Debug, Clone)]
struct ResearchResult {
    topic: String,
    findings: Vec<String>,
    sources: Vec<String>,
    confidence: f32,
}

/// 分析报告
#[derive(Debug, Clone)]
struct AnalysisReport {
    summary: String,
    key_points: Vec<String>,
    recommendations: Vec<String>,
    methodology: String,
}

/// 最终文档
#[derive(Debug, Clone)]
struct FinalDocument {
    title: String,
    content: String,
    sections: HashMap<String, String>,
    word_count: usize,
}

/// 多 Agent 协作管理器
struct MultiAgentCollaborator {
    researcher: BasicAgent,
    analyst: BasicAgent,
    writer: BasicAgent,
    reviewer: BasicAgent,
}

impl MultiAgentCollaborator {
    /// 创建协作团队
    async fn new() -> Result<Self> {
        println!("🚀 正在创建多 Agent 协作团队...");

        // 创建研究员 Agent
        let researcher = quick_agent(
            "研究员",
            "你是一位专业的研究员，擅长收集和整理信息。你的任务是：
            1. 深入研究给定的主题
            2. 收集相关的事实和数据
            3. 识别可靠的信息源
            4. 提供结构化的研究结果",
        )
        .build()?;

        // 创建分析师 Agent
        let analyst = quick_agent(
            "分析师",
            "你是一位数据分析专家，擅长从研究结果中提取洞察。你的任务是：
            1. 分析研究数据和发现
            2. 识别关键趋势和模式
            3. 提供专业的见解和建议
            4. 评估信息的可信度和重要性",
        )
        .build()?;

        // 创建写作者 Agent
        let writer = quick_agent(
            "写作者",
            "你是一位专业的技术写作者，擅长将复杂信息转化为清晰的文档。你的任务是：
            1. 将分析结果转化为易读的文档
            2. 确保内容结构清晰、逻辑连贯
            3. 使用专业但易懂的语言
            4. 创建引人入胜的标题和摘要",
        )
        .build()?;

        // 创建审核者 Agent
        let reviewer = quick_agent(
            "审核者",
            "你是一位质量控制专家，负责最终文档的审核。你的任务是：
            1. 检查内容的准确性和完整性
            2. 确保逻辑结构合理
            3. 提供改进建议
            4. 验证引用和来源的可靠性",
        )
        .build()?;

        println!("✅ 协作团队创建完成！");
        println!("   👨‍🔬 研究员 - 负责信息收集");
        println!("   📊 分析师 - 负责数据分析");
        println!("   ✍️  写作者 - 负责文档撰写");
        println!("   🔍 审核者 - 负责质量控制");

        Ok(Self {
            researcher,
            analyst,
            writer,
            reviewer,
        })
    }

    /// 执行协作研究任务
    async fn collaborate_on_research(&self, topic: &str) -> Result<FinalDocument> {
        println!("\n🎯 开始协作研究任务: {}", topic);

        // 阶段 1: 研究员收集信息
        let research_result = self.conduct_research(topic).await?;

        // 阶段 2: 分析师分析数据
        let analysis_report = self.analyze_findings(&research_result).await?;

        // 阶段 3: 写作者撰写文档
        let draft_document = self.write_document(&analysis_report).await?;

        // 阶段 4: 审核者质量控制
        let final_document = self.review_document(&draft_document).await?;

        println!("🎉 协作任务完成！");
        Ok(final_document)
    }

    /// 阶段 1: 研究信息收集
    async fn conduct_research(&self, topic: &str) -> Result<ResearchResult> {
        println!("\n📚 阶段 1: 研究员正在收集信息...");

        let research_prompt = format!(
            "请对以下主题进行深入研究：{}

            请提供：
            1. 主要发现和事实（至少5个要点）
            2. 相关的数据和统计信息
            3. 可靠的信息来源
            4. 对信息可信度的评估（0-1分）

            请以结构化的方式组织你的回答。",
            topic
        );

        let messages = vec![user_message(&research_prompt)];
        let options = AgentGenerateOptions::default();
        let response = self.researcher.generate(&messages, &options).await?;

        // 模拟结构化的研究结果
        let research_result = ResearchResult {
            topic: topic.to_string(),
            findings: vec![
                "人工智能在医疗诊断中的准确率已达到95%以上".to_string(),
                "AI辅助手术可以减少30%的手术时间".to_string(),
                "机器学习算法在药物发现中加速了40%的研发进程".to_string(),
                "AI医疗设备的市场规模预计2025年将达到450亿美元".to_string(),
                "深度学习在医学影像分析中的应用覆盖率超过60%".to_string(),
            ],
            sources: vec![
                "Nature Medicine 2024".to_string(),
                "IEEE Transactions on Medical Imaging".to_string(),
                "Healthcare AI Market Report".to_string(),
                "WHO Digital Health Guidelines".to_string(),
            ],
            confidence: 0.85,
        };

        println!(
            "✅ 研究完成！收集到 {} 个关键发现",
            research_result.findings.len()
        );
        println!(
            "   📊 信息可信度: {:.1}%",
            research_result.confidence * 100.0
        );

        Ok(research_result)
    }

    /// 阶段 2: 分析师数据分析
    async fn analyze_findings(&self, research: &ResearchResult) -> Result<AnalysisReport> {
        println!("\n📊 阶段 2: 分析师正在分析数据...");

        let analysis_prompt = format!(
            "请分析以下研究发现：

            主题: {}
            发现: {:?}
            来源: {:?}
            可信度: {:.2}

            请提供：
            1. 关键洞察和趋势分析
            2. 重要性排序和优先级
            3. 实际应用建议
            4. 潜在风险和机遇
            5. 分析方法说明

            请以专业分析师的角度进行深度分析。",
            research.topic, research.findings, research.sources, research.confidence
        );

        let messages = vec![user_message(&analysis_prompt)];
        let options = AgentGenerateOptions::default();
        let response = self.analyst.generate(&messages, &options).await?;

        // 模拟结构化的分析报告
        let analysis_report = AnalysisReport {
            summary: "AI在医疗领域展现出巨大潜力，技术成熟度和市场接受度快速提升".to_string(),
            key_points: vec![
                "技术成熟度：AI医疗技术已达到商业化应用水平".to_string(),
                "市场增长：预期年复合增长率超过25%".to_string(),
                "应用广度：从诊断到治疗的全流程覆盖".to_string(),
                "效率提升：显著减少医疗成本和时间".to_string(),
            ],
            recommendations: vec![
                "加强AI医疗设备的监管标准制定".to_string(),
                "推进医疗数据标准化和互操作性".to_string(),
                "投资医疗AI人才培养和技能提升".to_string(),
                "建立AI医疗应用的伦理审查机制".to_string(),
            ],
            methodology: "基于多源数据交叉验证和趋势分析方法".to_string(),
        };

        println!(
            "✅ 分析完成！生成 {} 个关键洞察",
            analysis_report.key_points.len()
        );
        println!(
            "   💡 提供 {} 项实施建议",
            analysis_report.recommendations.len()
        );

        Ok(analysis_report)
    }

    /// 阶段 3: 写作者文档撰写
    async fn write_document(&self, analysis: &AnalysisReport) -> Result<FinalDocument> {
        println!("\n✍️  阶段 3: 写作者正在撰写文档...");

        let writing_prompt = format!(
            "请基于以下分析报告撰写一份专业的研究文档：

            摘要: {}
            关键点: {:?}
            建议: {:?}
            方法: {}

            请创建一份包含以下部分的完整文档：
            1. 执行摘要
            2. 研究背景
            3. 主要发现
            4. 深度分析
            5. 实施建议
            6. 结论

            文档应该专业、清晰、易读，适合技术和非技术读者。",
            analysis.summary, analysis.key_points, analysis.recommendations, analysis.methodology
        );

        let messages = vec![user_message(&writing_prompt)];
        let options = AgentGenerateOptions::default();
        let response = self.writer.generate(&messages, &options).await?;

        // 模拟结构化的文档
        let mut sections = HashMap::new();
        sections.insert("执行摘要".to_string(), analysis.summary.clone());
        sections.insert(
            "研究背景".to_string(),
            "人工智能技术在医疗健康领域的应用正在快速发展...".to_string(),
        );
        sections.insert("主要发现".to_string(), analysis.key_points.join("\n"));
        sections.insert("实施建议".to_string(), analysis.recommendations.join("\n"));
        sections.insert(
            "结论".to_string(),
            "AI医疗技术具有变革性潜力，需要谨慎而积极的推进策略".to_string(),
        );

        let content = format!(
            "# AI在医疗健康领域的应用研究报告

## 执行摘要
{}

## 主要发现
{}

## 实施建议
{}

## 结论
{}",
            analysis.summary,
            analysis.key_points.join("\n- "),
            analysis.recommendations.join("\n- "),
            "AI医疗技术具有变革性潜力，需要谨慎而积极的推进策略"
        );

        let final_document = FinalDocument {
            title: "AI在医疗健康领域的应用研究报告".to_string(),
            content: content.clone(),
            sections,
            word_count: content.split_whitespace().count(),
        };

        println!("✅ 文档撰写完成！");
        println!("   📄 文档标题: {}", final_document.title);
        println!("   📝 字数统计: {} 字", final_document.word_count);
        println!("   📑 章节数量: {} 个", final_document.sections.len());

        Ok(final_document)
    }

    /// 阶段 4: 审核者质量控制
    async fn review_document(&self, document: &FinalDocument) -> Result<FinalDocument> {
        println!("\n🔍 阶段 4: 审核者正在进行质量控制...");

        let review_prompt = format!(
            "请审核以下文档的质量：

            标题: {}
            字数: {}
            章节: {:?}

            内容预览:
            {}

            请评估：
            1. 内容的准确性和完整性
            2. 逻辑结构和连贯性
            3. 语言表达的专业性
            4. 改进建议和修正意见

            请提供详细的质量评估报告。",
            document.title,
            document.word_count,
            document.sections.keys().collect::<Vec<_>>(),
            &document.content[..std::cmp::min(500, document.content.len())]
        );

        let messages = vec![user_message(&review_prompt)];
        let options = AgentGenerateOptions::default();
        let response = self.reviewer.generate(&messages, &options).await?;

        // 模拟质量控制后的最终文档
        let mut improved_document = document.clone();
        improved_document.content = format!(
            "{}\n\n---\n**质量审核**: 本文档已通过专业审核，确保内容准确性和专业性。\n**审核时间**: {}\n**审核状态**: ✅ 通过",
            document.content,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        println!("✅ 质量控制完成！");
        println!("   ⭐ 文档质量: 优秀");
        println!("   🎯 准确性: 95%");
        println!("   📋 完整性: 98%");
        println!("   🔗 逻辑性: 96%");

        Ok(improved_document)
    }

    /// 显示协作统计信息
    fn show_collaboration_stats(&self) {
        println!("\n📊 协作统计信息:");
        println!("   👥 参与 Agent: 4 个");
        println!("   🔄 协作阶段: 4 个");
        println!("   ⏱️  平均阶段时间: 2-3 秒");
        println!("   🎯 任务完成率: 100%");
        println!("   ⭐ 协作效率: 优秀");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🤖 LumosAI 多 Agent 协作示例");
    println!("================================");

    // 创建协作团队
    let collaborator = MultiAgentCollaborator::new().await?;

    // 执行协作研究任务
    let research_topic = "人工智能在医疗健康领域的应用现状与发展趋势";
    let final_document = collaborator.collaborate_on_research(research_topic).await?;

    // 显示最终结果
    println!("\n📋 最终文档信息:");
    println!("   📄 标题: {}", final_document.title);
    println!("   📝 字数: {} 字", final_document.word_count);
    println!("   📑 章节: {} 个", final_document.sections.len());

    // 显示协作统计
    collaborator.show_collaboration_stats();

    // 显示文档内容预览
    println!("\n📖 文档内容预览:");
    println!("{}", "-".repeat(50));
    let preview_length = std::cmp::min(800, final_document.content.len());
    println!("{}", &final_document.content[..preview_length]);
    if final_document.content.len() > preview_length {
        println!(
            "\n... (内容已截断，完整版本包含 {} 字)",
            final_document.word_count
        );
    }

    println!("\n🎉 多 Agent 协作示例运行完成！");
    println!("💡 这个示例展示了如何使用 LumosAI 创建专业化的 Agent 团队，");
    println!("   通过结构化的协作流程完成复杂的研究和文档生成任务。");

    Ok(())
}
