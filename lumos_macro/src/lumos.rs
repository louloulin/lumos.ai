use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{
    parse_macro_input, braced, Expr, Ident, LitStr, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

// 应用组件项目
struct ComponentItem {
    name: Ident,
    expr: Option<Expr>,
}

impl Parse for ComponentItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        
        let expr = if input.peek(Token![:]) {
            let _: Token![:] = input.parse()?;
            Some(input.parse()?)
        } else {
            None
        };
        
        Ok(ComponentItem { name, expr })
    }
}

// 应用组件集合
struct ComponentsConfig {
    items: Punctuated<ComponentItem, Token![,]>,
}

impl Parse for ComponentsConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let items = Punctuated::parse_terminated(&content)?;
        
        Ok(ComponentsConfig { items })
    }
}

// 应用定义
struct LumosAppDef {
    name: LitStr,
    description: Option<LitStr>,
    agents: Option<ComponentsConfig>,
    tools: Option<ComponentsConfig>,
    rags: Option<ComponentsConfig>,
    workflows: Option<ComponentsConfig>,
    mcp_endpoints: Option<Expr>,
}

impl Parse for LumosAppDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let mut name = None;
        let mut description = None;
        let mut agents = None;
        let mut tools = None;
        let mut rags = None;
        let mut workflows = None;
        let mut mcp_endpoints = None;
        
        while !content.is_empty() {
            let key: Ident = content.parse()?;
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
                _ => return Err(syn::Error::new(key.span(), "Unknown field in Lumos app definition")),
            }
        }
        
        let name = name.ok_or_else(|| syn::Error::new(content.span(), "Missing 'name' field in Lumos app definition"))?;
        
        Ok(LumosAppDef {
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

/// 实现lumos!宏
pub fn lumos(input: TokenStream) -> TokenStream {
    let app_def = parse_macro_input!(input as LumosAppDef);
    
    let app_name = &app_def.name;
    let app_name_str = app_name.value();
    let app_var_name = format_ident!("{}", app_name_str.to_lowercase().replace("-", "_"));
    
    // 描述
    let description = match &app_def.description {
        Some(desc) => quote! { .with_description(#desc) },
        None => quote! {},
    };
    
    // 处理代理
    let agent_registrations = if let Some(agents) = &app_def.agents {
        let agent_statements = agents.items.iter().map(|item| {
            let agent_name = &item.name;
            
            let agent_expr = match &item.expr {
                Some(expr) => quote! { #expr },
                None => quote! { #agent_name },
            };
            
            quote! {
                app.add_agent(#agent_name.to_string(), #agent_expr);
            }
        }).collect::<Vec<_>>();
        
        quote! {
            #(#agent_statements)*
        }
    } else {
        quote! {}
    };
    
    // 处理工具
    let tool_registrations = if let Some(tools) = &app_def.tools {
        let tool_statements = tools.items.iter().map(|item| {
            let tool_name = &item.name;
            
            let tool_expr = match &item.expr {
                Some(expr) => quote! { #expr() },
                None => quote! { #tool_name() },
            };
            
            quote! {
                app.add_tool(#tool_name.to_string(), #tool_expr);
            }
        }).collect::<Vec<_>>();
        
        quote! {
            #(#tool_statements)*
        }
    } else {
        quote! {}
    };
    
    // 处理RAG
    let rag_registrations = if let Some(rags) = &app_def.rags {
        let rag_statements = rags.items.iter().map(|item| {
            let rag_name = &item.name;
            
            let rag_expr = match &item.expr {
                Some(expr) => quote! { #expr },
                None => quote! { #rag_name },
            };
            
            quote! {
                app.add_rag(#rag_name.to_string(), #rag_expr);
            }
        }).collect::<Vec<_>>();
        
        quote! {
            #(#rag_statements)*
        }
    } else {
        quote! {}
    };
    
    // 处理工作流
    let workflow_registrations = if let Some(workflows) = &app_def.workflows {
        let workflow_statements = workflows.items.iter().map(|item| {
            let workflow_name = &item.name;
            
            let workflow_expr = match &item.expr {
                Some(expr) => quote! { #expr },
                None => quote! { #workflow_name },
            };
            
            quote! {
                app.add_workflow(#workflow_name.to_string(), #workflow_expr);
            }
        }).collect::<Vec<_>>();
        
        quote! {
            #(#workflow_statements)*
        }
    } else {
        quote! {}
    };
    
    // 处理MCP端点
    let mcp_config = if let Some(endpoints) = &app_def.mcp_endpoints {
        quote! {
            app.set_mcp_endpoints(#endpoints);
        }
    } else {
        quote! {}
    };
    
    let expanded = quote! {
        {
            use lumosai_core::app::LumosApp;
            
            let mut app = LumosApp::new(#app_name)
                #description;
                
            // 注册代理
            #agent_registrations
            
            // 注册工具
            #tool_registrations
            
            // 注册RAG
            #rag_registrations
            
            // 注册工作流
            #workflow_registrations
            
            // 配置MCP端点
            #mcp_config
            
            let #app_var_name = app;
            #app_var_name
        }
    };
    
    TokenStream::from(expanded)
} 