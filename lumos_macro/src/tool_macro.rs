use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use std::str::FromStr;
use syn::spanned::Spanned;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, FnArg, Ident, ItemFn, LitBool, LitStr, Pat, PatType, Token, Type,
};

use crate::parser::{parse_tool_macro, ToolDef};

/// Parameter attribute structure (local copy for tool_macro module)
#[derive(Debug, Clone)]
struct ParameterAttributes {
    name: LitStr,
    description: LitStr,
    type_: LitStr,
    required: bool,
}

impl Parse for ParameterAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut description = None;
        let mut type_ = None;
        let mut required = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            if ident == "name" {
                name = Some(input.parse()?);
            } else if ident == "description" {
                description = Some(input.parse()?);
            } else if ident == "r#type" || ident == "type" {
                type_ = Some(input.parse()?);
            } else if ident == "required" {
                let expr: Expr = input.parse()?;
                if let Expr::Lit(lit) = &expr {
                    if let syn::Lit::Bool(b) = &lit.lit {
                        required = Some(b.value);
                    }
                }
            } else {
                return Err(syn::Error::new(ident.span(), "Unknown parameter attribute"));
            }

            // Allow trailing comma
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        let name = name.ok_or_else(|| syn::Error::new(input.span(), "Missing name attribute"))?;
        let description = description
            .ok_or_else(|| syn::Error::new(input.span(), "Missing description attribute"))?;
        let type_ = type_.ok_or_else(|| syn::Error::new(input.span(), "Missing type attribute"))?;
        let required = required.unwrap_or(false);

        Ok(ParameterAttributes {
            name,
            description,
            type_,
            required,
        })
    }
}

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
                        proc_macro2::TokenStream::from_str(&format!("{{{inner_content_str}}}"))
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
                                            format!("Failed to parse parameter attributes: {e}"),
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
    let impl_fn_name = syn::Ident::new(&format!("{fn_name}_impl"), fn_name.span());
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
        pub fn #fn_name() -> Box<dyn crate::tool::Tool> {
            Box::new(crate::tool::FunctionTool::new(
                #tool_name.to_string(),
                #tool_description.to_string(),
                #schema_def,
                |params: serde_json::Value| -> crate::Result<serde_json::Value> {
                    // 将 Value 转换为对象以便访问参数
                    let params = params.as_object()
                        .ok_or_else(|| crate::Error::InvalidInput("Parameters must be a JSON object".to_string()))?;

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
            let error_msg = format!("Failed to parse tool macro: {e}");
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
            let error_msg = format!("Invalid handler expression: {e}");
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

/// 新的改进版 #[tool] 宏实现
///
/// 这个宏可以应用到函数上，自动生成 Tool trait 实现
///
/// # 示例
///
/// ```rust
/// #[tool(name = "calculator", description = "执行数学计算")]
/// async fn calculate(expression: String, precision: Option<u32>) -> Result<f64> {
///     // 实现计算逻辑
/// }
/// ```
pub fn tool_attribute_macro(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 解析属性
    let config = if attr.is_empty() {
        ToolConfig::default()
    } else {
        match syn::parse::<ToolConfig>(attr) {
            Ok(config) => config,
            Err(e) => {
                let error_msg = e.to_string();
                return quote! {
                    compile_error!(#error_msg);
                }
                .into();
            }
        }
    };

    // 解析函数
    let fn_item = match syn::parse::<ItemFn>(item) {
        Ok(fn_item) => fn_item,
        Err(e) => {
            let error_msg = e.to_string();
            return quote! {
                compile_error!(#error_msg);
            }
            .into();
        }
    };

    // 提取参数信息
    let params = match extract_parameters(&fn_item) {
        Ok(params) => params,
        Err(e) => {
            let error_msg = e.to_string();
            return quote! {
                compile_error!(#error_msg);
            }
            .into();
        }
    };

    // 生成工具实现
    match generate_tool_impl(config, fn_item, params) {
        Ok(tokens) => tokens.into(),
        Err(e) => {
            let error_msg = e.to_string();
            quote! {
                compile_error!(#error_msg);
            }
            .into()
        }
    }
}

/// 工具配置结构体
#[derive(Debug, Clone)]
pub struct ToolConfig {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub validate_params: bool,
    pub validate_output: bool,
    pub examples: Vec<String>,
    pub tags: Vec<String>,
    pub version: Option<String>,
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            name: None,
            description: None,
            category: None,
            validate_params: true,
            validate_output: false,
            examples: Vec::new(),
            tags: Vec::new(),
            version: None,
        }
    }
}

impl Parse for ToolConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut config = ToolConfig::default();

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;

            match key.to_string().as_str() {
                "name" => {
                    let name: LitStr = input.parse()?;
                    config.name = Some(name.value());
                }
                "description" => {
                    let desc: LitStr = input.parse()?;
                    config.description = Some(desc.value());
                }
                "category" => {
                    let cat: LitStr = input.parse()?;
                    config.category = Some(cat.value());
                }
                "validate_params" => {
                    let val: syn::LitBool = input.parse()?;
                    config.validate_params = val.value;
                }
                "validate_output" => {
                    let val: syn::LitBool = input.parse()?;
                    config.validate_output = val.value;
                }
                "examples" => {
                    // Parse array of strings: examples = ["example1", "example2"]
                    let content;
                    syn::bracketed!(content in input);
                    let examples: syn::punctuated::Punctuated<LitStr, Token![,]> =
                        content.parse_terminated(|input| input.parse::<LitStr>(), Token![,])?;
                    config.examples = examples.iter().map(|s| s.value()).collect();
                }
                "tags" => {
                    // Parse array of strings: tags = ["tag1", "tag2"]
                    let content;
                    syn::bracketed!(content in input);
                    let tags: syn::punctuated::Punctuated<LitStr, Token![,]> =
                        content.parse_terminated(|input| input.parse::<LitStr>(), Token![,])?;
                    config.tags = tags.iter().map(|s| s.value()).collect();
                }
                "version" => {
                    let version: LitStr = input.parse()?;
                    config.version = Some(version.value());
                }
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("Unknown tool attribute: {key}"),
                    ));
                }
            }

            let _: Option<Token![,]> = input.parse()?;
        }

        Ok(config)
    }
}

