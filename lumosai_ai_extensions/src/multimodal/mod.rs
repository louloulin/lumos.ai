//! 多模态处理模块
//! 
//! 支持图像、音频、视频和文档的处理和分析

pub mod vision;
pub mod audio;
pub mod video;
pub mod document;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::{MultimodalConfig, Result, AiExtensionError};

/// 多模态处理器
pub struct MultimodalProcessor {
    /// 视觉处理器
    pub vision: vision::VisionProcessor,
    
    /// 音频处理器
    pub audio: audio::AudioProcessor,
    
    /// 视频处理器
    pub video: video::VideoProcessor,
    
    /// 文档处理器
    pub document: document::DocumentProcessor,
}

/// 多模态输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MultimodalInput {
    /// 图像输入
    Image {
        data: Vec<u8>,
        format: String,
        metadata: HashMap<String, String>,
    },
    
    /// 音频输入
    Audio {
        data: Vec<u8>,
        format: String,
        sample_rate: u32,
        channels: u16,
        metadata: HashMap<String, String>,
    },
    
    /// 视频输入
    Video {
        data: Vec<u8>,
        format: String,
        duration: f32,
        fps: f32,
        resolution: (u32, u32),
        metadata: HashMap<String, String>,
    },
    
    /// 文档输入
    Document {
        data: Vec<u8>,
        format: String,
        filename: String,
        metadata: HashMap<String, String>,
    },
    
    /// 文本输入
    Text {
        content: String,
        language: Option<String>,
        metadata: HashMap<String, String>,
    },
    
    /// 混合输入
    Mixed {
        inputs: Vec<MultimodalInput>,
        metadata: HashMap<String, String>,
    },
}

/// 多模态输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalOutput {
    /// 处理结果
    pub results: Vec<ModalityResult>,
    
    /// 处理时间
    pub processing_time_ms: u64,
    
    /// 置信度
    pub confidence: f32,
    
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// 处理时间戳
    pub timestamp: DateTime<Utc>,
}

/// 模态处理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModalityResult {
    /// 视觉处理结果
    Vision(vision::VisionResult),
    
    /// 音频处理结果
    Audio(audio::AudioResult),
    
    /// 视频处理结果
    Video(video::VideoResult),
    
    /// 文档处理结果
    Document(document::DocumentResult),
    
    /// 文本处理结果
    Text(TextResult),
    
    /// 错误结果
    Error {
        modality: String,
        error: String,
    },
}

/// 文本处理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextResult {
    /// 原始文本
    pub original_text: String,
    
    /// 处理后文本
    pub processed_text: Option<String>,
    
    /// 语言检测结果
    pub detected_language: Option<String>,
    
    /// 情感分析
    pub sentiment: Option<SentimentAnalysis>,
    
    /// 实体识别
    pub entities: Vec<NamedEntity>,
    
    /// 关键词提取
    pub keywords: Vec<Keyword>,
    
    /// 摘要
    pub summary: Option<String>,
}

/// 情感分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentAnalysis {
    /// 情感标签
    pub label: String,
    
    /// 置信度
    pub confidence: f32,
    
    /// 情感分数
    pub score: f32,
}

/// 命名实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedEntity {
    /// 实体文本
    pub text: String,
    
    /// 实体类型
    pub entity_type: String,
    
    /// 开始位置
    pub start: usize,
    
    /// 结束位置
    pub end: usize,
    
    /// 置信度
    pub confidence: f32,
}

/// 关键词
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyword {
    /// 关键词文本
    pub text: String,
    
    /// 重要性分数
    pub score: f32,
    
    /// 词性
    pub pos_tag: Option<String>,
}

impl MultimodalProcessor {
    /// 创建新的多模态处理器
    pub async fn new(config: MultimodalConfig) -> Result<Self> {
        Ok(Self {
            vision: vision::VisionProcessor::new(config.vision).await?,
            audio: audio::AudioProcessor::new(config.audio).await?,
            video: video::VideoProcessor::new(config.video).await?,
            document: document::DocumentProcessor::new(config.document).await?,
        })
    }
    
