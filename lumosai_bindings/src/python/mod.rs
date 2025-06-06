//! Python绑定模块
//! 
//! 为Python提供Lumos.ai的完整绑定支持

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;
use std::sync::Arc;
use crate::core::{CrossLangAgent, CrossLangAgentBuilder, CrossLangTool, CrossLangResponse, ToolMetadata};
use crate::error::BindingError;
use crate::types::*;

pub mod tools;

/// Python Agent包装器
#[pyclass(name = "Agent")]
pub struct PyAgent {
    inner: CrossLangAgent,
}

/// Python AgentBuilder包装器
#[pyclass(name = "AgentBuilder")]
pub struct PyAgentBuilder {
    inner: CrossLangAgentBuilder,
}

/// Python Tool包装器
#[pyclass(name = "Tool")]
pub struct PyTool {
    inner: CrossLangTool,
}

/// Python Response包装器
#[pyclass(name = "Response")]
pub struct PyResponse {
    inner: CrossLangResponse,
}

/// Python错误类型
#[pyclass(name = "LumosError", extends = pyo3::exceptions::PyException)]
pub struct PyLumosError {
    inner: BindingError,
}

#[pymethods]
impl PyAgent {
    /// 生成响应
    #[pyo3(text_signature = "(self, input)")]
    fn generate(&self, input: &str) -> PyResult<PyResponse> {
        let response = self.inner.generate(input)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(PyResponse { inner: response })
    }
    
    /// 异步生成响应
    #[pyo3(text_signature = "(self, input)")]
    fn generate_async<'p>(&self, py: Python<'p>, input: &str) -> PyResult<&'p PyAny> {
        let input = input.to_string();
        let agent = self.inner.clone();
        
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let response = agent.generate_async(&input).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            
            Ok(PyResponse { inner: response })
        })
    }
    
    /// 获取配置
    #[pyo3(text_signature = "(self)")]
    fn get_config(&self) -> PyResult<PyObject> {
        let config = self.inner.get_config();
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            
            // 模型配置
            let model_dict = PyDict::new(py);
            model_dict.set_item("name", &config.model.name)?;
            model_dict.set_item("api_key", &config.model.api_key)?;
            model_dict.set_item("base_url", &config.model.base_url)?;
            dict.set_item("model", model_dict)?;
            
            // 工具配置
            let tools_list = PyList::new(py, &config.tools);
            dict.set_item("tools", tools_list)?;
            
            // 运行时配置
            let runtime_dict = PyDict::new(py);
            runtime_dict.set_item("timeout_seconds", config.runtime.timeout_seconds)?;
            runtime_dict.set_item("max_retries", config.runtime.max_retries)?;
            runtime_dict.set_item("concurrency_limit", config.runtime.concurrency_limit)?;
            runtime_dict.set_item("enable_logging", config.runtime.enable_logging)?;
            runtime_dict.set_item("log_level", &config.runtime.log_level)?;
            dict.set_item("runtime", runtime_dict)?;
            
            Ok(dict.into())
        })
    }
    
    /// 字符串表示
    fn __str__(&self) -> String {
        "Lumos.ai Agent".to_string()
    }
    
    /// 调试表示
    fn __repr__(&self) -> String {
        "Agent()".to_string()
    }
}

#[pymethods]
impl PyAgentBuilder {
    /// 创建新的AgentBuilder
    #[new]
    fn new() -> Self {
        Self {
            inner: CrossLangAgentBuilder::new(),
        }
    }
    
    /// 设置名称
    #[pyo3(text_signature = "(self, name)")]
    fn name(mut slf: PyRefMut<Self>, name: &str) -> PyRefMut<Self> {
        slf.inner = slf.inner.name(name);
        slf
    }
    
    /// 设置指令
    #[pyo3(text_signature = "(self, instructions)")]
    fn instructions(mut slf: PyRefMut<Self>, instructions: &str) -> PyRefMut<Self> {
        slf.inner = slf.inner.instructions(instructions);
        slf
    }
    
