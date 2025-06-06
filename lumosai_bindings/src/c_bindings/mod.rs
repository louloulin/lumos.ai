//! C绑定模块
//! 
//! 为Go、C++等语言提供C ABI兼容的绑定支持

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint, c_void};
use std::ptr;
use std::sync::Arc;
use libc;
use crate::core::{CrossLangAgent, CrossLangAgentBuilder, CrossLangTool, CrossLangResponse};
use crate::error::BindingError;

/// C Agent句柄
pub type CAgent = *mut c_void;

/// C AgentBuilder句柄
pub type CAgentBuilder = *mut c_void;

/// C Tool句柄
pub type CTool = *mut c_void;

/// C Response句柄
pub type CResponse = *mut c_void;

/// C错误码
#[repr(C)]
pub enum CErrorCode {
    Success = 0,
    InvalidParameter = 1,
    RuntimeError = 2,
    SerializationError = 3,
    NetworkError = 4,
    TimeoutError = 5,
    UnknownError = 99,
}

/// C配置结构
#[repr(C)]
pub struct CConfig {
    pub model_name: *const c_char,
    pub api_key: *const c_char,
    pub base_url: *const c_char,
    pub timeout_seconds: c_uint,
    pub max_retries: c_uint,
    pub enable_logging: c_int,
}

/// C响应结构
#[repr(C)]
pub struct CResponseData {
    pub content: *const c_char,
    pub response_type: *const c_char,
    pub error: *const c_char,
    pub execution_time_ms: c_uint,
    pub success: c_int,
}

/// 创建新的AgentBuilder
#[no_mangle]
pub extern "C" fn lumos_agent_builder_new() -> CAgentBuilder {
    let builder = Box::new(CrossLangAgentBuilder::new());
    Box::into_raw(builder) as CAgentBuilder
}

/// 设置Agent名称
#[no_mangle]
pub extern "C" fn lumos_agent_builder_name(
    builder: CAgentBuilder,
    name: *const c_char,
) -> CErrorCode {
    if builder.is_null() || name.is_null() {
        return CErrorCode::InvalidParameter;
    }
    
    unsafe {
        let builder_ref = &mut *(builder as *mut CrossLangAgentBuilder);
        let name_str = match CStr::from_ptr(name).to_str() {
            Ok(s) => s,
            Err(_) => return CErrorCode::InvalidParameter,
        };
        
        *builder_ref = std::mem::take(builder_ref).name(name_str);
        CErrorCode::Success
    }
}

/// 设置Agent指令
#[no_mangle]
pub extern "C" fn lumos_agent_builder_instructions(
    builder: CAgentBuilder,
    instructions: *const c_char,
) -> CErrorCode {
    if builder.is_null() || instructions.is_null() {
        return CErrorCode::InvalidParameter;
    }
    
    unsafe {
        let builder_ref = &mut *(builder as *mut CrossLangAgentBuilder);
        let instructions_str = match CStr::from_ptr(instructions).to_str() {
            Ok(s) => s,
            Err(_) => return CErrorCode::InvalidParameter,
        };
        
        *builder_ref = std::mem::take(builder_ref).instructions(instructions_str);
        CErrorCode::Success
    }
}

/// 设置模型
#[no_mangle]
pub extern "C" fn lumos_agent_builder_model(
    builder: CAgentBuilder,
    model: *const c_char,
) -> CErrorCode {
    if builder.is_null() || model.is_null() {
        return CErrorCode::InvalidParameter;
    }
    
    unsafe {
        let builder_ref = &mut *(builder as *mut CrossLangAgentBuilder);
        let model_str = match CStr::from_ptr(model).to_str() {
            Ok(s) => s,
            Err(_) => return CErrorCode::InvalidParameter,
        };
        
        *builder_ref = std::mem::take(builder_ref).model(model_str);
        CErrorCode::Success
    }
}

/// 添加工具
#[no_mangle]
pub extern "C" fn lumos_agent_builder_tool(
    builder: CAgentBuilder,
    tool: CTool,
) -> CErrorCode {
    if builder.is_null() || tool.is_null() {
        return CErrorCode::InvalidParameter;
    }
    
    unsafe {
        let builder_ref = &mut *(builder as *mut CrossLangAgentBuilder);
        let tool_ref = &*(tool as *const CrossLangTool);
        
        *builder_ref = std::mem::take(builder_ref).tool(tool_ref.clone());
        CErrorCode::Success
    }
}

/// 构建Agent
#[no_mangle]
pub extern "C" fn lumos_agent_builder_build(
    builder: CAgentBuilder,
    agent_out: *mut CAgent,
) -> CErrorCode {
    if builder.is_null() || agent_out.is_null() {
        return CErrorCode::InvalidParameter;
    }
    
    unsafe {
        let builder = Box::from_raw(builder as *mut CrossLangAgentBuilder);
        
        match builder.build() {
            Ok(agent) => {
                let agent_box = Box::new(agent);
                *agent_out = Box::into_raw(agent_box) as CAgent;
                CErrorCode::Success
            }
            Err(_) => CErrorCode::RuntimeError,
        }
    }
}