    /// 处理多模态输入
    pub async fn process(&self, input: MultimodalInput) -> Result<MultimodalOutput> {
        let start_time = std::time::Instant::now();
        let mut results = Vec::new();
        let mut overall_confidence = 0.0;
        let mut confidence_count = 0;
        
        match input {
            MultimodalInput::Image { data, format, metadata } => {
                match self.vision.process_image(data, format, metadata).await {
                    Ok(result) => {
                        overall_confidence += result.confidence;
                        confidence_count += 1;
                        results.push(ModalityResult::Vision(result));
                    }
                    Err(e) => {
                        results.push(ModalityResult::Error {
                            modality: "vision".to_string(),
                            error: e.to_string(),
                        });
                    }
                }
            }
            
            MultimodalInput::Audio { data, format, sample_rate, channels, metadata } => {
                match self.audio.process_audio(data, format, sample_rate, channels, metadata).await {
                    Ok(result) => {
                        overall_confidence += result.confidence;
                        confidence_count += 1;
                        results.push(ModalityResult::Audio(result));
                    }
                    Err(e) => {
                        results.push(ModalityResult::Error {
                            modality: "audio".to_string(),
                            error: e.to_string(),
                        });
                    }
                }
            }
            
            MultimodalInput::Video { data, format, duration, fps, resolution, metadata } => {
                match self.video.process_video(data, format, duration, fps, resolution, metadata).await {
                    Ok(result) => {
                        overall_confidence += result.confidence;
                        confidence_count += 1;
                        results.push(ModalityResult::Video(result));
                    }
                    Err(e) => {
                        results.push(ModalityResult::Error {
                            modality: "video".to_string(),
                            error: e.to_string(),
                        });
                    }
                }
            }
            
            MultimodalInput::Document { data, format, filename, metadata } => {
                match self.document.process_document(data, format, filename, metadata).await {
                    Ok(result) => {
                        overall_confidence += result.confidence;
                        confidence_count += 1;
                        results.push(ModalityResult::Document(result));
                    }
                    Err(e) => {
                        results.push(ModalityResult::Error {
                            modality: "document".to_string(),
                            error: e.to_string(),
                        });
                    }
                }
            }
            
            MultimodalInput::Text { content, language, metadata } => {
                match self.process_text(content, language, metadata).await {
                    Ok(result) => {
                        overall_confidence += 0.9; // 文本处理通常比较可靠
                        confidence_count += 1;
                        results.push(ModalityResult::Text(result));
                    }
                    Err(e) => {
                        results.push(ModalityResult::Error {
                            modality: "text".to_string(),
                            error: e.to_string(),
                        });
                    }
                }
            }
            
            MultimodalInput::Mixed { inputs, metadata: _ } => {
                for input in inputs {
                    let sub_result = self.process(input).await?;
                    results.extend(sub_result.results);
                    overall_confidence += sub_result.confidence;
                    confidence_count += 1;
                }
            }
        }
        
        let processing_time = start_time.elapsed();
        let final_confidence = if confidence_count > 0 {
            overall_confidence / confidence_count as f32
        } else {
            0.0
        };
        
        Ok(MultimodalOutput {
            results,
            processing_time_ms: processing_time.as_millis() as u64,
            confidence: final_confidence,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        })
    }
    
    /// 处理文本
    async fn process_text(
        &self,
        content: String,
        language: Option<String>,
        metadata: HashMap<String, String>,
    ) -> Result<TextResult> {
        // 基础文本处理
        let processed_text = self.preprocess_text(&content);
        
        // 语言检测
        let detected_language = language.or_else(|| self.detect_language(&content));
        
        // 情感分析
        let sentiment = self.analyze_sentiment(&content).await.ok();
        
        // 实体识别
        let entities = self.extract_entities(&content).await.unwrap_or_default();
        
        // 关键词提取
        let keywords = self.extract_keywords(&content).await.unwrap_or_default();
        
        // 摘要生成
        let summary = if content.len() > 500 {
            self.generate_summary(&content).await.ok()
        } else {
            None
        };
        
        Ok(TextResult {
            original_text: content,
            processed_text: Some(processed_text),
            detected_language,
            sentiment,
            entities,
            keywords,
            summary,
        })
    }
    
