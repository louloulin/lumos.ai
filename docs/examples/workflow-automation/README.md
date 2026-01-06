# 工作流自动化系统示例

这是一个完整的工作流自动化系统示例，展示了如何使用 LumosAI 构建复杂的自动化工作流，实现多步骤任务的编排和执行。

## 功能特性

- **工作流编排**: 定义和管理复杂的多步骤工作流
- **依赖管理**: 支持步骤间的依赖关系和顺序执行
- **并行执行**: 支持无依赖步骤的并行处理
- **状态跟踪**: 实时跟踪工作流和步骤的执行状态
- **错误处理**: 完善的错误处理和恢复机制
- **模板系统**: 预定义的工作流模板，快速创建常用流程
- **上下文管理**: 步骤间的数据传递和共享

## 快速开始

### 1. 安装依赖

```bash
cd docs/examples/workflow-automation
cargo build
```

### 2. 设置环境变量

```bash
# 设置 OpenAI API 密钥（或其他支持的模型提供商）
export OPENAI_API_KEY="your-api-key-here"

# 可选：设置其他模型提供商
export ANTHROPIC_API_KEY="your-anthropic-key"
export GOOGLE_API_KEY="your-google-key"
```

### 3. 运行示例

```bash
# 创建自定义工作流
cargo run -- create "内容创作流程" "从研究到发布的完整流程" --steps 4

# 使用预定义模板
cargo run -- template content-pipeline

# 执行工作流
cargo run -- execute <workflow-id> --input "人工智能在教育中的应用"

# 查看工作流状态
cargo run -- status <workflow-id>

# 列出所有工作流
cargo run -- list
```

## 工作流模板

### 1. 内容创作流水线 (content-pipeline)

```bash
cargo run -- template content-pipeline
```

完整的内容创作流程：
- **研究阶段**: 收集和分析相关信息
- **创作阶段**: 基于研究结果创作内容
- **审查阶段**: 质量检查和改进建议

### 2. 数据分析工作流 (data-analysis)

```bash
cargo run -- template data-analysis
```

数据分析和报告生成流程：
- **数据收集**: 收集和整理分析数据
- **数据分析**: 深入分析和模式识别
- **报告生成**: 生成专业分析报告

### 3. 项目审查工作流 (project-review)

```bash
cargo run -- template project-review
```

项目管理和质量控制流程：
- **项目规划**: 制定计划和里程碑
- **执行审查**: 审查执行情况和进度
- **质量检查**: 全面质量检查和验证

## 使用示例

### 基础工作流创建

```rust
use anyhow::Result;
use lumosai::prelude::SimpleAgent;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建专业 Agent
    let researcher = lumosai::agent::simple(
        "gpt-3.5-turbo",
        "你是专业研究员，擅长收集和分析信息。"
    ).await?;
    
    let writer = lumosai::agent::simple(
        "gpt-3.5-turbo", 
        "你是专业作家，擅长将研究结果转化为易读的内容。"
    ).await?;
    
    // 执行工作流步骤
    let topic = "人工智能在教育中的应用";
    
    // 步骤 1: 研究
    let research_result = researcher.chat(&format!(
        "请研究以下主题：{}", topic
    )).await?;
    
    // 步骤 2: 写作
    let article = writer.chat(&format!(
        "基于以下研究结果写文章：{}", research_result
    )).await?;
    
    println!("工作流完成！文章已生成。");
    Ok(())
}
```

### 并行工作流执行

```rust
use futures::future::join_all;

// 创建并行任务
let tasks = vec![
    market_analyst.chat("分析市场前景"),
    tech_analyst.chat("评估技术方案"),
    financial_analyst.chat("分析财务状况"),
    risk_analyst.chat("识别风险因素"),
];

// 并行执行
let results = join_all(tasks).await;
```

### 工作流状态管理

```rust
pub struct WorkflowEngine {
    workflows: Arc<Mutex<HashMap<String, Workflow>>>,
    agents: HashMap<String, SimpleAgent>,
}

impl WorkflowEngine {
    pub async fn execute_workflow(&self, workflow_id: &str) -> Result<()> {
        let mut workflow = self.get_workflow(workflow_id)?;
        workflow.status = WorkflowStatus::Running;
        
        for step in &mut workflow.steps {
            // 检查依赖
            if self.dependencies_satisfied(&workflow, step) {
                // 执行步骤
                let result = self.execute_step(step).await?;
                step.result = Some(result);
                step.status = StepStatus::Completed;
            }
        }
        
        workflow.status = WorkflowStatus::Completed;
        Ok(())
    }
}
```

