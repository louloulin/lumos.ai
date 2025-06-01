use crate::Result;
use crate::agent::Agent;
use crate::tool::Tool;
use std::collections::HashMap;
use std::sync::Arc;
use crate::rag::RagPipeline;
use crate::workflow::basic::Workflow;

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
        }
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