    /// 设置模型
    #[pyo3(text_signature = "(self, model)")]
    fn model(mut slf: PyRefMut<Self>, model: &str) -> PyRefMut<Self> {
        slf.inner = slf.inner.model(model);
        slf
    }
    
    /// 添加工具
    #[pyo3(text_signature = "(self, tool)")]
    fn tool(mut slf: PyRefMut<Self>, tool: &PyTool) -> PyRefMut<Self> {
        slf.inner = slf.inner.tool(tool.inner.clone());
        slf
    }
    
    /// 添加多个工具
    #[pyo3(text_signature = "(self, tools)")]
    fn tools(mut slf: PyRefMut<Self>, tools: &PyList) -> PyResult<PyRefMut<Self>> {
        for item in tools.iter() {
            let tool: &PyTool = item.extract()?;
            slf.inner = slf.inner.tool(tool.inner.clone());
        }
        Ok(slf)
    }
    
    /// 构建Agent
    #[pyo3(text_signature = "(self)")]
    fn build(self) -> PyResult<PyAgent> {
        let agent = self.inner.build()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(PyAgent { inner: agent })
    }
    
    /// 异步构建Agent
    #[pyo3(text_signature = "(self)")]
    fn build_async<'p>(self, py: Python<'p>) -> PyResult<&'p PyAny> {
        let builder = self.inner;
        
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let agent = builder.build_async().await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            
            Ok(PyAgent { inner: agent })
        })
    }
    
    /// 字符串表示
    fn __str__(&self) -> String {
        "Lumos.ai AgentBuilder".to_string()
    }
    
    /// 调试表示
    fn __repr__(&self) -> String {
        "AgentBuilder()".to_string()
    }
}

#[pymethods]
impl PyTool {
    /// 获取工具元数据
    #[pyo3(text_signature = "(self)")]
    fn metadata(&self) -> PyResult<PyObject> {
        let metadata = self.inner.metadata();
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            dict.set_item("name", &metadata.name)?;
            dict.set_item("description", &metadata.description)?;
            dict.set_item("tool_type", &metadata.tool_type)?;
            dict.set_item("is_async", metadata.is_async)?;
            
            // 参数模式转换为Python字典
            let params_str = serde_json::to_string(&metadata.parameters)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
            let params_dict: PyObject = py.eval(&format!("__import__('json').loads('{}')", params_str), None, None)?.into();
            dict.set_item("parameters", params_dict)?;
            
            Ok(dict.into())
        })
    }
    
    /// 执行工具
    #[pyo3(text_signature = "(self, **kwargs)")]
    fn execute(&self, kwargs: Option<&PyDict>) -> PyResult<PyObject> {
        let parameters = if let Some(kwargs) = kwargs {
            // 将Python字典转换为JSON值
            let mut params = serde_json::Map::new();
            for (key, value) in kwargs.iter() {
                let key_str: String = key.extract()?;
                let value_json = python_to_json(value)?;
                params.insert(key_str, value_json);
            }
            serde_json::Value::Object(params)
        } else {
            serde_json::Value::Object(serde_json::Map::new())
        };
        
        let result = self.inner.execute(parameters)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            dict.set_item("tool_name", &result.tool_name)?;
            dict.set_item("success", result.success)?;
            dict.set_item("execution_time_ms", result.execution_time_ms)?;
            
            // 结果转换为Python对象
            let result_str = serde_json::to_string(&result.result)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
            let result_obj: PyObject = py.eval(&format!("__import__('json').loads('{}')", result_str), None, None)?.into();
            dict.set_item("result", result_obj)?;
            
            if let Some(error) = &result.error {
                dict.set_item("error", error)?;
            }
            
            Ok(dict.into())
        })
    }
    
    /// 字符串表示
    fn __str__(&self) -> String {
        format!("Tool({})", self.inner.metadata().name)
    }
    
    /// 调试表示
    fn __repr__(&self) -> String {
        format!("Tool(name='{}', type='{}')", 
                self.inner.metadata().name, 
                self.inner.metadata().tool_type)
    }
}

