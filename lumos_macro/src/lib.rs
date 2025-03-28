extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, Expr, Ident, LitStr, Token, parse::{Parse, ParseStream}, punctuated::Punctuated};
use syn::{Attribute, Data, DataStruct, Fields};

/// Macro for defining a tool in a simplified way
/// 
/// # Example
/// ```
/// use lumos_macro::tool;
/// 
/// #[tool(
///     name = "calculator",
///     description = "Performs basic math operations"
/// )]
/// fn calculator(
///     #[parameter(
///         name = "operation",
///         description = "The operation to perform: add, subtract, multiply, divide",
///         r#type = "string", 
///         required = true
///     )]
///     operation: String,
///     
///     #[parameter(
///         name = "a",
///         description = "First number",
///         r#type = "number",
///         required = true
///     )]
///     a: f64,
///     
///     #[parameter(
///         name = "b",
///         description = "Second number",
///         r#type = "number",
///         required = true
///     )]
///     b: f64,
/// ) -> Result<serde_json::Value, lomusai_core::Error> {
///     // Function implementation
/// }
/// ```
#[proc_macro_attribute]
pub fn tool(attr: TokenStream, item: TokenStream) -> TokenStream {
    tool_macro::tool_impl(attr, item)
}

struct ToolAttributes {
    name: LitStr,
    description: LitStr,
}

impl Parse for ToolAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut description = None;
        
        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            
            if ident == "name" {
                name = Some(input.parse()?);
            } else if ident == "description" {
                description = Some(input.parse()?);
            } else {
                return Err(syn::Error::new(ident.span(), "Unknown attribute"));
            }
            
            // Allow trailing comma
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        
        let name = name.ok_or_else(|| syn::Error::new(input.span(), "Missing name attribute"))?;
        let description = description.ok_or_else(|| syn::Error::new(input.span(), "Missing description attribute"))?;
        
        Ok(ToolAttributes { name, description })
    }
}

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
        
        let content;
        syn::parenthesized!(content in input);
        let input = &content;
        
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
        let description = description.ok_or_else(|| syn::Error::new(input.span(), "Missing description attribute"))?;
        let type_ = type_.ok_or_else(|| syn::Error::new(input.span(), "Missing type attribute"))?;
        let required = required.unwrap_or(false);
        
        Ok(ParameterAttributes { name, description, type_, required })
    }
}

/// Macro for defining an agent with tools in a simplified way
/// 
/// # Example
/// ```
/// use lumos_macro::agent_attr;
/// 
/// #[agent_attr(
///     name = "math_agent",
///     instructions = "You are a helpful math assistant that can perform calculations.",
///     model = "gpt-4"
/// )]
/// struct MathAgent {
///     #[tool]
///     calculator: CalculatorTool,
///     
///     #[tool]
///     unit_converter: UnitConverterTool,
/// }
/// ```
#[proc_macro_attribute]
pub fn agent_attr(attr: TokenStream, item: TokenStream) -> TokenStream {
    agent_macro::agent_impl(attr, item)
}

struct AgentAttributes {
    name: LitStr,
    instructions: LitStr,
    model: LitStr,
}

impl Parse for AgentAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut instructions = None;
        let mut model = None;
        
        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            
            if ident == "name" {
                name = Some(input.parse()?);
            } else if ident == "instructions" {
                instructions = Some(input.parse()?);
            } else if ident == "model" {
                model = Some(input.parse()?);
            } else {
                return Err(syn::Error::new(ident.span(), "Unknown attribute"));
            }
            
            // Allow trailing comma
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        
        let name = name.ok_or_else(|| syn::Error::new(input.span(), "Missing name attribute"))?;
        let instructions = instructions.ok_or_else(|| syn::Error::new(input.span(), "Missing instructions attribute"))?;
        let model = model.ok_or_else(|| syn::Error::new(input.span(), "Missing model attribute"))?;
        
        Ok(AgentAttributes { name, instructions, model })
    }
}

/// A convenient derive macro for defining a model adapter
/// 
/// # Example
/// ```
/// use lumos_macro::LlmAdapter;
/// 
/// #[derive(LlmAdapter)]
/// struct OpenAIAdapter {
///     api_key: String,
///     model: String,
/// }
/// ```
#[proc_macro_derive(LlmAdapter)]
pub fn llm_adapter(input: TokenStream) -> TokenStream {
    llm_adapter_macro::llm_adapter_impl(input)
}

