# 多 Agent 协作系统示例

这是一个完整的多 Agent 协作系统示例，展示了如何使用 LumosAI 构建复杂的 Agent 团队来协作完成任务。

## 功能特性

- **团队协作**: 多个专业 Agent 协同工作
- **任务分配**: 智能分析任务并分配给合适的 Agent
- **并行处理**: 支持同时处理多个任务
- **结果整合**: 自动整合各个 Agent 的输出
- **角色专业化**: 每个 Agent 都有明确的专业领域

## 快速开始

### 1. 安装依赖

```bash
cd docs/examples/multi-agent
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
# 创建协作任务
cargo run -- create "为新产品制定营销策略" --agents 4

# 运行预定义场景
cargo run -- scenario product-launch

# 启动交互模式
cargo run -- interactive

# 查看任务状态
cargo run -- status <task-id>
```

## 使用场景

### 1. 产品发布协作

```bash
cargo run -- scenario product-launch
```

这个场景展示了如何协调多个 Agent 来制定产品发布计划：
- **市场分析师**: 分析市场趋势和竞争对手
- **产品经理**: 制定产品策略和功能规划
- **营销专家**: 设计营销活动和推广策略
- **风险评估师**: 识别潜在风险和应对措施

### 2. 研究项目协作

```bash
cargo run -- scenario research-project
```

展示学术研究项目的协作模式：
- **研究员**: 收集和分析数据
- **理论专家**: 提供理论框架
- **实验设计师**: 设计验证实验
- **报告撰写者**: 整理研究成果

### 3. 危机响应协作

```bash
cargo run -- scenario crisis-response
```

演示紧急情况下的快速响应协作：
- **问题诊断专家**: 快速定位问题根源
- **解决方案架构师**: 设计修复方案
- **执行协调员**: 协调实施步骤
- **沟通专家**: 处理对外沟通

## 示例代码

### 基础团队协作

```rust
use lumosai::prelude::SimpleAgent;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建专业团队
    let mut team = HashMap::new();
    
    // 市场分析师
    let analyst = lumosai::agent::simple(
        "gpt-3.5-turbo",
        "你是资深市场分析师，擅长市场调研和趋势预测。"
    ).await?;
    team.insert("analyst", analyst);
    
    // 产品经理
    let pm = lumosai::agent::simple(
        "gpt-3.5-turbo", 
        "你是经验丰富的产品经理，负责产品规划和用户体验。"
    ).await?;
    team.insert("pm", pm);
    
    // 协作执行任务
    let task = "为 AI 编程助手制定产品策略";
    
    // 各自分析
    let analyst_view = team["analyst"].chat(&format!(
        "请分析这个产品的市场前景：{}", task
    )).await?;
    
    let pm_view = team["pm"].chat(&format!(
        "请从产品角度分析：{}", task  
    )).await?;
    
    // 整合结果
    let final_strategy = team["pm"].chat(&format!(
        "基于以下分析，制定完整策略：\n市场分析：{}\n产品分析：{}", 
        analyst_view, pm_view
    )).await?;
    
    println!("最终策略：{}", final_strategy);
    Ok(())
}
```

### 并行协作处理

```rust
use futures::future::join_all;

// 并行处理多个子任务
let tasks = vec![
    "市场调研",
    "竞争分析", 
    "用户研究",
    "技术评估"
];

let futures: Vec<_> = tasks.iter().map(|task| {
    let agent = &team["analyst"];
    async move {
        agent.chat(&format!("请完成：{}", task)).await
    }
}).collect();

let results = join_all(futures).await;
```

## 架构设计

### 协作模式

1. **分层协作**: 
   - 协调层：任务分析和分配
   - 执行层：专业 Agent 执行
   - 整合层：结果汇总和优化

2. **角色专业化**:
   - 每个 Agent 有明确的专业领域
   - 专业指令确保输出质量
   - 角色间互补协作

3. **通信机制**:
   - 异步消息传递
   - 上下文共享
   - 结果反馈循环

### 任务管理

```rust
pub struct MultiAgentSystem {
    agents: HashMap<String, SimpleAgent>,
    tasks: Arc<Mutex<HashMap<String, Task>>>,
}

pub struct Task {
    id: String,
    description: String,
    status: TaskStatus,
    agents: Vec<AgentInfo>,
    messages: Vec<Message>,
    result: Option<String>,
}
```

### 执行流程

1. **任务分析**: 分析任务复杂度和所需专业领域
2. **团队组建**: 选择合适的专业 Agent
3. **并行执行**: Agent 同时处理各自的子任务
4. **结果整合**: 汇总和优化所有输出
5. **质量验证**: 检查结果完整性和一致性

## 高级功能

### 动态团队组建

```rust
impl MultiAgentSystem {
    pub async fn create_dynamic_team(&self, task: &str) -> Result<Vec<AgentInfo>> {
        // 分析任务需求
        let requirements = self.analyze_task_requirements(task).await?;
        
        // 选择合适的 Agent 角色
        let roles = self.select_optimal_roles(&requirements)?;
        
        // 创建专业 Agent
        let agents = self.create_specialized_agents(roles).await?;
        
        Ok(agents)
    }
}
```

### 智能负载均衡

```rust
impl MultiAgentSystem {
    pub async fn distribute_workload(&self, tasks: Vec<String>) -> Result<()> {
        // 评估每个 Agent 的当前负载
        let agent_loads = self.assess_agent_loads().await?;
        
        // 智能分配任务
        let assignments = self.optimize_task_assignment(tasks, agent_loads)?;
        
        // 并行执行
        self.execute_assignments(assignments).await?;
        
        Ok(())
    }
}
```

### 结果质量控制

```rust
impl MultiAgentSystem {
    pub async fn quality_control(&self, results: Vec<String>) -> Result<String> {
        // 一致性检查
        let consistency_score = self.check_consistency(&results).await?;
        
        // 完整性验证
        let completeness_score = self.verify_completeness(&results).await?;
        
        // 如果质量不达标，触发重新处理
        if consistency_score < 0.8 || completeness_score < 0.8 {
            return self.reprocess_with_feedback(results).await;
        }
        
        // 整合最终结果
        self.integrate_final_result(results).await
    }
}
```

## 性能优化

### 并发控制

- **连接池**: 复用 HTTP 连接减少延迟
- **批处理**: 合并相似请求提高效率
- **缓存机制**: 缓存常用结果避免重复计算

### 资源管理

- **内存优化**: 及时清理不需要的数据
- **超时控制**: 避免长时间等待
- **错误恢复**: 自动重试和降级处理

## 故障排除

### 常见问题

1. **Agent 响应慢**
   ```bash
   # 检查网络连接
   curl -I https://api.openai.com/v1/models
   
   # 启用详细日志
   RUST_LOG=debug cargo run
   ```

2. **协作结果不一致**
   ```rust
   // 增加上下文共享
   let shared_context = format!(
       "项目背景：{}\n其他 Agent 的观点：{}", 
       project_context, other_views
   );
   ```

3. **任务分配不合理**
   ```rust
   // 优化角色匹配算法
   let role_match_score = calculate_role_task_compatibility(role, task);
   ```

## 扩展示例

查看 `examples/` 目录中的更多示例：

- **team_collaboration.rs**: 基础团队协作演示
- **agent_coordination.rs**: 智能客服协调系统

## 相关示例

- **hello-world**: 基础 Agent 使用
- **chatbot**: 单 Agent 对话系统
- **research-assistant**: 工具增强的 Agent
- **rag-system**: 知识库问答系统
- **workflow-automation**: 工作流自动化（即将推出）

## 许可证

本示例遵循 MIT 许可证。