#[pymethods]
impl PyResponse {
    /// 获取响应内容
    #[getter]
    fn content(&self) -> &str {
        &self.inner.content
    }
    
    /// 获取响应类型
    #[getter]
    fn response_type(&self) -> String {
        format!("{:?}", self.inner.response_type)
    }
    
    /// 获取元数据
    #[getter]
    fn metadata(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            for (key, value) in &self.inner.metadata {
                let value_str = serde_json::to_string(value)
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
                let value_obj: PyObject = py.eval(&format!("__import__('json').loads('{}')", value_str), None, None)?.into();
                dict.set_item(key, value_obj)?;
            }
            Ok(dict.into())
        })
    }
    
    /// 获取工具调用结果
    #[getter]
    fn tool_calls(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let list = PyList::empty(py);
            for tool_call in &self.inner.tool_calls {
                let dict = PyDict::new(py);
                dict.set_item("tool_name", &tool_call.tool_name)?;
                dict.set_item("success", tool_call.success)?;
                dict.set_item("execution_time_ms", tool_call.execution_time_ms)?;
                
                let result_str = serde_json::to_string(&tool_call.result)
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
                let result_obj: PyObject = py.eval(&format!("__import__('json').loads('{}')", result_str), None, None)?.into();
                dict.set_item("result", result_obj)?;
                
                if let Some(error) = &tool_call.error {
                    dict.set_item("error", error)?;
                }
                
                list.append(dict)?;
            }
            Ok(list.into())
        })
    }
    
    /// 是否有错误
    #[getter]
    fn has_error(&self) -> bool {
        self.inner.error.is_some()
    }
    
    /// 获取错误信息
    #[getter]
    fn error(&self) -> Option<&str> {
        self.inner.error.as_deref()
    }
    
    /// 字符串表示
    fn __str__(&self) -> &str {
        &self.inner.content
    }
    
    /// 调试表示
    fn __repr__(&self) -> String {
        format!("Response(content='{}...', type={:?})", 
                &self.inner.content.chars().take(50).collect::<String>(),
                self.inner.response_type)
    }
}

/// 便利函数：快速创建Agent
#[pyfunction]
#[pyo3(text_signature = "(name, instructions)")]
pub fn quick_agent(name: &str, instructions: &str) -> PyAgentBuilder {
    PyAgentBuilder {
        inner: crate::core::quick_agent(name, instructions),
    }
}

/// 便利函数：创建AgentBuilder
#[pyfunction]
#[pyo3(text_signature = "()")]
pub fn create_agent_builder() -> PyAgentBuilder {
    PyAgentBuilder {
        inner: crate::core::create_agent_builder(),
    }
}

/// 将Python对象转换为JSON值
fn python_to_json(obj: &PyAny) -> PyResult<serde_json::Value> {
    if obj.is_none() {
        Ok(serde_json::Value::Null)
    } else if let Ok(s) = obj.extract::<String>() {
        Ok(serde_json::Value::String(s))
    } else if let Ok(i) = obj.extract::<i64>() {
        Ok(serde_json::Value::Number(serde_json::Number::from(i)))
    } else if let Ok(f) = obj.extract::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            Ok(serde_json::Value::Number(n))
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Invalid float value"))
        }
    } else if let Ok(b) = obj.extract::<bool>() {
        Ok(serde_json::Value::Bool(b))
    } else {
        // 对于复杂类型，尝试使用Python的json模块
        Python::with_gil(|py| {
            let json_module = py.import("json")?;
            let json_str: String = json_module.call_method1("dumps", (obj,))?.extract()?;
            serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
        })
    }
}
