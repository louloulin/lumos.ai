//! 事件驱动架构演示
//! 
//! 展示如何实现事件驱动的代理协作系统，包括：
//! - 事件总线设计
//! - 事件发布和订阅
//! - 异步事件处理
//! - 代理间协作

use lumosai_core::prelude::*;
use lumosai_core::agent::{AgentBuilder, BasicAgent};
use lumosai_core::events::{EventBus, Event, EventHandler, EventSubscription};
use lumosai_core::llm::{MockLlmProvider, Message, Role};
use std::sync::Arc;
use std::collections::HashMap;
use serde_json::{json, Value};
use tokio::sync::{Mutex, mpsc};
use tokio::time::{sleep, Duration};
use async_trait::async_trait;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📡 事件驱动架构演示");
    println!("====================");
    
    // 演示1: 基础事件系统
    demo_basic_event_system().await?;
    
    // 演示2: 代理事件协作
    demo_agent_event_collaboration().await?;
    
    // 演示3: 复杂事件流
    demo_complex_event_flow().await?;
    
    // 演示4: 事件驱动的客户服务系统
    demo_customer_service_system().await?;
    
    Ok(())
}

/// 演示基础事件系统
async fn demo_basic_event_system() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示1: 基础事件系统 ===");
    
    // 创建事件总线
    let event_bus = Arc::new(EventBus::new());
    
    // 创建事件处理器
    let handler1 = Arc::new(LoggingEventHandler::new("Handler1"));
    let handler2 = Arc::new(LoggingEventHandler::new("Handler2"));
    
    // 订阅事件
    event_bus.subscribe("user_action", handler1.clone()).await?;
    event_bus.subscribe("user_action", handler2.clone()).await?;
    event_bus.subscribe("system_event", handler1.clone()).await?;
    
    println!("事件总线已创建，处理器已订阅");
    
    // 发布测试事件
    let events = vec![
        Event::new("user_action", json!({"action": "login", "user_id": "user123"})),
        Event::new("user_action", json!({"action": "view_page", "page": "/dashboard"})),
        Event::new("system_event", json!({"type": "maintenance", "status": "started"})),
        Event::new("user_action", json!({"action": "logout", "user_id": "user123"})),
    ];
    
    println!("\n发布测试事件:");
    for (i, event) in events.iter().enumerate() {
        println!("  {}. 发布事件: {} - {:?}", i + 1, event.event_type, event.data);
        event_bus.publish(event.clone()).await?;
        sleep(Duration::from_millis(100)).await; // 短暂延迟以观察处理顺序
    }
    
    // 等待事件处理完成
    sleep(Duration::from_millis(500)).await;
    
    // 显示处理统计
    println!("\n事件处理统计:");
    println!("  Handler1 处理了 {} 个事件", handler1.get_processed_count().await);
    println!("  Handler2 处理了 {} 个事件", handler2.get_processed_count().await);
    
    Ok(())
}

