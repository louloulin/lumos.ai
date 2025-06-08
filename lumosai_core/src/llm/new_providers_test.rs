//! 新LLM提供商的单元测试

use super::*;

#[test]
fn test_cohere_provider_creation() {
    let provider = CohereProvider::new(
        "test-key".to_string(),
        "command-r-plus".to_string(),
    );

    assert_eq!(provider.name(), "cohere");
    assert!(!provider.supports_function_calling());
}

#[test]
fn test_gemini_provider_creation() {
    let provider = GeminiProvider::new(
        "test-key".to_string(),
        "gemini-1.5-pro".to_string(),
    );

    assert_eq!(provider.name(), "gemini");
    assert!(provider.supports_function_calling());
}

#[test]
fn test_ollama_provider_creation() {
    let provider = OllamaProvider::localhost("llama2".to_string());

    assert_eq!(provider.name(), "ollama");
    assert!(!provider.supports_function_calling());
}

#[test]
fn test_together_provider_creation() {
    let provider = TogetherProvider::new(
        "test-key".to_string(),
        "meta-llama/Llama-2-7b-chat-hf".to_string(),
    );

    assert_eq!(provider.name(), "together");
    assert!(!provider.supports_function_calling());
}

#[test]
fn test_zhipu_provider_creation() {
    let provider = ZhipuProvider::new(
        "test-key".to_string(),
        Some("glm-4".to_string()),
    );

    assert_eq!(provider.name(), "zhipu");
    assert!(provider.supports_function_calling());
}

#[test]
fn test_baidu_provider_creation() {
    let provider = BaiduProvider::new(
        "test-key".to_string(),
        "test-secret".to_string(),
        Some("ernie-bot".to_string()),
    );

    assert_eq!(provider.name(), "baidu");
    assert!(provider.supports_function_calling());
}









#[test]
fn test_llm_options_basic() {
    let options = LlmOptions::default()
        .with_temperature(0.7)
        .with_max_tokens(100);

    // Test that options can be created
    assert_eq!(options.temperature, Some(0.7));
    assert_eq!(options.max_tokens, Some(100));
}

#[test]
fn test_provider_from_env() {
    // Test that providers can be created from environment variables
    // These will fail if env vars are not set, but that's expected
    
    // Test Cohere
    let cohere_result = CohereProvider::from_env();
    assert!(cohere_result.is_err()); // Expected to fail without env var
    
    // Test Gemini
    let gemini_result = GeminiProvider::from_env();
    assert!(gemini_result.is_err()); // Expected to fail without env var
    
    // Test Together
    let together_result = TogetherProvider::from_env();
    assert!(together_result.is_err()); // Expected to fail without env var
    
    // Test Ollama (should work as it has defaults)
    let ollama_provider = OllamaProvider::from_env();
    assert_eq!(ollama_provider.name(), "ollama");
}

#[test]
fn test_all_providers_implement_trait() {
    // Test that all providers implement the LlmProvider trait correctly
    let providers: Vec<Box<dyn LlmProvider>> = vec![
        Box::new(CohereProvider::new("test".to_string(), "model".to_string())),
        Box::new(GeminiProvider::new("test".to_string(), "model".to_string())),
        Box::new(OllamaProvider::localhost("model".to_string())),
        Box::new(TogetherProvider::new("test".to_string(), "model".to_string())),
        Box::new(ZhipuProvider::new("test".to_string(), Some("model".to_string()))),
        Box::new(BaiduProvider::new("test".to_string(), "test-secret".to_string(), Some("model".to_string()))),
    ];

    let expected_names = ["cohere", "gemini", "ollama", "together", "zhipu", "baidu"];
    
    for (i, provider) in providers.iter().enumerate() {
        assert_eq!(provider.name(), expected_names[i]);
    }
}

#[tokio::test]
#[ignore] // 需要实际的API密钥才能运行
async fn test_cohere_integration() {
    let api_key = std::env::var("COHERE_API_KEY").expect("需要设置COHERE_API_KEY环境变量");
    let provider = CohereProvider::new(api_key, "command-r-plus".to_string());
    
    let options = LlmOptions::default()
        .with_temperature(0.7)
        .with_max_tokens(50);
    
    let response = provider.generate("Say hello", &options).await;
    assert!(response.is_ok());
    println!("Cohere response: {}", response.unwrap());
}

#[tokio::test]
#[ignore] // 需要实际的API密钥才能运行
async fn test_gemini_integration() {
    let api_key = std::env::var("GEMINI_API_KEY").expect("需要设置GEMINI_API_KEY环境变量");
    let provider = GeminiProvider::new(api_key, "gemini-1.5-pro".to_string());
    
    let options = LlmOptions::default()
        .with_temperature(0.7)
        .with_max_tokens(50);
    
    let response = provider.generate("Say hello", &options).await;
    assert!(response.is_ok());
    println!("Gemini response: {}", response.unwrap());
}

#[tokio::test]
#[ignore] // 需要本地运行Ollama服务
async fn test_ollama_integration() {
    let provider = OllamaProvider::localhost("llama2".to_string());
    
    let options = LlmOptions::default()
        .with_temperature(0.7)
        .with_max_tokens(50);
    
    let response = provider.generate("Say hello", &options).await;
    assert!(response.is_ok());
    println!("Ollama response: {}", response.unwrap());
}

#[tokio::test]
#[ignore] // 需要实际的API密钥才能运行
async fn test_together_integration() {
    let api_key = std::env::var("TOGETHER_API_KEY").expect("需要设置TOGETHER_API_KEY环境变量");
    let provider = TogetherProvider::new(api_key, "meta-llama/Llama-2-7b-chat-hf".to_string());
    
    let options = LlmOptions::default()
        .with_temperature(0.7)
        .with_max_tokens(50);
    
    let response = provider.generate("Say hello", &options).await;
    assert!(response.is_ok());
    println!("Together response: {}", response.unwrap());
}

#[tokio::test]
#[ignore] // 需要实际的API密钥才能运行
async fn test_zhipu_integration() {
    let api_key = std::env::var("ZHIPU_API_KEY").expect("需要设置ZHIPU_API_KEY环境变量");
    let provider = ZhipuProvider::new(api_key, Some("glm-4".to_string()));

    let options = LlmOptions::default()
        .with_temperature(0.7)
        .with_max_tokens(50);

    let response = provider.generate("你好", &options).await;
    assert!(response.is_ok());
    println!("智谱AI response: {}", response.unwrap());
}

#[tokio::test]
#[ignore] // 需要实际的API密钥才能运行
async fn test_baidu_integration() {
    let api_key = std::env::var("BAIDU_API_KEY").expect("需要设置BAIDU_API_KEY环境变量");
    let secret_key = std::env::var("BAIDU_SECRET_KEY").expect("需要设置BAIDU_SECRET_KEY环境变量");
    let provider = BaiduProvider::new(api_key, secret_key, Some("ernie-bot".to_string()));

    let options = LlmOptions::default()
        .with_temperature(0.7)
        .with_max_tokens(50);

    let response = provider.generate("你好", &options).await;
    assert!(response.is_ok());
    println!("百度ERNIE response: {}", response.unwrap());
}
