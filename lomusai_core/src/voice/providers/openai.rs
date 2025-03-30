//! OpenAI语音提供者，支持OpenAI的TTS和STT

use std::sync::Arc;
use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::io::AsyncRead;

use crate::base::{Base, BaseComponent};
use crate::error::{Error, Result};
use crate::logger::{Component, Logger};
use crate::telemetry::TelemetrySink;
use crate::voice::{VoiceProvider, VoiceOptions, ListenOptions, VoiceEvent};

/// OpenAI语音提供者配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIVoiceConfig {
    /// API密钥
    pub api_key: Option<String>,
    /// 组织ID
    pub org_id: Option<String>,
    /// API基础URL
    pub api_base: Option<String>,
    /// 默认语音ID
    pub default_voice: Option<String>,
    /// 默认模型
    pub default_model: Option<String>,
}

impl Default for OpenAIVoiceConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("OPENAI_API_KEY").ok(),
            org_id: std::env::var("OPENAI_ORG_ID").ok(),
            api_base: Some("https://api.openai.com/v1".to_string()),
            default_voice: Some("alloy".to_string()),
            default_model: Some("tts-1".to_string()),
        }
    }
}

/// OpenAI语音提供者
pub struct OpenAIVoice {
    /// 基础组件
    base: BaseComponent,
    /// 配置
    config: OpenAIVoiceConfig,
    /// HTTP客户端
    client: reqwest::Client,
}

impl OpenAIVoice {
    /// 创建新的OpenAI语音提供者
    pub fn new(config: OpenAIVoiceConfig) -> Result<Self> {
        if config.api_key.is_none() {
            return Err(Error::Config("OpenAI API密钥未设置，请设置OPENAI_API_KEY环境变量或在配置中提供".to_string()));
        }
        
        Ok(Self {
            base: BaseComponent::new(Some("OpenAIVoice"), Component::Voice),
            config,
            client: reqwest::Client::new(),
        })
    }
    
    /// 创建默认的OpenAI语音提供者
    pub fn default_with_api_key(api_key: impl Into<String>) -> Result<Self> {
        let mut config = OpenAIVoiceConfig::default();
        config.api_key = Some(api_key.into());
        Self::new(config)
    }
    
    /// 获取API请求头
    fn get_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        
        if let Some(api_key) = &self.config.api_key {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap(),
            );
        }
        
        if let Some(org_id) = &self.config.org_id {
            headers.insert(
                reqwest::header::HeaderName::from_static("openai-organization"),
                reqwest::header::HeaderValue::from_str(org_id).unwrap(),
            );
        }
        
        headers
    }
    
    /// 获取API基础URL
    fn get_api_base(&self) -> &str {
        self.config.api_base.as_deref().unwrap_or("https://api.openai.com/v1")
    }
}

#[async_trait]
impl VoiceProvider for OpenAIVoice {
    async fn speak(&self, text: &str, options: &VoiceOptions) -> Result<BoxStream<'static, Result<Vec<u8>>>> {
        self.logger().debug(&format!("OpenAIVoice: 将文本转换为语音: {}", text));
        
        let voice_id = options.voice_id.as_deref().unwrap_or_else(|| 
            self.config.default_voice.as_deref().unwrap_or("alloy")
        );
        
        let model = options.settings.as_ref()
            .and_then(|s| s.get("model").and_then(|m| m.as_str()))
            .unwrap_or_else(|| self.config.default_model.as_deref().unwrap_or("tts-1"));
        
        let url = format!("{}/audio/speech", self.get_api_base());
        
        let request_body = json!({
            "model": model,
            "input": text,
            "voice": voice_id,
            "response_format": options.format.as_deref().unwrap_or("mp3"),
            "speed": options.speed.unwrap_or(1.0)
        });
        
        // 发送请求
        let response = self.client.post(&url)
            .headers(self.get_headers())
            .json(&request_body)
            .send()
            .await?;
            
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(Error::External(format!(
                "OpenAI语音API错误 ({}): {}", 
                status, 
                error_text
            )));
        }
        
        // 获取完整的音频数据
        let audio_data = response.bytes().await?.to_vec();
        
        // 返回音频流
        Ok(stream::once(async { Ok(audio_data) }).boxed())
    }
    
    async fn listen(&self, audio: impl AsyncRead + Send + 'static, options: &ListenOptions) -> Result<String> {
        self.logger().debug("OpenAIVoice: 将语音转换为文本");
        
        // 目前，此功能未实现，使用MockVoice提供的模拟功能
        // 在实际实现中，我们需要使用OpenAI的API来转录音频

        Err(Error::Unsupported("OpenAIVoice STT功能尚未实现".to_string()))
    }
}

impl Base for OpenAIVoice {
    fn name(&self) -> Option<&str> {
        self.base.name()
    }
    
    fn component(&self) -> Component {
        self.base.component()
    }
    
    fn logger(&self) -> Arc<dyn Logger> {
        self.base.logger()
    }
    
    fn set_logger(&mut self, logger: Arc<dyn Logger>) {
        self.base.set_logger(logger);
    }
    
    fn telemetry(&self) -> Option<Arc<dyn TelemetrySink>> {
        self.base.telemetry()
    }
    
    fn set_telemetry(&mut self, telemetry: Arc<dyn TelemetrySink>) {
        self.base.set_telemetry(telemetry);
    }
} 