/// 演示代理事件协作
async fn demo_agent_event_collaboration() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示2: 代理事件协作 ===");
    
    let event_bus = Arc::new(EventBus::new());
    
    // 创建协作代理
    let data_processor = create_data_processor_agent().await?;
    let analyzer = create_analyzer_agent().await?;
    let reporter = create_reporter_agent().await?;
    
    // 创建代理事件处理器
    let processor_handler = Arc::new(AgentEventHandler::new(
        "data_processor",
        data_processor,
        "处理原始数据并发布处理结果"
    ));
    
    let analyzer_handler = Arc::new(AgentEventHandler::new(
        "analyzer",
        analyzer,
        "分析处理后的数据并生成洞察"
    ));
    
    let reporter_handler = Arc::new(AgentEventHandler::new(
        "reporter",
        reporter,
        "生成最终报告"
    ));
    
    // 设置事件订阅链
    event_bus.subscribe("raw_data", processor_handler.clone()).await?;
    event_bus.subscribe("processed_data", analyzer_handler.clone()).await?;
    event_bus.subscribe("analysis_complete", reporter_handler.clone()).await?;
    
    println!("代理协作链已设置:");
    println!("  raw_data -> data_processor -> processed_data");
    println!("  processed_data -> analyzer -> analysis_complete");
    println!("  analysis_complete -> reporter -> final_report");
    
    // 启动协作流程
    println!("\n启动数据处理协作流程...");
    
    let raw_data_event = Event::new("raw_data", json!({
        "source": "sales_database",
        "data": [
            {"date": "2024-01", "sales": 10000, "region": "north"},
            {"date": "2024-01", "sales": 15000, "region": "south"},
            {"date": "2024-02", "sales": 12000, "region": "north"},
            {"date": "2024-02", "sales": 18000, "region": "south"}
        ],
        "timestamp": "2024-01-15T10:00:00Z"
    }));
    
    event_bus.publish(raw_data_event).await?;
    
    // 等待协作完成
    sleep(Duration::from_secs(3)).await;
    
    // 显示协作结果
    println!("\n代理协作结果:");
    println!("  数据处理器: {} 个事件", processor_handler.get_processed_count().await);
    println!("  分析器: {} 个事件", analyzer_handler.get_processed_count().await);
    println!("  报告器: {} 个事件", reporter_handler.get_processed_count().await);
    
    Ok(())
}

/// 演示复杂事件流
async fn demo_complex_event_flow() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示3: 复杂事件流 ===");
    
    let event_bus = Arc::new(EventBus::new());
    
    // 创建事件流监控器
    let flow_monitor = Arc::new(EventFlowMonitor::new());
    
    // 订阅所有事件类型进行监控
    let monitor_handler = Arc::new(MonitoringEventHandler::new(flow_monitor.clone()));
    event_bus.subscribe_all(monitor_handler).await?;
    
    // 创建复杂的事件处理链
    let orchestrator = Arc::new(EventOrchestrator::new(event_bus.clone()));
    
    // 设置复杂的事件流规则
    orchestrator.add_rule(EventFlowRule {
        trigger_event: "order_created".to_string(),
        actions: vec![
            EventAction::PublishEvent("inventory_check".to_string()),
            EventAction::PublishEvent("payment_process".to_string()),
        ],
        condition: None,
    }).await;
    
    orchestrator.add_rule(EventFlowRule {
        trigger_event: "inventory_confirmed".to_string(),
        actions: vec![
            EventAction::PublishEvent("shipping_prepare".to_string()),
        ],
        condition: Some("payment_confirmed".to_string()),
    }).await;
    
    orchestrator.add_rule(EventFlowRule {
        trigger_event: "shipping_complete".to_string(),
        actions: vec![
            EventAction::PublishEvent("order_complete".to_string()),
            EventAction::PublishEvent("customer_notification".to_string()),
        ],
        condition: None,
    }).await;
    
    println!("复杂事件流规则已设置");
    
    // 模拟订单处理流程
    println!("\n模拟订单处理事件流...");
    
    let order_events = vec![
        Event::new("order_created", json!({"order_id": "ORD001", "customer_id": "CUST123"})),
        Event::new("inventory_confirmed", json!({"order_id": "ORD001", "items_available": true})),
        Event::new("payment_confirmed", json!({"order_id": "ORD001", "amount": 299.99})),
        Event::new("shipping_complete", json!({"order_id": "ORD001", "tracking_number": "TRK456"})),
    ];
    
    for (i, event) in order_events.iter().enumerate() {
        println!("  {}. 触发事件: {}", i + 1, event.event_type);
        event_bus.publish(event.clone()).await?;
        sleep(Duration::from_millis(200)).await;
    }
    
    // 等待事件流处理完成
    sleep(Duration::from_secs(2)).await;
    
    // 显示事件流统计
    let flow_stats = flow_monitor.get_statistics().await;
    println!("\n事件流统计:");
    println!("  总事件数: {}", flow_stats.total_events);
    println!("  事件类型数: {}", flow_stats.event_types.len());
    println!("  平均处理时间: {:?}", flow_stats.avg_processing_time);
    
    for (event_type, count) in &flow_stats.event_types {
        println!("    {}: {} 次", event_type, count);
    }
    
    Ok(())
}

