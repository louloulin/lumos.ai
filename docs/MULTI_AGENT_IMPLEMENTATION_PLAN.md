# 多智能体协作模式实现计划

> **目标**: 完善 LumosAI 的多智能体协作能力，对标 AutoGen、CrewAI、LangGraph  
> **时间**: 3 周  
> **优先级**: P1 (生产就绪的关键功能)

## 📊 当前状态分析

### 已实现模式 (7/12) ✅

| 模式 | 实现位置 | 完成度 | 测试覆盖 |
|------|---------|--------|---------|
| Sequential | `operators.rs` (AgentPipeline) | 100% | ✅ |
| Parallel | `operators.rs` (AgentParallel) | 100% | ✅ |
| Hierarchical | `collaboration.rs` (Crew) | 100% | ✅ |
| DAG Orchestration | `dag_orchestration.rs` | 100% | ✅ |
| SOP React | `sop_environment.rs` | 100% | ⚠️ |
| SOP ByOrder | `collaboration.rs` | 100% | ⚠️ |
| SOP PlanAndAct | `collaboration.rs` | 100% | ⚠️ |

### 待实现模式 (5/12) ⚠️

| 模式 | 优先级 | 预计工期 | 依赖 |
|------|--------|---------|------|
| Group Chat | P1 | 3 天 | Communication |
| Handoff | P1 | 2 天 | - |
| Reflection | P1 | 2 天 | - |
| Magentic | P2 | 3 天 | DAG |
| Debate | P2 | 2 天 | Group Chat |

---

## 🎯 Week 1: Group Chat + Handoff (P1)

### Day 1-3: Group Chat 实现

#### 目标
实现完整的 Group Chat 协作模式，支持:
- 基础群聊框架
- Debate 子模式
- Consensus 子模式
- Maker-Checker Loop

#### 实现步骤

**Step 1: 核心数据结构 (4 小时)**

```rust
// lumosai_core/src/agent/group_chat.rs

use std::sync::Arc;
use tokio::sync::RwLock;
use crate::agent::Agent;
use crate::Result;

/// 群聊消息
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub agent_id: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: Option<serde_json::Value>,
}

/// 群聊线程
#[derive(Debug, Clone)]
pub struct ChatThread {
    messages: Vec<ChatMessage>,
    max_messages: usize,
}

impl ChatThread {
    pub fn new(max_messages: usize) -> Self {
        Self {
            messages: Vec::new(),
            max_messages,
        }
    }
    
    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
    }
    
    pub fn get_context(&self, last_n: usize) -> String {
        self.messages
            .iter()
            .rev()
            .take(last_n)
            .rev()
            .map(|m| format!("{}: {}", m.agent_id, m.content))
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    pub fn has_consensus(&self) -> bool {
        // 简单实现: 检查最后 3 条消息是否包含 "agree" 或 "consensus"
        self.messages
            .iter()
            .rev()
            .take(3)
            .all(|m| {
                m.content.to_lowercase().contains("agree") ||
                m.content.to_lowercase().contains("consensus")
            })
    }
}

/// 终止条件
pub enum TerminationCondition {
    MaxRounds(usize),
    Consensus,
    Custom(Box<dyn Fn(&ChatThread) -> bool + Send + Sync>),
}

/// 群聊管理器
pub struct GroupChat {
    agents: Vec<(String, Arc<dyn Agent>)>,
    thread: Arc<RwLock<ChatThread>>,
    termination: TerminationCondition,
    max_rounds: usize,
    current_round: usize,
}
```

**Step 2: 群聊执行逻辑 (4 小时)**

