//! AI能力扩展演示
//!
//! 展示Lumos.ai AI能力扩展的完整功能

use lumosai_ai_extensions::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧠 Lumos.ai AI能力扩展演示");
    println!("=" .repeat(50));

    // 演示多模态处理
    demo_multimodal_processing().await?;

    // 演示推理能力
    demo_reasoning_capabilities().await?;

    // 演示领域适配
    demo_domain_adaptation().await?;

    // 演示知识图谱
    demo_knowledge_graph().await?;

    // 演示模型推理
    demo_model_inference().await?;

    // 演示综合AI能力
    demo_integrated_ai_capabilities().await?;

    println!("\n🎉 AI能力扩展演示完成！");
    println!("\n🚀 支持的AI能力:");
    println!("  👁️  多模态处理: 图像、音频、视频、文档");
    println!("  🧠 高级推理: 逻辑、因果、类比、归纳、演绎");
    println!("  🏢 领域适配: 金融、医疗、教育、法律等专业领域");
    println!("  🕸️  知识图谱: 实体识别、关系抽取、知识推理");
    println!("  ⚡ 模型推理: ONNX、PyTorch、TensorFlow模型支持");

    Ok(())
}

/// 演示多模态处理
async fn demo_multimodal_processing() -> Result<()> {
    println!("\n👁️  演示：多模态处理");
    println!("-" .repeat(30));

    // 创建多模态处理器
    let config = AiCapabilityConfig::default();
    let processor = MultimodalProcessor::new(config.multimodal).await?;

    // 演示图像处理
    println!("🖼️  图像处理:");
    let image_input = MultimodalInput::Image {
        data: vec![0u8; 1024], // 模拟图像数据
        format: "png".to_string(),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("source".to_string(), "camera".to_string());
            meta
        },
    };

    match processor.process(image_input).await {
        Ok(result) => {
            println!("   ✅ 处理成功");
            println!("   📊 结果数量: {}", result.results.len());
            println!("   🎯 置信度: {:.2}", result.confidence);
            println!("   ⏱️  处理时间: {}ms", result.processing_time_ms);

            for (i, modal_result) in result.results.iter().enumerate() {
                match modal_result {
                    ModalityResult::Vision(vision_result) => {
                        println!("   🔍 视觉分析 {}:", i + 1);
                        println!("      图像尺寸: {}x{}",
                                vision_result.image_info.width,
                                vision_result.image_info.height);
                        if let Some(ocr) = &vision_result.ocr_result {
                            println!("      OCR文本: {}", ocr.text);
                        }
                        if let Some(analysis) = &vision_result.analysis_result {
                            println!("      图像描述: {}", analysis.description);
                            println!("      标签数量: {}", analysis.tags.len());
                        }
                    }
                    _ => {}
                }
            }
        }
        Err(e) => println!("   ❌ 处理失败: {}", e),
    }

    // 演示音频处理
    println!("\n🎵 音频处理:");
    let audio_input = MultimodalInput::Audio {
        data: vec![0u8; 2048], // 模拟音频数据
        format: "wav".to_string(),
        sample_rate: 16000,
        channels: 1,
        metadata: HashMap::new(),
    };

    match processor.process(audio_input).await {
        Ok(result) => {
            println!("   ✅ 处理成功");
            println!("   🎯 置信度: {:.2}", result.confidence);

            for modal_result in &result.results {
                if let ModalityResult::Audio(audio_result) = modal_result {
                    println!("   🎤 音频分析:");
                    if let Some(transcription) = &audio_result.transcription {
                        println!("      转录文本: {}", transcription);
                    }
                    if let Some(language) = &audio_result.language {
                        println!("      检测语言: {}", language);
                    }
                    println!("      音频时长: {:.1}秒", audio_result.duration_seconds);
                }
            }
        }
        Err(e) => println!("   ❌ 处理失败: {}", e),
    }

    // 演示文本处理
    println!("\n📝 文本处理:");
    let text_input = MultimodalInput::Text {
        content: "这是一个很棒的AI系统，它能够处理多种模态的数据，包括文本、图像和音频。".to_string(),
        language: Some("zh".to_string()),
        metadata: HashMap::new(),
    };

    match processor.process(text_input).await {
        Ok(result) => {
            println!("   ✅ 处理成功");

            for modal_result in &result.results {
                if let ModalityResult::Text(text_result) = modal_result {
                    println!("   📄 文本分析:");
                    if let Some(language) = &text_result.detected_language {
                        println!("      检测语言: {}", language);
                    }
                    if let Some(sentiment) = &text_result.sentiment {
                        println!("      情感分析: {} (置信度: {:.2})", sentiment.label, sentiment.confidence);
                    }
                    println!("      实体数量: {}", text_result.entities.len());
                    println!("      关键词数量: {}", text_result.keywords.len());

                    if !text_result.keywords.is_empty() {
                        println!("      关键词:");
                        for keyword in text_result.keywords.iter().take(3) {
                            println!("        - {} (分数: {:.2})", keyword.text, keyword.score);
                        }
                    }
                }
            }
        }
        Err(e) => println!("   ❌ 处理失败: {}", e),
    }

    Ok(())
}

