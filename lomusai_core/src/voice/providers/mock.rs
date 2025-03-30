//! 模拟语音提供者，用于测试和开发

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use futures::{stream, StreamExt};
use futures::stream::BoxStream;
use tokio::io::AsyncRead;
use serde_json::Value;

use crate::base::{Base, BaseComponent, ComponentConfig};
use crate::error::{Error, Result};
use crate::logger::{Component, Logger};
use crate::telemetry::TelemetrySink;
use crate::voice::{
    VoiceProvider, VoiceListener, VoiceSender, VoiceListenerExt, VoiceSenderExt, 
    VoiceOptions, ListenOptions, VoiceEvent
};

/// 模拟语音提供者，用于测试
pub struct MockVoice {
    /// 基础组件
    base: BaseComponent,
    /// 模拟回复
    responses: Mutex<Vec<String>>,
}

impl MockVoice {
    /// 创建新的模拟语音提供者
    pub fn new() -> Self {
        let component_config = ComponentConfig {
            name: Some("MockVoice".to_string()),
            component: Component::Voice,
            log_level: None,
        };
        
        Self {
            base: BaseComponent::new(component_config),
            responses: Mutex::new(vec!["这是一个模拟的语音回复".to_string()]),
        }
    }
    
    /// 设置模拟回复
    pub fn set_responses(&self, responses: Vec<String>) {
        let mut guard = self.responses.lock().unwrap();
        *guard = responses;
    }
}

#[async_trait]
impl VoiceListener for MockVoice {
    async fn listen(&self, _audio: Vec<u8>, _options: &ListenOptions) -> Result<String> {
        self.logger().debug("MockVoice: 将语音转换为文本", None);
        
        let guard = self.responses.lock().unwrap();
        Ok(guard.first().unwrap_or(&"默认模拟回复".to_string()).clone())
    }
}

#[async_trait]
impl VoiceListenerExt for MockVoice {
    async fn listen_impl(&self, audio: impl AsyncRead + Send + Unpin + 'static, options: &ListenOptions) -> Result<String> {
        // Convert the AsyncRead to Vec<u8>
        let audio_data = crate::voice::get_audio_data(audio).await?;
        VoiceListener::listen(self, audio_data, options).await
    }
}

#[async_trait]
impl VoiceSender for MockVoice {
    async fn send(&self, _audio: Vec<u8>) -> Result<()> {
        self.logger().debug("MockVoice: 发送音频数据", None);
        Ok(())
    }
}

#[async_trait]
impl VoiceSenderExt for MockVoice {
    async fn send_impl(&self, audio: impl AsyncRead + Send + Unpin + 'static) -> Result<()> {
        // 读取音频数据
        let audio_data = crate::voice::get_audio_data(audio).await?;
        VoiceSender::send(self, audio_data).await
    }
}

#[async_trait]
impl VoiceProvider for MockVoice {
    async fn connect(&self) -> Result<()> {
        self.logger().debug("MockVoice: 连接服务", None);
        Ok(())
    }
    
    async fn close(&self) -> Result<()> {
        self.logger().debug("MockVoice: 关闭连接", None);
        Ok(())
    }
    
    async fn speak(&self, text: &str, _options: &VoiceOptions) -> Result<BoxStream<'_, Result<Vec<u8>>>> {
        self.logger().debug(&format!("MockVoice: 将文本转换为语音: {}", text), None);
        
        // 创建模拟的音频数据
        let chunks = vec![
            Ok(vec![1, 2, 3, 4]), 
            Ok(vec![5, 6, 7, 8]),
            Ok(vec![9, 10, 11, 12])
        ];
        
        let stream = stream::iter(chunks).boxed();
        Ok(stream)
    }
    
    async fn listen(&self, audio: Vec<u8>, options: &ListenOptions) -> Result<String> {
        VoiceListener::listen(self, audio, options).await
    }
    
    async fn send(&self, audio: Vec<u8>) -> Result<()> {
        VoiceSender::send(self, audio).await
    }
    
    fn as_listener(&self) -> Option<&dyn VoiceListener> {
        Some(self as &dyn VoiceListener)
    }
    
    fn as_sender(&self) -> Option<&dyn VoiceSender> {
        Some(self as &dyn VoiceSender)
    }
}

impl Base for MockVoice {
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