/// 生成响应
#[no_mangle]
pub extern "C" fn lumos_agent_generate(
    agent: CAgent,
    input: *const c_char,
    response_out: *mut CResponse,
) -> CErrorCode {
    if agent.is_null() || input.is_null() || response_out.is_null() {
        return CErrorCode::InvalidParameter;
    }
    
    unsafe {
        let agent_ref = &*(agent as *const CrossLangAgent);
        let input_str = match CStr::from_ptr(input).to_str() {
            Ok(s) => s,
            Err(_) => return CErrorCode::InvalidParameter,
        };
        
        match agent_ref.generate(input_str) {
            Ok(response) => {
                let response_box = Box::new(response);
                *response_out = Box::into_raw(response_box) as CResponse;
                CErrorCode::Success
            }
            Err(e) => match e {
                BindingError::Network { .. } => CErrorCode::NetworkError,
                BindingError::Timeout { .. } => CErrorCode::TimeoutError,
                BindingError::Serialization { .. } => CErrorCode::SerializationError,
                _ => CErrorCode::RuntimeError,
            }
        }
    }
}

/// 获取响应数据
#[no_mangle]
pub extern "C" fn lumos_response_get_data(
    response: CResponse,
    data_out: *mut CResponseData,
) -> CErrorCode {
    if response.is_null() || data_out.is_null() {
        return CErrorCode::InvalidParameter;
    }
    
    unsafe {
        let response_ref = &*(response as *const CrossLangResponse);
        
        // 分配C字符串
        let content_cstr = match CString::new(response_ref.content.clone()) {
            Ok(s) => s,
            Err(_) => return CErrorCode::SerializationError,
        };
        
        let response_type_cstr = match CString::new(format!("{:?}", response_ref.response_type)) {
            Ok(s) => s,
            Err(_) => return CErrorCode::SerializationError,
        };
        
        let error_cstr = if let Some(error) = &response_ref.error {
            match CString::new(error.clone()) {
                Ok(s) => Some(s),
                Err(_) => return CErrorCode::SerializationError,
            }
        } else {
            None
        };
        
        (*data_out).content = content_cstr.into_raw();
        (*data_out).response_type = response_type_cstr.into_raw();
        (*data_out).error = error_cstr.map_or(ptr::null(), |s| s.into_raw());
        (*data_out).execution_time_ms = 0; // TODO: 添加执行时间跟踪
        (*data_out).success = if response_ref.error.is_none() { 1 } else { 0 };
        
        CErrorCode::Success
    }
}

/// 释放Agent
#[no_mangle]
pub extern "C" fn lumos_agent_free(agent: CAgent) {
    if !agent.is_null() {
        unsafe {
            let _ = Box::from_raw(agent as *mut CrossLangAgent);
        }
    }
}

/// 释放AgentBuilder
#[no_mangle]
pub extern "C" fn lumos_agent_builder_free(builder: CAgentBuilder) {
    if !builder.is_null() {
        unsafe {
            let _ = Box::from_raw(builder as *mut CrossLangAgentBuilder);
        }
    }
}

/// 释放Response
#[no_mangle]
pub extern "C" fn lumos_response_free(response: CResponse) {
    if !response.is_null() {
        unsafe {
            let _ = Box::from_raw(response as *mut CrossLangResponse);
        }
    }
}

/// 释放C字符串
#[no_mangle]
pub extern "C" fn lumos_string_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

/// 快速创建Agent
#[no_mangle]
pub extern "C" fn lumos_quick_agent(
    name: *const c_char,
    instructions: *const c_char,
    agent_out: *mut CAgent,
) -> CErrorCode {
    if name.is_null() || instructions.is_null() || agent_out.is_null() {
        return CErrorCode::InvalidParameter;
    }
    
    unsafe {
        let name_str = match CStr::from_ptr(name).to_str() {
            Ok(s) => s,
            Err(_) => return CErrorCode::InvalidParameter,
        };
        
        let instructions_str = match CStr::from_ptr(instructions).to_str() {
            Ok(s) => s,
            Err(_) => return CErrorCode::InvalidParameter,
        };
        
        let builder = crate::core::quick_agent(name_str, instructions_str);
        
        match builder.build() {
            Ok(agent) => {
                let agent_box = Box::new(agent);
                *agent_out = Box::into_raw(agent_box) as CAgent;
                CErrorCode::Success
            }
            Err(_) => CErrorCode::RuntimeError,
        }
    }
}

/// 工具相关函数

/// 创建Web搜索工具
#[no_mangle]
pub extern "C" fn lumos_tool_web_search(tool_out: *mut CTool) -> CErrorCode {
    if tool_out.is_null() {
        return CErrorCode::InvalidParameter;
    }
    
    let tool = lumosai_core::tools::web::web_search();
    let metadata = crate::core::ToolMetadata {
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
    
    let cross_lang_tool = CrossLangTool::new(tool, metadata);
    let tool_box = Box::new(cross_lang_tool);
    
    unsafe {
        *tool_out = Box::into_raw(tool_box) as CTool;
    }
    
    CErrorCode::Success
}

/// 创建计算器工具
#[no_mangle]
pub extern "C" fn lumos_tool_calculator(tool_out: *mut CTool) -> CErrorCode {
    if tool_out.is_null() {
        return CErrorCode::InvalidParameter;
    }
    
    let tool = lumosai_core::tools::math::calculator();
    let metadata = crate::core::ToolMetadata {
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
    
    let cross_lang_tool = CrossLangTool::new(tool, metadata);
    let tool_box = Box::new(cross_lang_tool);
    
    unsafe {
        *tool_out = Box::into_raw(tool_box) as CTool;
    }
    
    CErrorCode::Success
}

/// 释放Tool
#[no_mangle]
pub extern "C" fn lumos_tool_free(tool: CTool) {
    if !tool.is_null() {
        unsafe {
            let _ = Box::from_raw(tool as *mut CrossLangTool);
        }
    }
}

/// 获取版本信息
#[no_mangle]
pub extern "C" fn lumos_version() -> *const c_char {
    static VERSION: &str = "0.1.0\0";
    VERSION.as_ptr() as *const c_char
}
