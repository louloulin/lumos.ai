//! Agent 操作符支持
//!
//! 提供类似 CangjieMagic 风格的操作符，用于 Agent 之间的协作：
//! - `pipe()` - 管道操作符（类似 `|>`）：顺序执行
//! - `delegate()` - 委托操作符（类似 `<=`）：委托任务
//! - `parallel()` - 并行操作符（类似 `|`）：并行执行
//!
//! # 示例
//!
//! ```rust
//! use lumosai_core::agent::operators::*;
//!
//! // 管道操作：researcher |> analyzer |> writer
//! let result = researcher
//!     .pipe(analyzer)
//!     .pipe(writer)
//!     .execute("Research AI trends")
//!     .await?;
//!
//! // 委托操作：manager <= worker
//! let result = manager
//!     .delegate(worker, "Complete the task")
//!     .await?;
//!
//! // 并行操作：agent1 | agent2 | agent3
//! let result = agent1
//!     .parallel(agent2)
//!     .parallel(agent3)
//!     .execute("Process data")
//!     .await?;
//! ```

use crate::agent::Agent;
use crate::error::Result;
use crate::llm::{Message, Role};
    use crate::llm::test_helpers::{create_test_zhipu_provider, create_test_zhipu_provider_arc};
use std::sync::Arc;

/// Agent 管道链
///
/// 用于实现 Agent 之间的顺序执行（管道操作符 `|>`）
pub struct AgentPipeline {
    agents: Vec<Arc<dyn Agent>>,
}

impl AgentPipeline {
    /// 创建新的管道链
    pub fn new(agent: Arc<dyn Agent>) -> Self {
        Self {
            agents: vec![agent],
        }
    }

    /// 添加下一个 Agent 到管道链（类似 `|>` 操作符）
    ///
    /// # 示例
    ///
    /// ```rust
    /// let pipeline = AgentPipeline::new(researcher)
    ///     .pipe(analyzer)
    ///     .pipe(writer);
    /// ```
    pub fn pipe(mut self, agent: Arc<dyn Agent>) -> Self {
        self.agents.push(agent);
        self
    }

    /// 执行管道链
    ///
    /// 每个 Agent 的输出会作为下一个 Agent 的输入
    pub async fn execute(&self, input: &str) -> Result<String> {
        let mut current_input = input.to_string();

        for agent in &self.agents {
            let messages = vec![Message {
                role: Role::User,
                content: current_input.clone(),
                metadata: None,
                name: None,
            }];
            let result = agent.generate(&messages, &Default::default()).await?;
            current_input = result.response;
        }

        Ok(current_input)
    }

    /// 获取管道中的所有 Agent
    pub fn agents(&self) -> &[Arc<dyn Agent>] {
        &self.agents
    }
}

/// Agent 并行执行器
///
/// 用于实现 Agent 之间的并行执行（并行操作符 `|`）
pub struct AgentParallel {
    agents: Vec<Arc<dyn Agent>>,
}

impl AgentParallel {
    /// 创建新的并行执行器
    pub fn new(agent: Arc<dyn Agent>) -> Self {
        Self {
            agents: vec![agent],
        }
    }

    /// 添加并行执行的 Agent（类似 `|` 操作符）
    ///
    /// # 示例
    ///
    /// ```rust
    /// let parallel = AgentParallel::new(agent1)
    ///     .parallel(agent2)
    ///     .parallel(agent3);
    /// ```
    pub fn parallel(mut self, agent: Arc<dyn Agent>) -> Self {
        self.agents.push(agent);
        self
    }

    /// 并行执行所有 Agent
    ///
    /// 所有 Agent 接收相同的输入，并行执行，返回所有结果
    pub async fn execute(&self, input: &str) -> Result<Vec<String>> {
        let messages = vec![Message {
            role: Role::User,
            content: input.to_string(),
            metadata: None,
            name: None,
        }];
        let mut tasks = Vec::new();

        for agent in &self.agents {
            let agent = Arc::clone(agent);
            let messages = messages.clone();
            let task =
                tokio::spawn(async move { agent.generate(&messages, &Default::default()).await });
            tasks.push(task);
        }

        let mut results = Vec::new();
        for task in tasks {
            let result = task
                .await
                .map_err(|e| crate::error::Error::Agent(format!("Task join error: {}", e)))??;
            results.push(result.response);
        }

        Ok(results)
    }

    /// 获取并行执行的所有 Agent
    pub fn agents(&self) -> &[Arc<dyn Agent>] {
        &self.agents
    }
}

/// Agent 委托操作
///
/// 用于实现 Agent 之间的任务委托（委托操作符 `<=`）
pub struct AgentDelegation {
    manager: Arc<dyn Agent>,
    worker: Arc<dyn Agent>,
    task_description: String,
}

impl AgentDelegation {
    /// 创建新的委托操作
    ///
    /// # 参数
    ///
    /// - `manager`: 管理者 Agent
    /// - `worker`: 工作者 Agent
    /// - `task_description`: 任务描述
    pub fn new(manager: Arc<dyn Agent>, worker: Arc<dyn Agent>, task_description: String) -> Self {
        Self {
            manager,
            worker,
            task_description,
        }
    }

