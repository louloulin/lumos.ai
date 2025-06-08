//! 便利函数用于快速创建LLM providers

use crate::Result;
use super::*;

/// 便利函数用于创建各种LLM providers

/// 创建OpenAI provider
pub fn openai(api_key: String, model: Option<String>) -> OpenAiProvider {
    OpenAiProvider::new(api_key, model.unwrap_or_else(|| "gpt-3.5-turbo".to_string()))
}

/// 创建Anthropic provider
pub fn anthropic(api_key: String, model: Option<String>) -> AnthropicProvider {
    AnthropicProvider::new(api_key, model.unwrap_or_else(|| "claude-3-sonnet-20240229".to_string()))
}

/// 创建Claude provider
pub fn claude(api_key: String, model: Option<String>) -> ClaudeProvider {
    ClaudeProvider::new(api_key, model.unwrap_or_else(|| "claude-3-sonnet-20240229".to_string()))
}

/// 创建DeepSeek provider
pub fn deepseek(api_key: String, model: Option<String>) -> DeepSeekProvider {
    DeepSeekProvider::new(api_key, model)
}

/// 创建Qwen provider
pub fn qwen(api_key: String, model: Option<String>) -> QwenProvider {
    QwenProvider::new_with_defaults(api_key, model.unwrap_or_else(|| "qwen-turbo".to_string()))
}

/// 创建Cohere provider
pub fn cohere(api_key: String, model: String) -> CohereProvider {
    CohereProvider::new(api_key, model)
}

/// 创建Gemini provider
pub fn gemini(api_key: String, model: String) -> GeminiProvider {
    GeminiProvider::new(api_key, model)
}

/// 创建Ollama provider (本地)
pub fn ollama_local(model: String) -> OllamaProvider {
    OllamaProvider::localhost(model)
}

/// 创建Ollama provider (自定义URL)
pub fn ollama(base_url: String, model: String) -> OllamaProvider {
    OllamaProvider::new(base_url, model)
}

/// 创建Together provider
pub fn together(api_key: String, model: String) -> TogetherProvider {
    TogetherProvider::new(api_key, model)
}

/// 创建智谱AI provider
pub fn zhipu(api_key: String, model: Option<String>) -> ZhipuProvider {
    ZhipuProvider::new(api_key, model)
}

/// 创建百度ERNIE provider
pub fn baidu(api_key: String, secret_key: String, model: Option<String>) -> BaiduProvider {
    BaiduProvider::new(api_key, secret_key, model)
}

/// 从环境变量创建providers的便利函数

/// 从环境变量创建OpenAI provider
/// 需要环境变量: OPENAI_API_KEY
pub fn openai_from_env() -> Result<OpenAiProvider> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| crate::Error::Llm("OPENAI_API_KEY environment variable not set".to_string()))?;
    Ok(openai(api_key, None))
}

/// 从环境变量创建Anthropic provider
/// 需要环境变量: ANTHROPIC_API_KEY
pub fn anthropic_from_env() -> Result<AnthropicProvider> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| crate::Error::Llm("ANTHROPIC_API_KEY environment variable not set".to_string()))?;
    Ok(anthropic(api_key, None))
}

/// 从环境变量创建Claude provider
/// 需要环境变量: CLAUDE_API_KEY
pub fn claude_from_env() -> Result<ClaudeProvider> {
    let api_key = std::env::var("CLAUDE_API_KEY")
        .map_err(|_| crate::Error::Llm("CLAUDE_API_KEY environment variable not set".to_string()))?;
    Ok(claude(api_key, None))
}

/// 从环境变量创建DeepSeek provider
/// 需要环境变量: DEEPSEEK_API_KEY
pub fn deepseek_from_env() -> Result<DeepSeekProvider> {
    let api_key = std::env::var("DEEPSEEK_API_KEY")
        .map_err(|_| crate::Error::Llm("DEEPSEEK_API_KEY environment variable not set".to_string()))?;
    Ok(deepseek(api_key, None))
}

/// 从环境变量创建Qwen provider
/// 需要环境变量: QWEN_API_KEY
pub fn qwen_from_env() -> Result<QwenProvider> {
    let api_key = std::env::var("QWEN_API_KEY")
        .map_err(|_| crate::Error::Llm("QWEN_API_KEY environment variable not set".to_string()))?;
    Ok(qwen(api_key, None))
}

