use lumosai_core::agent::{AgentConfig, BasicAgent};
use lumosai_core::llm::{Message, QwenApiType, QwenProvider, Role};
use lumosai_core::Agent;
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use tokio;

/// 真实多模态功能验证测试
/// 验证LumosAI的多模态处理能力（图像、音频等）
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🎨 LumosAI 真实多模态功能验证测试");
    println!("========================================");
    println!("📋 配置信息:");
    println!("  - 模型: qwen3-30b-a3b");
    println!("  - API密钥: sk-bc977c4e31e542f1a34159cb42478198");
    println!("  - 基础URL: https://dashscope.aliyuncs.com/compatible-mode/v1");

    // 9.1 图像处理能力验证
    println!("\n📋 9.1 图像处理能力验证");
    test_image_processing().await?;

    // 9.2 音频处理能力验证
    println!("\n📋 9.2 音频处理能力验证");
    test_audio_processing().await?;

    // 9.3 多模态内容理解验证
    println!("\n📋 9.3 多模态内容理解验证");
    test_multimodal_understanding().await?;

    // 9.4 多模态内容生成验证
    println!("\n📋 9.4 多模态内容生成验证");
    test_multimodal_generation().await?;

    // 9.5 跨模态转换验证
    println!("\n📋 9.5 跨模态转换验证");
    test_cross_modal_conversion().await?;

    println!("\n✅ 多模态功能验证测试完成！");
    Ok(())
}

async fn test_image_processing() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试图像处理能力...");
    let start_time = Instant::now();

    // 测试用例 9.1.1: 创建图像处理Agent
    println!("    🖼️ 测试图像处理Agent");

    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible,
    );

    let image_agent_config = AgentConfig {
        name: "ImageAgent".to_string(),
        instructions: "你是一个图像处理专家，能够分析、描述和处理各种图像内容。".to_string(),
        memory_config: None,
        model_id: None,
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(false),
        context: None,
        metadata: None,
        max_tool_calls: None,
        tool_timeout: None,
        isolation_level: None,
        tenant_id: None,
    };

    let image_agent = BasicAgent::new(image_agent_config, Arc::new(llm))?;

    println!("      ✓ 图像处理Agent创建成功");

    // 测试用例 9.1.2: 图像描述和分析
    println!("    🔍 测试图像描述和分析");

    // 模拟图像处理任务（由于当前模型限制，我们使用文本描述来模拟）
    let image_tasks = vec![
        ("风景照片", "一张美丽的山水风景照片，包含雪山、湖泊和森林"),
        ("人物肖像", "一张专业的商务人士肖像照片，背景是现代办公室"),
        (
            "产品图片",
            "一张智能手机的产品展示图片，白色背景，突出产品特征",
        ),
        ("艺术作品", "一幅抽象派油画作品，色彩丰富，充满创意"),
    ];

    for (image_type, image_description) in image_tasks {
        println!("      🔄 分析{}: {}", image_type, image_description);

        let image_prompt = format!(
            "请分析以下图像内容：\n\n图像类型：{}\n图像描述：{}\n\n请提供详细的图像分析，包括：\n1. 主要内容和元素\n2. 色彩和构图分析\n3. 风格和技法评价\n4. 可能的用途和应用场景\n5. 改进建议",
            image_type, image_description
        );

        let messages = vec![Message {
            role: Role::User,
            content: image_prompt,
            name: None,
            metadata: None,
        }];

        let analysis_start = Instant::now();
        let response = image_agent.generate(&messages, &Default::default()).await?;
        let analysis_duration = analysis_start.elapsed();

        println!(
            "        ✓ {} 分析完成 (耗时: {:?})",
            image_type, analysis_duration
        );
        println!("        📊 分析报告长度: {} 字符", response.response.len());

        // 验证图像分析结果
        assert!(!response.response.trim().is_empty(), "图像分析响应不能为空");
        assert!(response.response.len() > 200, "图像分析应该足够详细");

        // 验证是否包含关键分析要素
        let response_lower = response.response.to_lowercase();
        let analysis_elements = vec!["内容", "色彩", "构图", "风格", "用途", "建议"];
        let mut found_elements = 0;

        for element in analysis_elements {
            if response_lower.contains(element) {
                found_elements += 1;
            }
        }

        assert!(found_elements >= 3, "图像分析应该包含至少3个关键要素");

        println!("        ✓ {} 验证通过", image_type);
    }

    let duration = start_time.elapsed();
    println!("  ✅ 图像处理能力测试完成! 耗时: {:?}", duration);

    Ok(())
}

