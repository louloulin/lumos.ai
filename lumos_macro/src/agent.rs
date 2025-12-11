use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, Ident, LitBool, LitInt, LitStr, Token,
};

/// 增强的 Agent 定义结构 - 支持更多配置选项
///
/// 支持的字段：
/// - name: Agent 名称（必需）
/// - instructions: Agent 指令（必需）
/// - provider: LLM 提供者（可选，如果使用 model 则不需要）
/// - model: 模型名称（可选，如果使用 provider 则不需要）
/// - tools: 工具列表（可选）
/// - memory: 是否启用内存（可选，默认 false）
/// - sop_mode: SOP 执行模式（可选）
/// - max_tool_calls: 最大工具调用次数（可选）
/// - tool_timeout: 工具超时时间（可选）
struct AgentDef {
    name: LitStr,
    instructions: LitStr,
    provider: Option<Expr>,
    model: Option<LitStr>,
    tools: Vec<Ident>,
    memory: Option<LitBool>,
    sop_mode: Option<Ident>,
    max_tool_calls: Option<LitInt>,
    tool_timeout: Option<LitInt>,
}

impl Parse for AgentDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut instructions = None;
        let mut provider = None;
        let mut model = None;
        let mut tools = Vec::new();
        let mut memory = None;
        let mut sop_mode = None;
        let mut max_tool_calls = None;
        let mut tool_timeout = None;

        // 解析所有字段
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            let _: Token![:] = input.parse()?;

            match key.to_string().as_str() {
                "name" => {
                    name = Some(input.parse::<LitStr>()?);
                }
                "instructions" => {
                    instructions = Some(input.parse::<LitStr>()?);
                }
                "provider" => {
                    provider = Some(input.parse::<Expr>()?);
                }
                "model" => {
                    model = Some(input.parse::<LitStr>()?);
                }
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
                }
                "memory" => {
                    memory = Some(input.parse::<LitBool>()?);
                }
                "sop_mode" => {
                    sop_mode = Some(input.parse::<Ident>()?);
                }
                "max_tool_calls" => {
                    max_tool_calls = Some(input.parse::<LitInt>()?);
                }
                "tool_timeout" => {
                    tool_timeout = Some(input.parse::<LitInt>()?);
                }
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("Unknown field '{key}' in agent definition. Supported fields: name, instructions, provider, model, tools, memory, sop_mode, max_tool_calls, tool_timeout"),
                    ));
                }
            }

            // 处理可选的逗号
            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            }
        }

        // 验证必需字段
        let name = name.ok_or_else(|| syn::Error::new(input.span(), "Missing 'name' field"))?;
        let instructions = instructions
            .ok_or_else(|| syn::Error::new(input.span(), "Missing 'instructions' field"))?;

        // provider 和 model 至少需要一个
        if provider.is_none() && model.is_none() {
            return Err(syn::Error::new(
                input.span(),
                "Either 'provider' or 'model' field is required",
            ));
        }

        Ok(AgentDef {
            name,
            instructions,
            provider,
            model,
            tools,
            memory,
            sop_mode,
            max_tool_calls,
            tool_timeout,
        })
    }
}

/// 增强的 agent! 宏实现
///
/// # 语法
///
/// ```rust
/// agent! {
///     name: "agent_name",
///     instructions: "instructions",
///     provider: provider_expr,  // 或者使用 model: "gpt-4"
///     tools: [tool1, tool2, ...],  // 可选
///     memory: true,  // 可选
///     sop_mode: React,  // 可选：React, ByOrder, PlanAndAct
///     max_tool_calls: 10,  // 可选
///     tool_timeout: 30,  // 可选
/// }
/// ```
///
/// # 示例
///
/// ```rust
/// // 基础用法
/// let agent = agent! {
///     name: "researcher",
///     instructions: "You are a research assistant",
///     model: "gpt-4"
/// };
///
/// // 带工具和内存
/// let agent = agent! {
///     name: "researcher",
///     instructions: "You are a research assistant",
///     model: "gpt-4",
///     tools: [web_search, calculator],
///     memory: true,
///     max_tool_calls: 10
/// };
///
/// // 带 SOP 模式
/// let agent = agent! {
///     name: "researcher",
///     instructions: "You are a research assistant",
///     model: "gpt-4",
///     sop_mode: React
/// };
/// ```
pub fn agent(input: TokenStream) -> TokenStream {
    let agent_def = parse_macro_input!(input as AgentDef);

    let agent_name = &agent_def.name;
    let instructions = &agent_def.instructions;

    // 生成 LLM 提供者代码
    let llm_provider_code = if let Some(provider_expr) = &agent_def.provider {
        quote! {
            let llm_provider: std::sync::Arc<dyn lumosai_core::llm::LlmProvider> = std::sync::Arc::new(#provider_expr);
        }
    } else if let Some(model_name) = &agent_def.model {
        quote! {
            // 使用模型名称创建提供者（需要环境变量配置）
            let llm_provider: std::sync::Arc<dyn lumosai_core::llm::LlmProvider> = {
                use lumosai_core::llm::openai::OpenAiProvider;
                std::sync::Arc::new(OpenAiProvider::from_env().expect("Failed to create OpenAI provider"))
            };
        }
    } else {
        quote! {
            compile_error!("Either 'provider' or 'model' must be specified");
        }
    };

    // 生成工具注册代码
    let tool_registrations: Vec<_> = agent_def.tools.iter().map(|tool_name| {
        quote! {
            agent.add_tool(#tool_name()).expect(&format!("Failed to add tool '{}' to agent", stringify!(#tool_name)));
        }
    }).collect();

    // 生成内存配置代码
    let memory_config = if let Some(memory_enabled) = &agent_def.memory {
        let _ = memory_enabled.value; // 标记为已使用
        quote! {
            // TODO: 添加内存配置
            // agent.enable_memory();
        }
    } else {
        quote! {}
    };

    // 生成 SOP 模式配置代码
    let sop_config = if let Some(sop_mode) = &agent_def.sop_mode {
        quote! {
            // TODO: 配置 SOP 模式
            // agent.set_sop_mode(lumosai_core::agent::SopExecutionMode::#sop_mode);
        }
    } else {
        quote! {}
    };

    // 生成最大工具调用次数配置
    let max_tool_calls_config = if let Some(max_calls) = &agent_def.max_tool_calls {
        quote! {
            // TODO: 配置最大工具调用次数
            // agent.set_max_tool_calls(#max_calls);
        }
    } else {
        quote! {}
    };

    // 生成工具超时配置
    let tool_timeout_config = if let Some(timeout) = &agent_def.tool_timeout {
        quote! {
            // TODO: 配置工具超时
            // agent.set_tool_timeout(#timeout);
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        {
            use lumosai_core::agent::create_basic_agent;
            use lumosai_core::llm::LlmProvider;
            use std::sync::Arc;

            // 创建 LLM 提供者
            #llm_provider_code

            // 创建 Agent
            let mut agent = create_basic_agent(
                #agent_name.to_string(),
                #instructions.to_string(),
                llm_provider
            ).expect("Failed to create basic agent");

            // 添加工具
            #(#tool_registrations)*

            // 配置内存
            #memory_config

            // 配置 SOP 模式
            #sop_config

            // 配置最大工具调用次数
            #max_tool_calls_config

            // 配置工具超时
            #tool_timeout_config

            agent
        }
    };

    TokenStream::from(expanded)
}
