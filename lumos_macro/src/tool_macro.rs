use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::str::FromStr;
use syn::spanned::Spanned;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, FnArg, Ident, ItemFn, LitStr, PatType, Token, Type,
};

use crate::parser::{parse_tool_macro, ToolDef};

// 工具属性解析
pub struct ToolAttributes {
    pub name: LitStr,
    pub description: LitStr,
}

impl Parse for ToolAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut description = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;

            match key.to_string().as_str() {
                "name" => {
                    name = Some(input.parse()?);
                    let _: Option<Token![,]> = input.parse()?;
                }
                "description" => {
                    description = Some(input.parse()?);
                    let _: Option<Token![,]> = input.parse()?;
                }
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        "Unknown attribute in tool definition",
                    ))
                }
            }
        }

        let name = name.ok_or_else(|| {
            syn::Error::new(input.span(), "Missing 'name' attribute in tool definition")
        })?;
        let description = description.ok_or_else(|| {
            syn::Error::new(
                input.span(),
                "Missing 'description' attribute in tool definition",
            )
        })?;

        Ok(ToolAttributes { name, description })
    }
}

// Import ParameterAttributes from parent module
use crate::ParameterAttributes;

// ToolExecuteArgs结构体定义
pub struct ToolExecuteArgs {
    pub tool: Expr,
    pub params: Expr,
}

impl Parse for ToolExecuteArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = syn::braced!(content in input);

        let mut tool = None;
        let mut params = None;

        while !content.is_empty() {
            let key: Ident = content.parse()?;
            let _: Token![:] = content.parse()?;

            match key.to_string().as_str() {
                "tool" => {
                    tool = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                }
                "params" => {
                    let _params_str: LitStr; // 标记为未使用
                    let inner_content;
                    let _ = syn::braced!(inner_content in content);

                    // 直接创建一个表达式
                    let inner_content_str = inner_content.to_string();
                    params = Some(Expr::Verbatim(
                        proc_macro2::TokenStream::from_str(&format!("{{{}}}", inner_content_str))
                            .unwrap(),
                    ));

                    let _: Option<Token![,]> = content.parse()?;
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &key,
                        "Unknown key in tool execution, expected 'tool' or 'params'",
                    ))
                }
            }
        }

        let tool = tool.ok_or_else(|| {
            syn::Error::new(Span::call_site(), "Missing 'tool' in tool execution")
        })?;
        let params = params.ok_or_else(|| {
            syn::Error::new(Span::call_site(), "Missing 'params' in tool execution")
        })?;

        Ok(ToolExecuteArgs { tool, params })
    }
}

