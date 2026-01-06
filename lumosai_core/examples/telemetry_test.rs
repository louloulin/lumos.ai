//! OpenTelemetry 和 Metrics 集成测试示例
//!
//! 运行方式: cargo run --example telemetry_test

use lumosai_core::telemetry::metrics::PrometheusMetrics;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== OpenTelemetry 和 Metrics 集成测试 ===\n");

    // 测试 1: OpenTelemetry Tracer 配置
    println!("测试 1: OpenTelemetry Tracer 配置");
    let tracer_config = lumosai_core::telemetry::tracing::TracerConfig::builder()
        .with_service_name("test-agent")
        .with_sample_rate(0.5)
        .with_attribute("env", "test")
        .build()?;

    println!("✅ TracerConfig 创建成功");
    println!("  - 服务名: {}", tracer_config.service_name);
    println!("  - 采样率: {}", tracer_config.sample_rate);
    println!("  - 属性数: {}\n", tracer_config.attributes.len());

    // 测试 2: Prometheus Metrics 基础功能
    println!("测试 2: Prometheus Metrics 基础功能");
    let metrics = PrometheusMetrics::new(Default::default());

    // 注册 Counter
    metrics.register_counter("test_requests_total", "Total test requests")?;
    metrics.increment_counter("test_requests_total")?;
    metrics.increment_counter_by("test_requests_total", 5.0)?;
    println!("✅ Counter 注册和更新成功");

    // 注册 Gauge
    metrics.register_gauge("test_active_connections", "Active test connections")?;
    metrics.set_gauge("test_active_connections", 42.0)?;
    println!("✅ Gauge 注册和设置成功");

    // 注册 Histogram
    metrics.register_histogram("test_latency_seconds", "Test request latency")?;
    metrics.record_latency("test_latency_seconds", 0.1)?;
    metrics.record_latency("test_latency_seconds", 0.5)?;
    metrics.record_latency("test_latency_seconds", 1.0)?;
    println!("✅ Histogram 注册和记录成功");

    // 测试 3: 标准 Agent 指标
    println!("\n测试 3: 预定义标准 Agent 指标");
    let agent_metrics = PrometheusMetrics::new(Default::default());

    agent_metrics.register_agent_metrics()?;
    println!("✅ Agent 指标注册成功");

    agent_metrics.register_llm_metrics()?;
    println!("✅ LLM 指标注册成功");

    agent_metrics.register_rag_metrics()?;
    println!("✅ RAG 指标注册成功");

    agent_metrics.register_vector_db_metrics()?;
    println!("✅ Vector DB 指标注册成功");

    // 测试 4: 指标导出
    println!("\n测试 4: 指标导出");
    let exported = metrics.export()?;
    println!("✅ 指标导出成功");
    println!("  - 导出字节数: {} bytes", exported.len());
    println!("\n导出内容预览:");
    println!("{}", exported.lines().take(10).collect::<Vec<&str>>().join("\n"));

    println!("\n=== 所有测试通过! ✅ ===");
    Ok(())
}
