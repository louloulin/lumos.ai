//! WebAssembly绑定模块
//! 
//! 为Web浏览器提供Lumos.ai的WebAssembly绑定支持

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use js_sys::{Object, Promise};
use web_sys::console;
use std::collections::HashMap;
use crate::core::{CrossLangAgent, CrossLangAgentBuilder, CrossLangTool, CrossLangResponse};
use crate::error::BindingError;
use crate::types::*;

/// WebAssembly Agent包装器
#[wasm_bindgen]
pub struct WasmAgent {
    inner: CrossLangAgent,
}

/// WebAssembly AgentBuilder包装器
#[wasm_bindgen]
pub struct WasmAgentBuilder {
    inner: CrossLangAgentBuilder,
}

/// WebAssembly Tool包装器
#[wasm_bindgen]
pub struct WasmTool {
    inner: CrossLangTool,
}

/// WebAssembly Response包装器
#[wasm_bindgen]
pub struct WasmResponse {
    inner: CrossLangResponse,
}

#[wasm_bindgen]
impl WasmAgent {
    /// 生成响应
    #[wasm_bindgen]
    pub fn generate(&self, input: &str) -> Result<WasmResponse, JsValue> {
        let response = self.inner.generate(input)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        Ok(WasmResponse { inner: response })
    }
    
    /// 异步生成响应
    #[wasm_bindgen]
    pub async fn generate_async(&self, input: &str) -> Result<WasmResponse, JsValue> {
        let response = self.inner.generate_async(input).await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        Ok(WasmResponse { inner: response })
    }
    
    /// 获取配置
    #[wasm_bindgen]
    pub fn get_config(&self) -> Result<JsValue, JsValue> {
        let config = self.inner.get_config();
        
        let js_config = Object::new();
        
        // 模型配置
        let model_obj = Object::new();
        js_sys::Reflect::set(&model_obj, &"name".into(), &config.model.name.into())?;
        if let Some(api_key) = &config.model.api_key {
            js_sys::Reflect::set(&model_obj, &"apiKey".into(), &api_key.into())?;
        }
        if let Some(base_url) = &config.model.base_url {
            js_sys::Reflect::set(&model_obj, &"baseUrl".into(), &base_url.into())?;
        }
        js_sys::Reflect::set(&js_config, &"model".into(), &model_obj)?;
        
        // 工具配置
        let tools_array = js_sys::Array::new();
        for tool in &config.tools {
            tools_array.push(&tool.into());
        }
        js_sys::Reflect::set(&js_config, &"tools".into(), &tools_array)?;
        
        // 运行时配置
        let runtime_obj = Object::new();
        js_sys::Reflect::set(&runtime_obj, &"timeoutSeconds".into(), &(config.runtime.timeout_seconds as f64).into())?;
        js_sys::Reflect::set(&runtime_obj, &"maxRetries".into(), &(config.runtime.max_retries as f64).into())?;
        js_sys::Reflect::set(&runtime_obj, &"concurrencyLimit".into(), &(config.runtime.concurrency_limit as f64).into())?;
        js_sys::Reflect::set(&runtime_obj, &"enableLogging".into(), &config.runtime.enable_logging.into())?;
        js_sys::Reflect::set(&runtime_obj, &"logLevel".into(), &config.runtime.log_level.into())?;
        js_sys::Reflect::set(&js_config, &"runtime".into(), &runtime_obj)?;
        
        Ok(js_config.into())
    }
}

#[wasm_bindgen]
impl WasmAgentBuilder {
    /// 创建新的AgentBuilder
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: CrossLangAgentBuilder::new(),
        }
    }
    
    /// 设置名称
    #[wasm_bindgen]
    pub fn name(mut self, name: &str) -> Self {
        self.inner = self.inner.name(name);
        self
    }
    
    /// 设置指令
    #[wasm_bindgen]
    pub fn instructions(mut self, instructions: &str) -> Self {
        self.inner = self.inner.instructions(instructions);
        self
    }
    
    /// 设置模型
    #[wasm_bindgen]
    pub fn model(mut self, model: &str) -> Self {
        self.inner = self.inner.model(model);
        self
    }
    
    /// 添加工具
    #[wasm_bindgen]
    pub fn tool(mut self, tool: &WasmTool) -> Self {
        self.inner = self.inner.tool(tool.inner.clone());
        self
    }
    
    /// 构建Agent
    #[wasm_bindgen]
    pub fn build(self) -> Result<WasmAgent, JsValue> {
        let agent = self.inner.build()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        Ok(WasmAgent { inner: agent })
    }
    
    /// 异步构建Agent
    #[wasm_bindgen]
    pub async fn build_async(self) -> Result<WasmAgent, JsValue> {
        let agent = self.inner.build_async().await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        Ok(WasmAgent { inner: agent })
    }
}

#[wasm_bindgen]
impl WasmTool {
    /// 获取工具元数据
    #[wasm_bindgen]
    pub fn metadata(&self) -> Result<JsValue, JsValue> {
        let metadata = self.inner.metadata();
        
        let js_metadata = Object::new();
        js_sys::Reflect::set(&js_metadata, &"name".into(), &metadata.name.into())?;
        js_sys::Reflect::set(&js_metadata, &"description".into(), &metadata.description.into())?;
        js_sys::Reflect::set(&js_metadata, &"toolType".into(), &metadata.tool_type.into())?;
        js_sys::Reflect::set(&js_metadata, &"isAsync".into(), &metadata.is_async.into())?;
        
        // 参数模式转换
        let params_str = serde_json::to_string(&metadata.parameters)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let params_obj = js_sys::JSON::parse(&params_str)?;
        js_sys::Reflect::set(&js_metadata, &"parameters".into(), &params_obj)?;
        
        Ok(js_metadata.into())
    }
    