```rust
impl GroupChat {
    pub fn new() -> Self {
        Self {
            agents: Vec::new(),
            thread: Arc::new(RwLock::new(ChatThread::new(100))),
            termination: TerminationCondition::MaxRounds(10),
            max_rounds: 10,
            current_round: 0,
        }
    }
    
    pub fn add_agent(mut self, id: &str, agent: Arc<dyn Agent>) -> Self {
        self.agents.push((id.to_string(), agent));
        self
    }
    
    pub fn max_rounds(mut self, rounds: usize) -> Self {
        self.max_rounds = rounds;
        self.termination = TerminationCondition::MaxRounds(rounds);
        self
    }
    
    pub fn termination_condition<F>(mut self, condition: F) -> Self
    where
        F: Fn(&ChatThread) -> bool + Send + Sync + 'static,
    {
        self.termination = TerminationCondition::Custom(Box::new(condition));
        self
    }
    
    pub async fn execute(&mut self, initial_message: &str) -> Result<String> {
        // 添加初始消息
        {
            let mut thread = self.thread.write().await;
            thread.add_message(ChatMessage {
                agent_id: "system".to_string(),
                content: initial_message.to_string(),
                timestamp: chrono::Utc::now(),
                metadata: None,
            });
        }
        
        // 执行群聊循环
        while !self.should_terminate().await {
            self.current_round += 1;
            
            // 每个 Agent 依次发言
            for (agent_id, agent) in &self.agents {
                let context = {
                    let thread = self.thread.read().await;
                    thread.get_context(10)
                };
                
                // Agent 生成响应
                let prompt = format!(
                    "You are participating in a group discussion.\n\nConversation so far:\n{}\n\nYour response:",
                    context
                );
                
                let response = agent.generate_simple(&prompt).await?;
                
                // 添加到线程
                {
                    let mut thread = self.thread.write().await;
                    thread.add_message(ChatMessage {
                        agent_id: agent_id.clone(),
                        content: response,
                        timestamp: chrono::Utc::now(),
                        metadata: None,
                    });
                }
            }
        }
        
        // 返回最终结果
        let thread = self.thread.read().await;
        Ok(thread.get_context(self.agents.len()))
    }
    
    async fn should_terminate(&self) -> bool {
        match &self.termination {
            TerminationCondition::MaxRounds(max) => self.current_round >= *max,
            TerminationCondition::Consensus => {
                let thread = self.thread.read().await;
                thread.has_consensus()
            }
            TerminationCondition::Custom(f) => {
                let thread = self.thread.read().await;
                f(&*thread)
            }
        }
    }
}
```

**Step 3: Maker-Checker Loop (2 小时)**

```rust
/// Maker-Checker 循环
pub struct MakerCheckerLoop {
    maker: Arc<dyn Agent>,
    checker: Arc<dyn Agent>,
    max_iterations: usize,
}

impl MakerCheckerLoop {
    pub fn new(maker: Arc<dyn Agent>, checker: Arc<dyn Agent>) -> Self {
        Self {
            maker,
            checker,
            max_iterations: 5,
        }
    }
    
    pub fn max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }
    
    pub async fn execute(&self, task: &str) -> Result<String> {
        let mut current_output = String::new();
        
        for iteration in 0..self.max_iterations {
            // Maker 生成/改进
            let maker_prompt = if iteration == 0 {
                format!("Create: {}", task)
            } else {
                format!("Improve based on feedback:\nTask: {}\nPrevious: {}\nFeedback: {}", 
                    task, current_output, "...")
            };
            
            current_output = self.maker.generate_simple(&maker_prompt).await?;
            
            // Checker 审核
            let checker_prompt = format!(
                "Review this work:\nTask: {}\nWork: {}\n\nProvide feedback or say 'APPROVED' if acceptable.",
                task, current_output
            );
            
            let feedback = self.checker.generate_simple(&checker_prompt).await?;
            
            if feedback.to_lowercase().contains("approved") {
                return Ok(current_output);
            }
        }
        
        Ok(current_output)
    }
}
```

**Step 4: 集成到 Crew (2 小时)**

```rust
// lumosai_core/src/agent/collaboration.rs

impl Crew {
    async fn execute_group_chat(&self) -> Result<Vec<AgentTask>> {
        use super::group_chat::GroupChat;
        
        let mut chat = GroupChat::new().max_rounds(10);
        
        // 添加所有 Agent
        let agents = self.agents.read().await;
        for (id, agent) in agents.iter() {
            chat = chat.add_agent(id, agent.clone());
        }
        
        // 执行群聊
        let result = chat.execute("Begin collaboration").await?;
        
        // 转换为 AgentTask
        Ok(vec![AgentTask {
            id: "group_chat_result".to_string(),
            description: result,
            status: TaskStatus::Completed,
            ..Default::default()
        }])
    }
}
```