## 架构设计

### 核心组件

1. **WorkflowEngine**: 工作流执行引擎
   - 工作流管理和调度
   - 步骤执行和状态跟踪
   - 错误处理和恢复

2. **Workflow**: 工作流定义
   - 步骤定义和依赖关系
   - 执行状态和上下文
   - 元数据和配置

3. **WorkflowStep**: 工作流步骤
   - Agent 角色分配
   - 输入输出映射
   - 执行状态跟踪

### 执行模型

```
工作流创建 → 步骤规划 → 依赖检查 → 并行执行 → 结果整合 → 状态更新
     ↓           ↓           ↓           ↓           ↓           ↓
   定义步骤   → 分析依赖   → 验证条件   → 执行任务   → 收集输出   → 更新状态
```

### 数据流

```
输入数据 → 工作流上下文 → 步骤输入 → Agent 处理 → 步骤输出 → 上下文更新
    ↓           ↓           ↓           ↓           ↓           ↓
  用户输入   → 全局状态   → 局部数据   → AI 处理   → 处理结果   → 状态同步
```

## 高级功能

### 条件执行

```rust
pub struct ConditionalStep {
    condition: String,
    true_step: WorkflowStep,
    false_step: Option<WorkflowStep>,
}
```

### 循环执行

```rust
pub struct LoopStep {
    condition: String,
    body: Vec<WorkflowStep>,
    max_iterations: usize,
}
```

### 错误恢复

```rust
pub struct ErrorHandler {
    retry_count: usize,
    fallback_step: Option<WorkflowStep>,
    error_notification: bool,
}
```

### 动态工作流

```rust
impl WorkflowEngine {
    pub async fn create_dynamic_workflow(&self, requirements: &str) -> Result<Workflow> {
        let planner = self.get_agent("workflow_planner")?;
        let plan = planner.chat(&format!(
            "根据需求创建工作流计划：{}", requirements
        )).await?;
        
        self.parse_workflow_plan(&plan)
    }
}
```

## 性能优化

### 并发控制

- **连接池**: 复用 HTTP 连接
- **批处理**: 合并相似请求
- **缓存**: 缓存中间结果

### 资源管理

- **内存优化**: 及时清理不需要的数据
- **超时控制**: 避免长时间等待
- **负载均衡**: 分散 Agent 负载

### 监控指标

- **执行时间**: 工作流和步骤的执行时间
- **成功率**: 工作流和步骤的成功率
- **资源使用**: CPU、内存、网络使用情况
- **错误统计**: 错误类型和频率分析

## 扩展示例

查看 `examples/` 目录中的更多示例：

- **simple_workflow.rs**: 基础工作流概念演示
- **parallel_workflow.rs**: 并行执行工作流示例

## 故障排除

### 常见问题

1. **工作流执行失败**
   ```bash
   # 检查步骤依赖
   cargo run -- status <workflow-id>
   
   # 启用详细日志
   RUST_LOG=debug cargo run
   ```

2. **步骤超时**
   ```rust
   // 增加超时时间
   step.timeout = Duration::from_secs(300);
   ```

3. **依赖循环**
   ```rust
   // 检查步骤依赖图
   fn validate_dependencies(steps: &[WorkflowStep]) -> Result<()> {
       // 拓扑排序检查循环依赖
   }
   ```

### 调试技巧

```bash
# 启用详细日志
export RUST_LOG=debug

# 查看工作流执行轨迹
cargo run -- status <workflow-id>

# 检查 Agent 响应
export RUST_LOG=lumosai=trace
```

## 相关示例

- **hello-world**: 基础 Agent 使用
- **chatbot**: 单 Agent 对话系统
- **research-assistant**: 工具增强的 Agent
- **rag-system**: 知识库问答系统
- **multi-agent**: 多 Agent 协作系统

## 许可证

本示例遵循 MIT 许可证。