/// 参数验证器类型
#[derive(Debug, Clone)]
pub enum ParameterValidator {
    None,
    Url,
    Email,
    Range(f64, f64),
    MinLength(usize),
    MaxLength(usize),
    Regex(String),
    NonEmpty,
    Custom(String),
}

/// 参数信息
#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub name: String,
    pub rust_type: Type,
    pub description: Option<String>,
    pub required: bool,
    pub validator: ParameterValidator,
    pub default_value: Option<String>,
    pub examples: Vec<String>,
}

/// 从函数签名提取参数信息（支持 #[parameter] 属性）
pub fn extract_parameters(fn_item: &ItemFn) -> syn::Result<Vec<ParameterInfo>> {
    let mut params = Vec::new();

    for input in &fn_item.sig.inputs {
        match input {
            FnArg::Typed(pat_type) => {
                if let Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                    let param_name = pat_ident.ident.to_string();
                    let (rust_type, auto_required) = analyze_type(&pat_type.ty);

                    // 尝试从 #[parameter] 属性中提取信息
                    let param_attr = extract_parameter_attribute(&pat_type.attrs)?;

                    let name = param_attr
                        .as_ref()
                        .and_then(|attr| Some(attr.name.value()))
                        .unwrap_or(param_name.clone());

                    let description = param_attr
                        .as_ref()
                        .and_then(|attr| Some(attr.description.value()))
                        .or_else(|| extract_param_description(&fn_item.attrs, &param_name));

                    let required = param_attr
                        .as_ref()
                        .map(|attr| attr.required)
                        .unwrap_or(auto_required);

                    params.push(ParameterInfo {
                        name,
                        rust_type,
                        description,
                        required,
                        validator: ParameterValidator::None,
                        default_value: None,
                        examples: Vec::new(),
                    });
                }
            }
            FnArg::Receiver(_) => {
                return Err(syn::Error::new_spanned(
                    input,
                    "Tool functions cannot have self parameters",
                ));
            }
        }
    }

    Ok(params)
}