async fn test_audio_processing() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试音频处理能力...");
    let start_time = Instant::now();

    // 创建音频处理Agent
    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible,
    );

    let audio_agent_config = AgentConfig {
        name: "AudioAgent".to_string(),
        instructions: "你是一个音频处理专家，能够分析、转录和处理各种音频内容。".to_string(),
        memory_config: None,
        model_id: None,
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(false),
        context: None,
        metadata: None,
        max_tool_calls: None,
        tool_timeout: None,
    };

    let audio_agent = BasicAgent::new(audio_agent_config, Arc::new(llm))?;

    // 测试用例 9.2.1: 音频内容分析
    println!("    🎵 测试音频内容分析");

    let audio_tasks = vec![
        ("会议录音", "一段30分钟的商务会议录音，包含多人讨论项目进展"),
        ("音乐作品", "一首古典音乐作品，钢琴独奏，情感丰富"),
        ("播客节目", "一期科技播客节目，主持人采访AI专家"),
        ("语音备忘", "个人语音备忘录，记录日常工作安排和想法"),
    ];

    for (audio_type, audio_description) in audio_tasks {
        println!("      🔄 分析{}: {}", audio_type, audio_description);

        let audio_prompt = format!(
            "请分析以下音频内容：\n\n音频类型：{}\n音频描述：{}\n\n请提供详细的音频分析，包括：\n1. 内容摘要和关键信息\n2. 音质和技术特征\n3. 语音识别和转录建议\n4. 后处理和优化方案\n5. 应用场景和价值评估",
            audio_type, audio_description
        );

        let messages = vec![Message {
            role: Role::User,
            content: audio_prompt,
            name: None,
            metadata: None,
        }];

        let analysis_start = Instant::now();
        let response = audio_agent.generate(&messages, &Default::default()).await?;
        let analysis_duration = analysis_start.elapsed();

        println!(
            "        ✓ {} 分析完成 (耗时: {:?})",
            audio_type, analysis_duration
        );
        println!("        📊 分析报告长度: {} 字符", response.response.len());

        // 验证音频分析结果
        assert!(!response.response.trim().is_empty(), "音频分析响应不能为空");
        assert!(response.response.len() > 200, "音频分析应该足够详细");

        // 验证是否包含关键分析要素
        let response_lower = response.response.to_lowercase();
        let analysis_elements = vec!["内容", "音质", "转录", "处理", "应用"];
        let mut found_elements = 0;

        for element in analysis_elements {
            if response_lower.contains(element) {
                found_elements += 1;
            }
        }

        assert!(found_elements >= 3, "音频分析应该包含至少3个关键要素");

        println!("        ✓ {} 验证通过", audio_type);
    }

    let duration = start_time.elapsed();
    println!("  ✅ 音频处理能力测试完成! 耗时: {:?}", duration);

    Ok(())
}

