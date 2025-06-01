use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{
    parse_macro_input, braced, Expr, ExprClosure, Ident, LitBool, LitStr, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

// 工具参数
struct ToolParameter {
    name: LitStr,
    description: LitStr,
    type_: LitStr,
    required: Option<LitBool>,
    default: Option<Expr>,
}

impl Parse for ToolParameter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let mut name = None;
        let mut description = None;
        let mut type_ = None;
        let mut required = None;
        let mut default = None;
        
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
                "type" | "r#type" => {
                    type_ = Some(content.parse()?);
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
        let type_ = type_.ok_or_else(|| syn::Error::new(content.span(), "Missing 'type' field in parameter definition"))?;
        
        Ok(ToolParameter {
            name,
            description,
            type_,
            required,
            default,
        })
    }
}

// 工具定义
struct ToolDef {
    name: LitStr,
    description: LitStr,
    parameters: Punctuated<ToolParameter, Token![,]>,
    handler: ExprClosure,
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
                "parameters" => {
                    let params_content;
                    braced!(params_content in content);
                    parameters_content = Some(params_content);
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
        
        let parameters_content = parameters_content.ok_or_else(|| 
            syn::Error::new(content.span(), "Missing 'parameters' field in tool definition")
        )?;
        
        let parameters = Punctuated::parse_terminated(&parameters_content)?;
        
        let handler = handler.ok_or_else(|| syn::Error::new(content.span(), "Missing 'handler' field in tool definition"))?;
        
        Ok(ToolDef {
            name,
            description,
            parameters,
            handler,
        })
    }
}

// 整个工具集合
struct ToolsInput {
    tools: Punctuated<ToolDef, Token![,]>,
}

impl Parse for ToolsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let tools = Punctuated::parse_terminated(input)?;
        Ok(ToolsInput { tools })
    }
}

/// 实现tools!宏
pub fn tools(input: TokenStream) -> TokenStream {
    let tools_input = parse_macro_input!(input as ToolsInput);
    
    let mut tool_fn_defs = Vec::new();
    
    for tool in tools_input.tools.iter() {
        let name = &tool.name;
        let name_str = name.value();
        let tool_fn_name = format_ident!("{}", name_str.to_lowercase().replace("-", "_"));
        
        let description = &tool.description;
        let handler = &tool.handler;
        
        // 构建参数列表
        let parameter_defs = tool.parameters.iter().map(|param| {
            let param_name = &param.name;
            let param_desc = &param.description;
            let param_type = &param.type_;
            
            let required = match &param.required {
                Some(req) => {
                    let value = req.value;
                    quote! { #value }
                },
                None => quote! { false },
            };
            
            let default = match &param.default {
                Some(def) => quote! { Some(serde_json::Value::from(#def)) },
                None => quote! { None },
            };
            
            quote! {
                lumosai_core::tool::ParameterSchema {
                    name: #param_name.to_string(),
                    description: #param_desc.to_string(),
                    r#type: #param_type.to_string(),
                    required: #required,
                    properties: None,
                    default: #default,
                }
            }
        }).collect::<Vec<_>>();
        
        let tool_def = quote! {
            pub fn #tool_fn_name() -> Box<dyn lumosai_core::tool::Tool> {
                use serde_json::json;
                use lumosai_core::tool::{FunctionTool, ParameterSchema, ToolSchema};

                let schema = ToolSchema::new(vec![
                    #(#parameter_defs),*
                ]);

                Box::new(FunctionTool::new(
                    #name.to_string(),
                    #description.to_string(),
                    schema,
                    #handler
                ))
            }
        };
        
        tool_fn_defs.push(tool_def);
    }
    
    let expanded = quote! {
        #(#tool_fn_defs)*
    };
    
    TokenStream::from(expanded)
} 