/// 演示事件驱动的客户服务系统
async fn demo_customer_service_system() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示4: 事件驱动的客户服务系统 ===");
    
    let event_bus = Arc::new(EventBus::new());
    
    // 创建客户服务代理
    let customer_service = create_customer_service_agent().await?;
    let technical_support = create_technical_support_agent().await?;
    let escalation_handler = create_escalation_agent().await?;
    
    // 创建客户服务系统
    let service_system = CustomerServiceSystem::new(
        event_bus.clone(),
        customer_service,
        technical_support,
        escalation_handler,
    ).await?;
    
    println!("客户服务系统已启动");
    
    // 模拟客户服务场景
    let customer_scenarios = vec![
        ("customer_inquiry", json!({
            "customer_id": "CUST001",
            "message": "我的软件无法启动，显示错误代码 0x001",
            "priority": "high",
            "channel": "email"
        })),
        ("customer_inquiry", json!({
            "customer_id": "CUST002", 
            "message": "请问如何升级到最新版本？",
            "priority": "normal",
            "channel": "chat"
        })),
        ("customer_inquiry", json!({
            "customer_id": "CUST003",
            "message": "系统运行很慢，可能是什么原因？",
            "priority": "medium",
            "channel": "phone"
        })),
        ("customer_inquiry", json!({
            "customer_id": "CUST004",
            "message": "我需要技术支持来解决数据库连接问题",
            "priority": "high",
            "channel": "email"
        })),
    ];
    
    println!("\n处理客户服务场景:");
    for (i, (event_type, data)) in customer_scenarios.iter().enumerate() {
        println!("  {}. 客户咨询: {}", i + 1, 
            data["message"].as_str().unwrap_or("未知"));
        
        let event = Event::new(event_type, data.clone());
        event_bus.publish(event).await?;
        
        sleep(Duration::from_millis(500)).await;
    }
    
    // 等待所有服务请求处理完成
    sleep(Duration::from_secs(3)).await;
    
    // 显示服务统计
    let service_stats = service_system.get_statistics().await;
    println!("\n客户服务统计:");
    println!("  总咨询数: {}", service_stats.total_inquiries);
    println!("  一级支持处理: {}", service_stats.first_level_handled);
    println!("  技术支持处理: {}", service_stats.technical_support_handled);
    println!("  升级处理: {}", service_stats.escalated_cases);
    println!("  平均响应时间: {:?}", service_stats.avg_response_time);
    
    Ok(())
}

// ============================================================================
// 事件系统组件
// ============================================================================

/// 日志事件处理器
struct LoggingEventHandler {
    name: String,
    processed_count: Arc<Mutex<usize>>,
}

impl LoggingEventHandler {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            processed_count: Arc::new(Mutex::new(0)),
        }
    }
    
    async fn get_processed_count(&self) -> usize {
        *self.processed_count.lock().await
    }
}

#[async_trait]
impl EventHandler for LoggingEventHandler {
    async fn handle(&self, event: &Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut count = self.processed_count.lock().await;
        *count += 1;
        
        println!("    {} 处理事件: {} - {:?}", 
            self.name, event.event_type, event.data);
        
        Ok(())
    }
}

/// 代理事件处理器
struct AgentEventHandler {
    name: String,
    agent: Arc<dyn BasicAgent>,
    description: String,
    processed_count: Arc<Mutex<usize>>,
}

impl AgentEventHandler {
    fn new(name: &str, agent: Arc<dyn BasicAgent>, description: &str) -> Self {
        Self {
            name: name.to_string(),
            agent,
            description: description.to_string(),
            processed_count: Arc::new(Mutex::new(0)),
        }
    }
    
    async fn get_processed_count(&self) -> usize {
        *self.processed_count.lock().await
    }
}

