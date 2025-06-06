//! Lumos.ai 多语言绑定
//! 
//! 为Lumos.ai提供Python、JavaScript/TypeScript、Go等语言的绑定支持

pub mod core;
pub mod error;
pub mod types;

#[cfg(feature = "python")]
pub mod python;

#[cfg(feature = "nodejs")]
pub mod nodejs;

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(feature = "c-bindings")]
pub mod c_bindings;

// 重新导出核心类型
pub use crate::core::*;
pub use crate::error::*;
pub use crate::types::*;

// Python绑定入口
#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pymodule]
fn lumosai(_py: Python, m: &PyModule) -> PyResult<()> {
    // 注册核心类型
    m.add_class::<python::PyAgent>()?;
    m.add_class::<python::PyAgentBuilder>()?;
    m.add_class::<python::PyTool>()?;
    m.add_class::<python::PyResponse>()?;
    
    // 注册工具模块
    let tools_module = PyModule::new(_py, "tools")?;
    python::tools::register_tools(tools_module)?;
    m.add_submodule(tools_module)?;
    
    // 注册错误类型
    m.add("LumosError", _py.get_type::<python::PyLumosError>())?;
    
    // 注册便利函数
    m.add_function(wrap_pyfunction!(python::quick_agent, m)?)?;
    m.add_function(wrap_pyfunction!(python::create_agent_builder, m)?)?;
    
    Ok(())
}

// Node.js绑定入口
#[cfg(feature = "nodejs")]
#[napi_derive::napi]
pub fn register_nodejs_bindings() {
    // Node.js绑定将通过napi-derive自动生成
}

// WebAssembly绑定入口
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
#[wasm_bindgen(start)]
pub fn wasm_main() {
    // 初始化WebAssembly环境
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
}
