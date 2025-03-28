use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, LitStr, Expr, Token, braced, parse::{Parse, ParseStream}};
use syn::punctuated::Punctuated;

// 参数定义
struct ParameterDef {
    name: LitStr,
    description: LitStr,
    r#type: LitStr,
    required: Option<Expr>,
    default: Option<Expr>,
}

impl Parse for ParameterDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let mut name = None;
        let mut description = None;
        let mut r#type = None;
        let mut required = None;
        let mut default = None;
        
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
                "type" => {
                    r#type = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "required" => {
                    required = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "default" => {
                    default = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new(key.span(), "Unknown field in parameter definition")),
            }
        }
        
        let name = name.ok_or_else(|| syn::Error::new(content.span(), "Missing 'name' field in parameter definition"))?;
        let description = description.ok_or_else(|| syn::Error::new(content.span(), "Missing 'description' field in parameter definition"))?;
        let r#type = r#type.ok_or_else(|| syn::Error::new(content.span(), "Missing 'type' field in parameter definition"))?;
        
        Ok(ParameterDef {
            name,
            description,
            r#type,
            required,
            default,
        })
    }
}

// 工具定义
struct ToolDef {
    name: LitStr,
    description: LitStr,
    parameters: Punctuated<ParameterDef, Token![,]>,
    handler: Expr,
}

impl Parse for ToolDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let mut name = None;
        let mut description = None;
        let mut parameters_content = None;
        let mut handler = None;
        
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
                "parameters" => {
                    let params_braced;
                    braced!(params_braced in content);
                    parameters_content = Some(params_braced);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "handler" => {
                    handler = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new(key.span(), "Unknown field in tool definition")),
            }
        }
        
        let name = name.ok_or_else(|| syn::Error::new(content.span(), "Missing 'name' field in tool definition"))?;
        let description = description.ok_or_else(|| syn::Error::new(content.span(), "Missing 'description' field in tool definition"))?;
        let handler = handler.ok_or_else(|| syn::Error::new(content.span(), "Missing 'handler' field in tool definition"))?;
        
        let parameters = if let Some(params_content) = parameters_content {
            Punctuated::parse_terminated(&params_content)?
        } else {
            Punctuated::new()
        };
        
        Ok(ToolDef {
            name,
            description,
            parameters,
            handler,
        })
    }
}

// 工具集定义
struct ToolsSetDef {
    tools: Punctuated<ToolDef, Token![,]>,
}

impl Parse for ToolsSetDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let tools = Punctuated::parse_terminated(&content)?;
        
        Ok(ToolsSetDef { tools })
    }
}

/// 一次性定义多个工具，参考Mastra的工具API设计
/// 
/// # 示例
/// 
/// ```rust
/// tools! {
///     {
///         name: "calculator",
///         description: "执行基本的数学运算",
///         parameters: {
///             {
///                 name: "operation",
///                 description: "要执行的操作: add, subtract, multiply, divide",
///                 type: "string",
///                 required: true
///             },
///             {
///                 name: "a",
///                 description: "第一个数字",
///                 type: "number",
///                 required: true
///             },
///             {
///                 name: "b",
///                 description: "第二个数字",
///                 type: "number",
///                 required: true
///             }
///         },
///         handler: |params| async move {
///             let operation = params.get("operation").unwrap().as_str().unwrap();
///             let a = params.get("a").unwrap().as_f64().unwrap();
///             let b = params.get("b").unwrap().as_f64().unwrap();
///             
///             let result = match operation {
///                 "add" => a + b,
///                 "subtract" => a - b,
///                 "multiply" => a * b,
///                 "divide" => a / b,
///                 _ => return Err(Error::InvalidInput("Unknown operation".into()))
///             };
///             
///             Ok(json!({ "result": result }))
///         }
///     },
///     {
///         name: "weather",
///         description: "获取指定城市的天气信息",
///         parameters: {
///             {
///                 name: "city",
///                 description: "城市名称",
///                 type: "string",
///                 required: true
///             }
///         },
///         handler: get_weather_data
///     }
/// }
/// ```
pub fn tools(input: TokenStream) -> TokenStream {
    let tools_set = parse_macro_input!(input as ToolsSetDef);
    
    let mut tool_function_defs = Vec::new();
    
    for tool in tools_set.tools.iter() {
        let tool_name = &tool.name;
        let tool_name_str = tool_name.value();
        let tool_fn_name = format_ident!("{}", tool_name_str.to_lowercase().replace("-", "_").replace(" ", "_"));
        let description = &tool.description;
        let handler = &tool.handler;
        
        // 处理参数
        let parameters = tool.parameters.iter().map(|param| {
            let param_name = &param.name;
            let param_description = &param.description;
            let param_type = &param.r#type;
            
            let required = match &param.required {
                Some(req) => quote! { #req },
                None => quote! { false },
            };
            
            let default = match &param.default {
                Some(def) => quote! { Some(#def) },
                None => quote! { None },
            };
            
            quote! {
                lomusai_core::tool::ParameterSchema {
                    name: #param_name.to_string(),
                    description: #param_description.to_string(),
                    r#type: #param_type.to_string(),
                    required: #required,
                    properties: None,
                    default: #default,
                }
            }
        }).collect::<Vec<_>>();
        
        let tool_def = quote! {
            pub fn #tool_fn_name() -> Box<dyn lomusai_core::tool::Tool> {
                Box::new(lomusai_core::tool::FunctionTool::new(
                    #tool_name.to_string(),
                    #description.to_string(),
                    lomusai_core::tool::ToolSchema {
                        parameters: vec![
                            #(#parameters),*
                        ]
                    },
                    #handler,
                ))
            }
        };
        
        tool_function_defs.push(tool_def);
    }
    
    let expanded = quote! {
        {
            #(#tool_function_defs)*
        }
    };
    
    TokenStream::from(expanded)
} 