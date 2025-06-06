//! 简化API演示
//! 
//! 展示新的简化API设计，提供更友好的开发体验

use lumosai_core::prelude::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Lumos.ai 简化API演示");
    println!("=" .repeat(50));
    
    // 演示最简单的使用方式
    demo_quick_agent().await?;
    
    // 演示标准构建器模式
    demo_builder_pattern().await?;
    
    // 演示预配置的专用Agent
    demo_specialized_agents().await?;
    
    // 演示智能默认值系统
    demo_smart_defaults().await?;
    
    // 演示链式配置
    demo_fluent_configuration().await?;
    
    println!("\n🎉 简化API演示完成！");
    println!("\n💡 API设计原则:");
    println!("  ✅ 渐进式复杂度：从简单到复杂");
    println!("  ✅ 智能默认值：减少配置负担");
    println!("  ✅ 链式调用：流畅的开发体验");
    println!("  ✅ 类型安全：编译时错误检查");
    println!("  ✅ 向后兼容：保护现有投资");
    
    Ok(())
}

/// 演示最简单的使用方式
async fn demo_quick_agent() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚡ 演示：最简单的使用方式");
    println!("-" .repeat(30));
    
    // 最简单的Agent创建 - 只需要名称和指令
    let agent = Agent::quick("assistant", "你是一个友好的AI助手")
        .build()
        .await?;
    
    println!("✅ 快速创建Agent成功");
    println!("   名称: assistant");
    println!("   指令: 你是一个友好的AI助手");
    println!("   模型: 自动选择最佳模型");
    
    // 简单的对话
    let response = agent.generate("你好！").await?;
    println!("   对话测试: {}", response);
    
    Ok(())
}

/// 演示标准构建器模式
async fn demo_builder_pattern() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🏗️  演示：标准构建器模式");
    println!("-" .repeat(30));
    
    // 使用构建器模式进行详细配置
    let agent = Agent::builder()
        .name("research_assistant")
        .instructions("你是一个专业的研究助手，擅长信息收集和分析")
        .model("deepseek-chat")
        .temperature(0.7)
        .max_tokens(2000)
        .tools(vec![
            tools::web_search(),
            tools::calculator(),
            tools::file_reader(),
        ])
        .memory_type(MemoryType::Semantic)
        .memory_capacity(1000)
        .build()
        .await?;
    
    println!("✅ 构建器模式创建Agent成功");
    println!("   名称: research_assistant");
    println!("   模型: deepseek-chat");
    println!("   工具数量: 3");
    println!("   内存类型: 语义内存");
    
    // 测试工具调用
    let response = agent.generate("帮我搜索最新的AI技术趋势").await?;
    println!("   工具调用测试: {}", response);
    
    Ok(())
}

/// 演示预配置的专用Agent
async fn demo_specialized_agents() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎯 演示：预配置的专用Agent");
    println!("-" .repeat(30));
    
    // Web专用Agent - 预配置了所有Web相关工具
    let web_agent = Agent::web_agent("web_crawler")
        .instructions("你是一个专业的网络爬虫助手")
        .build()
        .await?;
    
    println!("🌐 Web Agent:");
    println!("   预配置工具: web_search, http_request, url_extractor");
    
    // 文件专用Agent - 预配置了所有文件操作工具
    let file_agent = Agent::file_agent("file_manager")
        .instructions("你是一个文件管理助手")
        .build()
        .await?;
    
    println!("📁 File Agent:");
    println!("   预配置工具: file_reader, file_writer, directory_scanner");
    
    // 数据分析专用Agent - 预配置了数据处理工具
    let data_agent = Agent::data_agent("data_analyst")
        .instructions("你是一个数据分析专家")
        .build()
        .await?;
    
    println!("📊 Data Agent:");
    println!("   预配置工具: csv_processor, json_transformer, excel_reader");
    
    // AI服务专用Agent - 预配置了AI服务集成工具
    let ai_agent = Agent::ai_agent("ai_orchestrator")
        .instructions("你是一个AI服务编排助手")
        .build()
        .await?;
    
    println!("🤖 AI Agent:");
    println!("   预配置工具: openai_client, anthropic_client, huggingface_client");
    
    Ok(())
}

/// 演示智能默认值系统
async fn demo_smart_defaults() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧠 演示：智能默认值系统");
    println!("-" .repeat(30));
    
    // 启用自动配置
    let agent = Agent::builder()
        .name("smart_assistant")
        .instructions("你是一个智能助手")
        .auto_configure(true)  // 启用智能默认值
        .build()
        .await?;
    
    println!("✅ 智能默认值配置完成:");
    
    // 获取自动配置的详情
    let config = agent.get_config();
    println!("   自动选择模型: {}", config.model.name);
    println!("   自动配置内存: {} (容量: {})", 
            config.memory.memory_type, 
            config.memory.capacity);
    println!("   自动优化参数: 温度={}, 最大令牌={}", 
            config.model.temperature, 
            config.model.max_tokens);
    
    // 根据使用场景自动推荐工具
    let recommended_tools = agent.get_recommended_tools();
    println!("   推荐工具: {:?}", recommended_tools);
    
    Ok(())
}

