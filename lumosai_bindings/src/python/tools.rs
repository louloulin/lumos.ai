//! Python工具绑定
//! 
//! 为Python提供Lumos.ai工具的绑定支持

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::sync::Arc;
use lumosai_core::tools;
use crate::core::{CrossLangTool, ToolMetadata};
use crate::python::PyTool;

/// 注册所有工具到Python模块
pub fn register_tools(module: &PyModule) -> PyResult<()> {
    // Web工具
    module.add_function(wrap_pyfunction!(web_search, module)?)?;
    module.add_function(wrap_pyfunction!(http_request, module)?)?;
    module.add_function(wrap_pyfunction!(url_extractor, module)?)?;
    
    // 文件工具
    module.add_function(wrap_pyfunction!(file_reader, module)?)?;
    module.add_function(wrap_pyfunction!(file_writer, module)?)?;
    module.add_function(wrap_pyfunction!(directory_scanner, module)?)?;
    
    // 数据处理工具
    module.add_function(wrap_pyfunction!(json_processor, module)?)?;
    module.add_function(wrap_pyfunction!(csv_processor, module)?)?;
    module.add_function(wrap_pyfunction!(xml_processor, module)?)?;
    
    // 计算工具
    module.add_function(wrap_pyfunction!(calculator, module)?)?;
    module.add_function(wrap_pyfunction!(math_evaluator, module)?)?;
    
    // 系统工具
    module.add_function(wrap_pyfunction!(shell_executor, module)?)?;
    module.add_function(wrap_pyfunction!(environment_reader, module)?)?;
    
    // 网络工具
    module.add_function(wrap_pyfunction!(ping_tool, module)?)?;
    module.add_function(wrap_pyfunction!(dns_resolver, module)?)?;
    
    // 时间工具
    module.add_function(wrap_pyfunction!(datetime_formatter, module)?)?;
    module.add_function(wrap_pyfunction!(timezone_converter, module)?)?;
    
    Ok(())
}

/// Web搜索工具
#[pyfunction]
#[pyo3(text_signature = "()")]
pub fn web_search() -> PyResult<PyTool> {
    let tool = tools::web::web_search();
    let metadata = ToolMetadata {
        name: "web_search".to_string(),
        description: "搜索网络内容".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "搜索查询"
                },
                "max_results": {
                    "type": "integer",
                    "description": "最大结果数",
                    "default": 10
                }
            },
            "required": ["query"]
        }),
        tool_type: "web".to_string(),
        is_async: true,
    };
    
    Ok(PyTool {
        inner: CrossLangTool::new(tool, metadata),
    })
}

/// HTTP请求工具
#[pyfunction]
#[pyo3(text_signature = "()")]
pub fn http_request() -> PyResult<PyTool> {
    let tool = tools::web::http_request();
    let metadata = ToolMetadata {
        name: "http_request".to_string(),
        description: "发送HTTP请求".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "请求URL"
                },
                "method": {
                    "type": "string",
                    "description": "HTTP方法",
                    "enum": ["GET", "POST", "PUT", "DELETE"],
                    "default": "GET"
                },
                "headers": {
                    "type": "object",
                    "description": "请求头"
                },
                "body": {
                    "type": "string",
                    "description": "请求体"
                }
            },
            "required": ["url"]
        }),
        tool_type: "web".to_string(),
        is_async: true,
    };
    
    Ok(PyTool {
        inner: CrossLangTool::new(tool, metadata),
    })
}

/// URL提取工具
#[pyfunction]
#[pyo3(text_signature = "()")]
pub fn url_extractor() -> PyResult<PyTool> {
    let tool = tools::web::url_extractor();
    let metadata = ToolMetadata {
        name: "url_extractor".to_string(),
        description: "从文本中提取URL".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "输入文本"
                }
            },
            "required": ["text"]
        }),
        tool_type: "text".to_string(),
        is_async: false,
    };
    
    Ok(PyTool {
        inner: CrossLangTool::new(tool, metadata),
    })
}

/// 文件读取工具
#[pyfunction]
#[pyo3(text_signature = "()")]
pub fn file_reader() -> PyResult<PyTool> {
    let tool = tools::file::file_reader();
    let metadata = ToolMetadata {
        name: "file_reader".to_string(),
        description: "读取文件内容".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "文件路径"
                },
                "encoding": {
                    "type": "string",
                    "description": "文件编码",
                    "default": "utf-8"
                }
            },
            "required": ["path"]
        }),
        tool_type: "file".to_string(),
        is_async: true,
    };
    
    Ok(PyTool {
        inner: CrossLangTool::new(tool, metadata),
    })
}