/// 演示推理能力
async fn demo_reasoning_capabilities() -> Result<()> {
    println!("\n🧠 演示：推理能力");
    println!("-" .repeat(30));

    // 创建推理引擎
    let config = AiCapabilityConfig::default();
    let reasoning_engine = ReasoningEngine::new(config.reasoning).await?;

    // 演示逻辑推理
    println!("🔍 逻辑推理:");
    let logical_query = ReasoningQuery {
        query_type: ReasoningType::Logical,
        premises: vec![
            "所有人都是会死的".to_string(),
            "苏格拉底是人".to_string(),
        ],
        question: "苏格拉底是否会死？".to_string(),
        context: HashMap::new(),
        parameters: ReasoningParameters::default(),
    };

    match reasoning_engine.reason(logical_query).await {
        Ok(result) => {
            println!("   ✅ 推理成功");
            println!("   📝 结论: {}", result.conclusion);
            println!("   🎯 置信度: {:.2}", result.confidence);
            println!("   🔗 推理步数: {}", result.reasoning_chain.len());
            println!("   ⏱️  推理时间: {}ms", result.statistics.reasoning_time_ms);

            if !result.reasoning_chain.is_empty() {
                println!("   🧩 推理链:");
                for step in &result.reasoning_chain {
                    println!("      步骤{}: {} -> {}",
                            step.step_number,
                            step.rule,
                            step.output_conclusion);
                }
            }
        }
        Err(e) => println!("   ❌ 推理失败: {}", e),
    }

    // 演示因果推理
    println!("\n🔄 因果推理:");
    let causal_query = ReasoningQuery {
        query_type: ReasoningType::Causal,
        premises: vec![
            "下雨时地面会湿".to_string(),
            "地面是湿的".to_string(),
        ],
        question: "是否下雨了？".to_string(),
        context: HashMap::new(),
        parameters: ReasoningParameters::default(),
    };

    match reasoning_engine.reason(causal_query).await {
        Ok(result) => {
            println!("   ✅ 推理成功");
            println!("   📝 结论: {}", result.conclusion);
            println!("   🎯 置信度: {:.2}", result.confidence);
            println!("   📊 探索假设数: {}", result.statistics.hypotheses_explored);
        }
        Err(e) => println!("   ❌ 推理失败: {}", e),
    }

    // 演示类比推理
    println!("\n🔗 类比推理:");
    let analogical_query = ReasoningQuery {
        query_type: ReasoningType::Analogical,
        premises: vec![
            "原子核就像太阳系的中心".to_string(),
            "电子围绕原子核运动".to_string(),
        ],
        question: "电子的运动类似于什么？".to_string(),
        context: HashMap::new(),
        parameters: ReasoningParameters::default(),
    };

    match reasoning_engine.reason(analogical_query).await {
        Ok(result) => {
            println!("   ✅ 推理成功");
            println!("   📝 结论: {}", result.conclusion);
            println!("   🎯 置信度: {:.2}", result.confidence);
        }
        Err(e) => println!("   ❌ 推理失败: {}", e),
    }

    Ok(())
}

/// 演示领域适配
async fn demo_domain_adaptation() -> Result<()> {
    println!("\n🏢 演示：领域适配");
    println!("-" .repeat(30));

    // 创建领域适配器
    let config = AiCapabilityConfig::default();
    let domain_adapter = DomainAdapter::new(config.domain).await?;

    let domains = vec![
        ("finance", "分析这家公司的财务状况和投资价值"),
        ("healthcare", "患者出现发热、咳嗽症状，请提供诊断建议"),
        ("education", "如何提高学生的数学学习效果"),
        ("legal", "分析这份合同的法律风险和注意事项"),
    ];

    for (domain, content) in domains {
        println!("🎯 {} 领域适配:", domain);

        let input = DomainInput {
            content: content.to_string(),
            domain: domain.to_string(),
            context: HashMap::new(),
        };

        match domain_adapter.adapt(domain, input).await {
            Ok(result) => {
                println!("   ✅ 适配成功");
                println!("   📝 适配内容: {}", result.adapted_content);
                println!("   🎯 置信度: {:.2}", result.confidence);
                println!("   💡 领域洞察: {:?}", result.domain_insights);
            }
            Err(e) => println!("   ❌ 适配失败: {}", e),
        }
        println!();
    }

    Ok(())
}

/// 演示知识图谱
async fn demo_knowledge_graph() -> Result<()> {
    println!("\n🕸️  演示：知识图谱");
    println!("-" .repeat(30));

    // 创建知识图谱
    let config = AiCapabilityConfig::default();
    let knowledge_graph = KnowledgeGraph::new(config.knowledge).await?;

    println!("🔍 知识查询:");
    let query = KnowledgeQuery {
        query_type: "entity_search".to_string(),
        entities: vec!["人工智能".to_string(), "机器学习".to_string()],
        relations: vec!["包含".to_string(), "应用于".to_string()],
        constraints: HashMap::new(),
    };

    match knowledge_graph.query(query).await {
        Ok(result) => {
            println!("   ✅ 查询成功");
            println!("   📊 实体数量: {}", result.entities.len());
            println!("   🔗 关系数量: {}", result.relations.len());
            println!("   🎯 置信度: {:.2}", result.confidence);

            if !result.entities.is_empty() {
                println!("   🏷️  实体列表:");
                for entity in &result.entities {
                    println!("      - {} (类型: {})", entity.name, entity.entity_type);
                }
            }

            if !result.relations.is_empty() {
                println!("   🔗 关系列表:");
                for relation in &result.relations {
                    println!("      - {} -> {} ({})",
                            relation.source_entity,
                            relation.target_entity,
                            relation.relation_type);
                }
            }
        }
        Err(e) => println!("   ❌ 查询失败: {}", e),
    }

    Ok(())
}

