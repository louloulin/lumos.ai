use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub fn llm_adapter_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    // Generate the LlmProvider implementation
    let expanded = quote! {
        #[async_trait::async_trait]
        impl lumosai_core::llm::LlmProvider for #name {
            async fn generate(&self, prompt: &str, options: &lumosai_core::llm::LlmOptions) -> lumosai_core::Result<String> {
                // Default implementation delegates to generate_with_messages
                let message = lumosai_core::Message {
                    role: lumosai_core::Role::User,
                    content: Some(prompt.to_string()),
                    name: None,
                    metadata: std::collections::HashMap::new(),
                };
                self.generate_with_messages(&[message], options).await
            }
            
            async fn generate_with_messages(&self, messages: &[lumosai_core::Message], options: &lumosai_core::llm::LlmOptions) -> lumosai_core::Result<String> {
                // This needs to be implemented by the concrete type
                unimplemented!("generate_with_messages not implemented")
            }
            
            async fn generate_stream<'a>(
                &'a self,
                prompt: &'a str,
                options: &'a lumosai_core::llm::LlmOptions,
            ) -> lumosai_core::Result<futures::stream::BoxStream<'a, lumosai_core::Result<String>>> {
                // Default implementation - concrete types should override this
                unimplemented!("Streaming not implemented")
            }
            
            async fn get_embedding(&self, text: &str) -> lumosai_core::Result<Vec<f32>> {
                // Default implementation - concrete types should override this
                unimplemented!("Embeddings not implemented")
            }
        }
    };
    
    TokenStream::from(expanded)
} 