#[async_trait]
impl EventHandler for AgentEventHandler {
    async fn handle(&self, event: &Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut count = self.processed_count.lock().await;
        *count += 1;
        drop(count);
        
        println!("    {} 处理事件: {}", self.name, event.event_type);
        
        // 使用代理处理事件
        let input = format!("处理事件: {} - 数据: {}", event.event_type, event.data);
        let response = self.agent.generate(&input).await?;
        
        println!("      代理响应: {}", response.content);
        
        // 根据事件类型发布后续事件（模拟）
        // 这里应该有实际的事件总线引用，为了简化演示，我们只打印
        match event.event_type.as_str() {
            "raw_data" => {
                println!("      -> 发布 processed_data 事件");
            }
            "processed_data" => {
                println!("      -> 发布 analysis_complete 事件");
            }
            "analysis_complete" => {
                println!("      -> 发布 final_report 事件");
            }
            _ => {}
        }
        
        Ok(())
    }
}

/// 事件流监控器
struct EventFlowMonitor {
    statistics: Arc<Mutex<EventFlowStatistics>>,
}

impl EventFlowMonitor {
    fn new() -> Self {
        Self {
            statistics: Arc::new(Mutex::new(EventFlowStatistics::default())),
        }
    }
    
    async fn get_statistics(&self) -> EventFlowStatistics {
        self.statistics.lock().await.clone()
    }
}

#[derive(Clone, Default)]
struct EventFlowStatistics {
    total_events: usize,
    event_types: HashMap<String, usize>,
    avg_processing_time: Duration,
}

/// 监控事件处理器
struct MonitoringEventHandler {
    monitor: Arc<EventFlowMonitor>,
}

impl MonitoringEventHandler {
    fn new(monitor: Arc<EventFlowMonitor>) -> Self {
        Self { monitor }
    }
}

#[async_trait]
impl EventHandler for MonitoringEventHandler {
    async fn handle(&self, event: &Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut stats = self.monitor.statistics.lock().await;
        stats.total_events += 1;
        *stats.event_types.entry(event.event_type.clone()).or_insert(0) += 1;
        
        Ok(())
    }
}

/// 事件编排器
struct EventOrchestrator {
    event_bus: Arc<EventBus>,
    rules: Arc<Mutex<Vec<EventFlowRule>>>,
}

impl EventOrchestrator {
    fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            event_bus,
            rules: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    async fn add_rule(&self, rule: EventFlowRule) {
        let mut rules = self.rules.lock().await;
        rules.push(rule);
    }
}

/// 事件流规则
struct EventFlowRule {
    trigger_event: String,
    actions: Vec<EventAction>,
    condition: Option<String>,
}

/// 事件动作
enum EventAction {
    PublishEvent(String),
}

/// 客户服务系统
struct CustomerServiceSystem {
    event_bus: Arc<EventBus>,
    statistics: Arc<Mutex<CustomerServiceStatistics>>,
}

impl CustomerServiceSystem {
    async fn new(
        event_bus: Arc<EventBus>,
        customer_service: Arc<dyn BasicAgent>,
        technical_support: Arc<dyn BasicAgent>,
        escalation_handler: Arc<dyn BasicAgent>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let system = Self {
            event_bus: event_bus.clone(),
            statistics: Arc::new(Mutex::new(CustomerServiceStatistics::default())),
        };
        
        // 设置事件处理器
        // 这里应该设置实际的处理器，为了简化演示，我们只创建结构
        
        Ok(system)
    }
    
    async fn get_statistics(&self) -> CustomerServiceStatistics {
        self.statistics.lock().await.clone()
    }
}

#[derive(Clone, Default)]
struct CustomerServiceStatistics {
    total_inquiries: usize,
    first_level_handled: usize,
    technical_support_handled: usize,
    escalated_cases: usize,
    avg_response_time: Duration,
}

// ============================================================================
// 代理创建函数
// ============================================================================

