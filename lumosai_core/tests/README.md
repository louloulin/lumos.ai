# LumosAI Core 测试套件

本目录包含 LumosAI Core 库的综合测试套件，验证各个组件的功能和集成。

## 测试文件

### `mastra_integration_comprehensive_test.rs`

这是一个全面的集成测试文件，测试 LumosAI Core 的各个核心功能：

#### 测试覆盖的功能

1. **流式集成测试** (`test_comprehensive_streaming_integration`)
   - 测试 AI 代理的流式响应功能
   - 验证 MockLlmProvider 的流式输出
   - 检查消息处理和响应生成

2. **动态参数和运行时上下文** (`test_dynamic_arguments_and_runtime_context`)
   - 测试运行时上下文的创建和管理
   - 验证元数据的设置和获取
   - 测试动态参数处理

3. **评估指标系统** (`test_evaluation_metrics_system`)
   - 测试 RelevanceMetric 的评估功能
   - 验证评估结果的分数范围
   - 检查评估指标的基本功能

4. **内存处理器系统** (`test_memory_processors_system`)
   - 测试 MessageLimitProcessor 的消息限制功能
   - 验证消息历史的管理
   - 检查内存处理器的基本操作

5. **函数调用集成** (`test_function_calling_integration`)
   - 测试自定义工具的实现和执行
   - 验证 Tool trait 的正确实现
   - 检查工具参数处理和结果返回

#### 测试组件

- **MockLlmProvider**: 模拟 LLM 提供者，用于测试
- **CalculatorTool**: 自定义工具实现示例
- **RuntimeContext**: 运行时上下文管理
- **RelevanceMetric**: 相关性评估指标
- **MessageLimitProcessor**: 消息限制处理器

#### 运行测试

```bash
# 运行所有测试
cargo test --package lumosai_core --test mastra_integration_comprehensive_test

# 运行特定测试
cargo test --package lumosai_core --test mastra_integration_comprehensive_test test_comprehensive_streaming_integration
```

#### 测试特点

- **异步测试**: 所有测试都使用 `tokio::test` 进行异步测试
- **模拟组件**: 使用模拟组件避免外部依赖
- **全面覆盖**: 测试涵盖核心功能的各个方面
- **集成测试**: 测试组件之间的交互和集成

#### 测试输出示例

```
🧪 Testing comprehensive streaming integration...
✅ Streaming integration test passed:
   - Provider: MockLlmProvider
   - Response length: 42
   - Stream chunks: 3

🧪 Testing dynamic arguments and runtime context...
✅ Dynamic arguments test passed:
   - Context metadata: test
   - Dynamic processing: enabled

🧪 Testing evaluation metrics system...
✅ Evaluation metrics test passed:
   - Metric name: relevance
   - Score range: (0.0, 1.0)

🧪 Testing memory processors system...
✅ Memory processors test passed:
   - Original messages: 5
   - Processed messages: 3
   - Limit applied: true

🧪 Testing function calling integration...
✅ Function calling test passed:
   - Tool name: Some("calculator")
   - Tool ID: calculator
```

## 完整测试列表

### 单元测试 (84 个)
- **Agent 模块**: 18 个测试
- **LLM 模块**: 20 个测试
- **Memory 模块**: 12 个测试
- **Storage 模块**: 3 个测试
- **Telemetry 模块**: 12 个测试
- **Tool 模块**: 5 个测试
- **Vector 模块**: 6 个测试
- **Workflow 模块**: 3 个测试
- **其他**: 5 个测试

### 集成测试 (34 个)
- **agent_memory_test.rs**: 7 个测试
- **function_calling.rs**: 4 个测试
- **llm_qwen_test.rs**: 4 个测试
- **mastra_integration_comprehensive_test.rs**: 5 个测试
- **mastra_validation_test.rs**: 5 个测试
- **websocket_streaming_tests.rs**: 9 个测试

### 文档测试 (1 个)
- **FunctionSchema 文档示例**: 1 个测试

## 运行所有测试

```bash
# 运行所有测试（单元测试 + 集成测试 + 文档测试）
cargo test --package lumosai_core

# 只运行单元测试
cargo test --package lumosai_core --lib

# 只运行集成测试
cargo test --package lumosai_core --tests

# 运行特定的集成测试文件
cargo test --package lumosai_core --test mastra_integration_comprehensive_test

# 列出所有测试
cargo test --package lumosai_core -- --list
```

## 注意事项

- 测试使用模拟组件，不需要真实的 AI 服务
- 所有测试都应该快速执行且可重复
- 测试覆盖了错误处理和边界情况
- 测试验证了 API 的正确性和一致性
- 总计 **119 个测试**，确保代码质量和功能完整性

## 贡献

添加新测试时，请确保：
1. 使用适当的测试名称和文档
2. 包含必要的断言和验证
3. 使用模拟组件避免外部依赖
4. 添加适当的错误处理测试
5. 在 Cargo.toml 中正确配置集成测试
