//! 中文LLM提供商示例
//! 
//! 这个示例展示了如何使用新添加的中文LLM提供商：
//! - 智谱AI (GLM)
//! - 百度ERNIE

use lumosai_core::llm::{
    LlmProvider, LlmOptions, Message, Role,
    zhipu::ZhipuProvider,
    baidu::BaiduProvider,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 LumosAI 中文LLM提供商示例");
    println!("================================");

    // 智谱AI (GLM) 示例
    test_zhipu_provider().await?;
    
    // 百度ERNIE 示例
    test_baidu_provider().await?;
    
    // 演示统一接口使用
    demo_unified_interface().await?;

    println!("\n🎉 所有中文LLM提供商测试完成！");
    Ok(())
}

async fn test_zhipu_provider() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📡 智谱AI (GLM) 示例:");
    println!("-------------------");
    
    let zhipu_provider = ZhipuProvider::new(
        "your-zhipu-api-key".to_string(),
        Some("glm-4".to_string())
    );
    
    println!("✅ 智谱AI Provider 创建成功");
    println!("   - 提供商名称: {}", zhipu_provider.name());
    println!("   - 模型: {}", zhipu_provider.model());
    println!("   - 基础URL: {}", zhipu_provider.base_url());
    println!("   - 支持函数调用: {}", zhipu_provider.supports_function_calling());
    
    // 创建测试消息
    let messages = vec![
        Message {
            role: Role::System,
            content: "你是一个有用的AI助手，请用中文回答问题。".to_string(),
            metadata: None,
            name: None,
        },
        Message {
            role: Role::User,
            content: "你好，请简单介绍一下你自己。".to_string(),
            metadata: None,
            name: None,
        },
    ];
    
    // 创建测试选项
    let options = LlmOptions {
        model: None,
        temperature: Some(0.7),
        max_tokens: Some(100),
        stop: None,
        stream: false,
        extra: serde_json::Map::new(),
    };
    
    println!("✅ 测试配置:");
    println!("   - 消息数量: {}", messages.len());
    println!("   - 温度: {:?}", options.temperature);
    println!("   - 最大令牌数: {:?}", options.max_tokens);
    println!("   - 测试消息: \"{}\"", messages[1].content);
    
    println!("⚠️  注意: 需要有效的智谱AI API密钥才能进行实际调用");
    println!("   获取地址: https://open.bigmodel.cn");
    
    Ok(())
}

async fn test_baidu_provider() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔥 百度ERNIE 示例:");
    println!("------------------");
    
    let baidu_provider = BaiduProvider::new(
        "your-baidu-api-key".to_string(),
        "your-baidu-secret-key".to_string(),
        Some("ernie-bot-4".to_string())
    );
    
    println!("✅ 百度ERNIE Provider 创建成功");
    println!("   - 提供商名称: {}", baidu_provider.name());
    println!("   - 模型: {}", baidu_provider.model());
    println!("   - 基础URL: {}", baidu_provider.base_url());
    println!("   - 支持函数调用: {}", baidu_provider.supports_function_calling());
    
    // 创建测试消息
    let messages = vec![
        Message {
            role: Role::User,
            content: "请用中文介绍一下人工智能的发展历程。".to_string(),
            metadata: None,
            name: None,
        },
    ];
    
    // 创建测试选项
    let mut extra = serde_json::Map::new();
    extra.insert("top_p".to_string(), serde_json::json!(0.9));

    let options = LlmOptions {
        model: None,
        temperature: Some(0.8),
        max_tokens: Some(200),
        stop: None,
        stream: false,
        extra,
    };
    
    println!("✅ 测试配置:");
    println!("   - 消息数量: {}", messages.len());
    println!("   - 温度: {:?}", options.temperature);
    println!("   - 最大令牌数: {:?}", options.max_tokens);
    println!("   - Top-p: {:?}", options.extra.get("top_p"));
    println!("   - 测试消息: \"{}\"", messages[0].content);
    
    println!("⚠️  注意: 需要有效的百度API密钥和密钥才能进行实际调用");
    println!("   获取地址: https://cloud.baidu.com");
    
    Ok(())
}

async fn demo_unified_interface() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 统一接口演示:");
    println!("----------------");
    
    // 创建不同的中文LLM提供商
    let providers: Vec<Box<dyn LlmProvider>> = vec![
        Box::new(ZhipuProvider::new(
            "test-key".to_string(),
            Some("glm-4".to_string())
        )),
        Box::new(BaiduProvider::new(
            "test-key".to_string(),
            "test-secret".to_string(),
            Some("ernie-bot-4".to_string())
        )),
    ];
    
    println!("✅ 创建了 {} 个中文LLM提供商", providers.len());
    
    // 统一处理所有提供商
    for (i, provider) in providers.iter().enumerate() {
        println!("   {}. 提供商: {}", i + 1, provider.name());
        println!("      支持函数调用: {}", provider.supports_function_calling());
    }
    
    println!("\n🌊 流式生成功能:");
    println!("   - 智谱AI支持流式响应，可实时获取生成内容");
    println!("   - 百度ERNIE支持流式响应，适合长文本生成");
    
    println!("\n🔗 嵌入功能:");
    println!("   - 智谱AI支持文本嵌入 (embedding-2 模型)");
    println!("   - 百度ERNIE支持文本嵌入，适合中文语义理解");
    
    println!("\n⚙️  函数调用功能:");
    println!("   - 智谱AI支持工具调用 (Tools API)");
    println!("   - 百度ERNIE支持函数调用，可集成外部工具");
    
    println!("\n📊 模型选择建议:");
    println!("   智谱AI GLM:");
    println!("   - glm-4: 最新版本，性能优秀");
    println!("   - glm-4-plus: 增强版本，更强推理能力");
    println!("   - glm-3-turbo: 快速版本，响应迅速");
    
    println!("   百度ERNIE:");
    println!("   - ernie-bot-4: 最新版本，综合能力强");
    println!("   - ernie-bot: 标准版本，稳定可靠");
    println!("   - ernie-bot-turbo: 快速版本，高并发场景");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_zhipu_provider_creation() {
        let provider = ZhipuProvider::new(
            "test-key".to_string(),
            Some("glm-4".to_string())
        );

        assert_eq!(provider.name(), "zhipu");
        assert_eq!(provider.model(), "glm-4");
        assert!(provider.supports_function_calling());
    }
    
    #[test]
    fn test_baidu_provider_creation() {
        let provider = BaiduProvider::new(
            "test-key".to_string(),
            "test-secret".to_string(),
            Some("ernie-bot-4".to_string())
        );

        assert_eq!(provider.name(), "baidu");
        assert_eq!(provider.model(), "ernie-bot-4");
        assert!(provider.supports_function_calling());
    }
    
    #[test]
    fn test_provider_options() {
        let mut extra = serde_json::Map::new();
        extra.insert("top_p".to_string(), serde_json::json!(0.9));

        let options = LlmOptions {
            model: None,
            temperature: Some(0.8),
            max_tokens: Some(200),
            stop: None,
            stream: false,
            extra,
        };

        assert_eq!(options.temperature, Some(0.8));
        assert_eq!(options.max_tokens, Some(200));
        assert!(options.extra.contains_key("top_p"));
    }
    
    #[test]
    fn test_message_creation() {
        let message = Message {
            role: Role::User,
            content: "测试消息".to_string(),
            metadata: None,
            name: None,
        };
        
        assert_eq!(message.role, Role::User);
        assert_eq!(message.content, "测试消息");
        assert!(message.metadata.is_none());
        assert!(message.name.is_none());
    }
}
