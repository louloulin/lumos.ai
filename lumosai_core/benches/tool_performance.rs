//! 工具性能基准测试
//!
//! 对比宏驱动工具和手动实现工具的性能差异

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use lumosai_core::tool::builtin::macro_tools::*;
use lumosai_core::tool::{Tool, ToolExecutionContext, ToolExecutionOptions};
use serde_json::json;
use tokio::runtime::Runtime;

/// 创建测试用的执行上下文
fn create_test_context() -> ToolExecutionContext {
    ToolExecutionContext {
        thread_id: Some("bench-thread".to_string()),
        resource_id: Some("bench-resource".to_string()),
        run_id: Some("bench-run".to_string()),
        tool_call_id: Some("bench-call".to_string()),
        messages: None,
        abort_signal: None,
    }
}

/// 创建测试用的执行选项
fn create_test_options() -> ToolExecutionOptions {
    ToolExecutionOptions {
        context: None,
        validate_params: true,
        validate_output: false,
    }
}

/// 基准测试：JSON 解析工具
fn bench_json_parser(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let tool = parse_json_tool();
    let context = create_test_context();
    let options = create_test_options();
    
    let mut group = c.benchmark_group("json_parser");
    
    // 测试不同大小的 JSON
    for size in [10, 100, 1000].iter() {
        let json_data = generate_json_data(*size);
        
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}fields", size)),
            &json_data,
            |b, data| {
                b.to_async(&rt).iter(|| async {
                    let params = json!({
                        "json_string": data,
                        "validate_schema": false
                    });
                    
                    tool.execute(
                        black_box(params),
                        black_box(context.clone()),
                        black_box(&options)
                    ).await.unwrap()
                });
            },
        );
    }
    
    group.finish();
}

/// 基准测试：文本处理工具
fn bench_text_processor(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let tool = process_text_tool();
    let context = create_test_context();
    let options = create_test_options();
    
    let mut group = c.benchmark_group("text_processor");
    
    // 测试不同操作
    let operations = vec![
        ("uppercase", "hello world"),
        ("lowercase", "HELLO WORLD"),
        ("trim", "  hello world  "),
        ("reverse", "hello world"),
        ("length", "hello world"),
    ];
    
    for (op, text) in operations.iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(op),
            &(op, text),
            |b, (operation, input_text)| {
                b.to_async(&rt).iter(|| async {
                    let params = json!({
                        "text": input_text,
                        "operation": operation
                    });
                    
                    tool.execute(
                        black_box(params),
                        black_box(context.clone()),
                        black_box(&options)
                    ).await.unwrap()
                });
            },
        );
    }
    
    group.finish();
}

/// 基准测试：HTTP 工具
fn bench_http_tools(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let context = create_test_context();
    let options = create_test_options();

    let mut group = c.benchmark_group("http_tools");

    // HTTP GET
    let get_tool = http_get_tool();
    group.bench_function("http_get", |b| {
        b.to_async(&rt).iter(|| async {
            let params = json!({
                "url": "https://api.example.com/test",
                "timeout_seconds": 30
            });

            get_tool.execute(
                black_box(params),
                black_box(context.clone()),
                black_box(&options)
            ).await.unwrap()
        });
    });

    // HTTP POST
    let post_tool = http_post_tool();
    group.bench_function("http_post", |b| {
        b.to_async(&rt).iter(|| async {
            let params = json!({
                "url": "https://api.example.com/test",
                "body": {"test": "data", "value": 123},
                "timeout_seconds": 30
            });

            post_tool.execute(
                black_box(params),
                black_box(context.clone()),
                black_box(&options)
            ).await.unwrap()
        });
    });

    // API Call
    let api_tool = api_call_tool();
    group.bench_function("api_call", |b| {
        b.to_async(&rt).iter(|| async {
            let params = json!({
                "url": "https://api.example.com/test",
                "method": "GET"
            });

            api_tool.execute(
                black_box(params),
                black_box(context.clone()),
                black_box(&options)
            ).await.unwrap()
        });
    });

    group.finish();
}

/// 基准测试：工具创建开销
fn bench_tool_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("tool_creation");
    
    group.bench_function("create_json_parser", |b| {
        b.iter(|| {
            black_box(parse_json_tool())
        });
    });
    
    group.bench_function("create_text_processor", |b| {
        b.iter(|| {
            black_box(process_text_tool())
        });
    });
    
    group.bench_function("create_http_get", |b| {
        b.iter(|| {
            black_box(http_get_tool())
        });
    });
    
    group.bench_function("create_all_tools", |b| {
        b.iter(|| {
            black_box(get_all_macro_tools())
        });
    });
    
    group.finish();
}

/// 生成测试用的 JSON 数据
fn generate_json_data(fields: usize) -> String {
    let mut data = String::from("{");
    for i in 0..fields {
        if i > 0 {
            data.push_str(",");
        }
        data.push_str(&format!(r#""field{}":"value{}""#, i, i));
    }
    data.push_str("}");
    data
}

criterion_group!(
    benches,
    bench_json_parser,
    bench_text_processor,
    bench_http_tools,
    bench_tool_creation
);

criterion_main!(benches);

