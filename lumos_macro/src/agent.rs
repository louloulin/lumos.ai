use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, braced, Expr, Ident, LitStr, Token, parse::{Parse, ParseStream}};
use syn::punctuated::Punctuated;

// LLM配置
struct LlmConfig {
    provider: Expr,
    model: LitStr,
}

impl Parse for LlmConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let mut provider = None;
        let mut model = None;
        
        while !content.is_empty() {
            let key: Ident = content.parse()?;
            let _: Token![:] = content.parse()?;
            
            match key.to_string().as_str() {
                "provider" => {
                    provider = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "model" => {
                    model = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new(key.span(), "Unknown field in LLM config, expected 'provider' or 'model'")),
            }
        }
        
        let provider = provider.ok_or_else(|| syn::Error::new(input.span(), "Missing 'provider' field in LLM config"))?;
        let model = model.ok_or_else(|| syn::Error::new(input.span(), "Missing 'model' field in LLM config"))?;
        
        Ok(LlmConfig { provider, model })
    }
}

// 内存配置
struct MemoryConfig {
    store_type: LitStr,
    capacity: Option<Expr>,
}

impl Parse for MemoryConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let mut store_type = None;
        let mut capacity = None;
        
        while !content.is_empty() {
            let key: Ident = content.parse()?;
            let _: Token![:] = content.parse()?;
            
            match key.to_string().as_str() {
                "store_type" => {
                    store_type = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "capacity" => {
                    capacity = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new(key.span(), "Unknown field in memory config")),
            }
        }
        
        let store_type = store_type.ok_or_else(|| syn::Error::new(input.span(), "Missing 'store_type' field in memory config"))?;
        
        Ok(MemoryConfig { store_type, capacity })
    }
}

// 工具配置
struct ToolItem {
    name: Ident,
    options: Option<Expr>,
}

impl Parse for ToolItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        
        let options = if input.peek(Token![:]) || input.peek(Token![=]) {
            let _: Token![:] = input.parse()?;
            Some(input.parse()?)
        } else {
            None
        };
        
        Ok(ToolItem { name, options })
    }
}

// 工具集合
struct ToolsConfig {
    tools: Punctuated<ToolItem, Token![,]>,
}

impl Parse for ToolsConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let tools = Punctuated::parse_terminated(&content)?;
        
        Ok(ToolsConfig { tools })
    }
}

// 代理配置
struct AgentDef {
    name: LitStr,
    instructions: LitStr,
    llm: LlmConfig,
    memory: Option<MemoryConfig>,
    tools: ToolsConfig,
}

impl Parse for AgentDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let mut name = None;
        let mut instructions = None;
        let mut llm = None;
        let mut memory = None;
        let mut tools = None;
        
        while !content.is_empty() {
            let key: Ident = content.parse()?;
            let _: Token![:] = content.parse()?;
            
            match key.to_string().as_str() {
                "name" => {
                    name = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "instructions" => {
                    instructions = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "llm" => {
                    llm = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "memory" => {
                    memory = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "tools" => {
                    tools = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new(key.span(), "Unknown field in agent definition")),
            }
        }
        
        let name = name.ok_or_else(|| syn::Error::new(content.span(), "Missing 'name' field in agent definition"))?;
        let instructions = instructions.ok_or_else(|| syn::Error::new(content.span(), "Missing 'instructions' field in agent definition"))?;
        let llm = llm.ok_or_else(|| syn::Error::new(content.span(), "Missing 'llm' field in agent definition"))?;
        let tools = tools.ok_or_else(|| syn::Error::new(content.span(), "Missing 'tools' field in agent definition"))?;
        
        Ok(AgentDef {
            name,
            instructions,
            llm,
            memory,
            tools,
        })
    }
}

/// 实现agent!宏
pub fn agent(input: TokenStream) -> TokenStream {
    let agent_def = parse_macro_input!(input as AgentDef);
    
    let agent_name = &agent_def.name;
    let agent_name_str = agent_name.value();
    let agent_var_name = format_ident!("{}", agent_name_str.to_lowercase().replace("-", "_"));
    
    let instructions = &agent_def.instructions;
    
    let llm_provider = &agent_def.llm.provider;
    let llm_model = &agent_def.llm.model;
    
    // 处理内存配置
    let memory_config = if let Some(memory) = &agent_def.memory {
        let store_type = &memory.store_type;
        
        let capacity = if let Some(cap) = &memory.capacity {
            quote! {
                .with_capacity(#cap)
            }
        } else {
            quote! {}
        };
        
        quote! {
            let memory_config = lumosai_core::memory::MemoryConfig::new(#store_type)
                #capacity;
            
            agent_config.with_memory_config(memory_config);
        }
    } else {
        quote! {}
    };
    
    // 处理工具
    let mut tool_registrations = Vec::new();
    for tool in agent_def.tools.tools.iter() {
        let tool_name = &tool.name;
        
        if let Some(options) = &tool.options {
            tool_registrations.push(quote! {
                agent.add_tool_with_options(#tool_name(), #options).expect("Failed to add tool to agent");
            });
        } else {
            tool_registrations.push(quote! {
                agent.add_tool(#tool_name()).expect("Failed to add tool to agent");
            });
        }
    }
    
    let expanded = quote! {
        {
            use lumosai_core::agent::{Agent, create_basic_agent};
            use lumosai_core::llm::LlmProvider;
            use std::sync::Arc;

            // 创建LLM提供者
            let llm_provider: Arc<dyn LlmProvider> = Arc::new(#llm_provider);

            // 创建代理
            let mut agent = create_basic_agent(
                #agent_name.to_string(),
                #instructions.to_string(),
                llm_provider
            );

            // 添加工具
            #(#tool_registrations)*

            let #agent_var_name = agent;
            #agent_var_name
        }
    };
    
    TokenStream::from(expanded)
} 