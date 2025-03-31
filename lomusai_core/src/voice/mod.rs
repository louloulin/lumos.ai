//! 语音模块，提供文本到语音和语音到文本的功能

use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncRead;
use std::sync::Arc;

use crate::base::{Base, BaseComponent, ComponentConfig};
use crate::error::Result;
use crate::logger::{Logger, Component};
use crate::telemetry::TelemetrySink;

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

/// 语音事件处理器接口
pub trait VoiceEventHandler<E: VoiceEvent>: Send + Sync {
    /// 处理事件
    fn handle(&self, event: E);
}

/// 语音监听接口
#[async_trait]
pub trait VoiceListener: Send + Sync {
    /// 将语音转换为文本 (非泛型方法版本)
    async fn listen(&self, audio: Vec<u8>, options: &ListenOptions) -> Result<String>;
}

/// 语音发送接口
#[async_trait]
pub trait VoiceSender: Send + Sync {
    /// 发送语音数据 (非泛型方法版本)
    async fn send(&self, audio: Vec<u8>) -> Result<()>;
}

/// 语音监听接口扩展 (不对象安全)
#[async_trait]
pub trait VoiceListenerExt: VoiceListener {
    /// 将语音转换为文本 (泛型方法版本)
    async fn listen_impl(&self, audio: impl AsyncRead + Send + Unpin + 'static, options: &ListenOptions) -> Result<String>;
}

/// 语音发送接口扩展 (不对象安全)
#[async_trait]
pub trait VoiceSenderExt: VoiceSender {
    /// 发送语音数据 (泛型方法版本)
    async fn send_impl(&self, audio: impl AsyncRead + Send + Unpin + 'static) -> Result<()>;
}

/// 语音事件处理扩展 (不对象安全)
pub trait VoiceEventHandlerExt: Send + Sync {
    /// 获取事件处理器
    fn as_event_handler<E: VoiceEvent>(&self) -> Option<&dyn VoiceEventHandler<E>>;
}

/// 语音提供者接口
#[async_trait]
pub trait VoiceProvider: Base + Send + Sync {
    /// 连接语音服务
    async fn connect(&self) -> Result<()>;
    
    /// 关闭语音服务连接
    async fn close(&self) -> Result<()>;
    
    /// 文本转语音
    async fn speak(&self, text: &str, options: &VoiceOptions) -> Result<BoxStream<'_, Result<Vec<u8>>>>;
    
    /// 语音转文本
    async fn listen(&self, audio: Vec<u8>, options: &ListenOptions) -> Result<String>;
    
    /// 发送音频数据 (用于实时交互)
    async fn send(&self, audio: Vec<u8>) -> Result<()>;
    
    /// 获取语音监听器
    fn as_listener(&self) -> Option<&dyn VoiceListener>;
    
    /// 获取语音发送器
    fn as_sender(&self) -> Option<&dyn VoiceSender>;
}

/// 复合语音提供者，支持独立的TTS和STT提供者
pub struct CompositeVoice {
    /// 基础组件
    base: BaseComponent,
    /// 语音合成提供者
    speak_provider: Arc<dyn VoiceProvider>,
    /// 语音识别提供者
    listen_provider: Arc<dyn VoiceProvider>,
}

impl CompositeVoice {
    /// 创建新的复合语音提供者
    pub fn new(speak_provider: Arc<dyn VoiceProvider>, listen_provider: Arc<dyn VoiceProvider>) -> Self {
        let component_config = ComponentConfig {
            name: Some("CompositeVoice".to_string()),
            component: Component::Voice,
            log_level: None,
        };
        
        Self {
            base: BaseComponent::new(component_config),
            speak_provider,
            listen_provider,
        }
    }
}

#[async_trait]
impl VoiceProvider for CompositeVoice {
    async fn connect(&self) -> Result<()> {
        self.speak_provider.connect().await?;
        self.listen_provider.connect().await?;
        Ok(())
    }
    
    async fn close(&self) -> Result<()> {
        let _ = self.speak_provider.close().await;
        let _ = self.listen_provider.close().await;
        Ok(())
    }
    
    async fn speak(&self, text: &str, options: &VoiceOptions) -> Result<BoxStream<'_, Result<Vec<u8>>>> {
        self.speak_provider.speak(text, options).await
    }
    
    async fn listen(&self, audio: Vec<u8>, options: &ListenOptions) -> Result<String> {
        self.listen_provider.listen(audio, options).await
    }
    
    async fn send(&self, audio: Vec<u8>) -> Result<()> {
        self.listen_provider.send(audio).await
    }
    
    fn as_listener(&self) -> Option<&dyn VoiceListener> {
        self.listen_provider.as_listener()
    }
    
    fn as_sender(&self) -> Option<&dyn VoiceSender> {
        self.listen_provider.as_sender()
    }
}

impl Base for CompositeVoice {
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

// 重新导出子模块
pub mod providers;

// 重新导出主要类型
pub use providers::*;

/// 获取音频数据辅助函数
pub async fn get_audio_data(audio: impl AsyncRead + Send + Unpin + 'static) -> Result<Vec<u8>> {
    use tokio::io::AsyncReadExt;
    let mut buffer = Vec::new();
    let mut reader = tokio::io::BufReader::new(audio);
    reader.read_to_end(&mut buffer).await?;
    Ok(buffer)
} 