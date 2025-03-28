use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, LitStr, Expr, Token, braced, parse::{Parse, ParseStream}};
use syn::punctuated::Punctuated;

// 配置项结构
struct AgentRef {
    name: syn::Ident,
    expr: Option<Expr>,
}

impl Parse for AgentRef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        
        let expr = if input.peek(Token![:]) {
            let _: Token![:] = input.parse()?;
            Some(input.parse()?)
        } else {
            None
        };
        
        Ok(AgentRef { name, expr })
    }
}

// 代理配置集合
struct AgentsConfig {
    agents: Punctuated<AgentRef, Token![,]>,
}

impl Parse for AgentsConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);
        
        let agents = Punctuated::parse_terminated(&content)?;
        
        Ok(AgentsConfig { agents })
    }
}

// 工具配置集合
struct ToolsConfig {
    tools: Punctuated<AgentRef, Token![,]>,
}

impl Parse for ToolsConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);
        
        let tools = Punctuated::parse_terminated(&content)?;
        
        Ok(ToolsConfig { tools })
    }
}

// RAG配置集合
struct RagConfig {
    rags: Punctuated<AgentRef, Token![,]>,
}

impl Parse for RagConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);
        
        let rags = Punctuated::parse_terminated(&content)?;
        
        Ok(RagConfig { rags })
    }
}

// 工作流配置集合
struct WorkflowsConfig {
    workflows: Punctuated<AgentRef, Token![,]>,
}

impl Parse for WorkflowsConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);
        
        let workflows = Punctuated::parse_terminated(&content)?;
        
        Ok(WorkflowsConfig { workflows })
    }
}

// Lumos应用配置
struct LumosConfig {
    name: Option<LitStr>,
    description: Option<LitStr>,
    agents: Option<AgentsConfig>,
    tools: Option<ToolsConfig>,
    rags: Option<RagConfig>,
    workflows: Option<WorkflowsConfig>,
    mcp_endpoints: Option<Expr>,
}

impl Parse for LumosConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);
        
        let mut name = None;
        let mut description = None;
        let mut agents = None;
        let mut tools = None;
        let mut rags = None;
        let mut workflows = None;
        let mut mcp_endpoints = None;
        
        while !content.is_empty() {
            let key: syn::Ident = content.parse()?;
            let _: Token![:] = content.parse()?;
            
            match key.to_string().as_str() {
                "name" => {
                    name = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "description" => {
                    description = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "agents" => {
                    agents = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "tools" => {
                    tools = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "rags" => {
                    rags = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "workflows" => {
                    workflows = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "mcp_endpoints" => {
                    mcp_endpoints = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new(key.span(), "Unknown field in Lumos configuration")),
            }
        }
        
        Ok(LumosConfig {
            name,
            description,
            agents,
            tools,
            rags,
            workflows,
            mcp_endpoints,
        })
    }
}

/// 配置整个Lumos应用，参考Mastra的应用级API
/// 
/// # 示例
/// 
/// ```rust
/// let app = lumos! {
///     name: "stock_assistant",
///     description: "一个能够提供股票信息的AI助手",
///     
///     agents: {
///         stockAgent
///     },
///     
///     tools: {
///         stockPriceTool,
///         stockInfoTool
///     },
///     
///     rags: {
///         stockKnowledgeBase
///     },
///     
///     workflows: {
///         stockAnalysisWorkflow
///     },
///     
///     mcp_endpoints: vec!["https://api.example.com/mcp"]
/// };
/// ```
pub fn lumos(input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(input as LumosConfig);
    
    // 应用名称和描述
    let name = match &config.name {
        Some(name) => quote! { .with_name(#name) },
        None => quote! {},
    };
    
    let description = match &config.description {
        Some(desc) => quote! { .with_description(#desc) },
        None => quote! {},
    };
    
    // 处理代理配置
    let agents_config = if let Some(agents_cfg) = &config.agents {
        let agent_refs = agents_cfg.agents.iter().map(|agent| {
            let agent_name = &agent.name;
            
            if let Some(expr) = &agent.expr {
                quote! { app.add_agent(#agent_name, #expr); }
            } else {
                quote! { app.add_agent(#agent_name, #agent_name); }
            }
        }).collect::<Vec<_>>();
        
        quote! {
            // 添加代理
            #(#agent_refs)*
        }
    } else {
        quote! {}
    };
    
    // 处理工具配置
    let tools_config = if let Some(tools_cfg) = &config.tools {
        let tool_refs = tools_cfg.tools.iter().map(|tool| {
            let tool_name = &tool.name;
            
            if let Some(expr) = &tool.expr {
                quote! { app.add_tool(#tool_name, #expr); }
            } else {
                quote! { app.add_tool(#tool_name, #tool_name()); }
            }
        }).collect::<Vec<_>>();
        
        quote! {
            // 添加工具
            #(#tool_refs)*
        }
    } else {
        quote! {}
    };
    
    // 处理RAG配置
    let rags_config = if let Some(rags_cfg) = &config.rags {
        let rag_refs = rags_cfg.rags.iter().map(|rag| {
            let rag_name = &rag.name;
            
            if let Some(expr) = &rag.expr {
                quote! { app.add_rag(#rag_name, #expr); }
            } else {
                quote! { app.add_rag(#rag_name, #rag_name); }
            }
        }).collect::<Vec<_>>();
        
        quote! {
            // 添加RAG
            #(#rag_refs)*
        }
    } else {
        quote! {}
    };
    
    // 处理工作流配置
    let workflows_config = if let Some(workflows_cfg) = &config.workflows {
        let workflow_refs = workflows_cfg.workflows.iter().map(|workflow| {
            let workflow_name = &workflow.name;
            
            if let Some(expr) = &workflow.expr {
                quote! { app.add_workflow(#workflow_name, #expr); }
            } else {
                quote! { app.add_workflow(#workflow_name, #workflow_name); }
            }
        }).collect::<Vec<_>>();
        
        quote! {
            // 添加工作流
            #(#workflow_refs)*
        }
    } else {
        quote! {}
    };
    
    // 处理MCP配置
    let mcp_config = if let Some(endpoints) = &config.mcp_endpoints {
        quote! {
            // 添加MCP配置
            app.configure_mcp(#endpoints);
        }
    } else {
        quote! {}
    };
    
    let expanded = quote! {
        {
            // 创建应用实例
            let mut app = lomusai_core::app::LumosApp::new()
                #name
                #description;
            
            // 配置组件
            #agents_config
            #tools_config
            #rags_config
            #workflows_config
            #mcp_config
            
            // 返回配置好的应用实例
            app
        }
    };
    
    TokenStream::from(expanded)
} 