async fn create_data_processor_agent() -> Result<Arc<dyn BasicAgent>, Box<dyn std::error::Error>> {
    let responses = vec![
        "数据处理完成。我已经清洗和标准化了原始销售数据，识别出了关键指标和趋势。处理后的数据已准备好进行进一步分析。".to_string(),
    ];
    
    let llm_provider = Arc::new(MockLlmProvider::new(responses));
    
    Ok(Arc::new(
        AgentBuilder::new()
            .name("data_processor")
            .instructions("你是一个数据处理专家，负责清洗、标准化和预处理原始数据。")
            .model(llm_provider)
            .build()?
    ))
}

async fn create_analyzer_agent() -> Result<Arc<dyn BasicAgent>, Box<dyn std::error::Error>> {
    let responses = vec![
        "分析完成。基于处理后的数据，我发现了以下关键洞察：销售趋势呈上升态势，南部地区表现优于北部地区，2月份增长率达到20%。".to_string(),
    ];
    
    let llm_provider = Arc::new(MockLlmProvider::new(responses));
    
    Ok(Arc::new(
        AgentBuilder::new()
            .name("analyzer")
            .instructions("你是一个数据分析专家，负责从处理后的数据中提取洞察和趋势。")
            .model(llm_provider)
            .build()?
    ))
}

async fn create_reporter_agent() -> Result<Arc<dyn BasicAgent>, Box<dyn std::error::Error>> {
    let responses = vec![
        "报告生成完成。我已经创建了一份综合性的销售分析报告，包含了关键指标、趋势分析和业务建议。报告格式清晰，适合管理层查阅。".to_string(),
    ];
    
    let llm_provider = Arc::new(MockLlmProvider::new(responses));
    
    Ok(Arc::new(
        AgentBuilder::new()
            .name("reporter")
            .instructions("你是一个报告生成专家，负责将分析结果转化为清晰的业务报告。")
            .model(llm_provider)
            .build()?
    ))
}

async fn create_customer_service_agent() -> Result<Arc<dyn BasicAgent>, Box<dyn std::error::Error>> {
    let responses = vec![
        "我已经处理了客户的咨询。对于软件启动问题，我提供了基础的故障排除步骤。如果问题持续，建议转给技术支持团队。".to_string(),
        "我为客户提供了软件升级的详细指导，包括下载链接和安装步骤。客户表示满意。".to_string(),
    ];
    
    let llm_provider = Arc::new(MockLlmProvider::new(responses));
    
    Ok(Arc::new(
        AgentBuilder::new()
            .name("customer_service")
            .instructions("你是一个友好的客服代表，专门处理客户咨询和基础问题。")
            .model(llm_provider)
            .build()?
    ))
}

async fn create_technical_support_agent() -> Result<Arc<dyn BasicAgent>, Box<dyn std::error::Error>> {
    let responses = vec![
        "技术支持已完成。我诊断了系统性能问题，发现是内存使用过高导致的。已提供了优化建议和解决方案。".to_string(),
        "数据库连接问题已解决。问题是由于防火墙配置导致的，我已经提供了详细的配置修改步骤。".to_string(),
    ];
    
    let llm_provider = Arc::new(MockLlmProvider::new(responses));
    
    Ok(Arc::new(
        AgentBuilder::new()
            .name("technical_support")
            .instructions("你是一个技术支持专家，专门解决复杂的技术问题和故障。")
            .model(llm_provider)
            .build()?
    ))
}

async fn create_escalation_agent() -> Result<Arc<dyn BasicAgent>, Box<dyn std::error::Error>> {
    let responses = vec![
        "升级处理完成。我已经协调了相关技术团队，为客户提供了专门的解决方案，并安排了后续跟进。".to_string(),
    ];
    
    let llm_provider = Arc::new(MockLlmProvider::new(responses));
    
    Ok(Arc::new(
        AgentBuilder::new()
            .name("escalation_handler")
            .instructions("你是高级技术专家，处理复杂的升级问题和特殊情况。")
            .model(llm_provider)
            .build()?
    ))
}
