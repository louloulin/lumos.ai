//! Procedural macros for LumosAI
//! 
//! This crate provides derive macros to automatically generate function schemas
//! for use with OpenAI function calling from Rust structs.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Type, Attribute, Meta};
use std::collections::HashMap;

/// Derive macro to automatically generate OpenAI function schema from a Rust struct
/// 
/// This macro generates a `function_definition()` method that returns a `FunctionDefinition`
/// compatible with OpenAI's function calling API.
/// 
/// # Example
/// 
/// ```rust
/// use lumosai_derive::FunctionSchema;
/// use serde::{Serialize, Deserialize};
/// 
/// #[derive(Serialize, Deserialize, FunctionSchema)]
/// #[function(
///     name = "calculate",
///     description = "Performs mathematical calculations"
/// )]
/// pub struct CalculatorParams {
///     /// The mathematical expression to evaluate
///     pub expression: String,
///     /// Number of decimal places for precision (optional)
///     pub precision: Option<u32>,
/// }
/// ```
#[proc_macro_derive(FunctionSchema, attributes(function, field))]
pub fn derive_function_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    match generate_function_schema(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn generate_function_schema(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    
    // Parse function-level attributes
    let (function_name, function_description) = parse_function_attributes(&input.attrs)?;
    
    // Extract field information
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => return Err(syn::Error::new_spanned(input, "Only structs with named fields are supported")),
        },
        _ => return Err(syn::Error::new_spanned(input, "Only structs are supported")),
    };
    
    // Generate JSON schema for each field
    let mut properties = HashMap::new();
    let mut required_fields = Vec::new();
    
    for field in fields {
        let field_name = field.ident.as_ref().unwrap().to_string();
        let field_type = &field.ty;
        
        // Parse field attributes
        let field_description = parse_field_description(&field.attrs);
        
        // Determine if field is optional
        let is_optional = is_option_type(field_type);
        if !is_optional {
            required_fields.push(field_name.clone());
        }
        
        // Generate property schema
        let type_info = get_type_info(field_type, is_optional);
        properties.insert(field_name.clone(), (type_info, field_description));
    }
    
    // Generate the implementation
    let properties_code = generate_properties_code(&properties);
    let required_code = generate_required_code(&required_fields);
    
    let expanded = quote! {
        impl #name {
            /// Generate OpenAI function definition for this struct
            pub fn function_definition() -> ::lumosai_core::llm::function_calling::FunctionDefinition {
                use ::serde_json::{json, Value, Map};
                use ::lumosai_core::llm::function_calling::FunctionDefinition;
                
                let mut properties = Map::new();
                #properties_code
                
                let parameters = json!({
                    "type": "object",
                    "properties": properties,
                    "required": [#required_code]
                });
                
                FunctionDefinition::new(
                    #function_name.to_string(),
                    Some(#function_description.to_string()),
                    parameters
                )
            }
        }
        
        impl ::lumosai_core::tool::FunctionSchema for #name {
            fn function_definition() -> ::lumosai_core::llm::function_calling::FunctionDefinition {
                Self::function_definition()
            }
        }
    };
    
    Ok(expanded)
}

fn parse_function_attributes(attrs: &[Attribute]) -> syn::Result<(String, String)> {
    let mut function_name = None;
    let mut function_description = None;
    
    for attr in attrs {
        if attr.path().is_ident("function") {
            match &attr.meta {
                Meta::List(list) => {
                    // Parse name and description from attribute arguments
                    let nested = list.parse_args_with(syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated)?;
                    for meta in nested {
                        match meta {
                            Meta::NameValue(nv) if nv.path.is_ident("name") => {
                                if let syn::Expr::Lit(lit) = &nv.value {
                                    if let syn::Lit::Str(lit_str) = &lit.lit {
                                        function_name = Some(lit_str.value());
                                    }
                                }
                            },
                            Meta::NameValue(nv) if nv.path.is_ident("description") => {
                                if let syn::Expr::Lit(lit) = &nv.value {
                                    if let syn::Lit::Str(lit_str) = &lit.lit {
                                        function_description = Some(lit_str.value());
                                    }
                                }
                            },
                            _ => {},
                        }
                    }
                },
                _ => {},
            }
        }
    }
    
    // Default values if not specified
    let name = function_name.unwrap_or_else(|| "unnamed_function".to_string());
    let description = function_description.unwrap_or_else(|| "No description provided".to_string());
    
    Ok((name, description))
}

fn parse_field_description(attrs: &[Attribute]) -> String {
    // First check for #[field(description = "...")] attribute
    for attr in attrs {
        if attr.path().is_ident("field") {
            if let Meta::List(list) = &attr.meta {
                if let Ok(nested) = list.parse_args_with(syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated) {
                    for meta in nested {
                        if let Meta::NameValue(nv) = meta {
                            if nv.path.is_ident("description") {
                                if let syn::Expr::Lit(lit) = &nv.value {
                                    if let syn::Lit::Str(lit_str) = &lit.lit {
                                        return lit_str.value();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Fallback to doc comments
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Meta::NameValue(nv) = &attr.meta {
                if let syn::Expr::Lit(lit) = &nv.value {
                    if let syn::Lit::Str(lit_str) = &lit.lit {
                        let doc = lit_str.value();
                        let trimmed = doc.trim();
                        if !trimmed.is_empty() {
                            return trimmed.to_string();
                        }
                    }
                }
            }
        }
    }
    
    "No description".to_string()
}

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

fn get_type_info(ty: &Type, is_optional: bool) -> (&'static str, &'static str) {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let type_name = segment.ident.to_string();
            
            match type_name.as_str() {
                "String" | "str" => ("string", "string"),
                "i8" | "i16" | "i32" | "i64" | "isize" => ("integer", "integer"),
                "u8" | "u16" | "u32" | "u64" | "usize" => ("integer", "integer"),
                "f32" | "f64" => ("number", "number"),
                "bool" => ("boolean", "boolean"),
                "Vec" => ("array", "array"),
                "HashMap" | "Map" => ("object", "object"),
                "Option" => {
                    // Extract the inner type for Option<T>
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            return get_type_info(inner_ty, true);
                        }
                    }
                    ("string", "string")
                },
                _ => ("string", "string"), // Default to string for unknown types
            }
        } else {
            ("string", "string")
        }
    } else {
        ("string", "string")
    }
}

fn generate_properties_code(properties: &HashMap<String, ((&'static str, &'static str), String)>) -> proc_macro2::TokenStream {
    let mut code = proc_macro2::TokenStream::new();
    
    for (field_name, ((json_type, _), description)) in properties {
        let field_name_lit = field_name;
        let json_type_lit = json_type;
        let description_lit = description;
        
        let property_code = quote! {
            properties.insert(#field_name_lit.to_string(), json!({
                "type": #json_type_lit,
                "description": #description_lit
            }));
        };
        
        code.extend(property_code);
    }
    
    code
}

fn generate_required_code(required_fields: &[String]) -> proc_macro2::TokenStream {
    let mut code = proc_macro2::TokenStream::new();
    
    for field in required_fields {
        let field_lit = field;
        let field_code = quote! { #field_lit, };
        code.extend(field_code);
    }
    
    code
}