    /// 执行委托操作
    ///
    /// 1. Manager 接收任务并生成详细指令
    /// 2. Worker 根据指令执行任务
    /// 3. Manager 审查 Worker 的结果
    pub async fn execute(&self, input: &str) -> Result<String> {
        // 1. Manager 生成详细指令
        let manager_prompt = format!(
            "Task: {}\nInput: {}\n\nGenerate detailed instructions for a worker agent to complete this task.",
            self.task_description, input
        );
        let messages = vec![Message {
            role: Role::User,
            content: manager_prompt,
            metadata: None,
            name: None,
        }];
        let instructions = self
            .manager
            .generate(&messages, &Default::default())
            .await?;

        // 2. Worker 执行任务
        let worker_prompt = format!(
            "Instructions: {}\n\nComplete the task according to these instructions.",
            instructions.response
        );
        let messages = vec![Message {
            role: Role::User,
            content: worker_prompt,
            metadata: None,
            name: None,
        }];
        let worker_result = self.worker.generate(&messages, &Default::default()).await?;

        // 3. Manager 审查结果
        let review_prompt = format!(
            "Original task: {}\nWorker's result: {}\n\nReview and finalize the result.",
            self.task_description, worker_result.response
        );
        let messages = vec![Message {
            role: Role::User,
            content: review_prompt,
            metadata: None,
            name: None,
        }];
        let final_result = self
            .manager
            .generate(&messages, &Default::default())
            .await?;

        Ok(final_result.response)
    }
}

/// Agent 操作符扩展函数
///
/// 提供便捷的操作符函数，用于 Agent 协作

/// 创建管道链（类似 `|>` 操作符）
///
/// # 示例
///
/// ```rust
/// use lumosai_core::agent::operators::pipe;
///
/// let pipeline = pipe(researcher, analyzer);
/// let result = pipeline.execute("Research AI trends").await?;
/// ```
pub fn pipe(first: Arc<dyn Agent>, second: Arc<dyn Agent>) -> AgentPipeline {
    AgentPipeline::new(first).pipe(second)
}

/// 创建并行执行器（类似 `|` 操作符）
///
/// # 示例
///
/// ```rust
/// use lumosai_core::agent::operators::parallel;
///
/// let parallel_exec = parallel(agent1, agent2);
/// let results = parallel_exec.execute("Process data").await?;
/// ```
pub fn parallel(first: Arc<dyn Agent>, second: Arc<dyn Agent>) -> AgentParallel {
    AgentParallel::new(first).parallel(second)
}

/// 创建委托操作（类似 `<=` 操作符）
///
/// # 示例
///
/// ```rust
/// use lumosai_core::agent::operators::delegate;
///
/// let delegation = delegate(manager, worker, "Complete the task");
/// let result = delegation.execute("Task input").await?;
/// ```
pub fn delegate(
    manager: Arc<dyn Agent>,
    worker: Arc<dyn Agent>,
    task_description: &str,
) -> AgentDelegation {
    AgentDelegation::new(manager, worker, task_description.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::BasicAgent;
    use crate::llm::test_helpers::create_test_zhipu_provider_arc;
    use std::time::Duration;

    /// Helper function to retry API calls with exponential backoff
    async fn retry_with_backoff<F, Fut, T>(
        mut f: F,
        max_retries: u32,
        initial_delay_ms: u64,
    ) -> crate::Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = crate::Result<T>>,
    {
        let mut delay = initial_delay_ms;
        for attempt in 0..max_retries {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    let error_msg = format!("{:?}", e);
                    if error_msg.contains("429") || error_msg.contains("Too Many Requests") || error_msg.contains("1302") {
                        if attempt < max_retries - 1 {
                            eprintln!("⚠️  Rate limit hit (attempt {}/{}), retrying after {}ms...", attempt + 1, max_retries, delay);
                            tokio::time::sleep(Duration::from_millis(delay)).await;
                            delay *= 2; // Exponential backoff
                            continue;
                        }
                    }
                    return Err(e);
                }
            }
        }
        unreachable!()
    }

    #[tokio::test]
    async fn test_agent_pipeline() {
        let llm1 = create_test_zhipu_provider_arc();
        let llm2 = create_test_zhipu_provider_arc();

        let agent1 = Arc::new(BasicAgent::new(
            crate::agent::AgentConfig {
                name: "agent1".to_string(),
                instructions: "Process step 1".to_string(),
                ..Default::default()
            },
            llm1,
        ));

        let agent2 = Arc::new(BasicAgent::new(
            crate::agent::AgentConfig {
                name: "agent2".to_string(),
                instructions: "Process step 2".to_string(),
                ..Default::default()
            },
            llm2,
        ));

        // Add delay to avoid rate limiting
        tokio::time::sleep(Duration::from_millis(1000)).await;

        let pipeline = AgentPipeline::new(agent1).pipe(agent2);

        let result = retry_with_backoff(
            || async { pipeline.execute("input").await },
            5,
            2000,
        ).await;

        // Real LLM returns variable responses, just check it's successful
        assert!(result.is_ok(), "Pipeline execution failed: {:?}", result.err());
        let output = result.unwrap();
        assert!(!output.is_empty(), "Pipeline output should not be empty");
    }
}
