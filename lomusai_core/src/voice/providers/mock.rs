//! 模拟语音提供者，用于测试和开发

use std::sync::Arc;
use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use tokio::io::AsyncRead;

use crate::base::{Base, BaseComponent};
use crate::error::Result;
use crate::logger::{Component, Logger};
use crate::telemetry::TelemetrySink;
use crate::voice::{VoiceProvider, VoiceProviderExt, VoiceOptions, ListenOptions, VoiceEvent};

/// 模拟语音提供者
pub struct MockVoice {
    base: BaseComponent,
}

impl MockVoice {
    /// 创建一个新的模拟语音提供者
    pub fn new() -> Self {
        Self {
            base: BaseComponent::new(Some("MockVoice"), Component::Voice),
        }
    }
}

impl Default for MockVoice {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VoiceProviderExt for MockVoice {
    async fn listen_impl(&self, _audio: impl AsyncRead + Send + 'static, _options: &ListenOptions) -> Result<String> {
        self.logger().debug("MockVoice: 将语音转换为文本");
        
        // 返回一些模拟的文本
        Ok("这是由MockVoice生成的文本".to_string())
    }
    
    async fn send_impl(&self, _audio: impl AsyncRead + Send + 'static) -> Result<()> {
        self.logger().debug("MockVoice: 发送音频数据");
        Ok(())
    }
    
    fn on_impl<E: VoiceEvent>(&self, _callback: Box<dyn FnMut(E) + Send + 'static>) -> Result<()> {
        self.logger().debug("MockVoice: 注册回调函数");
        Ok(())
    }
}

#[async_trait]
impl VoiceProvider for MockVoice {
    async fn speak(&self, text: &str, _options: &VoiceOptions) -> Result<BoxStream<'static, Result<Vec<u8>>>> {
        self.logger().debug(&format!("MockVoice: 将文本转换为语音: {}", text));
        
        // 生成一些模拟的音频数据
        let data = text.as_bytes().to_vec();
        let chunks = vec![Ok(data)];
        
        Ok(stream::iter(chunks).boxed())
    }
    
    async fn listen(&self, audio: Vec<u8>, options: &ListenOptions) -> Result<String> {
        // 使用内存读取器转换Vec<u8>为AsyncRead
        let cursor = std::io::Cursor::new(audio);
        self.listen_impl(cursor, options).await
    }
    
    async fn send(&self, audio: Vec<u8>) -> Result<()> {
        let cursor = std::io::Cursor::new(audio);
        self.send_impl(cursor).await
    }
    
    fn as_ext(&self) -> &dyn VoiceProviderExt {
        self
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