pub fn tool_macro(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let attrs = parse_macro_input!(attr as ToolAttributes);

    let fn_name = &input.sig.ident;
    let fn_params = &input.sig.inputs;
    let fn_body = &input.block;
    let _fn_output = &input.sig.output; // 标记为未使用

    let tool_name = attrs.name.value();
    let tool_description = attrs.description.value();

    // Extract parameter metadata from attributes
    let mut parameters = Vec::new();
    let mut param_extractions = Vec::new();
    let mut param_names = Vec::new();

    for param in fn_params.iter() {
        if let syn::FnArg::Typed(pat_type) = param {
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                let param_name = &pat_ident.ident;
                let param_type = &pat_type.ty;

                // Find parameter attributes
                for attr in &pat_type.attrs {
                    if attr.path().is_ident("parameter") {
                        let param_attr = match &attr.meta {
                            syn::Meta::List(meta_list) => {
                                match syn::parse2::<ParameterAttributes>(meta_list.tokens.clone()) {
                                    Ok(attr) => attr,
                                    Err(e) => {
                                        let error = syn::Error::new(
                                            attr.span(),
                                            format!("Failed to parse parameter attributes: {}", e),
                                        );
                                        return error.to_compile_error().into();
                                    }
                                }
                            }
                            _ => {
                                let error = syn::Error::new(
                                    attr.span(),
                                    "Parameter attribute must be a list",
                                );
                                return error.to_compile_error().into();
                            }
                        };
                        let name = param_attr.name.value();
                        let description = param_attr.description.value();
                        let type_ = param_attr.type_.value();
                        let required = param_attr.required;

                        let param_schema = quote! {
                            ParameterSchema {
                                name: #name.to_string(),
                                description: #description.to_string(),
                                r#type: #type_.to_string(),
                                required: #required,
                                properties: None,
                                default: None,
                            }
                        };
                        parameters.push(param_schema);

                        // Generate parameter extraction code
                        let extraction = match type_.as_str() {
                            "string" => quote! {
                                let #param_name: #param_type = params.get(#name)
                                    .and_then(|v| v.as_str())
                                    .ok_or_else(|| lumosai_core::Error::InvalidInput(format!("Missing or invalid parameter: {}", #name)))?
                                    .to_string();
                            },
                            "number" => quote! {
                                let #param_name: #param_type = params.get(#name)
                                    .and_then(|v| v.as_f64())
                                    .ok_or_else(|| lumosai_core::Error::InvalidInput(format!("Missing or invalid parameter: {}", #name)))?;
                            },
                            "integer" => quote! {
                                let #param_name: #param_type = params.get(#name)
                                    .and_then(|v| v.as_i64())
                                    .ok_or_else(|| lumosai_core::Error::InvalidInput(format!("Missing or invalid parameter: {}", #name)))?;
                            },
                            "boolean" => quote! {
                                let #param_name: #param_type = params.get(#name)
                                    .and_then(|v| v.as_bool())
                                    .ok_or_else(|| lumosai_core::Error::InvalidInput(format!("Missing or invalid parameter: {}", #name)))?;
                            },
                            _ => quote! {
                                let #param_name: #param_type = serde_json::from_value(
                                    params.get(#name).cloned().unwrap_or(serde_json::Value::Null)
                                ).map_err(|e| lumosai_core::Error::InvalidInput(format!("Invalid parameter {}: {}", #name, e)))?;
                            },
                        };
                        param_extractions.push(extraction);
                        param_names.push(param_name);
                    }
                }
            }
        }
    }

    // Generate the ToolSchema
    let schema_def = quote! {
        ToolSchema {
            parameters: vec![
                #(#parameters),*
            ],
            json_schema: None,
            format: SchemaFormat::Parameters,
            output_schema: None,
        }
    };

    // Create a renamed version of the original function with parameter attributes removed
    let impl_fn_name = syn::Ident::new(&format!("{}_impl", fn_name), fn_name.span());
    let mut impl_input = input.clone();
    impl_input.sig.ident = impl_fn_name.clone();

    // Remove parameter attributes from the implementation function
    for param in impl_input.sig.inputs.iter_mut() {
        if let syn::FnArg::Typed(pat_type) = param {
            pat_type
                .attrs
                .retain(|attr| !attr.path().is_ident("parameter"));
        }
    }

    // Generate the FunctionTool implementation
    let expanded = quote! {
        // Preserve the original function with a renamed identifier
        #impl_input

        // Import necessary types (but avoid conflicts)
        #[allow(unused_imports)]
        use lumosai_core::tool::{FunctionTool, ParameterSchema, SchemaFormat, ToolSchema};
        #[allow(unused_imports)]
        use lumosai_core::Error;
        #[allow(unused_imports)]
        use serde_json::Value;

        // Create the tool factory function
        pub fn #fn_name() -> Box<dyn lumosai_core::tool::Tool> {
            Box::new(FunctionTool::new(
                #tool_name.to_string(),
                #tool_description.to_string(),
                #schema_def,
                |params: Value| -> lumosai_core::Result<Value> {
                    // 将 Value 转换为对象以便访问参数
                    let params = params.as_object()
                        .ok_or_else(|| lumosai_core::Error::InvalidInput("Parameters must be a JSON object".to_string()))?;

                    #(#param_extractions)*

                    // 调用重命名的原始函数
                    let result = #impl_fn_name(#(#param_names),*)?;

                    // 将结果转换为 Value
                    Ok(result)
                },
            ))
        }
    };

    TokenStream::from(expanded)
}

// 基于nom的tool!宏实现
pub fn tool_nom_macro(input: TokenStream) -> TokenStream {
    // 将TokenStream转换为字符串
    let input_str = input.to_string();

    // 使用nom解析器解析输入
    let tool_def = match parse_tool_macro(&input_str) {
        Ok(def) => def,
        Err(e) => {
            let error_msg = format!("Failed to parse tool macro: {}", e);
            return quote! {
                compile_error!(#error_msg);
            }
            .into();
        }
    };

    // 生成代码
    generate_tool_code(tool_def)
}

/// 生成tool代码
fn generate_tool_code(tool_def: ToolDef) -> TokenStream {
    let tool_name = &tool_def.name;
    let tool_description = &tool_def.description;
    let handler_expr = &tool_def.handler;

    // 解析handler表达式
    let handler_tokens: proc_macro2::TokenStream = match handler_expr.parse() {
        Ok(tokens) => tokens,
        Err(e) => {
            let error_msg = format!("Invalid handler expression: {}", e);
            return quote! {
                compile_error!(#error_msg);
            }
            .into();
        }
    };

    // 生成参数schema
    let parameters: Vec<proc_macro2::TokenStream> = tool_def
        .parameters
        .iter()
        .map(|param| {
            let name = &param.name;
            let description = &param.description;
            let param_type = &param.param_type;
            let required = param.required;

            quote! {
                ParameterSchema {
                    name: #name.to_string(),
                    description: #description.to_string(),
                    r#type: #param_type.to_string(),
                    required: #required,
                    properties: None,
                    default: None,
                }
            }
        })
        .collect();

    let expanded = quote! {
        pub fn #handler_tokens() -> Box<dyn Tool> {
            Box::new(FunctionTool::new(
                #tool_name.to_string(),
                #tool_description.to_string(),
                ToolSchema {
                    parameters: vec![
                        #(#parameters),*
                    ]
                },
                |params| {
                    // 这里需要实际的处理器实现
                    // 暂时返回一个占位符
                    Box::pin(async move {
                        Ok("Tool executed successfully".to_string())
                    })
                },
            ))
        }
    };

    TokenStream::from(expanded)
}

// lumos_execute_tool宏的实现
pub fn lumos_execute_tool(input: TokenStream) -> TokenStream {
    let input_parsed = parse_macro_input!(input as ToolExecuteArgs);

    let tool_name = &input_parsed.tool;
    let params = &input_parsed.params;

    let expanded = quote! {
        {
            let mut params_map = HashMap::new();
            #params
            let options = ToolExecutionOptions::default();
            let tool = #tool_name();
            tool.execute(params_map, &options).await.expect("Tool execution failed")
        }
    };

    TokenStream::from(expanded)
}
