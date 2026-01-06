//! 图像处理工具演示程序
//!
//! 演示如何使用图像处理工具进行图像分析、格式转换和压缩优化。

use lumosai_core::tool::builtin::image_processing::*;
use lumosai_core::tool::{ToolExecutionContext, ToolExecutionOptions};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("🖼️  图像处理工具演示\n");
    println!("{}", "=".repeat(80));
    println!();

    // 创建运行时上下文
    let context = ToolExecutionContext::default();
    let options = ToolExecutionOptions::default();

    // 测试 1: 图像信息分析
    println!("📊 测试 1: 图像信息分析");
    println!("{}", "-".repeat(80));

    let tools = get_all_image_processing_tools();
    let info_tool = &tools[0];

    println!("工具名称: {}", info_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", info_tool.description());
    println!();

    let test_image = "/path/to/photos/vacation.jpg";
    println!("测试图像: {}", test_image);
    println!();

    let params = json!({
        "image_path": test_image,
        "extract_metadata": true,
        "analyze_colors": true
    });

    match info_tool.execute(params, context.clone(), &options).await {
        Ok(result) => {
            println!("✅ 图像信息:");
            println!("  格式: {}", result["format"]);
            println!("  尺寸: {}x{} 像素", result["width"], result["height"]);
            println!("  文件大小: {} MB", result["file_size_mb"]);
            println!("  颜色模式: {}", result["color_mode"]);
            println!("  宽高比: {}", result["aspect_ratio"]);
            println!("  像素数: {} MP", result["megapixels"]);

            if let Some(metadata) = result.get("metadata") {
                println!("\n  📷 元数据:");
                println!("    相机: {}", metadata["camera"]);
                println!("    镜头: {}", metadata["lens"]);
                println!("    ISO: {}", metadata["iso"]);
                println!("    光圈: {}", metadata["aperture"]);
                println!("    快门速度: {}", metadata["shutter_speed"]);
                println!("    焦距: {}", metadata["focal_length"]);
                println!("    拍摄时间: {}", metadata["date_taken"]);
                println!("    GPS: {}", metadata["gps"]["location"]);
            }

            if let Some(colors) = result.get("color_analysis") {
                println!("\n  🎨 颜色分析:");
                println!("    主要颜色:");
                if let Some(dominant) = colors["dominant_colors"].as_array() {
                    for (i, color) in dominant.iter().enumerate() {
                        println!(
                            "      {}. {} ({}%)",
                            i + 1,
                            color["color"],
                            color["percentage"]
                        );
                    }
                }
                println!("    平均亮度: {}", colors["average_brightness"]);
                println!("    色温: {}", colors["color_temperature"]);
                println!("    饱和度: {}", colors["saturation"]);
            }
        }
        Err(e) => println!("❌ 分析失败: {}", e),
    }

    println!("\n");

    // 测试 2: 图像格式转换
    println!("🔄 测试 2: 图像格式转换");
    println!("{}", "-".repeat(80));

    let convert_tool = &tools[1];

    println!("工具名称: {}", convert_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", convert_tool.description());
    println!();

    let conversions = vec![
        (
            "PNG → JPEG",
            "/path/to/input.png",
            "/path/to/output.jpg",
            "jpeg",
            90,
        ),
        (
            "JPEG → WebP",
            "/path/to/input.jpg",
            "/path/to/output.webp",
            "webp",
            85,
        ),
        (
            "PNG → GIF",
            "/path/to/input.png",
            "/path/to/output.gif",
            "gif",
            80,
        ),
    ];

    for (desc, input, output, format, quality) in conversions {
        println!("转换: {}", desc);

        let params = json!({
            "input_path": input,
            "output_path": output,
            "output_format": format,
            "quality": quality,
            "preserve_metadata": true
        });

        match convert_tool
            .execute(params, context.clone(), &options)
            .await
        {
            Ok(result) => {
                if result["success"] == true {
                    println!("  ✅ 转换成功:");
                    println!(
                        "    输入: {} ({})",
                        result["input_format"], result["input_size_mb"]
                    );
                    println!(
                        "    输出: {} ({})",
                        result["output_format"], result["output_size_mb"]
                    );
                    println!("    质量: {}", result["quality"]);
                    println!("    压缩比: {}", result["compression_ratio"]);
                    println!("    处理时间: {} ms", result["processing_time_ms"]);
                } else {
                    println!("  ❌ 转换失败: {}", result["error"]);
                }
            }
            Err(e) => println!("  ❌ 转换失败: {}", e),
        }
        println!();
    }

    // 测试 3: 图像压缩优化
    println!("🗜️  测试 3: 图像压缩优化");
    println!("{}", "-".repeat(80));

    let compress_tool = &tools[2];

    println!("工具名称: {}", compress_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", compress_tool.description());
    println!();

    let compression_scenarios = vec![
        (
            "场景 1: 目标大小压缩",
            json!({
                "input_path": "/path/to/large.jpg",
                "output_path": "/path/to/compressed_200kb.jpg",
                "target_size_kb": 200,
                "quality": 85,
                "optimize": true
            }),
        ),
        (
            "场景 2: 尺寸调整压缩",
            json!({
                "input_path": "/path/to/large.jpg",
                "output_path": "/path/to/resized.jpg",
                "max_width": 1024,
                "max_height": 768,
                "quality": 80,
                "optimize": true
            }),
        ),
        (
            "场景 3: 质量压缩",
            json!({
                "input_path": "/path/to/large.jpg",
                "output_path": "/path/to/quality_compressed.jpg",
                "quality": 70,
                "optimize": true
            }),
        ),
    ];

    for (desc, params) in compression_scenarios {
        println!("{}", desc);

        match compress_tool
            .execute(params, context.clone(), &options)
            .await
        {
            Ok(result) => {
                println!("  ✅ 压缩结果:");
                println!(
                    "    原始大小: {} ({})",
                    result["original_dimensions"], result["original_size_mb"]
                );
                println!(
                    "    压缩后大小: {} ({})",
                    result["output_dimensions"], result["compressed_size_mb"]
                );
                println!("    压缩比: {}", result["compression_ratio"]);
                println!("    质量: {}", result["quality_used"]);
                println!("    优化: {}", result["optimize_enabled"]);
                println!("    处理时间: {} ms", result["processing_time_ms"]);

                if let Some(suggestions) = result.get("suggestions") {
                    if let Some(arr) = suggestions.as_array() {
                        if !arr.is_empty() {
                            println!("\n  💡 优化建议:");
                            for (i, suggestion) in arr.iter().enumerate() {
                                println!("    {}. {}", i + 1, suggestion);
                            }
                        }
                    }
                }
            }
            Err(e) => println!("  ❌ 压缩失败: {}", e),
        }
        println!();
    }

    // 测试 4: 参数验证
    println!("🔍 测试 4: 参数验证");
    println!("{}", "-".repeat(80));

    println!("测试 4.1: 无效的输出格式");
    let params = json!({
        "input_path": "/path/to/input.png",
        "output_path": "/path/to/output.bmp",
        "output_format": "bmp"
    });

    match convert_tool
        .execute(params, context.clone(), &options)
        .await
    {
        Ok(result) => {
            if result["success"] == false {
                println!("  ✅ 正确拒绝: {}", result["error"]);
            } else {
                println!("  ❌ 应该拒绝但接受了");
            }
        }
        Err(e) => println!("  ❌ 执行失败: {}", e),
    }
    println!();

    println!("测试 4.2: 无效的质量参数");
    let params = json!({
        "input_path": "/path/to/input.png",
        "output_path": "/path/to/output.jpg",
        "output_format": "jpeg",
        "quality": 150
    });

    match convert_tool
        .execute(params, context.clone(), &options)
        .await
    {
        Ok(result) => {
            if result["success"] == false {
                println!("  ✅ 正确拒绝: {}", result["error"]);
            } else {
                println!("  ❌ 应该拒绝但接受了");
            }
        }
        Err(e) => println!("  ❌ 执行失败: {}", e),
    }
    println!();

    // 测试 5: 批量获取工具
    println!("📦 测试 5: 获取所有图像处理工具");
    println!("{}", "-".repeat(80));

    let all_tools = get_all_image_processing_tools();
    println!("总共 {} 个图像处理工具:\n", all_tools.len());

    for (i, tool) in all_tools.iter().enumerate() {
        println!(
            "{}. {} - {}",
            i + 1,
            tool.name().unwrap_or("unknown"),
            tool.description()
        );
    }
    println!();

    // 总结
    println!("{}", "=".repeat(80));
    println!("✅ 图像处理工具演示完成！");
    println!("{}", "=".repeat(80));
    println!();
    println!("📊 工具统计:");
    println!("  - 图像信息分析工具: image_info");
    println!("  - 图像格式转换工具: image_convert");
    println!("  - 图像压缩优化工具: image_compress");
    println!();
    println!("💡 使用建议:");
    println!("  1. 图像信息分析: 用于获取图像的详细信息和元数据");
    println!("  2. 格式转换: 支持 PNG, JPEG, WebP, GIF 之间的转换");
    println!("  3. 压缩优化: 通过质量调整和尺寸调整减小文件大小");
    println!();
    println!("🚀 下一步:");
    println!("  - 集成真实的图像处理库（如 image, imageproc）");
    println!("  - 添加更多图像处理功能（裁剪、旋转、滤镜）");
    println!("  - 支持批量处理和异步处理");
    println!("{}", "=".repeat(80));
}
