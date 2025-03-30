//! 语音模块，提供文本到语音和语音到文本的功能

use std::pin::Pin;
use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncRead;

use crate::base::Base;
use crate::error::{Error, Result};

/// 语音事件类型
pub trait VoiceEvent: Send + 'static {}

/// 语音数据事件
#[derive(Debug, Clone)]
pub struct SpeakingEvent {
    /// 音频数据
    pub audio: Vec<u8>,
}

impl VoiceEvent for SpeakingEvent {}

/// 文本事件
#[derive(Debug, Clone)]
pub struct WritingEvent {
    /// 文本内容
    pub text: String,
    /// 角色（user或assistant）
    pub role: String,
}

impl VoiceEvent for WritingEvent {}

/// 语音选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceOptions {
    /// 语音ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_id: Option<String>,
    /// 音频格式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    /// 语速
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f32>,
    /// 音高
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pitch: Option<f32>,
    /// 音量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f32>,
    /// 其他设置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<serde_json::Value>,
}

impl Default for VoiceOptions {
    fn default() -> Self {
        Self {
            voice_id: None,
            format: Some("mp3".to_string()),
            speed: Some(1.0),
            pitch: Some(1.0),
            volume: Some(1.0),
            settings: None,
        }
    }
}

/// 语音识别选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenOptions {
    /// 音频格式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filetype: Option<String>,
    /// 语言
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// 其他设置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<serde_json::Value>,
}

impl Default for ListenOptions {
    fn default() -> Self {
        Self {
            filetype: None,
            language: Some("en".to_string()),
            settings: None,
        }
    }
}

/// 用于实现具有泛型参数的方法的扩展trait
#[async_trait]
pub trait VoiceProviderExt: Send + Sync {
    /// 将语音转换为文本
    async fn listen_impl(&self, audio: impl AsyncRead + Send + 'static, options: &ListenOptions) -> Result<String>;
    
    /// 发送语音数据
    async fn send_impl(&self, audio: impl AsyncRead + Send + 'static) -> Result<()>;
    
    /// 注册事件回调
    fn on_impl<E: VoiceEvent>(&self, callback: Box<dyn FnMut(E) + Send + 'static>) -> Result<()>;
}

/// 语音提供者接口
#[async_trait]
pub trait VoiceProvider: Base + Send + Sync {
    /// 将文本转换为语音
    async fn speak(&self, text: &str, options: &VoiceOptions) -> Result<BoxStream<'static, Result<Vec<u8>>>>;
    
    /// 将语音转换为文本的包装方法
    async fn listen(&self, audio: Vec<u8>, options: &ListenOptions) -> Result<String>;
    
    /// 建立实时语音连接
    async fn connect(&self) -> Result<()> {
        Ok(()) // 默认实现，不做任何事
    }
    
    /// 发送语音数据的包装方法
    async fn send(&self, audio: Vec<u8>) -> Result<()>;
    
    /// 关闭连接
    async fn close(&self) -> Result<()> {
        Ok(()) // 默认实现，不做任何事
    }
    
    /// 获取扩展接口
    fn as_ext(&self) -> &dyn VoiceProviderExt;
}

/// 组合式语音提供者，可以使用不同的提供者进行语音合成和识别
pub struct CompositeVoice {
    /// 基础组件
    base: crate::base::BaseComponent,
    /// 语音识别提供者
    listen_provider: Box<dyn VoiceProvider>,
    /// 语音合成提供者
    speak_provider: Box<dyn VoiceProvider>,
}

impl CompositeVoice {
    /// 创建新的组合式语音提供者
    pub fn new(
        listen_provider: Box<dyn VoiceProvider>,
        speak_provider: Box<dyn VoiceProvider>,
    ) -> Self {
        Self {
            base: crate::base::BaseComponent::new(Some("CompositeVoice"), crate::logger::Component::Voice),
            listen_provider,
            speak_provider,
        }
    }
}

#[async_trait]
impl VoiceProvider for CompositeVoice {
    async fn speak(&self, text: &str, options: &VoiceOptions) -> Result<BoxStream<'static, Result<Vec<u8>>>> {
        self.speak_provider.speak(text, options).await
    }
    
    async fn listen(&self, audio: Vec<u8>, options: &ListenOptions) -> Result<String> {
        // 使用内存读取器将Vec<u8>转换为可读流
        let cursor = std::io::Cursor::new(audio);
        self.listen_provider.as_ext().listen_impl(cursor, options).await
    }
    
    async fn connect(&self) -> Result<()> {
        self.speak_provider.connect().await?;
        self.listen_provider.connect().await?;
        Ok(())
    }
    
    async fn send(&self, audio: Vec<u8>) -> Result<()> {
        let cursor = std::io::Cursor::new(audio);
        self.listen_provider.as_ext().send_impl(cursor).await
    }
    
    async fn close(&self) -> Result<()> {
        let _ = self.speak_provider.close().await;
        let _ = self.listen_provider.close().await;
        Ok(())
    }
    
    fn as_ext(&self) -> &dyn VoiceProviderExt {
        self.listen_provider.as_ext()
    }
}

// CompositeVoice的VoiceProviderExt实现，将调用转发到适当的提供者
#[async_trait]
impl VoiceProviderExt for CompositeVoice {
    async fn listen_impl(&self, audio: impl AsyncRead + Send + 'static, options: &ListenOptions) -> Result<String> {
        self.listen_provider.as_ext().listen_impl(audio, options).await
    }
    
    async fn send_impl(&self, audio: impl AsyncRead + Send + 'static) -> Result<()> {
        self.listen_provider.as_ext().send_impl(audio).await
    }
    
    fn on_impl<E: VoiceEvent>(&self, callback: Box<dyn FnMut(E) + Send + 'static>) -> Result<()> {
        // 尝试在speak_provider上注册回调，如果失败则尝试在listen_provider上注册
        match self.speak_provider.as_ext().on_impl(callback.clone()) {
            Ok(_) => Ok(()),
            Err(_) => self.listen_provider.as_ext().on_impl(callback),
        }
    }
}

impl Base for CompositeVoice {
    fn name(&self) -> Option<&str> {
        self.base.name()
    }
    
    fn component(&self) -> crate::logger::Component {
        self.base.component()
    }
    
    fn logger(&self) -> std::sync::Arc<dyn crate::logger::Logger> {
        self.base.logger()
    }
    
    fn set_logger(&mut self, logger: std::sync::Arc<dyn crate::logger::Logger>) {
        self.base.set_logger(logger);
    }
    
    fn telemetry(&self) -> Option<std::sync::Arc<dyn crate::telemetry::TelemetrySink>> {
        self.base.telemetry()
    }
    
    fn set_telemetry(&mut self, telemetry: std::sync::Arc<dyn crate::telemetry::TelemetrySink>) {
        self.base.set_telemetry(telemetry);
    }
}

// 重新导出子模块
pub mod providers;

// 重新导出主要类型
pub use providers::*; 