/// A macro for quick tool execution setup
///
/// # Example
/// ```
/// lumos_execute_tool! {
///     tool: calculator,
///     params: {
///         "operation": "add",
///         "a": 10.5,
///         "b": 20.3
///     }
/// }
/// ```
#[proc_macro]
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

struct ToolExecuteArgs {
    tool: Expr,
    params: Expr,
}

impl Parse for ToolExecuteArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut tool = None;
        let mut params = None;
        
        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![:]>()?;
            
            if ident == "tool" {
                tool = Some(input.parse()?);
            } else if ident == "params" {
                params = Some(input.parse()?);
            } else {
                return Err(syn::Error::new(ident.span(), "Unknown field"));
            }
            
            // Allow trailing comma
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        
        let tool = tool.ok_or_else(|| syn::Error::new(input.span(), "Missing tool field"))?;
        let params = params.ok_or_else(|| syn::Error::new(input.span(), "Missing params field"))?;
        
        Ok(ToolExecuteArgs { tool, params })
    }
}

mod tool_macro;
mod agent_macro;
mod llm_adapter_macro;
mod workflow;
mod rag;
mod eval;
mod mcp;
mod agent_dsl;
mod tools_dsl;
mod lumos;

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
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn workflow(input: TokenStream) -> TokenStream {
    workflow::workflow(input)
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
///             chunk_overlap: 200
///         },
///         
///         embed: {
///             model: "text-embedding-3-small",
///             dimensions: 1536
///         },
///         
///         store: {
///             db: "pgvector",
///             collection: "embeddings"
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
    rag::rag_pipeline(input)
}

/// 创建一个评估套件，参考Mastra的Eval框架
/// 
/// # 示例
/// 
/// ```rust
/// eval_suite! {
///     name: "agent_performance",
///     
///     metrics: {
///         accuracy: AccuracyMetric::new(0.8),
///         relevance: RelevanceMetric::new(0.7),
///         completeness: CompletenessMetric::new(0.6)
///     },
///     
///     test_cases: {
///         basic_queries: "./tests/basic_queries.json",
///         complex_queries: "./tests/complex_queries.json"
///     },
///     
///     reporting: {
///         format: "html",
///         output: "./reports/eval_results.html"
///     }
/// }
/// ```
#[proc_macro]
pub fn eval_suite(input: TokenStream) -> TokenStream {
    eval::eval_suite(input)
}

/// 创建一个MCP客户端配置，参考Mastra的MCP支持
/// 
/// # 示例
/// 
/// ```rust
/// mcp_client! {
///     discovery: {
///         endpoints: ["https://tools.example.com/mcp", "https://api.mcp.run"],
///         auto_register: true
///     },
///     
///     tools: {
///         data_analysis: {
///             enabled: true,
///             auth: {
///                 type: "api_key",
///                 key_env: "DATA_ANALYSIS_API_KEY"
///             }
///         },
///         image_processing: {
///             enabled: true,
///             rate_limit: 100
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn mcp_client(input: TokenStream) -> TokenStream {
    mcp::mcp_client(input)
}

/// 创建一个代理定义，参考Mastra的Agent API设计
/// 
/// # 示例
/// 
/// ```rust
/// agent! {
///     name: "research_assistant",
///     instructions: "你是一个专业的研究助手，擅长收集和整理信息。",
///     
///     llm: {
///         provider: openai_adapter,
///         model: "gpt-4"
///     },
///     
///     memory: {
///         store_type: "buffer",
///         capacity: 10
///     },
///     
///     tools: {
///         search_tool,
///         calculator_tool: { precision: 2 },
///         web_browser: { javascript: true, screenshots: true }
///     }
/// }
/// ```
#[proc_macro]
pub fn agent(input: TokenStream) -> TokenStream {
    agent_dsl::agent(input)
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
#[proc_macro]
pub fn tools(input: TokenStream) -> TokenStream {
    tools_dsl::tools(input)
}

/// 配置整个Lumos应用，参考Mastra的应用级API
/// 
/// # 示例
/// 
/// ```rust
/// let app = lumos! {
///     name: "stock_assistant",
///     description: "一个能够提供股票信息的AI助手",
///     
///     agents: {
///         stockAgent
///     },
///     
///     tools: {
///         stockPriceTool,
///         stockInfoTool
///     },
///     
///     rags: {
///         stockKnowledgeBase
///     },
///     
///     workflows: {
///         stockAnalysisWorkflow
///     },
///     
///     mcp_endpoints: vec!["https://api.example.com/mcp"]
/// };
/// ```
#[proc_macro]
pub fn lumos(input: TokenStream) -> TokenStream {
    lumos::lumos(input)
} 