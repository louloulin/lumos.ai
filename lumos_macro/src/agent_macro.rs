use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput, Ident, LitStr, Token, parse::{Parse, ParseStream}};
use syn::{Data, DataStruct, Fields};

// 代理属性解析
pub struct AgentAttributes {
    pub name: LitStr,
    pub instructions: LitStr,
    pub model: LitStr,
}

impl Parse for AgentAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = syn::parenthesized!(content in input);
        
        let mut name = None;
        let mut instructions = None;
        let mut model = None;
        
        while !content.is_empty() {
            let key: Ident = content.parse()?;
            let _: Token![:] = content.parse()?;
            
            match key.to_string().as_str() {
                "name" => {
                    name = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "instructions" => {
                    instructions = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "model" => {
                    model = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new_spanned(&key, "Unknown attribute in agent definition")),
            }
        }
        
        let name = name.ok_or_else(|| syn::Error::new_spanned(&content, "Missing 'name' attribute in agent definition"))?;
        let instructions = instructions.ok_or_else(|| syn::Error::new_spanned(&content, "Missing 'instructions' attribute in agent definition"))?;
        let model = model.ok_or_else(|| syn::Error::new_spanned(&content, "Missing 'model' attribute in agent definition"))?;
        
        Ok(AgentAttributes {
            name,
            instructions,
            model,
        })
    }
}

pub fn agent_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let attrs = parse_macro_input!(attr as AgentAttributes);
    
    let struct_name = &input.ident;
    let agent_name = attrs.name.value();
    let instructions = attrs.instructions.value();
    let model = attrs.model.value(); 
    
    // Extract tool fields
    let mut tools = Vec::new();
    
    if let Data::Struct(DataStruct { fields: Fields::Named(named_fields), .. }) = &input.data {
        for field in &named_fields.named {
            for attr in &field.attrs {
                if attr.path().is_ident("tool") {
                    let field_name = &field.ident;
                    tools.push(quote! {
                        #field_name: #field_name()
                    });
                }
            }
        }
    }
    
    // Generate agent creation function
    let agent_fn_name = format_ident!("create_{}", struct_name.to_string().to_lowercase());
    
    let expanded = quote! {
        pub fn #agent_fn_name(llm_provider: std::sync::Arc<dyn lomusai_core::llm::LlmProvider>) -> impl lomusai_core::agent::Agent {
            let config = lomusai_core::agent::AgentConfig {
                name: #agent_name.to_string(),
                instructions: #instructions.to_string(),
                memory_config: None,
            };
            
            let mut agent = lomusai_core::agent::create_basic_agent(config, llm_provider);
            
            // Add tools
            #(agent.add_tool(#tools).expect("Failed to add tool to agent");)*
            
            agent
        }
    };
    
    TokenStream::from(expanded)
} 