**Step 5: 测试 (4 小时)**

```rust
// tests/e2e/group_chat_tests.rs

#[tokio::test]
async fn test_group_chat_basic() {
    let ctx = E2ETestContext::setup().await.unwrap();
    
    let agent1 = Arc::new(ctx.create_test_agent("expert1", "You are expert 1").unwrap());
    let agent2 = Arc::new(ctx.create_test_agent("expert2", "You are expert 2").unwrap());
    
    let mut chat = GroupChat::new()
        .add_agent("expert1", agent1)
        .add_agent("expert2", agent2)
        .max_rounds(3);
    
    let result = chat.execute("Discuss AI safety").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_maker_checker_loop() {
    let ctx = E2ETestContext::setup().await.unwrap();
    
    let maker = Arc::new(ctx.create_test_agent("maker", "You create content").unwrap());
    let checker = Arc::new(ctx.create_test_agent("checker", "You review content").unwrap());
    
    let loop_exec = MakerCheckerLoop::new(maker, checker).max_iterations(3);
    
    let result = loop_exec.execute("Write a function to sort numbers").await;
    assert!(result.is_ok());
}
```

---

### Day 4-5: Handoff 实现

#### 目标
实现动态任务移交模式

#### 实现步骤

**Step 1: 核心结构 (3 小时)**

```rust
// lumosai_core/src/agent/handoff.rs

/// 移交条件
pub enum HandoffCondition {
    Contains(String),
    Regex(regex::Regex),
    Custom(Box<dyn Fn(&str) -> bool + Send + Sync>),
}

impl HandoffCondition {
    pub fn matches(&self, content: &str) -> bool {
        match self {
            Self::Contains(keyword) => content.to_lowercase().contains(&keyword.to_lowercase()),
            Self::Regex(re) => re.is_match(content),
            Self::Custom(f) => f(content),
        }
    }
}

/// 移交规则
pub struct HandoffRule {
    from_agent: String,
    to_agent: String,
    condition: HandoffCondition,
}

/// 移交编排器
pub struct HandoffOrchestrator {
    agents: HashMap<String, Arc<dyn Agent>>,
    rules: Vec<HandoffRule>,
    initial_agent: String,
    max_handoffs: usize,
}
```

**Step 2: 执行逻辑 (3 小时)**

```rust
impl HandoffOrchestrator {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            rules: Vec::new(),
            initial_agent: String::new(),
            max_handoffs: 10,
        }
    }
    
    pub fn add_agent(mut self, id: &str, agent: Arc<dyn Agent>) -> Self {
        if self.initial_agent.is_empty() {
            self.initial_agent = id.to_string();
        }
        self.agents.insert(id.to_string(), agent);
        self
    }
    
    pub fn add_handoff(mut self, from: &str, to: &str, condition: HandoffCondition) -> Self {
        self.rules.push(HandoffRule {
            from_agent: from.to_string(),
            to_agent: to.to_string(),
            condition,
        });
        self
    }
    
    pub async fn execute(&self, input: &str) -> Result<String> {
        let mut current_agent_id = self.initial_agent.clone();
        let mut current_input = input.to_string();
        let mut handoff_count = 0;
        
        loop {
            if handoff_count >= self.max_handoffs {
                return Err(Error::Agent("Max handoffs exceeded".to_string()));
            }
            
            // 获取当前 Agent
            let agent = self.agents.get(&current_agent_id)
                .ok_or_else(|| Error::Agent(format!("Agent not found: {}", current_agent_id)))?;
            
            // 执行 Agent
            let response = agent.generate_simple(&current_input).await?;
            
            // 检查是否需要移交
            let mut next_agent = None;
            for rule in &self.rules {
                if rule.from_agent == current_agent_id && rule.condition.matches(&response) {
                    next_agent = Some(rule.to_agent.clone());
                    break;
                }
            }
            
            match next_agent {
                Some(next) => {
                    current_agent_id = next;
                    current_input = response;
                    handoff_count += 1;
                }
                None => {
                    // 没有移交，返回结果
                    return Ok(response);
                }
            }
        }
    }
}
```

