//! 多模态能力演示程序
//!
//! 演示语音处理（STT/TTS）和视觉处理（图像理解/生成）功能

use lumosai_multimodal::{
    GenerationOptions, ImageSize, OpenAIVision, OpenAIVoice, SynthesisOptions,
    TranscriptionOptions, VisionOptions, VisionProvider, VoiceProvider,
};

#[tokio::main]
async fn main() {
    println!("🎭 LumosAI 多模态能力演示\n");
    println!("{}", "=".repeat(80));
    println!();

    // 注意：这些示例需要有效的 OpenAI API 密钥
    // 可以通过环境变量 OPENAI_API_KEY 设置
    let api_key =
        std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "your-api-key-here".to_string());

    if api_key == "your-api-key-here" {
        println!("⚠️  请设置环境变量 OPENAI_API_KEY 以运行实际测试");
        println!("   export OPENAI_API_KEY=your-api-key");
        println!();
        println!("📝 以下是功能演示（使用 Mock 数据）：\n");
        demo_capabilities();
        return;
    }

    // 测试 1: 语音处理
    println!("🎤 测试 1: 语音处理");
    println!("{}", "-".repeat(80));
    test_voice_processing(&api_key).await;
    println!();

    // 测试 2: 视觉处理
    println!("👁️  测试 2: 视觉处理");
    println!("{}", "-".repeat(80));
    test_vision_processing(&api_key).await;
    println!();

    // 总结
    println!("{}", "=".repeat(80));
    println!("✅ 多模态能力演示完成！");
    println!("{}", "=".repeat(80));
    println!();
    println!("📊 功能统计:");
    println!("  - 语音识别 (STT): OpenAI Whisper");
    println!("  - 语音合成 (TTS): OpenAI TTS");
    println!("  - 图像理解: GPT-4V");
    println!("  - 图像生成: DALL-E 3");
    println!();
    println!("💡 使用建议:");
    println!("  1. 语音处理: 支持多种音频格式，最大 25MB");
    println!("  2. 视觉处理: 支持 PNG/JPEG/WebP/GIF，最大 20MB");
    println!("  3. 多模态对话: 可以同时处理文本、图像、音频");
    println!();
    println!("🚀 下一步:");
    println!("  - 集成更多提供商（Azure, Google, etc.)");
    println!("  - 支持流式处理");
    println!("  - 添加本地模型支持");
    println!("{}", "=".repeat(80));
}

/// 演示功能能力
fn demo_capabilities() {
    println!("📋 语音处理能力:");
    println!("  - 支持语言: 中文、英文、日语、韩语等 10+ 种");
    println!("  - 支持格式: MP3, WAV, FLAC, M4A, WebM");
    println!("  - 语音模型: alloy, echo, fable, onyx, nova, shimmer");
    println!("  - 最大时长: 10 分钟");
    println!("  - 最大文件: 25 MB");
    println!();

    println!("📋 视觉处理能力:");
    println!("  - 支持格式: PNG, JPEG, WebP, GIF");
    println!("  - 图像理解: ✅ (GPT-4V)");
    println!("  - 图像生成: ✅ (DALL-E 3)");
    println!("  - 图像编辑: ✅");
    println!("  - 最大尺寸: 4096x4096");
    println!("  - 最大文件: 20 MB");
    println!();

    println!("📋 API 使用示例:");
    println!();
    println!("```rust");
    println!("// 语音识别");
    println!("let voice = OpenAIVoice::new(api_key);");
    println!("let text = voice.transcribe_file(\"audio.mp3\", None).await?;");
    println!();
    println!("// 语音合成");
    println!("let audio = voice.synthesize(\"Hello\", None).await?;");
    println!();
    println!("// 图像理解");
    println!("let vision = OpenAIVision::new(api_key);");
    println!("let desc = vision.describe_image(\"image.jpg\", \"What's this?\", None).await?;");
    println!();
    println!("// 图像生成");
    println!("let url = vision.generate_image(\"A sunset\", None).await?;");
    println!("```");
}

/// 测试语音处理
async fn test_voice_processing(api_key: &str) {
    let voice = OpenAIVoice::new(api_key);

    println!("提供商: {}", voice.name());

    let caps = voice.capabilities();
    println!("支持语言: {} 种", caps.supported_languages.len());
    println!("支持格式: {} 种", caps.supported_formats.len());
    println!("支持语音: {} 种", caps.supported_voices.len());
    println!();

    // 场景 1: 文本转语音
    println!("场景 1: 文本转语音");
    let text = "你好，这是一个语音合成测试。";
    let options = SynthesisOptions {
        voice: "alloy".to_string(),
        model: Some("tts-1".to_string()),
        speed: Some(1.0),
        response_format: Some("mp3".to_string()),
    };

    match voice.synthesize(text, Some(options)).await {
        Ok(audio_data) => {
            println!("  ✅ 合成成功");
            println!("  音频大小: {} 字节", audio_data.len());
        }
        Err(e) => {
            println!("  ❌ 合成失败: {}", e);
        }
    }
    println!();

    // 场景 2: 语音转文本（需要实际音频文件）
    println!("场景 2: 语音转文本");
    println!("  ℹ️  需要实际音频文件，跳过此测试");
    println!("  示例: voice.transcribe_file(\"audio.mp3\", None).await");
}

/// 测试视觉处理
async fn test_vision_processing(api_key: &str) {
    let vision = OpenAIVision::new(api_key);

    println!("提供商: {}", vision.name());

    let caps = vision.capabilities();
    println!("支持格式: {} 种", caps.supported_formats.len());
    println!(
        "图像理解: {}",
        if caps.supports_understanding {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "图像生成: {}",
        if caps.supports_generation {
            "✅"
        } else {
            "❌"
        }
    );
    println!();

    // 场景 1: 图像生成
    println!("场景 1: 图像生成");
    let prompt = "A beautiful sunset over the ocean";
    let options = GenerationOptions {
        model: Some("dall-e-3".to_string()),
        size: ImageSize::Large,
        quality: Some("standard".to_string()),
        n: Some(1),
        style: Some("vivid".to_string()),
    };

    match vision.generate_image(prompt, Some(options)).await {
        Ok(image_url) => {
            println!("  ✅ 生成成功");
            println!("  图像 URL: {}", image_url);
        }
        Err(e) => {
            println!("  ❌ 生成失败: {}", e);
        }
    }
    println!();

    // 场景 2: 图像理解（需要实际图像文件或 URL）
    println!("场景 2: 图像理解");
    println!("  ℹ️  需要实际图像文件，跳过此测试");
    println!("  示例: vision.describe_image(\"image.jpg\", \"What's in this image?\", None).await");
    println!();

    // 场景 3: 图像理解（使用 URL）
    println!("场景 3: 图像理解（使用公开 URL）");
    let image_url = "https://upload.wikimedia.org/wikipedia/commons/thumb/d/dd/Gfp-wisconsin-madison-the-nature-boardwalk.jpg/2560px-Gfp-wisconsin-madison-the-nature-boardwalk.jpg";
    let prompt = "What's in this image?";
    let options = VisionOptions {
        model: Some("gpt-4o".to_string()),
        max_tokens: Some(300),
        detail: Some("auto".to_string()),
    };

    match vision
        .describe_image_url(image_url, prompt, Some(options))
        .await
    {
        Ok(description) => {
            println!("  ✅ 理解成功");
            println!("  描述: {}", description);
        }
        Err(e) => {
            println!("  ❌ 理解失败: {}", e);
        }
    }
}
