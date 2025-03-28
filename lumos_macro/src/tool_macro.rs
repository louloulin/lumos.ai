use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Expr, Ident, LitStr, Token, parse::{Parse, ParseStream}};

// 工具属性解析
pub struct ToolAttributes {
    pub name: LitStr,
    pub description: LitStr,
}

impl Parse for ToolAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = syn::parenthesized!(content in input);
        
        let mut name = None;
        let mut description = None;
        
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
                _ => return Err(syn::Error::new(key.span(), "Unknown attribute in tool definition")),
            }
        }
        
        let name = name.ok_or_else(|| syn::Error::new(content.span(), "Missing 'name' attribute in tool definition"))?;
        let description = description.ok_or_else(|| syn::Error::new(content.span(), "Missing 'description' attribute in tool definition"))?;
        
        Ok(ToolAttributes {
            name,
            description,
        })
    }
}

// 参数属性解析
pub struct ParameterAttributes {
    pub name: LitStr,
    pub description: LitStr,
    pub type_: LitStr,
    pub required: bool,
}

impl Parse for ParameterAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = syn::parenthesized!(content in input);
        
        let mut name = None;
        let mut description = None;
        let mut type_ = None;
        let mut required = None;
        
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
                "r#type" | "type" => {
                    type_ = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "required" => {
                    let expr: Expr = content.parse()?;
                    if let Expr::Lit(syn::ExprLit { lit: syn::Lit::Bool(b), .. }) = expr {
                        required = Some(b.value);
                    } else {
                        return Err(syn::Error::new(expr.span(), "Expected boolean literal for 'required' attribute"));
                    }
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new(key.span(), "Unknown attribute in parameter definition")),
            }
        }
        
        let name = name.ok_or_else(|| syn::Error::new(content.span(), "Missing 'name' attribute in parameter definition"))?;
        let description = description.ok_or_else(|| syn::Error::new(content.span(), "Missing 'description' attribute in parameter definition"))?;
        let type_ = type_.ok_or_else(|| syn::Error::new(content.span(), "Missing 'type' attribute in parameter definition"))?;
        let required = required.unwrap_or(false);
        
        Ok(ParameterAttributes {
            name,
            description,
            type_,
            required,
        })
    }
}

pub fn tool_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let attrs = parse_macro_input!(attr as ToolAttributes);
    
    let fn_name = &input.sig.ident;
    let fn_params = &input.sig.inputs;
    let fn_body = &input.block;
    let fn_output = &input.sig.output;
    
    let tool_name = attrs.name.value();
    let tool_description = attrs.description.value();
    
    // Extract parameter metadata from attributes
    let mut parameters = Vec::new();
    for param in fn_params.iter() {
        if let syn::FnArg::Typed(pat_type) = param {
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                let param_name = &pat_ident.ident;
                
                // Find parameter attributes
                for attr in &pat_ident.attrs {
                    if attr.path().is_ident("parameter") {
                        let param_attr = syn::parse2::<ParameterAttributes>(attr.meta.require_list().unwrap().tokens.clone()).unwrap();
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
            ]
        }
    };
    
    // Generate the FunctionTool implementation
    let expanded = quote! {
        pub fn #fn_name() -> Box<dyn Tool> {
            Box::new(FunctionTool::new(
                #tool_name.to_string(),
                #tool_description.to_string(),
                #schema_def,
                |params| {
                    // The actual implementation function
                    #fn_body
                },
            ))
        }
    };
    
    TokenStream::from(expanded)
} 