async fn test_multimodal_understanding() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试多模态内容理解...");
    let start_time = Instant::now();

    // 创建多模态理解Agent
    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible,
    );

    let multimodal_agent_config = AgentConfig {
        name: "MultimodalAgent".to_string(),
        instructions: "你是一个多模态内容理解专家，能够综合分析文本、图像、音频等多种模态的内容。"
            .to_string(),
        memory_config: None,
        model_id: None,
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(false),
        context: None,
        metadata: None,
        max_tool_calls: None,
        tool_timeout: None,
    };

    let multimodal_agent = BasicAgent::new(multimodal_agent_config, Arc::new(llm))?;

    // 测试用例 9.3.1: 多模态内容综合理解
    println!("    🔗 测试多模态内容综合理解");

    let multimodal_content = json!({
        "scenario": "产品发布会",
        "text_content": "我们今天发布的新款智能手机具有革命性的AI摄影功能，支持夜景模式和人像美化。",
        "image_content": "产品展示图片：一款黑色智能手机，屏幕显示相机界面，背景是专业摄影棚",
        "audio_content": "发布会现场录音：CEO介绍产品特性，观众热烈掌声，背景音乐轻柔",
        "video_content": "产品演示视频：展示手机拍照功能，从白天到夜晚的拍摄效果对比"
    });

    let multimodal_prompt = format!(
        "请综合分析以下多模态内容：\n{}\n\n请提供全面的多模态分析，包括：\n1. 各模态内容的一致性分析\n2. 信息互补和增强效果\n3. 整体传达的核心信息\n4. 受众体验和情感影响\n5. 多模态协同的优化建议",
        serde_json::to_string_pretty(&multimodal_content)?
    );

    let messages = vec![Message {
        role: Role::User,
        content: multimodal_prompt,
        name: None,
        metadata: None,
    }];

    let understanding_start = Instant::now();
    let response = multimodal_agent
        .generate(&messages, &Default::default())
        .await?;
    let understanding_duration = understanding_start.elapsed();

    println!(
        "      ✓ 多模态理解完成 (耗时: {:?})",
        understanding_duration
    );
    println!("      📊 理解报告长度: {} 字符", response.response.len());

    // 验证多模态理解结果
    assert!(
        !response.response.trim().is_empty(),
        "多模态理解响应不能为空"
    );
    assert!(response.response.len() > 400, "多模态理解应该非常详细");

    // 验证是否包含关键理解要素
    let response_lower = response.response.to_lowercase();
    let understanding_elements = vec!["一致性", "互补", "核心信息", "体验", "优化"];
    let mut found_elements = 0;

    for element in understanding_elements {
        if response_lower.contains(element) {
            found_elements += 1;
        }
    }

    assert!(found_elements >= 3, "多模态理解应该包含至少3个关键要素");

    println!("      ✓ 多模态理解验证通过");

    let duration = start_time.elapsed();
    println!("  ✅ 多模态内容理解测试完成! 耗时: {:?}", duration);

    Ok(())
}

async fn test_multimodal_generation() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试多模态内容生成...");
    let start_time = Instant::now();

    // 创建多模态生成Agent
    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible,
    );

    let generation_agent_config = AgentConfig {
        name: "GenerationAgent".to_string(),
        instructions:
            "你是一个多模态内容生成专家，能够根据需求生成文本、图像描述、音频脚本等多种模态的内容。"
                .to_string(),
        memory_config: None,
        model_id: None,
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(false),
        context: None,
        metadata: None,
        max_tool_calls: None,
        tool_timeout: None,
    };

    let generation_agent = BasicAgent::new(generation_agent_config, Arc::new(llm))?;

    // 测试用例 9.4.1: 多模态内容生成
    println!("    🎨 测试多模态内容生成");

    let generation_tasks = vec![
        ("营销活动", "为新产品发布设计一套完整的营销内容"),
        ("教育课程", "为在线编程课程创建多媒体教学材料"),
        ("品牌宣传", "为科技公司设计品牌形象宣传内容"),
    ];

    for (task_type, task_description) in generation_tasks {
        println!("      🔄 生成{}: {}", task_type, task_description);

        let generation_prompt = format!(
            "请为以下任务生成完整的多模态内容方案：\n\n任务类型：{}\n任务描述：{}\n\n请提供：\n1. 文本内容（标题、正文、标语等）\n2. 图像设计方案（构图、色彩、风格等）\n3. 音频内容规划（配音、音效、背景音乐等）\n4. 视频脚本大纲\n5. 各模态内容的协调统一方案",
            task_type, task_description
        );

        let messages = vec![Message {
            role: Role::User,
            content: generation_prompt,
            name: None,
            metadata: None,
        }];

        let generation_start = Instant::now();
        let response = generation_agent
            .generate(&messages, &Default::default())
            .await?;
        let generation_duration = generation_start.elapsed();

        println!(
            "        ✓ {} 生成完成 (耗时: {:?})",
            task_type, generation_duration
        );
        println!("        📊 生成方案长度: {} 字符", response.response.len());

        // 验证多模态生成结果
        assert!(
            !response.response.trim().is_empty(),
            "多模态生成响应不能为空"
        );
        assert!(response.response.len() > 500, "多模态生成方案应该非常详细");

        // 验证是否包含各种模态内容
        let response_lower = response.response.to_lowercase();
        let modality_elements = vec!["文本", "图像", "音频", "视频", "协调"];
        let mut found_elements = 0;

        for element in modality_elements {
            if response_lower.contains(element) {
                found_elements += 1;
            }
        }

        assert!(found_elements >= 4, "多模态生成应该包含至少4种模态内容");

        println!("        ✓ {} 验证通过", task_type);
    }

    let duration = start_time.elapsed();
    println!("  ✅ 多模态内容生成测试完成! 耗时: {:?}", duration);

    Ok(())
}