/// 从环境变量创建Cohere provider
/// 需要环境变量: COHERE_API_KEY
pub fn cohere_from_env() -> Result<CohereProvider> {
    let api_key = std::env::var("COHERE_API_KEY")
        .map_err(|_| crate::Error::Llm("COHERE_API_KEY environment variable not set".to_string()))?;
    Ok(cohere(api_key, "command-r-plus".to_string()))
}

/// 从环境变量创建Gemini provider
/// 需要环境变量: GEMINI_API_KEY
pub fn gemini_from_env() -> Result<GeminiProvider> {
    let api_key = std::env::var("GEMINI_API_KEY")
        .map_err(|_| crate::Error::Llm("GEMINI_API_KEY environment variable not set".to_string()))?;
    Ok(gemini(api_key, "gemini-1.5-pro".to_string()))
}

/// 从环境变量创建Together provider
/// 需要环境变量: TOGETHER_API_KEY
pub fn together_from_env() -> Result<TogetherProvider> {
    let api_key = std::env::var("TOGETHER_API_KEY")
        .map_err(|_| crate::Error::Llm("TOGETHER_API_KEY environment variable not set".to_string()))?;
    Ok(together(api_key, "meta-llama/Llama-2-7b-chat-hf".to_string()))
}

/// 从环境变量创建智谱AI provider
/// 需要环境变量: ZHIPU_API_KEY
pub fn zhipu_from_env() -> Result<ZhipuProvider> {
    let api_key = std::env::var("ZHIPU_API_KEY")
        .map_err(|_| crate::Error::Llm("ZHIPU_API_KEY environment variable not set".to_string()))?;
    Ok(zhipu(api_key, None))
}

/// 从环境变量创建百度ERNIE provider
/// 需要环境变量: BAIDU_API_KEY, BAIDU_SECRET_KEY
pub fn baidu_from_env() -> Result<BaiduProvider> {
    let api_key = std::env::var("BAIDU_API_KEY")
        .map_err(|_| crate::Error::Llm("BAIDU_API_KEY environment variable not set".to_string()))?;
    let secret_key = std::env::var("BAIDU_SECRET_KEY")
        .map_err(|_| crate::Error::Llm("BAIDU_SECRET_KEY environment variable not set".to_string()))?;
    Ok(baidu(api_key, secret_key, None))
}

/// 智能provider选择器
/// 根据环境变量自动选择可用的provider
pub fn auto_provider() -> Result<Box<dyn LlmProvider>> {
    // 按优先级尝试不同的providers
    if let Ok(provider) = openai_from_env() {
        return Ok(Box::new(provider));
    }
    
    if let Ok(provider) = claude_from_env() {
        return Ok(Box::new(provider));
    }
    
    if let Ok(provider) = anthropic_from_env() {
        return Ok(Box::new(provider));
    }
    
    if let Ok(provider) = deepseek_from_env() {
        return Ok(Box::new(provider));
    }
    
    if let Ok(provider) = zhipu_from_env() {
        return Ok(Box::new(provider));
    }
    
    if let Ok(provider) = baidu_from_env() {
        return Ok(Box::new(provider));
    }
    
    if let Ok(provider) = qwen_from_env() {
        return Ok(Box::new(provider));
    }
    
    if let Ok(provider) = gemini_from_env() {
        return Ok(Box::new(provider));
    }
    
    if let Ok(provider) = cohere_from_env() {
        return Ok(Box::new(provider));
    }
    
    if let Ok(provider) = together_from_env() {
        return Ok(Box::new(provider));
    }
    
    // 最后尝试本地Ollama
    Ok(Box::new(ollama_local("llama2".to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation_functions() {
        // Test that all provider creation functions work
        let _openai = openai("test".to_string(), None);
        let _anthropic = anthropic("test".to_string(), None);
        let _claude = claude("test".to_string(), None);
        let _deepseek = deepseek("test".to_string(), None);
        let _qwen = qwen("test".to_string(), None);
        let _cohere = cohere("test".to_string(), "model".to_string());
        let _gemini = gemini("test".to_string(), "model".to_string());
        let _ollama = ollama_local("model".to_string());
        let _together = together("test".to_string(), "model".to_string());
        let _zhipu = zhipu("test".to_string(), None);
        let _baidu = baidu("test".to_string(), "secret".to_string(), None);
    }

    #[test]
    fn test_auto_provider_fallback() {
        // Test that auto_provider falls back to Ollama when no env vars are set
        let provider = auto_provider().unwrap();
        assert_eq!(provider.name(), "ollama");
    }
}
