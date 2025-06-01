use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, braced, bracketed, Expr, Ident, LitStr, Token, parse::{Parse, ParseStream}};

/// 简化的Agent定义结构 - 使用syn直接解析
struct SimpleAgentDef {
    name: LitStr,
    instructions: LitStr,
    provider: Expr,
    tools: Vec<Ident>,
}

impl Parse for SimpleAgentDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut instructions = None;
        let mut provider = None;
        let mut tools = Vec::new();

        // 直接解析字段，不需要额外的花括号
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            let _: Token![:] = input.parse()?;

            match key.to_string().as_str() {
                "name" => {
                    name = Some(input.parse::<LitStr>()?);
                },
                "instructions" => {
                    instructions = Some(input.parse::<LitStr>()?);
                },
                "provider" => {
                    provider = Some(input.parse::<Expr>()?);
                },
                "tools" => {
                    // 解析工具数组 [tool1, tool2, ...]
                    let tools_content;
                    let _ = syn::bracketed!(tools_content in input);

                    while !tools_content.is_empty() {
                        tools.push(tools_content.parse::<Ident>()?);

                        // 处理逗号
                        if tools_content.peek(Token![,]) {
                            let _: Token![,] = tools_content.parse()?;
                        }
                    }
                },
                _ => {
                    return Err(syn::Error::new(key.span(), format!("Unknown field '{}' in agent definition", key)));
                }
            }

            // 处理可选的逗号
            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            }
        }

        // 验证必需字段
        let name = name.ok_or_else(|| syn::Error::new(input.span(), "Missing 'name' field"))?;
        let instructions = instructions.ok_or_else(|| syn::Error::new(input.span(), "Missing 'instructions' field"))?;
        let provider = provider.ok_or_else(|| syn::Error::new(input.span(), "Missing 'provider' field"))?;

        Ok(SimpleAgentDef {
            name,
            instructions,
            provider,
            tools,
        })
    }
}

/// 简化的agent!宏实现 - 回到syn解析
///
/// 语法：
/// ```
/// agent! {
///     name: "agent_name",
///     instructions: "instructions",
///     provider: provider_expr,
///     tools: [tool1, tool2, ...]
/// }
/// ```
pub fn agent(input: TokenStream) -> TokenStream {
    let agent_def = parse_macro_input!(input as SimpleAgentDef);

    let agent_name = &agent_def.name;
    let instructions = &agent_def.instructions;
    let provider_expr = &agent_def.provider;

    // 生成工具注册代码
    let tool_registrations: Vec<_> = agent_def.tools.iter().map(|tool_name| {
        quote! {
            agent.add_tool(#tool_name()).expect(&format!("Failed to add tool '{}' to agent", stringify!(#tool_name)));
        }
    }).collect();

    let expanded = quote! {
        {
            use lumosai_core::agent::create_basic_agent;
            use lumosai_core::llm::LlmProvider;
            use std::sync::Arc;

            // 创建LLM提供者
            let llm_provider: Arc<dyn LlmProvider> = Arc::new(#provider_expr);

            // 创建代理
            let mut agent = create_basic_agent(
                #agent_name.to_string(),
                #instructions.to_string(),
                llm_provider
            );

            // 添加工具
            #(#tool_registrations)*

            agent
        }
    };

    TokenStream::from(expanded)
}