/// 文件写入工具
#[pyfunction]
#[pyo3(text_signature = "()")]
pub fn file_writer() -> PyResult<PyTool> {
    let tool = tools::file::file_writer();
    let metadata = ToolMetadata {
        name: "file_writer".to_string(),
        description: "写入文件内容".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "文件路径"
                },
                "content": {
                    "type": "string",
                    "description": "文件内容"
                },
                "append": {
                    "type": "boolean",
                    "description": "是否追加",
                    "default": false
                }
            },
            "required": ["path", "content"]
        }),
        tool_type: "file".to_string(),
        is_async: true,
    };
    
    Ok(PyTool {
        inner: CrossLangTool::new(tool, metadata),
    })
}

/// 目录扫描工具
#[pyfunction]
#[pyo3(text_signature = "()")]
pub fn directory_scanner() -> PyResult<PyTool> {
    let tool = tools::file::directory_scanner();
    let metadata = ToolMetadata {
        name: "directory_scanner".to_string(),
        description: "扫描目录内容".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "目录路径"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "是否递归扫描",
                    "default": false
                },
                "pattern": {
                    "type": "string",
                    "description": "文件名模式"
                }
            },
            "required": ["path"]
        }),
        tool_type: "file".to_string(),
        is_async: true,
    };
    
    Ok(PyTool {
        inner: CrossLangTool::new(tool, metadata),
    })
}

/// JSON处理工具
#[pyfunction]
#[pyo3(text_signature = "()")]
pub fn json_processor() -> PyResult<PyTool> {
    let tool = tools::data::json_processor();
    let metadata = ToolMetadata {
        name: "json_processor".to_string(),
        description: "处理JSON数据".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "data": {
                    "type": "string",
                    "description": "JSON数据"
                },
                "operation": {
                    "type": "string",
                    "description": "操作类型",
                    "enum": ["parse", "stringify", "validate", "format"]
                }
            },
            "required": ["data", "operation"]
        }),
        tool_type: "data".to_string(),
        is_async: false,
    };
    
    Ok(PyTool {
        inner: CrossLangTool::new(tool, metadata),
    })
}

/// CSV处理工具
#[pyfunction]
#[pyo3(text_signature = "()")]
pub fn csv_processor() -> PyResult<PyTool> {
    let tool = tools::data::csv_processor();
    let metadata = ToolMetadata {
        name: "csv_processor".to_string(),
        description: "处理CSV数据".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "data": {
                    "type": "string",
                    "description": "CSV数据"
                },
                "operation": {
                    "type": "string",
                    "description": "操作类型",
                    "enum": ["parse", "format", "filter", "sort"]
                },
                "delimiter": {
                    "type": "string",
                    "description": "分隔符",
                    "default": ","
                }
            },
            "required": ["data", "operation"]
        }),
        tool_type: "data".to_string(),
        is_async: false,
    };
    
    Ok(PyTool {
        inner: CrossLangTool::new(tool, metadata),
    })
}

/// XML处理工具
#[pyfunction]
#[pyo3(text_signature = "()")]
pub fn xml_processor() -> PyResult<PyTool> {
    let tool = tools::data::xml_processor();
    let metadata = ToolMetadata {
        name: "xml_processor".to_string(),
        description: "处理XML数据".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "data": {
                    "type": "string",
                    "description": "XML数据"
                },
                "operation": {
                    "type": "string",
                    "description": "操作类型",
                    "enum": ["parse", "validate", "format", "extract"]
                },
                "xpath": {
                    "type": "string",
                    "description": "XPath表达式"
                }
            },
            "required": ["data", "operation"]
        }),
        tool_type: "data".to_string(),
        is_async: false,
    };
    
    Ok(PyTool {
        inner: CrossLangTool::new(tool, metadata),
    })
}

/// 计算器工具
#[pyfunction]
#[pyo3(text_signature = "()")]
pub fn calculator() -> PyResult<PyTool> {
    let tool = tools::math::calculator();
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
    
    Ok(PyTool {
        inner: CrossLangTool::new(tool, metadata),
    })
}

/// 数学表达式求值工具
#[pyfunction]
#[pyo3(text_signature = "()")]
pub fn math_evaluator() -> PyResult<PyTool> {
    let tool = tools::math::math_evaluator();
    let metadata = ToolMetadata {
        name: "math_evaluator".to_string(),
        description: "高级数学表达式求值".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "数学表达式"
                },
                "variables": {
                    "type": "object",
                    "description": "变量定义"
                }
            },
            "required": ["expression"]
        }),
        tool_type: "math".to_string(),
        is_async: false,
    };
    
    Ok(PyTool {
        inner: CrossLangTool::new(tool, metadata),
    })
}

