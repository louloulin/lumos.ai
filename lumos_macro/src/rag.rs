use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, LitStr, Expr, Token, braced, parse::{Parse, ParseStream}};

// RAG管道的Chunk选项
struct ChunkOptions {
    chunk_size: Expr,
    chunk_overlap: Option<Expr>,
    separator: Option<LitStr>,
    strategy: Option<LitStr>,
}

impl Parse for ChunkOptions {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let mut chunk_size = None;
        let mut chunk_overlap = None;
        let mut separator = None;
        let mut strategy = None;
        
        while !content.is_empty() {
            let key: syn::Ident = content.parse()?;
            let _: Token![:] = content.parse()?;
            
            match key.to_string().as_str() {
                "chunk_size" => {
                    chunk_size = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "chunk_overlap" => {
                    chunk_overlap = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "separator" => {
                    separator = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "strategy" => {
                    strategy = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new(key.span(), "Unknown field in chunk options")),
            }
        }
        
        let chunk_size = chunk_size.ok_or_else(|| syn::Error::new(content.span(), "Missing 'chunk_size' field in chunk options"))?;
        
        Ok(ChunkOptions {
            chunk_size,
            chunk_overlap,
            separator,
            strategy,
        })
    }
}

// RAG管道的Embed选项
struct EmbedOptions {
    model: LitStr,
    dimensions: Option<Expr>,
    max_retries: Option<Expr>,
}

impl Parse for EmbedOptions {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let mut model = None;
        let mut dimensions = None;
        let mut max_retries = None;
        
        while !content.is_empty() {
            let key: syn::Ident = content.parse()?;
            let _: Token![:] = content.parse()?;
            
            match key.to_string().as_str() {
                "model" => {
                    model = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "dimensions" => {
                    dimensions = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "max_retries" => {
                    max_retries = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new(key.span(), "Unknown field in embed options")),
            }
        }
        
        let model = model.ok_or_else(|| syn::Error::new(content.span(), "Missing 'model' field in embed options"))?;
        
        Ok(EmbedOptions {
            model,
            dimensions,
            max_retries,
        })
    }
}

// RAG管道的Store选项
struct StoreOptions {
    db: LitStr,
    collection: LitStr,
    connection_string: Option<Expr>,
}

impl Parse for StoreOptions {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let mut db = None;
        let mut collection = None;
        let mut connection_string = None;
        
        while !content.is_empty() {
            let key: syn::Ident = content.parse()?;
            let _: Token![:] = content.parse()?;
            
            match key.to_string().as_str() {
                "db" => {
                    db = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "collection" => {
                    collection = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "connection_string" => {
                    connection_string = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new(key.span(), "Unknown field in store options")),
            }
        }
        
        let db = db.ok_or_else(|| syn::Error::new(content.span(), "Missing 'db' field in store options"))?;
        let collection = collection.ok_or_else(|| syn::Error::new(content.span(), "Missing 'collection' field in store options"))?;
        
        Ok(StoreOptions {
            db,
            collection,
            connection_string,
        })
    }
}

// RAG管道的查询选项
struct QueryOptions {
    rerank: Option<Expr>,
    top_k: Option<Expr>,
    filter: Option<LitStr>,
}

impl Parse for QueryOptions {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let mut rerank = None;
        let mut top_k = None;
        let mut filter = None;
        
        while !content.is_empty() {
            let key: syn::Ident = content.parse()?;
            let _: Token![:] = content.parse()?;
            
            match key.to_string().as_str() {
                "rerank" => {
                    rerank = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "top_k" => {
                    top_k = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "filter" => {
                    filter = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new(key.span(), "Unknown field in query options")),
            }
        }
        
        Ok(QueryOptions {
            rerank,
            top_k,
            filter,
        })
    }
}

// RAG Pipeline的Pipeline部分
struct PipelineOptions {
    chunk: ChunkOptions,
    embed: EmbedOptions,
    store: StoreOptions,
}

impl Parse for PipelineOptions {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let mut chunk = None;
        let mut embed = None;
        let mut store = None;
        
        while !content.is_empty() {
            let key: syn::Ident = content.parse()?;
            let _: Token![:] = content.parse()?;
            
            match key.to_string().as_str() {
                "chunk" => {
                    chunk = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "embed" => {
                    embed = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "store" => {
                    store = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new(key.span(), "Unknown field in pipeline options")),
            }
        }
        
        let chunk = chunk.ok_or_else(|| syn::Error::new(content.span(), "Missing 'chunk' field in pipeline options"))?;
        let embed = embed.ok_or_else(|| syn::Error::new(content.span(), "Missing 'embed' field in pipeline options"))?;
        let store = store.ok_or_else(|| syn::Error::new(content.span(), "Missing 'store' field in pipeline options"))?;
        
        Ok(PipelineOptions {
            chunk,
            embed,
            store,
        })
    }
}

// 整个RAG Pipeline定义
struct RagPipelineDef {
    name: LitStr,
    source: Expr,
    pipeline: PipelineOptions,
    query_pipeline: Option<QueryOptions>,
}

impl Parse for RagPipelineDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let mut name = None;
        let mut source = None;
        let mut pipeline = None;
        let mut query_pipeline = None;
        
        while !content.is_empty() {
            let key: syn::Ident = content.parse()?;
            let _: Token![:] = content.parse()?;
            
            match key.to_string().as_str() {
                "name" => {
                    name = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "source" => {
                    source = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "pipeline" => {
                    pipeline = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "query_pipeline" => {
                    query_pipeline = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new(key.span(), "Unknown field in RAG pipeline definition")),
            }
        }
        
        let name = name.ok_or_else(|| syn::Error::new(content.span(), "Missing 'name' field in RAG pipeline definition"))?;
        let source = source.ok_or_else(|| syn::Error::new(content.span(), "Missing 'source' field in RAG pipeline definition"))?;
        let pipeline = pipeline.ok_or_else(|| syn::Error::new(content.span(), "Missing 'pipeline' field in RAG pipeline definition"))?;
        
        Ok(RagPipelineDef {
            name,
            source,
            pipeline,
            query_pipeline,
        })
    }
}

/// 创建一个RAG管道，参考Mastra的RAG原语API设计
/// 
/// # 示例
/// 
/// ```rust
/// rag_pipeline! {
///     name: "knowledge_base",
///     
///     source: DocumentSource::from_directory("./docs"),
///     
///     pipeline: {
///         chunk: {
///             chunk_size: 1000,
///             chunk_overlap: 200,
///             separator: "\n",
///             strategy: "recursive"
///         },
///         
///         embed: {
///             model: "text-embedding-3-small",
///             dimensions: 1536,
///             max_retries: 3
///         },
///         
///         store: {
///             db: "pgvector",
///             collection: "embeddings",
///             connection_string: env!("DATABASE_URL")
///         }
///     },
///     
///     query_pipeline: {
///         rerank: true,
///         top_k: 5,
///         filter: r#"{ "type": { "$in": ["article", "faq"] } }"#
///     }
/// }
/// ```
#[proc_macro]
pub fn rag_pipeline(input: TokenStream) -> TokenStream {
    let rag_pipeline_def = parse_macro_input!(input as RagPipelineDef);
    
    let name = &rag_pipeline_def.name;
    let source = &rag_pipeline_def.source;
    
    // 处理chunk选项
    let chunk = &rag_pipeline_def.pipeline.chunk;
    let chunk_size = &chunk.chunk_size;
    
    let chunk_overlap = match &chunk.chunk_overlap {
        Some(overlap) => quote! { .with_overlap(#overlap) },
        None => quote! {},
    };
    
    let separator = match &chunk.separator {
        Some(sep) => quote! { .with_separator(#sep) },
        None => quote! {},
    };
    
    let strategy = match &chunk.strategy {
        Some(strat) => quote! { .with_strategy(#strat) },
        None => quote! {},
    };
    
    // 处理embed选项
    let embed = &rag_pipeline_def.pipeline.embed;
    let embed_model = &embed.model;
    
    let dimensions = match &embed.dimensions {
        Some(dim) => quote! { .with_dimensions(#dim) },
        None => quote! {},
    };
    
    let max_retries = match &embed.max_retries {
        Some(retries) => quote! { .with_max_retries(#retries) },
        None => quote! {},
    };
    
    // 处理store选项
    let store = &rag_pipeline_def.pipeline.store;
    let db_type = &store.db;
    let collection_name = &store.collection;
    
    let connection_string = match &store.connection_string {
        Some(conn) => quote! { .with_connection_string(#conn) },
        None => quote! {},
    };
    
    // 处理query选项
    let query_options = match &rag_pipeline_def.query_pipeline {
        Some(query) => {
            let rerank = match &query.rerank {
                Some(rerank) => quote! { .with_rerank(#rerank) },
                None => quote! {},
            };
            
            let top_k = match &query.top_k {
                Some(top_k) => quote! { .with_top_k(#top_k) },
                None => quote! {},
            };
            
            let filter = match &query.filter {
                Some(filter) => quote! { .with_filter(#filter) },
                None => quote! {},
            };
            
            quote! {
                let query_config = lomusai_core::rag::QueryConfig::new()
                    #rerank
                    #top_k
                    #filter;
                    
                pipeline.with_query_config(query_config);
            }
        },
        None => quote! {},
    };
    
    let name_ident = format_ident!("{}", name.value());
    
    let expanded = quote! {
        {
            use lomusai_core::rag::*;
            
            let chunk_config = ChunkConfig::new(#chunk_size)
                #chunk_overlap
                #separator
                #strategy;
                
            let embed_config = EmbedConfig::new(#embed_model)
                #dimensions
                #max_retries;
                
            let store_config = StoreConfig::new(#db_type, #collection_name)
                #connection_string;
                
            let mut pipeline = RagPipeline::new(#name)
                .with_source(#source)
                .with_chunk_config(chunk_config)
                .with_embed_config(embed_config)
                .with_store_config(store_config);
                
            #query_options
            
            let #name_ident = pipeline;
            #name_ident
        }
    };
    
    TokenStream::from(expanded)
} 