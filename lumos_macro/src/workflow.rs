use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, LitStr, Expr, Token, braced, parse::{Parse, ParseStream}};
use syn::punctuated::Punctuated;

// 工作流步骤定义
struct WorkflowStep {
    name: LitStr,
    agent: Expr,
    instructions: Option<LitStr>,
    tools: Option<Expr>,
    when: Option<Expr>,
}

impl Parse for WorkflowStep {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let mut name = None;
        let mut agent = None;
        let mut instructions = None;
        let mut tools = None;
        let mut when = None;
        
        while !content.is_empty() {
            let key: syn::Ident = content.parse()?;
            let _: Token![:] = content.parse()?;
            
            match key.to_string().as_str() {
                "name" => {
                    name = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "agent" => {
                    agent = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "instructions" => {
                    instructions = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "tools" => {
                    tools = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "when" => {
                    when = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new(key.span(), "Unknown field in workflow step")),
            }
        }
        
        let name = name.ok_or_else(|| syn::Error::new(content.span(), "Missing 'name' field in workflow step"))?;
        let agent = agent.ok_or_else(|| syn::Error::new(content.span(), "Missing 'agent' field in workflow step"))?;
        
        Ok(WorkflowStep {
            name,
            agent,
            instructions,
            tools,
            when,
        })
    }
}

// 工作流定义
struct WorkflowDef {
    name: LitStr,
    description: Option<LitStr>,
    steps: Punctuated<WorkflowStep, Token![,]>,
}

impl Parse for WorkflowDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let mut name = None;
        let mut description = None;
        let mut steps_content = None;
        
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
                "steps" => {
                    let steps_braced;
                    braced!(steps_braced in content);
                    steps_content = Some(steps_braced);
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new(key.span(), "Unknown field in workflow definition")),
            }
        }
        
        let name = name.ok_or_else(|| syn::Error::new(content.span(), "Missing 'name' field in workflow definition"))?;
        
        let steps_content = steps_content.ok_or_else(|| 
            syn::Error::new(content.span(), "Missing 'steps' field in workflow definition")
        )?;
        
        let steps = Punctuated::parse_terminated(&steps_content)?;
        
        Ok(WorkflowDef {
            name,
            description,
            steps,
        })
    }
}

/// 创建一个工作流定义，参考Mastra的工作流API设计
/// 
/// # 示例
/// 
/// ```rust
/// workflow! {
///     name: "content_creation",
///     description: "创建高质量的内容",
///     steps: {
///         {
///             name: "research",
///             agent: researcher,
///             instructions: "进行深入的主题研究",
///         },
///         {
///             name: "writing",
///             agent: writer,
///             instructions: "将研究结果整理成文章",
///             when: { completed("research") },
///         },
///         {
///             name: "review",
///             agent: reviewer,
///             instructions: "检查文章质量和准确性",
///             when: { completed("writing") },
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn workflow(input: TokenStream) -> TokenStream {
    let workflow_def = parse_macro_input!(input as WorkflowDef);
    
    let workflow_name = &workflow_def.name;
    let workflow_name_str = workflow_name.value();
    let workflow_var_name = format_ident!("{}", workflow_name_str.to_lowercase());
    
    let description = match &workflow_def.description {
        Some(desc) => quote! { .with_description(#desc) },
        None => quote! {},
    };
    
    let mut step_defs = Vec::new();
    let mut step_registrations = Vec::new();
    
    for step in workflow_def.steps.iter() {
        let step_name = &step.name;
        let step_name_str = step_name.value();
        let step_var_name = format_ident!("step_{}", step_name_str.to_lowercase());
        let agent = &step.agent;
        
        let instructions = match &step.instructions {
            Some(instr) => quote! { .with_instructions(#instr) },
            None => quote! {},
        };
        
        let tools = match &step.tools {
            Some(tools_expr) => quote! { .with_tools(#tools_expr) },
            None => quote! {},
        };
        
        let step_def = quote! {
            let #step_var_name = lomusai_core::workflow::WorkflowStep::new(#step_name)
                .with_agent(#agent)
                #instructions
                #tools;
        };
        
        step_defs.push(step_def);
        
        let step_reg = if let Some(when) = &step.when {
            quote! {
                workflow_def.add_step_with_condition(#step_var_name, |ctx| {
                    #when
                });
            }
        } else {
            quote! {
                workflow_def.add_step(#step_var_name);
            }
        };
        
        step_registrations.push(step_reg);
    }
    
    let expanded = quote! {
        {
            #(#step_defs)*
            
            let mut workflow_def = lomusai_core::workflow::WorkflowDefinition::new(None)
                .with_name(#workflow_name)
                #description;
                
            #(#step_registrations)*
            
            let #workflow_var_name = workflow_def;
            #workflow_var_name
        }
    };
    
    TokenStream::from(expanded)
} 