**Step 3: 测试 (2 小时)**

```rust
#[tokio::test]
async fn test_handoff_orchestration() {
    let ctx = E2ETestContext::setup().await.unwrap();
    
    let triage = Arc::new(ctx.create_test_agent("triage", "You triage requests").unwrap());
    let technical = Arc::new(ctx.create_test_agent("technical", "You handle technical issues").unwrap());
    
    let orchestrator = HandoffOrchestrator::new()
        .add_agent("triage", triage)
        .add_agent("technical", technical)
        .add_handoff("triage", "technical", HandoffCondition::Contains("network"));
    
    let result = orchestrator.execute("My network is down").await;
    assert!(result.is_ok());
}
```

---

## 🎯 Week 2: Reflection + Magentic (P1/P2)

### Day 6-7: Reflection 实现

**核心代码**:

```rust
// lumosai_core/src/agent/reflection.rs

pub struct ReflectionLoop {
    generator: Arc<dyn Agent>,
    critic: Arc<dyn Agent>,
    max_iterations: usize,
    improvement_threshold: f32,
}

impl ReflectionLoop {
    pub async fn execute(&self, task: &str) -> Result<String> {
        let mut current_output = String::new();
        let mut previous_score = 0.0;
        
        for iteration in 0..self.max_iterations {
            // 生成
            let gen_prompt = if iteration == 0 {
                task.to_string()
            } else {
                format!("Improve: {}\nPrevious: {}", task, current_output)
            };
            
            current_output = self.generator.generate_simple(&gen_prompt).await?;
            
            // 评估
            let eval_prompt = format!("Rate this (0-1): {}", current_output);
            let score_str = self.critic.generate_simple(&eval_prompt).await?;
            let score = score_str.parse::<f32>().unwrap_or(0.5);
            
            if score >= self.improvement_threshold {
                return Ok(current_output);
            }
            
            if score <= previous_score {
                break; // 没有改进
            }
            
            previous_score = score;
        }
        
        Ok(current_output)
    }
}
```

### Day 8-10: Magentic 实现

**核心代码**:

```rust
// lumosai_core/src/agent/magentic.rs

pub struct TaskLedger {
    tasks: Vec<Task>,
    completed: HashSet<String>,
}

pub struct MagenticOrchestrator {
    manager: Arc<dyn Agent>,
    workers: HashMap<String, Arc<dyn Agent>>,
    ledger: Arc<RwLock<TaskLedger>>,
    max_iterations: usize,
}

impl MagenticOrchestrator {
    pub async fn execute(&self, goal: &str) -> Result<(String, TaskLedger)> {
        // 1. Manager 生成初始任务清单
        // 2. 迭代执行任务
        // 3. Manager 评估进度并调整计划
        // 4. 返回结果和完整清单
        todo!()
    }
}
```

---

## 🎯 Week 3: Debate + 统一 API + 测试

### Day 11-12: Debate 实现
### Day 13-14: 统一 API 重构
### Day 15: 完整测试和文档

---

## 📝 验收标准

### 功能完整性
- [ ] 所有 12 种模式实现完成
- [ ] 统一 API 设计完成
- [ ] 所有模式可通过 `CollaborationMode` 枚举访问

### 测试覆盖
- [ ] 每种模式至少 3 个 E2E 测试
- [ ] 单元测试覆盖率 > 80%
- [ ] 集成测试覆盖所有模式组合

### 文档完善
- [ ] API 文档完整
- [ ] 每种模式有完整示例
- [ ] 模式选择指南
- [ ] 性能基准测试报告

### 性能指标
- [ ] Sequential: < 2x 单 Agent 延迟
- [ ] Parallel: < 1.2x 单 Agent 延迟
- [ ] Group Chat: < 10 轮收敛
- [ ] Handoff: < 5 次移交

---

## 🚀 下一步行动

1. **立即开始**: 实现 Group Chat (Day 1-3)
2. **并行准备**: 编写 Handoff 设计文档
3. **持续集成**: 每完成一个模式立即添加测试

**预计完成时间**: 2025-12-02