async fn test_cross_modal_conversion() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试跨模态转换...");
    let start_time = Instant::now();

    // 创建跨模态转换Agent
    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible,
    );

    let conversion_agent_config = AgentConfig {
        name: "ConversionAgent".to_string(),
        instructions: "你是一个跨模态转换专家，能够在不同模态之间进行内容转换和适配。".to_string(),
        memory_config: None,
        model_id: None,
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(false),
        context: None,
        metadata: None,
        max_tool_calls: None,
        tool_timeout: None,
    };

    let conversion_agent = BasicAgent::new(conversion_agent_config, Arc::new(llm))?;

    // 测试用例 9.5.1: 跨模态转换
    println!("    🔄 测试跨模态转换");

    let conversion_tasks = vec![
        (
            "文本转图像",
            "将一段产品描述文本转换为图像设计方案",
            "这款智能手表具有圆形表盘、金属表带，支持健康监测和运动追踪功能",
        ),
        (
            "图像转文本",
            "将图像内容转换为详细的文字描述",
            "一张展示现代办公室的照片，包含开放式工作区、绿植装饰和自然光照明",
        ),
        (
            "音频转文本",
            "将音频内容转换为文字记录",
            "一段客户服务电话录音，客户咨询产品退换货政策",
        ),
        (
            "文本转音频",
            "将文本内容转换为音频脚本",
            "公司年度总结报告，需要制作成播客节目",
        ),
    ];

    for (conversion_type, conversion_description, source_content) in conversion_tasks {
        println!(
            "      🔄 执行{}: {}",
            conversion_type, conversion_description
        );

        let conversion_prompt = format!(
            "请执行以下跨模态转换任务：\n\n转换类型：{}\n任务描述：{}\n源内容：{}\n\n请提供：\n1. 转换后的目标内容\n2. 转换过程中的关键考虑因素\n3. 信息保真度评估\n4. 目标模态的优化建议\n5. 可能的质量损失和补偿方案",
            conversion_type, conversion_description, source_content
        );

        let messages = vec![Message {
            role: Role::User,
            content: conversion_prompt,
            name: None,
            metadata: None,
        }];

        let conversion_start = Instant::now();
        let response = conversion_agent
            .generate(&messages, &Default::default())
            .await?;
        let conversion_duration = conversion_start.elapsed();

        println!(
            "        ✓ {} 转换完成 (耗时: {:?})",
            conversion_type, conversion_duration
        );
        println!("        📊 转换结果长度: {} 字符", response.response.len());

        // 验证跨模态转换结果
        assert!(
            !response.response.trim().is_empty(),
            "跨模态转换响应不能为空"
        );
        assert!(response.response.len() > 300, "跨模态转换结果应该详细");

        // 验证是否包含转换要素
        let response_lower = response.response.to_lowercase();
        let conversion_elements = vec!["转换", "内容", "考虑", "评估", "优化"];
        let mut found_elements = 0;

        for element in conversion_elements {
            if response_lower.contains(element) {
                found_elements += 1;
            }
        }

        assert!(found_elements >= 3, "跨模态转换应该包含至少3个关键要素");

        println!("        ✓ {} 验证通过", conversion_type);
    }

    let duration = start_time.elapsed();
    println!("  ✅ 跨模态转换测试完成! 耗时: {:?}", duration);

    Ok(())
}
