use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, LitStr, Expr, Token, braced, parse::{Parse, ParseStream}};
use syn::punctuated::Punctuated;

// 代理工具定义
struct AgentTool {
    name: syn::Ident,
    options: Option<Expr>,
}

impl Parse for AgentTool {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: syn::Ident = input.parse()?;
        
        let options = if input.peek(Token![:]) {
            let _: Token![:] = input.parse()?;
            Some(input.parse()?)
        } else {
            None
        };
        
        Ok(AgentTool { name, options })
    }
}

// 代理内存配置
struct MemoryConfig {
    store_type: LitStr,
    capacity: Option<Expr>,
    options: Option<Expr>,
}

impl Parse for MemoryConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let mut store_type = None;
        let mut capacity = None;
        let mut options = None;
        
        while !content.is_empty() {
            let key: syn::Ident = content.parse()?;
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
                "options" => {
                    options = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new(key.span(), "Unknown field in memory config")),
            }
        }
        
        let store_type = store_type.ok_or_else(|| syn::Error::new(content.span(), "Missing 'store_type' field in memory config"))?;
        
        Ok(MemoryConfig {
            store_type,
            capacity,
            options,
        })
    }
}

// 代理LLM配置
struct LlmConfig {
    provider: Expr,
    model: Option<LitStr>,
    options: Option<Expr>,
}

impl Parse for LlmConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let mut provider = None;
        let mut model = None;
        let mut options = None;
        
        while !content.is_empty() {
            let key: syn::Ident = content.parse()?;
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
                "options" => {
                    options = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new(key.span(), "Unknown field in LLM config")),
            }
        }
        
        let provider = provider.ok_or_else(|| syn::Error::new(content.span(), "Missing 'provider' field in LLM config"))?;
        
        Ok(LlmConfig {
            provider,
            model,
            options,
        })
    }
}

// 代理定义
struct AgentDef {
    name: LitStr,
    instructions: LitStr,
    llm: LlmConfig,
    memory: Option<MemoryConfig>,
    tools: Punctuated<AgentTool, Token![,]>,
}

impl Parse for AgentDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let mut name = None;
        let mut instructions = None;
        let mut llm = None;
        let mut memory = None;
        let mut tools_content = None;
        
        while !content.is_empty() {
            let key: syn::Ident = content.parse()?;
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
                    let tools_braced;
                    braced!(tools_braced in content);
                    tools_content = Some(tools_braced);
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new(key.span(), "Unknown field in agent definition")),
            }
        }
        
        let name = name.ok_or_else(|| syn::Error::new(content.span(), "Missing 'name' field in agent definition"))?;
        let instructions = instructions.ok_or_else(|| syn::Error::new(content.span(), "Missing 'instructions' field in agent definition"))?;
        let llm = llm.ok_or_else(|| syn::Error::new(content.span(), "Missing 'llm' field in agent definition"))?;
        
        let tools_content = tools_content.ok_or_else(|| 
            syn::Error::new(content.span(), "Missing 'tools' field in agent definition")
        )?;
        
        let tools = Punctuated::parse_terminated(&tools_content)?;
        
        Ok(AgentDef {
            name,
            instructions,
            llm,
            memory,
            tools,
        })
    }
}

/// 创建一个代理定义，参考Mastra的Agent API设计
/// 
/// # 示例
/// 
/// ```rust
/// agent! {
///     name: "research_assistant",
///     instructions: "你是一个专业的研究助手，擅长收集和整理信息。",
///     
///     llm: {
///         provider: openai_adapter,
///         model: "gpt-4"
///     },
///     
///     memory: {
///         store_type: "buffer",
///         capacity: 10
///     },
///     
///     tools: {
///         search_tool,
///         calculator_tool: { precision: 2 },
///         web_browser: { javascript: true, screenshots: true }
///     }
/// }
/// ```
pub fn agent(input: TokenStream) -> TokenStream {
    let agent_def = parse_macro_input!(input as AgentDef);
    
    let agent_name = &agent_def.name;
    let agent_name_str = agent_def.name.value();
    let agent_var_name = format_ident!("{}", agent_name_str.to_lowercase().replace("-", "_").replace(" ", "_"));
    let instructions = &agent_def.instructions;
    
    // 处理LLM配置
    let llm_provider = &agent_def.llm.provider;
    let llm_model = match &agent_def.llm.model {
        Some(model) => quote! { .with_model(#model) },
        None => quote! {},
    };
    
    let llm_options = match &agent_def.llm.options {
        Some(options) => quote! { .with_options(#options) },
        None => quote! {},
    };
    
    // 处理内存配置
    let memory_config = if let Some(memory) = &agent_def.memory {
        let store_type = &memory.store_type;
        
        let capacity = match &memory.capacity {
            Some(cap) => quote! { .with_capacity(#cap) },
            None => quote! {},
        };
        
        let options = match &memory.options {
            Some(opts) => quote! { .with_options(#opts) },
            None => quote! {},
        };
        
        quote! {
            let memory_config = lomusai_core::memory::MemoryConfig::new(#store_type)
                #capacity
                #options;
                
            agent.set_memory(memory_config);
        }
    } else {
        quote! {}
    };
    
    // 处理工具
    let tool_registrations = agent_def.tools.iter().map(|tool| {
        let tool_name = &tool.name;
        
        if let Some(options) = &tool.options {
            quote! {
                agent.add_tool_with_options(#tool_name(), #options).expect("Failed to add tool");
            }
        } else {
            quote! {
                agent.add_tool(#tool_name()).expect("Failed to add tool");
            }
        }
    }).collect::<Vec<_>>();
    
    let expanded = quote! {
        {
            // 创建LLM提供者
            let llm_provider = #llm_provider
                #llm_model
                #llm_options;
            
            // 创建代理
            let mut agent = lomusai_core::agent::SimpleAgent::new(
                #agent_name,
                #instructions,
                std::sync::Arc::new(llm_provider),
            );
            
            // 配置内存
            #memory_config
            
            // 注册工具
            #(#tool_registrations)*
            
            let #agent_var_name = agent;
            #agent_var_name
        }
    };
    
    TokenStream::from(expanded)
} 