    /// 执行工具
    #[wasm_bindgen]
    pub fn execute(&self, parameters: &JsValue) -> Result<JsValue, JsValue> {
        // 将JavaScript对象转换为JSON
        let params_str = js_sys::JSON::stringify(parameters)
            .map_err(|_| JsValue::from_str("Failed to stringify parameters"))?;
        let params_str = params_str.as_string()
            .ok_or_else(|| JsValue::from_str("Invalid parameters"))?;
        
        let parameters: serde_json::Value = serde_json::from_str(&params_str)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        let result = self.inner.execute(parameters)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        // 构建结果对象
        let js_result = Object::new();
        js_sys::Reflect::set(&js_result, &"toolName".into(), &result.tool_name.into())?;
        js_sys::Reflect::set(&js_result, &"success".into(), &result.success.into())?;
        js_sys::Reflect::set(&js_result, &"executionTimeMs".into(), &(result.execution_time_ms as f64).into())?;
        
        // 结果数据转换
        let result_str = serde_json::to_string(&result.result)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let result_obj = js_sys::JSON::parse(&result_str)?;
        js_sys::Reflect::set(&js_result, &"result".into(), &result_obj)?;
        
        if let Some(error) = &result.error {
            js_sys::Reflect::set(&js_result, &"error".into(), &error.into())?;
        }
        
        Ok(js_result.into())
    }
}

#[wasm_bindgen]
impl WasmResponse {
    /// 获取响应内容
    #[wasm_bindgen(getter)]
    pub fn content(&self) -> String {
        self.inner.content.clone()
    }
    
    /// 获取响应类型
    #[wasm_bindgen(getter)]
    pub fn response_type(&self) -> String {
        format!("{:?}", self.inner.response_type)
    }
    
    /// 获取元数据
    #[wasm_bindgen]
    pub fn metadata(&self) -> Result<JsValue, JsValue> {
        let js_metadata = Object::new();
        
        for (key, value) in &self.inner.metadata {
            let value_str = serde_json::to_string(value)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            let value_obj = js_sys::JSON::parse(&value_str)?;
            js_sys::Reflect::set(&js_metadata, &key.into(), &value_obj)?;
        }
        
        Ok(js_metadata.into())
    }
    
    /// 获取工具调用结果
    #[wasm_bindgen]
    pub fn tool_calls(&self) -> Result<JsValue, JsValue> {
        let js_array = js_sys::Array::new();
        
        for tool_call in &self.inner.tool_calls {
            let js_call = Object::new();
            js_sys::Reflect::set(&js_call, &"toolName".into(), &tool_call.tool_name.into())?;
            js_sys::Reflect::set(&js_call, &"success".into(), &tool_call.success.into())?;
            js_sys::Reflect::set(&js_call, &"executionTimeMs".into(), &(tool_call.execution_time_ms as f64).into())?;
            
            let result_str = serde_json::to_string(&tool_call.result)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            let result_obj = js_sys::JSON::parse(&result_str)?;
            js_sys::Reflect::set(&js_call, &"result".into(), &result_obj)?;
            
            if let Some(error) = &tool_call.error {
                js_sys::Reflect::set(&js_call, &"error".into(), &error.into())?;
            }
            
            js_array.push(&js_call.into());
        }
        
        Ok(js_array.into())
    }
    
    /// 是否有错误
    #[wasm_bindgen(getter)]
    pub fn has_error(&self) -> bool {
        self.inner.error.is_some()
    }
    
    /// 获取错误信息
    #[wasm_bindgen(getter)]
    pub fn error(&self) -> Option<String> {
        self.inner.error.clone()
    }
}

/// 便利函数：快速创建Agent
#[wasm_bindgen]
pub fn quick_agent(name: &str, instructions: &str) -> WasmAgentBuilder {
    WasmAgentBuilder {
        inner: crate::core::quick_agent(name, instructions),
    }
}

/// 便利函数：创建AgentBuilder
#[wasm_bindgen]
pub fn create_agent_builder() -> WasmAgentBuilder {
    WasmAgentBuilder {
        inner: crate::core::create_agent_builder(),
    }
}

/// 工具模块
pub mod tools {
    use super::*;
    use crate::core::ToolMetadata;
    
    /// Web搜索工具
    #[wasm_bindgen]
    pub fn web_search() -> WasmTool {
        let tool = lumosai_core::tools::web::web_search();
        let metadata = ToolMetadata {
            name: "web_search".to_string(),
            description: "搜索网络内容".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "搜索查询"
                    }
                },
                "required": ["query"]
            }),
            tool_type: "web".to_string(),
            is_async: true,
        };
        
        WasmTool {
            inner: CrossLangTool::new(tool, metadata),
        }
    }
    
    /// 计算器工具
    #[wasm_bindgen]
    pub fn calculator() -> WasmTool {
        let tool = lumosai_core::tools::math::calculator();
        let metadata = ToolMetadata {
            name: "calculator".to_string(),
            description: "基础数学计算".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "数学表达式"
                    }
                },
                "required": ["expression"]
            }),
            tool_type: "math".to_string(),
            is_async: false,
        };
        
        WasmTool {
            inner: CrossLangTool::new(tool, metadata),
        }
    }
}

/// 初始化WebAssembly模块
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
    
    console::log_1(&"Lumos.ai WebAssembly module initialized".into());
}
