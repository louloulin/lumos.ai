//! AI工具集
//! 
//! 提供图像分析、文本摘要、情感分析、翻译、OCR等AI功能

use crate::tool::{ToolSchema, ParameterSchema, FunctionTool};
use crate::error::Result;
use serde_json::{Value, json};

/// 图像分析工具
pub fn image_analyzer() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "image_data".to_string(),
            description: "图像数据（base64编码）或图像URL".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "analysis_type".to_string(),
            description: "分析类型：object_detection, scene_recognition, ocr, all".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: Some(json!("all")),
        },
        ParameterSchema {
            name: "confidence_threshold".to_string(),
            description: "置信度阈值（0.0-1.0）".to_string(),
            r#type: "number".to_string(),
            required: false,
            properties: None,
            default: Some(json!(0.5)),
        },
    ]);

    FunctionTool::new(
        "image_analyzer",
        "分析图像内容，支持物体检测、场景识别、文字提取等功能",
        schema,
        |params| {
            let _image_data = params.get("image_data")
                .and_then(|v| v.as_str())
                .ok_or_else(|| crate::error::Error::Tool("Missing image_data parameter".to_string()))?;
            
            let analysis_type = params.get("analysis_type")
                .and_then(|v| v.as_str())
                .unwrap_or("all");
            
            let confidence_threshold = params.get("confidence_threshold")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.5);

            // 模拟图像分析结果
            let mut results = json!({
                "image_info": {
                    "format": "JPEG",
                    "size": "1024x768",
                    "file_size": "245KB"
                },
                "analysis_type": analysis_type,
                "confidence_threshold": confidence_threshold
            });

            match analysis_type {
                "object_detection" | "all" => {
                    results["objects"] = json!([
                        {
                            "label": "person",
                            "confidence": 0.95,
                            "bbox": [100, 150, 300, 500]
                        },
                        {
                            "label": "car",
                            "confidence": 0.87,
                            "bbox": [400, 200, 600, 350]
                        }
                    ]);
                },
                "scene_recognition" => {
                    results["scene"] = json!({
                        "primary_scene": "street",
                        "confidence": 0.92,
                        "secondary_scenes": ["urban", "outdoor"]
                    });
                },
                "ocr" => {
                    results["text"] = json!({
                        "detected_text": "Sample text from image",
                        "language": "en",
                        "confidence": 0.89
                    });
                },
                _ => {}
            }

            Ok(json!({
                "success": true,
                "results": results,
                "processing_time_ms": 1250
            }))
        },
    )
}

/// 文本摘要工具
pub fn text_summarizer() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "text".to_string(),
            description: "要摘要的文本内容".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "max_length".to_string(),
            description: "摘要最大长度（字符数）".to_string(),
            r#type: "number".to_string(),
            required: false,
            properties: None,
            default: Some(json!(200)),
        },
        ParameterSchema {
            name: "strategy".to_string(),
            description: "摘要策略：extractive, abstractive, hybrid".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: Some(json!("abstractive")),
        },
        ParameterSchema {
            name: "language".to_string(),
            description: "文本语言（zh, en, auto）".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: Some(json!("auto")),
        },
    ]);

    FunctionTool::new(
        "text_summarizer",
        "生成文本摘要，支持多种摘要策略和长度控制",
        schema,
        |params| {
            let text = params.get("text")
                .and_then(|v| v.as_str())
                .ok_or_else(|| crate::error::Error::Tool("Missing text parameter".to_string()))?;
            
            let max_length = params.get("max_length")
                .and_then(|v| v.as_u64())
                .unwrap_or(200) as usize;
            
            let strategy = params.get("strategy")
                .and_then(|v| v.as_str())
                .unwrap_or("abstractive");
            
            let language = params.get("language")
                .and_then(|v| v.as_str())
                .unwrap_or("auto");

            // 简化的摘要生成逻辑
            let summary = if text.len() <= max_length {
                text.to_string()
            } else {
                let sentences: Vec<&str> = text.split('.').collect();
                let mut summary = String::new();
                
                for sentence in sentences {
                    if summary.len() + sentence.len() <= max_length {
                        summary.push_str(sentence);
                        summary.push('.');
                    } else {
                        break;
                    }
                }
                
                if summary.is_empty() {
                    text.chars().take(max_length).collect::<String>() + "..."
                } else {
                    summary
                }
            };

            Ok(json!({
                "success": true,
                "summary": summary,
                "original_length": text.len(),
                "summary_length": summary.len(),
                "compression_ratio": (summary.len() as f64) / (text.len() as f64),
                "strategy": strategy,
                "language": language
            }))
        },
    )
}