/// 从参数的属性中提取 #[parameter(...)] 信息
fn extract_parameter_attribute(
    attrs: &[syn::Attribute],
) -> syn::Result<Option<ParameterAttributes>> {
    for attr in attrs {
        if attr.path().is_ident("parameter") {
            // 解析 #[parameter(...)] 中的内容
            let param_attrs: ParameterAttributes = attr.parse_args()?;
            return Ok(Some(param_attrs));
        }
    }
    Ok(None)
}

/// 分析类型，判断是否为 Option 类型
fn analyze_type(ty: &Type) -> (Type, bool) {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                        return (inner_type.clone(), false);
                    }
                }
            }
        }
    }
    (ty.clone(), true)
}

/// 从属性中提取参数描述
/// 支持多种格式:
/// 1. `@param param_name: description` - 标准格式
/// 2. `@param param_name - description` - 简化格式
/// 3. `参数: param_name - description` - 中文格式
/// 4. 包含参数名的普通文档注释
fn extract_param_description(attrs: &[syn::Attribute], param_name: &str) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Ok(meta) = attr.meta.require_name_value() {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = &meta.value
                {
                    let doc = lit_str.value().trim().to_string();

                    // 格式 1: @param param_name: description
                    if doc.starts_with("@param ") || doc.starts_with("@parameter ") {
                        if let Some(desc) = parse_param_doc(&doc, param_name) {
                            return Some(desc);
                        }
                    }

                    // 格式 2: 参数: param_name - description (中文)
                    if doc.starts_with("参数:") || doc.starts_with("参数：") {
                        if let Some(desc) = parse_chinese_param_doc(&doc, param_name) {
                            return Some(desc);
                        }
                    }

                    // 格式 3: 包含参数名的普通文档注释（回退方案）
                    if doc.to_lowercase().contains(&param_name.to_lowercase()) {
                        return Some(doc);
                    }
                }
            }
        }
    }
    None
}

/// 解析 @param 格式的文档注释
/// 格式: @param param_name: description
/// 或: @param param_name - description
fn parse_param_doc(doc: &str, param_name: &str) -> Option<String> {
    let doc = doc.trim();

    // 移除 @param 或 @parameter 前缀
    let doc = doc
        .strip_prefix("@param ")
        .or_else(|| doc.strip_prefix("@parameter "))?;

    // 检查是否匹配参数名
    if !doc.starts_with(param_name) {
        return None;
    }

    // 移除参数名
    let doc = doc.strip_prefix(param_name)?.trim();

    // 移除分隔符 (: 或 -)
    let doc = doc
        .strip_prefix(':')
        .or_else(|| doc.strip_prefix('-'))?
        .trim();

    Some(doc.to_string())
}

/// 解析中文格式的参数文档
/// 格式: 参数: param_name - description
fn parse_chinese_param_doc(doc: &str, param_name: &str) -> Option<String> {
    let doc = doc.trim();

    // 移除 "参数:" 或 "参数：" 前缀
    let doc = doc
        .strip_prefix("参数:")
        .or_else(|| doc.strip_prefix("参数："))?
        .trim();

    // 检查是否匹配参数名
    if !doc.starts_with(param_name) {
        return None;
    }

    // 移除参数名
    let doc = doc.strip_prefix(param_name)?.trim();

    // 移除分隔符 (- 或 :)
    let doc = doc
        .strip_prefix('-')
        .or_else(|| doc.strip_prefix(':'))
        .or_else(|| doc.strip_prefix('：'))?
        .trim();

    Some(doc.to_string())
}

/// 将 Rust 类型转换为 JSON Schema 类型字符串
pub fn rust_type_to_json_type(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                match segment.ident.to_string().as_str() {
                    "String" | "str" => "string".to_string(),
                    "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32"
                    | "u64" | "u128" | "usize" => "integer".to_string(),
                    "f32" | "f64" => "number".to_string(),
                    "bool" => "boolean".to_string(),
                    "Vec" => "array".to_string(),
                    "Value" => "object".to_string(), // serde_json::Value
                    "HashMap" | "Map" => "object".to_string(),
                    _ => "string".to_string(),
                }
            } else {
                "string".to_string()
            }
        }
        _ => "string".to_string(),
    }
}