    /// 预处理文本
    fn preprocess_text(&self, text: &str) -> String {
        // 基础文本清理
        text.trim()
            .replace('\r', "")
            .replace('\t', " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    /// 检测语言
    fn detect_language(&self, text: &str) -> Option<String> {
        // 简单的语言检测逻辑
        let chinese_chars = text.chars().filter(|c| {
            *c >= '\u{4e00}' && *c <= '\u{9fff}'
        }).count();
        
        let total_chars = text.chars().count();
        
        if total_chars > 0 && chinese_chars as f32 / total_chars as f32 > 0.3 {
            Some("zh".to_string())
        } else {
            Some("en".to_string())
        }
    }
    
    /// 情感分析
    async fn analyze_sentiment(&self, text: &str) -> Result<SentimentAnalysis> {
        // 简单的情感分析逻辑
        let positive_words = ["good", "great", "excellent", "amazing", "wonderful", "好", "棒", "优秀"];
        let negative_words = ["bad", "terrible", "awful", "horrible", "worst", "坏", "糟糕", "差"];
        
        let text_lower = text.to_lowercase();
        let positive_count = positive_words.iter()
            .filter(|word| text_lower.contains(*word))
            .count();
        let negative_count = negative_words.iter()
            .filter(|word| text_lower.contains(*word))
            .count();
        
        let (label, score) = if positive_count > negative_count {
            ("positive", 0.6 + (positive_count as f32 * 0.1))
        } else if negative_count > positive_count {
            ("negative", -0.6 - (negative_count as f32 * 0.1))
        } else {
            ("neutral", 0.0)
        };
        
        Ok(SentimentAnalysis {
            label: label.to_string(),
            confidence: 0.7,
            score: score.clamp(-1.0, 1.0),
        })
    }
    
    /// 提取实体
    async fn extract_entities(&self, text: &str) -> Result<Vec<NamedEntity>> {
        let mut entities = Vec::new();
        
        // 简单的实体识别逻辑（邮箱、URL、电话号码等）
        
        // 邮箱识别
        let email_regex = regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
        for mat in email_regex.find_iter(text) {
            entities.push(NamedEntity {
                text: mat.as_str().to_string(),
                entity_type: "EMAIL".to_string(),
                start: mat.start(),
                end: mat.end(),
                confidence: 0.95,
            });
        }
        
        // URL识别
        let url_regex = regex::Regex::new(r"https?://[^\s]+").unwrap();
        for mat in url_regex.find_iter(text) {
            entities.push(NamedEntity {
                text: mat.as_str().to_string(),
                entity_type: "URL".to_string(),
                start: mat.start(),
                end: mat.end(),
                confidence: 0.9,
            });
        }
        
        Ok(entities)
    }
    
    /// 提取关键词
    async fn extract_keywords(&self, text: &str) -> Result<Vec<Keyword>> {
        // 简单的关键词提取逻辑
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut word_freq = HashMap::new();
        
        for word in words {
            let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
            if clean_word.len() > 3 && !is_stop_word(&clean_word) {
                *word_freq.entry(clean_word).or_insert(0) += 1;
            }
        }
        
        let mut keywords: Vec<_> = word_freq.into_iter()
            .map(|(word, freq)| Keyword {
                text: word,
                score: freq as f32,
                pos_tag: None,
            })
            .collect();
        
        keywords.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        keywords.truncate(10);
        
        Ok(keywords)
    }
    
    /// 生成摘要
    async fn generate_summary(&self, text: &str) -> Result<String> {
        // 简单的摘要生成逻辑（提取前几句）
        let sentences: Vec<&str> = text.split('.').collect();
        let summary_sentences = sentences.into_iter()
            .take(3)
            .filter(|s| !s.trim().is_empty())
            .collect::<Vec<_>>()
            .join(". ");
        
        Ok(format!("{}.", summary_sentences.trim()))
    }
}

/// 检查是否为停用词
fn is_stop_word(word: &str) -> bool {
    let stop_words = [
        "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
        "是", "的", "了", "在", "有", "和", "与", "或", "但", "为", "从", "到", "对", "把", "被"
    ];
    stop_words.contains(&word)
}