/// 演示链式配置
async fn demo_fluent_configuration() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔗 演示：链式配置");
    println!("-" .repeat(30));
    
    // 流畅的链式配置API
    let agent = Agent::quick("fluent_assistant", "你是一个流畅配置的助手")
        .model("gpt-4")
        .temperature(0.8)
        .add_tool(tools::web_search())
        .add_tool(tools::calculator())
        .with_memory(MemoryType::Buffer, 500)
        .with_logging(LogLevel::Info)
        .with_timeout(30)
        .with_retry(3)
        .build()
        .await?;
    
    println!("✅ 链式配置完成:");
    println!("   模型: gpt-4");
    println!("   温度: 0.8");
    println!("   工具: web_search, calculator");
    println!("   内存: Buffer (500)");
    println!("   超时: 30秒");
    println!("   重试: 3次");
    
    // 动态添加工具
    let enhanced_agent = agent
        .add_tool(tools::file_reader())
        .add_tool(tools::json_processor())
        .update_instructions("你现在还可以处理文件和JSON数据")
        .build()
        .await?;
    
    println!("   动态增强: 添加了文件和JSON处理能力");
    
    Ok(())
}

// 扩展Agent实现以支持新的API
impl Agent {
    /// 快速创建Agent - 最简单的使用方式
    pub fn quick(name: &str, instructions: &str) -> AgentBuilder {
        AgentBuilder::new()
            .name(name)
            .instructions(instructions)
            .auto_configure(true)
    }
    
    /// 标准构建器模式
    pub fn builder() -> AgentBuilder {
        AgentBuilder::new()
    }
    
    /// Web专用Agent
    pub fn web_agent(name: &str) -> AgentBuilder {
        Self::builder()
            .name(name)
            .tools(vec![
                tools::web_search(),
                tools::http_request(),
                tools::url_extractor(),
            ])
            .auto_configure(true)
    }
    
    /// 文件专用Agent
    pub fn file_agent(name: &str) -> AgentBuilder {
        Self::builder()
            .name(name)
            .tools(vec![
                tools::file_reader(),
                tools::file_writer(),
                tools::directory_scanner(),
            ])
            .auto_configure(true)
    }
    
    /// 数据分析专用Agent
    pub fn data_agent(name: &str) -> AgentBuilder {
        Self::builder()
            .name(name)
            .tools(vec![
                tools::csv_processor(),
                tools::json_processor(),
                tools::excel_reader(),
            ])
            .auto_configure(true)
    }
    
    /// AI服务专用Agent
    pub fn ai_agent(name: &str) -> AgentBuilder {
        Self::builder()
            .name(name)
            .tools(vec![
                tools::openai_client(),
                tools::anthropic_client(),
                tools::huggingface_client(),
            ])
            .auto_configure(true)
    }
    
    /// 获取推荐工具
    pub fn get_recommended_tools(&self) -> Vec<String> {
        // 基于Agent的指令和使用模式推荐工具
        vec![
            "web_search".to_string(),
            "calculator".to_string(),
            "file_reader".to_string(),
        ]
    }
}

// 扩展AgentBuilder以支持链式配置
impl AgentBuilder {
    /// 启用自动配置
    pub fn auto_configure(mut self, enabled: bool) -> Self {
        self.auto_configure = enabled;
        self
    }
    
    /// 设置温度
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.config.model.temperature = temperature;
        self
    }
    
    /// 设置最大令牌数
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.config.model.max_tokens = max_tokens;
        self
    }
    
    /// 添加单个工具
    pub fn add_tool(mut self, tool: Arc<dyn Tool>) -> Self {
        self.config.tools.push(tool);
        self
    }
    
    /// 配置内存
    pub fn with_memory(mut self, memory_type: MemoryType, capacity: usize) -> Self {
        self.config.memory.memory_type = memory_type;
        self.config.memory.capacity = capacity;
        self
    }
    
    /// 配置日志
    pub fn with_logging(mut self, level: LogLevel) -> Self {
        self.config.logging.level = level;
        self
    }
    
    /// 配置超时
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.config.runtime.timeout_seconds = timeout_seconds;
        self
    }
    
    /// 配置重试
    pub fn with_retry(mut self, max_retries: u32) -> Self {
        self.config.runtime.max_retries = max_retries;
        self
    }
    
    /// 更新指令
    pub fn update_instructions(mut self, instructions: &str) -> Self {
        self.config.instructions = instructions.to_string();
        self
    }
    
    /// 应用智能默认值
    fn apply_smart_defaults(mut self) -> Result<Self, Box<dyn std::error::Error>> {
        // 自动选择合适的模型
        if self.config.model.name.is_empty() {
            self.config.model.name = self.detect_best_model()?;
        }
        
        // 自动配置内存
        if self.config.memory.memory_type == MemoryType::None {
            self.config.memory = self.create_default_memory()?;
        }
        
        // 自动优化参数
        if self.config.model.temperature == 0.0 {
            self.config.model.temperature = 0.7; // 平衡创造性和准确性
        }
        
        if self.config.model.max_tokens == 0 {
            self.config.model.max_tokens = 1000; // 合理的默认值
        }
        
        Ok(self)
    }
    
    /// 检测最佳模型
    fn detect_best_model(&self) -> Result<String, Box<dyn std::error::Error>> {
        // 基于指令内容和可用资源选择最佳模型
        let instructions = &self.config.instructions;
        
        if instructions.contains("代码") || instructions.contains("编程") {
            Ok("deepseek-coder".to_string())
        } else if instructions.contains("数学") || instructions.contains("计算") {
            Ok("deepseek-math".to_string())
        } else {
            Ok("deepseek-chat".to_string()) // 通用模型
        }
    }
    
    /// 创建默认内存配置
    fn create_default_memory(&self) -> Result<MemoryConfig, Box<dyn std::error::Error>> {
        Ok(MemoryConfig {
            memory_type: MemoryType::Buffer,
            capacity: 1000,
            persistence: false,
        })
    }
}

// 支持类型定义
#[derive(Debug, Clone)]
pub enum MemoryType {
    None,
    Buffer,
    Semantic,
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone)]
pub struct MemoryConfig {
    pub memory_type: MemoryType,
    pub capacity: usize,
    pub persistence: bool,
}