/// 生成工具实现代码
pub fn generate_tool_impl(
    config: ToolConfig,
    fn_item: ItemFn,
    params: Vec<ParameterInfo>,
) -> syn::Result<proc_macro2::TokenStream> {
    let fn_name = &fn_item.sig.ident;
    let tool_name = config.name.unwrap_or_else(|| fn_name.to_string());
    let tool_description = config
        .description
        .unwrap_or_else(|| format!("Tool generated from function {fn_name}"));

    let tool_struct_name = format_ident!(
        "{}Tool",
        fn_name
            .to_string()
            .chars()
            .enumerate()
            .map(|(i, c)| if i == 0 {
                c.to_uppercase().collect::<String>()
            } else {
                c.to_string()
            })
            .collect::<String>()
    );

    // 生成参数 schema
    let param_schemas: Vec<proc_macro2::TokenStream> = params
        .iter()
        .map(|param| {
            let name = &param.name;
            let description = param.description.as_deref().unwrap_or("Parameter");
            let json_type = rust_type_to_json_type(&param.rust_type);
            let required = param.required;

            quote! {
                crate::tool::ParameterSchema {
                    name: #name.to_string(),
                    description: #description.to_string(),
                    r#type: #json_type.to_string(),
                    required: #required,
                    properties: None,
                    default: None,
                }
            }
        })
        .collect();

    // 生成参数提取代码
    let param_extractions: Vec<proc_macro2::TokenStream> = params.iter().map(|param| {
        let param_name = format_ident!("{}", param.name);
        let param_key = &param.name;
        let rust_type = &param.rust_type;
        let json_type = rust_type_to_json_type(&param.rust_type);

        // 根据类型生成不同的提取代码
        if param.required {
            match json_type.as_str() {
                "object" | "array" => {
                    // Value 类型直接克隆
                    quote! {
                        let #param_name: #rust_type = params.get(#param_key)
                            .ok_or_else(|| crate::error::Error::Tool(format!("Missing required parameter: {}", #param_key)))?
                            .clone();
                    }
                }
                "string" => {
                    quote! {
                        let #param_name: #rust_type = params.get(#param_key)
                            .ok_or_else(|| crate::error::Error::Tool(format!("Missing required parameter: {}", #param_key)))?
                            .as_str()
                            .ok_or_else(|| crate::error::Error::Tool(format!("Parameter {} must be a string", #param_key)))?
                            .to_string();
                    }
                }
                "integer" => {
                    quote! {
                        let #param_name: #rust_type = params.get(#param_key)
                            .ok_or_else(|| crate::error::Error::Tool(format!("Missing required parameter: {}", #param_key)))?
                            .as_i64()
                            .ok_or_else(|| crate::error::Error::Tool(format!("Parameter {} must be an integer", #param_key)))? as #rust_type;
                    }
                }
                "number" => {
                    quote! {
                        let #param_name: #rust_type = params.get(#param_key)
                            .ok_or_else(|| crate::error::Error::Tool(format!("Missing required parameter: {}", #param_key)))?
                            .as_f64()
                            .ok_or_else(|| crate::error::Error::Tool(format!("Parameter {} must be a number", #param_key)))? as #rust_type;
                    }
                }
                "boolean" => {
                    quote! {
                        let #param_name: #rust_type = params.get(#param_key)
                            .ok_or_else(|| crate::error::Error::Tool(format!("Missing required parameter: {}", #param_key)))?
                            .as_bool()
                            .ok_or_else(|| crate::error::Error::Tool(format!("Parameter {} must be a boolean", #param_key)))?;
                    }
                }
                _ => {
                    // 默认使用字符串解析
                    quote! {
                        let #param_name: #rust_type = params.get(#param_key)
                            .ok_or_else(|| crate::error::Error::Tool(format!("Missing required parameter: {}", #param_key)))?
                            .as_str()
                            .ok_or_else(|| crate::error::Error::Tool(format!("Parameter {} must be a string", #param_key)))?
                            .parse()
                            .map_err(|_| crate::error::Error::Tool(format!("Failed to parse parameter: {}", #param_key)))?;
                    }
                }
            }
        } else {
            match json_type.as_str() {
                "object" | "array" => {
                    // Option<Value> 类型直接克隆
                    quote! {
                        let #param_name: Option<#rust_type> = params.get(#param_key).cloned();
                    }
                }
                "string" => {
                    quote! {
                        let #param_name: Option<#rust_type> = params.get(#param_key)
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                    }
                }
                "integer" => {
                    quote! {
                        let #param_name: Option<#rust_type> = params.get(#param_key)
                            .and_then(|v| v.as_i64())
                            .map(|n| n as #rust_type);
                    }
                }
                "number" => {
                    quote! {
                        let #param_name: Option<#rust_type> = params.get(#param_key)
                            .and_then(|v| v.as_f64())
                            .map(|n| n as #rust_type);
                    }
                }
                "boolean" => {
                    quote! {
                        let #param_name: Option<#rust_type> = params.get(#param_key)
                            .and_then(|v| v.as_bool());
                    }
                }
                _ => {
                    // 默认使用字符串解析
                    quote! {
                        let #param_name: Option<#rust_type> = params.get(#param_key)
                            .and_then(|v| v.as_str())
                            .map(|s| s.parse().ok())
                            .flatten();
                    }
                }
            }
        }
    }).collect();

    let param_names: Vec<proc_macro2::Ident> =
        params.iter().map(|p| format_ident!("{}", p.name)).collect();
    let tool_fn_name = format_ident!("{}_tool", fn_name);

    // 检查函数是否是异步的
    let is_async = fn_item.sig.asyncness.is_some();
    let call_expr = if is_async {
        quote! { #fn_name(#(#param_names),*).await }
    } else {
        quote! { #fn_name(#(#param_names),*) }
    };

    Ok(quote! {
        #fn_item

        #[derive(Debug, Clone)]
        pub struct #tool_struct_name {
            base: crate::base::BaseComponent,
        }

        impl #tool_struct_name {
            pub fn new() -> Self {
                Self {
                    base: crate::base::BaseComponent::new_with_name(
                        #tool_name,
                        crate::compat::Component::Tool,
                    ),
                }
            }
        }

        impl crate::base::Base for #tool_struct_name {
            fn name(&self) -> Option<&str> {
                self.base.name()
            }

            fn component(&self) -> crate::compat::Component {
                self.base.component()
            }

            fn logger(&self) -> std::sync::Arc<dyn crate::logger::Logger> {
                self.base.logger()
            }

            fn set_logger(&mut self, logger: std::sync::Arc<dyn crate::logger::Logger>) {
                self.base.set_logger(logger);
            }

            fn telemetry(&self) -> Option<std::sync::Arc<dyn crate::telemetry::TelemetrySink>> {
                self.base.telemetry()
            }

            fn set_telemetry(&mut self, telemetry: std::sync::Arc<dyn crate::telemetry::TelemetrySink>) {
                self.base.set_telemetry(telemetry);
            }
        }

        #[async_trait::async_trait]
        impl crate::tool::Tool for #tool_struct_name {
            fn id(&self) -> &str {
                #tool_name
            }

            fn description(&self) -> &str {
                #tool_description
            }

            fn schema(&self) -> crate::tool::ToolSchema {
                crate::tool::ToolSchema {
                    parameters: vec![
                        #(#param_schemas),*
                    ],
                    ..Default::default()
                }
            }

            async fn execute(
                &self,
                params: serde_json::Value,
                _context: crate::tool::ToolExecutionContext,
                _options: &crate::tool::ToolExecutionOptions,
            ) -> crate::error::Result<serde_json::Value> {
                let params = params.as_object()
                    .ok_or_else(|| crate::error::Error::Tool("Parameters must be a JSON object".to_string()))?;

                #(#param_extractions)*

                let result = #call_expr?;

                Ok(serde_json::to_value(result)
                    .map_err(|e| crate::error::Error::Tool(format!("Failed to serialize result: {}", e)))?)
            }

            fn clone_box(&self) -> Box<dyn crate::tool::Tool> {
                Box::new(self.clone())
            }
        }

        pub fn #tool_fn_name() -> Box<dyn crate::tool::Tool> {
            Box::new(#tool_struct_name::new())
        }
    })
}
