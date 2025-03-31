use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, LitStr, Expr, Token, braced, parse::{Parse, ParseStream}};
use syn::punctuated::Punctuated;

// 评估指标的定义
struct MetricDef {
    name: syn::Ident,
    expr: Expr,
}

impl Parse for MetricDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: syn::Ident = input.parse()?;
        let _: Token![:] = input.parse()?;
        let expr: Expr = input.parse()?;
        Ok(MetricDef { name, expr })
    }
}

// 测试用例定义
struct TestCaseDef {
    name: syn::Ident,
    path: LitStr,
}

impl Parse for TestCaseDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: syn::Ident = input.parse()?;
        let _: Token![:] = input.parse()?;
        let path: LitStr = input.parse()?;
        Ok(TestCaseDef { name, path })
    }
}

// 报告选项
struct ReportingOptions {
    format: LitStr,
    output: Option<LitStr>,
}

impl Parse for ReportingOptions {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let mut format = None;
        let mut output = None;
        
        while !content.is_empty() {
            let key: syn::Ident = content.parse()?;
            let _: Token![:] = content.parse()?;
            
            match key.to_string().as_str() {
                "format" => {
                    format = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "output" => {
                    output = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new(key.span(), "Unknown field in reporting options")),
            }
        }
        
        let format = format.ok_or_else(|| syn::Error::new(content.span(), "Missing 'format' field in reporting options"))?;
        
        Ok(ReportingOptions {
            format,
            output,
        })
    }
}

// 整个评估套件定义
struct EvalSuiteDef {
    name: LitStr,
    metrics: Option<Punctuated<MetricDef, Token![,]>>,
    test_cases: Option<Punctuated<TestCaseDef, Token![,]>>,
    reporting: Option<ReportingOptions>,
}

impl Parse for EvalSuiteDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        
        let mut name = None;
        let mut metrics = None;
        let mut test_cases = None;
        let mut reporting = None;
        
        while !content.is_empty() {
            let key: syn::Ident = content.parse()?;
            let _: Token![:] = content.parse()?;
            
            match key.to_string().as_str() {
                "name" => {
                    name = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "metrics" => {
                    let metrics_content;
                    braced!(metrics_content in content);
                    metrics = Some(Punctuated::parse_terminated(&metrics_content)?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "test_cases" => {
                    let test_cases_content;
                    braced!(test_cases_content in content);
                    test_cases = Some(Punctuated::parse_terminated(&test_cases_content)?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                "reporting" => {
                    reporting = Some(content.parse()?);
                    let _: Option<Token![,]> = content.parse()?;
                },
                _ => return Err(syn::Error::new(key.span(), "Unknown field in eval suite definition")),
            }
        }
        
        let name = name.ok_or_else(|| syn::Error::new(content.span(), "Missing 'name' field in eval suite definition"))?;
        
        Ok(EvalSuiteDef {
            name,
            metrics,
            test_cases,
            reporting,
        })
    }
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
pub fn eval_suite_impl(input: TokenStream) -> TokenStream {
    let eval_suite_def = parse_macro_input!(input as EvalSuiteDef);
    
    let name = &eval_suite_def.name;
    let suite_name_str = name.value();
    let suite_var_name = format_ident!("{}", suite_name_str.to_lowercase().replace("-", "_"));
    
    // 处理指标
    let metrics_registration = if let Some(metrics) = &eval_suite_def.metrics {
        let metric_statements = metrics.iter().map(|metric| {
            let metric_name = &metric.name;
            let metric_expr = &metric.expr;
            quote! {
                suite.add_metric(stringify!(#metric_name), #metric_expr);
            }
        }).collect::<Vec<_>>();
        
        quote! {
            #(#metric_statements)*
        }
    } else {
        quote! {}
    };
    
    // 处理测试用例
    let test_cases_registration = if let Some(test_cases) = &eval_suite_def.test_cases {
        let test_case_statements = test_cases.iter().map(|test_case| {
            let test_name = &test_case.name;
            let test_path = &test_case.path;
            quote! {
                suite.add_test_case(stringify!(#test_name), #test_path);
            }
        }).collect::<Vec<_>>();
        
        quote! {
            #(#test_case_statements)*
        }
    } else {
        quote! {}
    };
    
    // 处理报告选项
    let reporting_config = if let Some(reporting) = &eval_suite_def.reporting {
        let format = &reporting.format;
        
        let output = if let Some(output_path) = &reporting.output {
            quote! {
                suite.set_report_output(#output_path);
            }
        } else {
            quote! {}
        };
        
        quote! {
            suite.set_report_format(#format);
            #output
        }
    } else {
        quote! {}
    };
    
    let expanded = quote! {
        {
            use lumosai_core::eval::*;
            
            let mut suite = EvaluationSuite::new(#name);
            
            // 注册指标
            #metrics_registration
            
            // 注册测试用例
            #test_cases_registration
            
            // 配置报告选项
            #reporting_config
            
            let #suite_var_name = suite;
            #suite_var_name
        }
    };
    
    TokenStream::from(expanded)
} 