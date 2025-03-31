use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr, Expr, Token, braced, parse::{Parse, ParseStream}};
use syn::punctuated::Punctuated;

// MCP服务器发现配置
struct DiscoveryConfig {
    endpoints: Expr,
    auto_register: Option<Expr>,
}

impl Parse for DiscoveryConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let mut endpoints = None;
        let mut auto_register = None;
        
        while !content.is_empty() {
            let key: syn::Ident = content.parse()?;
            let _: Token![:] = content.parse()?;
            
            match key.to_string().as_str() {
                "endpoints" => {
                    endpoints = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "auto_register" => {
                    auto_register = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new(key.span(), "Unknown field in discovery config")),
            }
        }
        
        let endpoints = endpoints.ok_or_else(|| syn::Error::new(content.span(), "Missing 'endpoints' field in discovery config"))?;
        
        Ok(DiscoveryConfig {
            endpoints,
            auto_register,
        })
    }
}

// MCP工具授权配置
struct AuthConfig {
    auth_type: LitStr,
    key_env: Option<LitStr>,
}

impl Parse for AuthConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let mut auth_type = None;
        let mut key_env = None;
        
        while !content.is_empty() {
            let key: syn::Ident = content.parse()?;
            let _: Token![:] = content.parse()?;
            
            match key.to_string().as_str() {
                "type" => {
                    auth_type = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "key_env" => {
                    key_env = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new(key.span(), "Unknown field in auth config")),
            }
        }
        
        let auth_type = auth_type.ok_or_else(|| syn::Error::new(content.span(), "Missing 'type' field in auth config"))?;
        
        Ok(AuthConfig {
            auth_type,
            key_env,
        })
    }
}

// MCP工具配置
struct ToolConfig {
    enabled: Option<Expr>,
    auth: Option<AuthConfig>,
    rate_limit: Option<Expr>,
}

impl Parse for ToolConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let mut enabled = None;
        let mut auth = None;
        let mut rate_limit = None;
        
        while !content.is_empty() {
            let key: syn::Ident = content.parse()?;
            let _: Token![:] = content.parse()?;
            
            match key.to_string().as_str() {
                "enabled" => {
                    enabled = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "auth" => {
                    auth = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "rate_limit" => {
                    rate_limit = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new(key.span(), "Unknown field in tool config")),
            }
        }
        
        Ok(ToolConfig {
            enabled,
            auth,
            rate_limit,
        })
    }
}

// MCP工具集合
struct ToolsDef {
    name: syn::Ident,
    config: ToolConfig,
}

impl Parse for ToolsDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: syn::Ident = input.parse()?;
        let _: Token![:] = input.parse()?;
        let config: ToolConfig = input.parse()?;
        
        Ok(ToolsDef { name, config })
    }
}

// MCP客户端配置
struct McpClientDef {
    discovery: DiscoveryConfig,
    tools: Option<Punctuated<ToolsDef, Token![,]>>,
}

impl Parse for McpClientDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let mut discovery = None;
        let mut tools = None;
        
        while !content.is_empty() {
            let key: syn::Ident = content.parse()?;
            let _: Token![:] = content.parse()?;
            
            match key.to_string().as_str() {
                "discovery" => {
                    discovery = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "tools" => {
                    let tools_content;
                    braced!(tools_content in content);
                    tools = Some(Punctuated::parse_terminated(&tools_content)?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new(key.span(), "Unknown field in MCP client definition")),
            }
        }
        
        let discovery = discovery.ok_or_else(|| syn::Error::new(content.span(), "Missing 'discovery' field in MCP client definition"))?;
        
        Ok(McpClientDef {
            discovery,
            tools,
        })
    }
}

/// 创建一个MCP客户端配置，参考Mastra的MCP支持
/// 
/// # 示例
/// 
/// ```rust
/// mcp_client! {
///     discovery: {
///         endpoints: ["https://tools.example.com/mcp", "https://api.mcp.run"],
///         auto_register: true
///     },
///     
///     tools: {
///         data_analysis: {
///             enabled: true,
///             auth: {
///                 type: "api_key",
///                 key_env: "DATA_ANALYSIS_API_KEY"
///             }
///         },
///         image_processing: {
///             enabled: true,
///             rate_limit: 100
///         }
///     }
/// }
/// ```
pub fn mcp_client_impl(input: TokenStream) -> TokenStream {
    let mcp_client_def = parse_macro_input!(input as McpClientDef);
    
    let discovery = &mcp_client_def.discovery;
    let endpoints = &discovery.endpoints;
    
    let auto_register = match &discovery.auto_register {
        Some(auto_reg) => quote! { .with_auto_register(#auto_reg) },
        None => quote! {},
    };
    
    let tool_registrations = if let Some(tools) = &mcp_client_def.tools {
        let tool_statements = tools.iter().map(|tool| {
            let tool_name = &tool.name;
            let tool_name_str = tool_name.to_string();
            
            let enabled = match &tool.config.enabled {
                Some(enabled) => quote! { .with_enabled(#enabled) },
                None => quote! {},
            };
            
            let rate_limit = match &tool.config.rate_limit {
                Some(limit) => quote! { .with_rate_limit(#limit) },
                None => quote! {},
            };
            
            let auth_config = match &tool.config.auth {
                Some(auth) => {
                    let auth_type = &auth.auth_type;
                    
                    let key_env = match &auth.key_env {
                        Some(key_env) => quote! { .with_key_env(#key_env) },
                        None => quote! {},
                    };
                    
                    quote! {
                        .with_auth_type(#auth_type)
                        #key_env
                    }
                },
                None => quote! {},
            };
            
            quote! {
                mcp_client.register_tool(#tool_name_str, ToolConfig::new()
                    #enabled
                    #rate_limit
                    #auth_config
                );
            }
        }).collect::<Vec<_>>();
        
        quote! {
            #(#tool_statements)*
        }
    } else {
        quote! {}
    };
    
    let expanded = quote! {
        {
            use lomusai_core::mcp::*;
            
            let mut mcp_client = McpClient::new()
                .with_endpoints(#endpoints)
                #auto_register;
                
            // 注册工具
            #tool_registrations
            
            mcp_client
        }
    };
    
    TokenStream::from(expanded)
} 