/// 演示模型推理
async fn demo_model_inference() -> Result<()> {
    println!("\n⚡ 演示：模型推理");
    println!("-" .repeat(30));

    // 创建推理引擎
    let config = AiCapabilityConfig::default();
    let inference_engine = InferenceEngine::new(config.inference).await?;

    println!("🤖 模型推理:");
    let input = InferenceInput {
        data: serde_json::json!({
            "image": [0.1, 0.2, 0.3, 0.4],
            "metadata": {
                "width": 224,
                "height": 224,
                "channels": 3
            }
        }),
        input_format: "tensor".to_string(),
        preprocessing: None,
    };

    match inference_engine.infer("image_classifier", input).await {
        Ok(result) => {
            println!("   ✅ 推理成功");
            println!("   📊 结果: {}", result.result);
            println!("   🎯 置信度: {:.2}", result.confidence);
            println!("   ⏱️  推理时间: {}ms", result.inference_time_ms);
            println!("   🤖 模型信息:");
            println!("      名称: {}", result.model_info.model_name);
            println!("      版本: {}", result.model_info.model_version);
            println!("      后端: {}", result.model_info.backend);
            println!("      输入形状: {:?}", result.model_info.input_shape);
            println!("      输出形状: {:?}", result.model_info.output_shape);
        }
        Err(e) => println!("   ❌ 推理失败: {}", e),
    }

    Ok(())
}

/// 演示综合AI能力
async fn demo_integrated_ai_capabilities() -> Result<()> {
    println!("\n🚀 演示：综合AI能力");
    println!("-" .repeat(30));

    // 创建AI扩展管理器
    let config = AiCapabilityConfig::default();
    let ai_manager = AiExtensionManager::new(config).await?;

    println!("🎯 综合场景：智能文档分析");

    // 1. 多模态处理 - 文档
    println!("   1️⃣ 文档处理...");
    let document_input = MultimodalInput::Document {
        data: vec![0u8; 4096], // 模拟PDF数据
        format: "pdf".to_string(),
        filename: "financial_report.pdf".to_string(),
        metadata: HashMap::new(),
    };

    let multimodal_result = ai_manager.process_multimodal(document_input).await?;
    println!("      ✅ 文档处理完成，置信度: {:.2}", multimodal_result.confidence);

    // 2. 领域适配 - 金融领域
    println!("   2️⃣ 金融领域适配...");
    let domain_input = DomainInput {
        content: "公司本季度营收增长15%，净利润率提升至12%".to_string(),
        domain: "finance".to_string(),
        context: HashMap::new(),
    };

    let domain_result = ai_manager.adapt_domain("finance", domain_input).await?;
    println!("      ✅ 领域适配完成，置信度: {:.2}", domain_result.confidence);

    // 3. 推理分析
    println!("   3️⃣ 推理分析...");
    let reasoning_query = ReasoningQuery {
        query_type: ReasoningType::Inductive,
        premises: vec![
            "营收增长15%".to_string(),
            "净利润率提升至12%".to_string(),
            "市场份额稳定".to_string(),
        ],
        question: "公司未来发展趋势如何？".to_string(),
        context: HashMap::new(),
        parameters: ReasoningParameters::default(),
    };

    let reasoning_result = ai_manager.reason(reasoning_query).await?;
    println!("      ✅ 推理分析完成，置信度: {:.2}", reasoning_result.confidence);

    // 4. 综合结论
    println!("   4️⃣ 综合结论:");
    println!("      📊 文档分析: 成功提取关键财务数据");
    println!("      🏢 领域适配: 应用金融分析框架");
    println!("      🧠 推理结论: {}", reasoning_result.conclusion);
    println!("      🎯 整体置信度: {:.2}",
            (multimodal_result.confidence + domain_result.confidence + reasoning_result.confidence) / 3.0);

    println!("\n💡 AI能力特性总结:");
    println!("   ✅ 多模态融合: 文本、图像、音频、视频统一处理");
    println!("   ✅ 智能推理: 6种推理类型，支持复杂逻辑分析");
    println!("   ✅ 领域专精: 金融、医疗、教育、法律等专业适配");
    println!("   ✅ 知识增强: 实体识别、关系抽取、知识推理");
    println!("   ✅ 高效推理: 多后端支持，优化的推理性能");
    println!("   ✅ 可扩展性: 模块化设计，支持自定义扩展");

    Ok(())
}