/// Shell执行工具
#[pyfunction]
#[pyo3(text_signature = "()")]
pub fn shell_executor() -> PyResult<PyTool> {
    let tool = tools::system::shell_executor();
    let metadata = ToolMetadata {
        name: "shell_executor".to_string(),
        description: "执行Shell命令".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Shell命令"
                },
                "working_dir": {
                    "type": "string",
                    "description": "工作目录"
                },
                "timeout": {
                    "type": "integer",
                    "description": "超时时间（秒）",
                    "default": 30
                }
            },
            "required": ["command"]
        }),
        tool_type: "system".to_string(),
        is_async: true,
    };
    
    Ok(PyTool {
        inner: CrossLangTool::new(tool, metadata),
    })
}

/// 环境变量读取工具
#[pyfunction]
#[pyo3(text_signature = "()")]
pub fn environment_reader() -> PyResult<PyTool> {
    let tool = tools::system::environment_reader();
    let metadata = ToolMetadata {
        name: "environment_reader".to_string(),
        description: "读取环境变量".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "variable": {
                    "type": "string",
                    "description": "环境变量名"
                }
            },
            "required": ["variable"]
        }),
        tool_type: "system".to_string(),
        is_async: false,
    };
    
    Ok(PyTool {
        inner: CrossLangTool::new(tool, metadata),
    })
}

/// Ping工具
#[pyfunction]
#[pyo3(text_signature = "()")]
pub fn ping_tool() -> PyResult<PyTool> {
    let tool = tools::network::ping_tool();
    let metadata = ToolMetadata {
        name: "ping_tool".to_string(),
        description: "网络连通性测试".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "host": {
                    "type": "string",
                    "description": "目标主机"
                },
                "count": {
                    "type": "integer",
                    "description": "ping次数",
                    "default": 4
                }
            },
            "required": ["host"]
        }),
        tool_type: "network".to_string(),
        is_async: true,
    };
    
    Ok(PyTool {
        inner: CrossLangTool::new(tool, metadata),
    })
}

/// DNS解析工具
#[pyfunction]
#[pyo3(text_signature = "()")]
pub fn dns_resolver() -> PyResult<PyTool> {
    let tool = tools::network::dns_resolver();
    let metadata = ToolMetadata {
        name: "dns_resolver".to_string(),
        description: "DNS域名解析".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "domain": {
                    "type": "string",
                    "description": "域名"
                },
                "record_type": {
                    "type": "string",
                    "description": "记录类型",
                    "enum": ["A", "AAAA", "CNAME", "MX", "TXT"],
                    "default": "A"
                }
            },
            "required": ["domain"]
        }),
        tool_type: "network".to_string(),
        is_async: true,
    };
    
    Ok(PyTool {
        inner: CrossLangTool::new(tool, metadata),
    })
}

/// 日期时间格式化工具
#[pyfunction]
#[pyo3(text_signature = "()")]
pub fn datetime_formatter() -> PyResult<PyTool> {
    let tool = tools::time::datetime_formatter();
    let metadata = ToolMetadata {
        name: "datetime_formatter".to_string(),
        description: "日期时间格式化".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "datetime": {
                    "type": "string",
                    "description": "日期时间字符串"
                },
                "format": {
                    "type": "string",
                    "description": "输出格式",
                    "default": "%Y-%m-%d %H:%M:%S"
                }
            },
            "required": ["datetime"]
        }),
        tool_type: "time".to_string(),
        is_async: false,
    };
    
    Ok(PyTool {
        inner: CrossLangTool::new(tool, metadata),
    })
}

/// 时区转换工具
#[pyfunction]
#[pyo3(text_signature = "()")]
pub fn timezone_converter() -> PyResult<PyTool> {
    let tool = tools::time::timezone_converter();
    let metadata = ToolMetadata {
        name: "timezone_converter".to_string(),
        description: "时区转换".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "datetime": {
                    "type": "string",
                    "description": "日期时间字符串"
                },
                "from_timezone": {
                    "type": "string",
                    "description": "源时区"
                },
                "to_timezone": {
                    "type": "string",
                    "description": "目标时区"
                }
            },
            "required": ["datetime", "from_timezone", "to_timezone"]
        }),
        tool_type: "time".to_string(),
        is_async: false,
    };
    
    Ok(PyTool {
        inner: CrossLangTool::new(tool, metadata),
    })
}
