use crate::Result;
use crate::agent::{trait_def::Agent, AgentBuilder, ModelResolver};
use crate::tool::Tool;
use crate::config::{ConfigLoader, YamlConfig, WorkflowConfig};
use std::collections::HashMap;
use std::sync::Arc;
use std::path::Path;
use crate::rag::RagPipeline;
use crate::workflow::Workflow;
use crate::workflow::EnhancedWorkflow;

pub mod enhanced;

pub use enhanced::{
    EnhancedApp,
    EnhancedAppConfig,
    ToolsConfig,
    RagConfig,
    ChunkingConfig,
    AppStats,
};

/// Lumosai应用主类，用于整合代理、工具、RAG和MCP等组件
pub struct LumosApp {
    name: String,
    description: Option<String>,
    agents: HashMap<String, Arc<dyn Agent>>,
    tools: HashMap<String, Arc<dyn Tool>>,
    rags: HashMap<String, Arc<dyn RagPipeline>>,
    workflows: HashMap<String, Arc<dyn Workflow>>,
    mcp_endpoints: Vec<String>,
    config: Option<YamlConfig>,
    model_resolver: ModelResolver,
}

impl LumosApp {
    /// 创建一个新的Lumosai应用实例
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            agents: HashMap::new(),
            tools: HashMap::new(),
            rags: HashMap::new(),
            workflows: HashMap::new(),
            mcp_endpoints: Vec::new(),
            config: None,
            model_resolver: ModelResolver::new(),
        }
    }

    /// 从配置文件创建应用实例
    pub async fn from_config<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        let config = ConfigLoader::load(config_path)?;
        Self::from_yaml_config(config).await
    }

    /// 从 YAML 配置创建应用实例
    pub async fn from_yaml_config(config: YamlConfig) -> Result<Self> {
        let mut app = Self {
            name: config.project.as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "lumosai_app".to_string()),
            description: config.project.as_ref()
                .and_then(|p| p.description.clone()),
            agents: HashMap::new(),
            tools: HashMap::new(),
            rags: HashMap::new(),
            workflows: HashMap::new(),
            mcp_endpoints: Vec::new(),
            config: Some(config.clone()),
            model_resolver: ModelResolver::new(),
        };

        // 创建配置中定义的 Agents
        if let Some(agents_config) = &config.agents {
            for (name, agent_config) in agents_config {
                let agent = app.create_agent_from_config(name, agent_config).await?;
                app.agents.insert(name.clone(), Arc::new(agent));
            }
        }

        // 创建配置中定义的 Workflows
        if let Some(workflows_config) = &config.workflows {
            for (name, workflow_config) in workflows_config {
                let workflow = app.create_workflow_from_config(name, workflow_config)?;
                app.workflows.insert(name.clone(), Arc::new(workflow));
            }
        }

        Ok(app)
    }

    /// 自动检测并加载配置文件
    pub async fn auto_load() -> Result<Self> {
        let config = ConfigLoader::auto_detect()?;
        Self::from_yaml_config(config).await
    }

    /// 从配置创建 Agent
    async fn create_agent_from_config(
        &self,
        name: &str,
        config: &crate::config::AgentConfig,
    ) -> Result<impl Agent> {
        let mut builder = AgentBuilder::new()
            .name(name)
            .instructions(&config.instructions)
            .model_name(&config.model);

        // 设置可选参数
        if let Some(temperature) = config.temperature {
            builder = builder.temperature(temperature);
        }

        if let Some(max_tokens) = config.max_tokens {
            builder = builder.max_tokens(max_tokens);
        }

        if let Some(timeout) = config.timeout {
            builder = builder.tool_timeout(timeout);
        }

        // 添加工具
        if let Some(tools) = &config.tools {
            for tool_name in tools {
                if let Some(tool) = self.resolve_tool(tool_name)? {
                    builder = builder.add_tool(tool);
                }
            }
        }

        builder.build_async().await
    }

    /// 解析工具名称到工具实例
    fn resolve_tool(&self, tool_name: &str) -> Result<Option<Arc<dyn crate::tool::Tool>>> {
        // 首先检查已注册的工具
        if let Some(tool) = self.tools.get(tool_name) {
            return Ok(Some(tool.clone()));
        }

        // 尝试创建内置工具
        match tool_name {
            "web_search" => {
                use crate::tool::builtin::WebSearchTool;
                Ok(Some(Arc::new(WebSearchTool::new())))
            },
            "calculator" => {
                use crate::tool::builtin::CalculatorTool;
                Ok(Some(Arc::new(CalculatorTool::new())))
            },
            "file_manager" => {
                use crate::tool::builtin::FileManagerTool;
                Ok(Some(Arc::new(FileManagerTool::new())))
            },
            "code_executor" => {
                use crate::tool::builtin::CodeExecutorTool;
                Ok(Some(Arc::new(CodeExecutorTool::new())))
            },
            _ => {
                tracing::warn!("Unknown tool: {}", tool_name);
                Ok(None)
            }
        }
    }

    /// 从配置创建工作流
    fn create_workflow_from_config(
        &self,
        name: &str,
        config: &WorkflowConfig,
    ) -> Result<EnhancedWorkflow> {
        // 设置基本信息
        let workflow_id = config.id.as_ref().map_or(name.to_string(), |id| id.clone());
        let description = config.description.clone();

        // 创建基本的 EnhancedWorkflow
        let mut workflow = EnhancedWorkflow::new(workflow_id, description);

        // TODO: 添加步骤配置的处理
        // 这里需要根据 config.steps 来构建工作流步骤
        // 由于 WorkflowStep 需要 StepExecutor，这里暂时创建一个空的工作流

        Ok(workflow)
    }

    /// 设置应用名称
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
    
    /// 设置应用描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// 添加代理到应用
    pub fn add_agent(&mut self, name: String, agent: impl Agent + 'static) {
        self.agents.insert(name, Arc::new(agent));
    }
    
    /// 添加工具到应用
    pub fn add_tool(&mut self, name: String, tool: impl Tool + 'static) {
        self.tools.insert(name, Arc::new(tool));
    }
    
    /// 添加RAG到应用
    pub fn add_rag(&mut self, name: String, rag: impl RagPipeline + 'static) {
        self.rags.insert(name, Arc::new(rag));
    }
    
    /// 添加工作流到应用
    pub fn add_workflow(&mut self, name: String, workflow: impl Workflow + 'static) {
        self.workflows.insert(name, Arc::new(workflow));
    }
    
    /// 配置MCP客户端
    pub fn set_mcp_endpoints(&mut self, endpoints: Vec<String>) {
        self.mcp_endpoints = endpoints;
    }
    
    /// 启动应用
    pub async fn start(&self) -> Result<()> {
        println!("Starting Lumosai application: {}", self.name);
        if let Some(desc) = &self.description {
            println!("Description: {}", desc);
        }
        
        println!("Registered components:");
        println!("- Agents: {}", self.agents.len());
        println!("- Tools: {}", self.tools.len());
        println!("- RAG pipelines: {}", self.rags.len());
        println!("- Workflows: {}", self.workflows.len());
        
        if !self.mcp_endpoints.is_empty() {
            println!("MCP endpoints: {}", self.mcp_endpoints.join(", "));
        }
        
        // 实际应用中，这里会执行更多的启动逻辑
        
        Ok(())
    }
    
    /// 执行用户请求
    pub async fn run(&self, request: impl Into<String>) -> Result<String> {
        let request_str = request.into();
        println!("Processing request: {}", request_str);
        
        // 简单实现：将请求转发给第一个可用的代理
        if let Some((agent_name, agent)) = self.agents.iter().next() {
            println!("Routing request to agent: {}", agent_name);
            
            // 创建用户消息
            let user_message = crate::llm::Message {
                role: crate::llm::Role::User,
                content: request_str,
                name: None,
                metadata: None,
            };
            
            // 调用代理
            let result = agent.generate(&[user_message], &crate::agent::types::AgentGenerateOptions::default()).await?;
            
            Ok(result.response)
        } else {
            Ok("No agents available to process the request".to_string())
        }
    }
    
    /// 获取应用名称
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// 获取应用描述
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    
    /// 获取所有代理列表
    pub fn agents(&self) -> &HashMap<String, Arc<dyn Agent>> {
        &self.agents
    }

    /// 获取指定名称的代理（便捷方法）
    pub fn agent(&self, name: &str) -> Result<&Arc<dyn Agent>> {
        self.agents.get(name)
            .ok_or_else(|| crate::Error::Configuration(format!("Agent '{}' not found", name)))
    }
    
    /// 获取所有工具列表
    pub fn tools(&self) -> &HashMap<String, Arc<dyn Tool>> {
        &self.tools
    }
    
    /// 获取RAG管道列表
    pub fn rags(&self) -> &HashMap<String, Arc<dyn RagPipeline>> {
        &self.rags
    }
    
    /// 获取工作流列表
    pub fn workflows(&self) -> &HashMap<String, Arc<dyn Workflow>> {
        &self.workflows
    }
    
    /// 获取MCP端点列表
    pub fn mcp_endpoints(&self) -> &[String] {
        &self.mcp_endpoints
    }
}

impl Default for LumosApp {
    fn default() -> Self {
        Self::new("lumosai_app")
    }
} 