/// 情感分析工具
pub fn sentiment_analyzer() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "text".to_string(),
            description: "要分析的文本内容".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "analysis_depth".to_string(),
            description: "分析深度：basic, detailed, emotions".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: Some(json!("basic")),
        },
        ParameterSchema {
            name: "language".to_string(),
            description: "文本语言（zh, en, auto）".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: Some(json!("auto")),
        },
    ]);

    FunctionTool::new(
        "sentiment_analyzer",
        "分析文本情感倾向，支持多维度情感分析",
        schema,
        |params| {
            let text = params.get("text")
                .and_then(|v| v.as_str())
                .ok_or_else(|| crate::error::Error::Tool("Missing text parameter".to_string()))?;
            
            let analysis_depth = params.get("analysis_depth")
                .and_then(|v| v.as_str())
                .unwrap_or("basic");
            
            let language = params.get("language")
                .and_then(|v| v.as_str())
                .unwrap_or("auto");

            // 简化的情感分析逻辑
            let positive_words = ["good", "great", "excellent", "amazing", "wonderful", "好", "棒", "优秀"];
            let negative_words = ["bad", "terrible", "awful", "horrible", "worst", "坏", "糟糕", "差"];
            
            let text_lower = text.to_lowercase();
            let positive_count = positive_words.iter().filter(|&&word| text_lower.contains(word)).count();
            let negative_count = negative_words.iter().filter(|&&word| text_lower.contains(word)).count();
            
            let sentiment_score = if positive_count + negative_count == 0 {
                0.0
            } else {
                (positive_count as f64 - negative_count as f64) / (positive_count + negative_count) as f64
            };
            
            let sentiment_label = if sentiment_score > 0.2 {
                "positive"
            } else if sentiment_score < -0.2 {
                "negative"
            } else {
                "neutral"
            };

            let mut result = json!({
                "success": true,
                "sentiment": {
                    "label": sentiment_label,
                    "score": sentiment_score,
                    "confidence": 0.85
                },
                "language": language,
                "text_length": text.len()
            });

            if analysis_depth == "detailed" || analysis_depth == "emotions" {
                result["detailed_analysis"] = json!({
                    "positive_indicators": positive_count,
                    "negative_indicators": negative_count,
                    "subjectivity": 0.7,
                    "intensity": sentiment_score.abs()
                });
            }

            if analysis_depth == "emotions" {
                result["emotions"] = json!({
                    "joy": if sentiment_score > 0.0 { sentiment_score * 0.8 } else { 0.1 },
                    "anger": if sentiment_score < 0.0 { sentiment_score.abs() * 0.6 } else { 0.1 },
                    "sadness": if sentiment_score < -0.3 { sentiment_score.abs() * 0.5 } else { 0.1 },
                    "fear": 0.1,
                    "surprise": 0.2,
                    "disgust": if sentiment_score < -0.5 { sentiment_score.abs() * 0.4 } else { 0.1 }
                });
            }

            Ok(result)
        },
    )
}

/// 获取所有AI工具
pub fn all_ai_tools() -> Vec<FunctionTool> {
    vec![
        image_analyzer(),
        text_summarizer(),
        sentiment_analyzer(),
    ]
}
