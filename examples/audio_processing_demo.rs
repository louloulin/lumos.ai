//! 音频处理工具演示程序

use lumosai_core::tool::builtin::audio_processing::*;
use lumosai_core::tool::{ToolExecutionContext, ToolExecutionOptions};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("🎵 音频处理工具演示\n");
    println!("{}", "=".repeat(80));
    println!();

    let context = ToolExecutionContext::default();
    let options = ToolExecutionOptions::default();

    // 测试 1: 音频信息分析
    println!("📊 测试 1: 音频信息分析");
    println!("{}", "-".repeat(80));

    let tools = get_all_audio_processing_tools();
    let info_tool = &tools[0];

    println!("工具名称: {}", info_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", info_tool.description());
    println!();

    let test_audio = "/path/to/music/song.mp3";
    println!("测试音频: {}", test_audio);
    println!();

    let params = json!({
        "audio_path": test_audio,
        "extract_metadata": true,
        "analyze_waveform": true
    });

    match info_tool.execute(params, context.clone(), &options).await {
        Ok(result) => {
            println!("✅ 音频信息:");
            println!("  格式: {}", result["format"]);
            println!(
                "  时长: {} ({}秒)",
                result["duration_formatted"], result["duration_seconds"]
            );
            println!("  采样率: {} Hz", result["sample_rate"]);
            println!("  比特率: {} kbps", result["bit_rate"]);
            println!("  声道: {}", result["channels"]);
            println!("  文件大小: {} MB", result["file_size_mb"]);

            if let Some(metadata) = result.get("metadata") {
                println!("\n  🎼 元数据:");
                println!("    标题: {}", metadata["title"]);
                println!("    艺术家: {}", metadata["artist"]);
                println!("    专辑: {}", metadata["album"]);
                println!("    年份: {}", metadata["year"]);
                println!("    流派: {}", metadata["genre"]);
            }

            if let Some(waveform) = result.get("waveform_analysis") {
                println!("\n  📈 波形分析:");
                println!("    峰值振幅: {}", waveform["peak_amplitude"]);
                println!("    RMS 电平: {} dB", waveform["rms_level"]);
                println!("    动态范围: {} dB", waveform["dynamic_range"]);
            }
        }
        Err(e) => println!("❌ 分析失败: {}", e),
    }

    println!("\n");

    // 测试 2: 音频格式转换
    println!("🔄 测试 2: 音频格式转换");
    println!("{}", "-".repeat(80));

    let convert_tool = &tools[1];

    println!("工具名称: {}", convert_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", convert_tool.description());
    println!();

    let conversions = vec![
        (
            "WAV → MP3",
            "/path/to/input.wav",
            "/path/to/output.mp3",
            "mp3",
            320,
        ),
        (
            "MP3 → AAC",
            "/path/to/input.mp3",
            "/path/to/output.aac",
            "aac",
            256,
        ),
        (
            "WAV → FLAC",
            "/path/to/input.wav",
            "/path/to/output.flac",
            "flac",
            192,
        ),
    ];

    for (desc, input, output, format, bitrate) in conversions {
        println!("转换: {}", desc);

        let params = json!({
            "input_path": input,
            "output_path": output,
            "output_format": format,
            "bit_rate": bitrate,
            "sample_rate": 44100
        });

        match convert_tool
            .execute(params, context.clone(), &options)
            .await
        {
            Ok(result) => {
                if result["success"] == true {
                    println!("  ✅ 转换成功:");
                    println!(
                        "    输入: {} ({} MB)",
                        result["input_format"], result["input_size"]
                    );
                    println!(
                        "    输出: {} ({} MB)",
                        result["output_format"], result["output_size"]
                    );
                    println!("    比特率: {} kbps", result["bit_rate"]);
                    println!("    采样率: {} Hz", result["sample_rate"]);
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

    // 测试 3: 音频处理
    println!("🎛️  测试 3: 音频处理");
    println!("{}", "-".repeat(80));

    let process_tool = &tools[2];

    println!("工具名称: {}", process_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", process_tool.description());
    println!();

    let processing_scenarios = vec![
        (
            "场景 1: 音量调整",
            json!({
                "input_path": "/path/to/input.mp3",
                "output_path": "/path/to/louder.mp3",
                "operation": "volume",
                "volume_db": 5
            }),
        ),
        (
            "场景 2: 音频剪辑",
            json!({
                "input_path": "/path/to/input.mp3",
                "output_path": "/path/to/trimmed.mp3",
                "operation": "trim",
                "start_time": 30,
                "end_time": 90
            }),
        ),
        (
            "场景 3: 降噪处理",
            json!({
                "input_path": "/path/to/noisy.mp3",
                "output_path": "/path/to/clean.mp3",
                "operation": "denoise",
                "noise_reduction": true
            }),
        ),
        (
            "场景 4: 音频标准化",
            json!({
                "input_path": "/path/to/input.mp3",
                "output_path": "/path/to/normalized.mp3",
                "operation": "normalize"
            }),
        ),
    ];

    for (desc, params) in processing_scenarios {
        println!("{}", desc);

        match process_tool
            .execute(params, context.clone(), &options)
            .await
        {
            Ok(result) => {
                println!("  ✅ 处理结果:");
                println!("    操作: {}", result["operation"]);

                if let Some(volume) = result.get("volume_adjustment_db") {
                    println!("    音量调整: {} dB", volume);
                    println!("    处理后峰值: {}", result["peak_after"]);
                }

                if let Some(start) = result.get("start_time") {
                    println!("    开始时间: {}秒", start);
                    println!("    结束时间: {}秒", result["end_time"]);
                    println!("    剪辑时长: {}秒", result["duration"]);
                }

                if let Some(noise_reduction) = result.get("noise_reduction_db") {
                    println!("    降噪: {} dB", noise_reduction);
                    println!("    信噪比改善: {} dB", result["snr_improvement"]);
                }

                if let Some(target) = result.get("target_level_db") {
                    println!("    目标电平: {} dB", target);
                    println!("    应用增益: {} dB", result["gain_applied_db"]);
                }

                println!("    处理时间: {} ms", result["processing_time_ms"]);
            }
            Err(e) => println!("  ❌ 处理失败: {}", e),
        }
        println!();
    }

    // 测试 4: 参数验证
    println!("🔍 测试 4: 参数验证");
    println!("{}", "-".repeat(80));

    println!("测试 4.1: 无效的输出格式");
    let params = json!({
        "input_path": "/path/to/input.mp3",
        "output_path": "/path/to/output.ogg",
        "output_format": "ogg"
    });

    match convert_tool
        .execute(params, context.clone(), &options)
        .await
    {
        Ok(result) => {
            if result["success"] == false {
                println!("  ✅ 正确拒绝: {}", result["error"]);
            }
        }
        Err(e) => println!("  ❌ 执行失败: {}", e),
    }
    println!();

    println!("测试 4.2: 无效的比特率");
    let params = json!({
        "input_path": "/path/to/input.mp3",
        "output_path": "/path/to/output.mp3",
        "output_format": "mp3",
        "bit_rate": 500
    });

    match convert_tool
        .execute(params, context.clone(), &options)
        .await
    {
        Ok(result) => {
            if result["success"] == false {
                println!("  ✅ 正确拒绝: {}", result["error"]);
            }
        }
        Err(e) => println!("  ❌ 执行失败: {}", e),
    }
    println!();

    // 测试 5: 批量获取工具
    println!("📦 测试 5: 获取所有音频处理工具");
    println!("{}", "-".repeat(80));

    let all_tools = get_all_audio_processing_tools();
    println!("总共 {} 个音频处理工具:\n", all_tools.len());

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
    println!("✅ 音频处理工具演示完成！");
    println!("{}", "=".repeat(80));
    println!();
    println!("📊 工具统计:");
    println!("  - 音频信息分析工具: audio_info");
    println!("  - 音频格式转换工具: audio_convert");
    println!("  - 音频处理工具: audio_process");
    println!();
    println!("💡 使用建议:");
    println!("  1. 音频信息分析: 用于获取音频的详细信息和元数据");
    println!("  2. 格式转换: 支持 MP3, WAV, AAC, FLAC 之间的转换");
    println!("  3. 音频处理: 支持音量调整、剪辑、降噪、标准化");
    println!();
    println!("🚀 下一步:");
    println!("  - 集成真实的音频处理库（如 rodio, symphonia）");
    println!("  - 添加更多音频效果（均衡器、混响、压缩）");
    println!("  - 支持批量处理和实时处理");
    println!("{}", "=".repeat(80));
}
