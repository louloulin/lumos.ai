// 基于nom的宏解析器 - 重新实现
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, multispace0, alpha1, alphanumeric1},
    combinator::{map, recognize},
    multi::{many0, separated_list0},
    sequence::{delimited, pair},
    IResult,
};
use std::collections::HashMap;

/// 解析结果类型
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Identifier(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Expression(String),
}

/// Agent定义结构
#[derive(Debug, Clone)]
pub struct AgentDef {
    pub name: String,
    pub instructions: String,
    pub provider: String,
    pub tools: Vec<String>,
}

/// Tool定义结构
#[derive(Debug, Clone)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ParameterDef>,
    pub handler: String,
}

/// 参数定义结构
#[derive(Debug, Clone)]
pub struct ParameterDef {
    pub name: String,
    pub param_type: String,
    pub description: String,
    pub required: bool,
}

// 基础解析器

/// 解析标识符
fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((tag("_"), alpha1)),
        many0(alt((
            alphanumeric1,
            tag("_"),
        ))),
    ))(input)
}

/// 解析字符串字面量
fn string_literal(input: &str) -> IResult<&str, String> {
    delimited(
        char('"'),
        map(take_while(|c| c != '"'), |s: &str| s.to_string()),
        char('"'),
    )(input)
}

/// 解析表达式（简化版本，支持函数调用）
fn expression(input: &str) -> IResult<&str, String> {
    // 解析到逗号、右括号或结束为止
    let (input, expr) = take_while1(|c: char| {
        c != ',' && c != '}' && c != ']' && c != '\n'
    })(input)?;
    Ok((input, expr.trim().to_string()))
}

/// 解析数组
fn array(input: &str) -> IResult<&str, Vec<Value>> {
    delimited(
        char('['),
        separated_list0(
            delimited(multispace0, char(','), multispace0),
            delimited(multispace0, value, multispace0),
        ),
        char(']'),
    )(input)
}

/// 解析值
fn value(input: &str) -> IResult<&str, Value> {
    alt((
        map(string_literal, Value::String),
        map(array, Value::Array),
        map(expression, |expr| {
            if expr.chars().all(|c| c.is_alphanumeric() || c == '_') {
                Value::Identifier(expr)
            } else {
                Value::Expression(expr)
            }
        }),
    ))(input)
}

/// 解析键值对
fn key_value_pair(input: &str) -> IResult<&str, (String, Value)> {
    let (input, key) = identifier(input)?;
    let (input, _) = delimited(multispace0, char(':'), multispace0)(input)?;
    let (input, val) = value(input)?;
    Ok((input, (key.to_string(), val)))
}

/// 解析对象
fn object(input: &str) -> IResult<&str, HashMap<String, Value>> {
    let (input, pairs) = delimited(
        char('{'),
        separated_list0(
            delimited(multispace0, char(','), multispace0),
            delimited(multispace0, key_value_pair, multispace0),
        ),
        char('}'),
    )(input)?;
    
    let mut map = HashMap::new();
    for (key, value) in pairs {
        map.insert(key, value);
    }
    Ok((input, map))
}

// 高级解析器

/// 解析agent!宏
pub fn parse_agent_macro(input: &str) -> Result<AgentDef, String> {
    let result = delimited(multispace0, object, multispace0)(input);
    
    match result {
        Ok((remaining, obj)) => {
            if !remaining.trim().is_empty() {
                return Err(format!("Unexpected content after agent definition: {}", remaining));
            }
            
            let name = extract_string(&obj, "name")?;
            let instructions = extract_string(&obj, "instructions")?;
            let provider = extract_expression(&obj, "provider")?;
            let tools = extract_tool_list(&obj, "tools")?;
            
            Ok(AgentDef {
                name,
                instructions,
                provider,
                tools,
            })
        }
        Err(e) => Err(format!("Failed to parse agent macro: {:?}", e)),
    }
}

/// 解析tool!宏
pub fn parse_tool_macro(input: &str) -> Result<ToolDef, String> {
    let result = delimited(multispace0, object, multispace0)(input);
    
    match result {
        Ok((remaining, obj)) => {
            if !remaining.trim().is_empty() {
                return Err(format!("Unexpected content after tool definition: {}", remaining));
            }
            
            let name = extract_string(&obj, "name")?;
            let description = extract_string(&obj, "description")?;
            let handler = extract_expression(&obj, "handler")?;
            
            // 解析参数（如果存在）
            let parameters = if let Some(Value::Array(params)) = obj.get("parameters") {
                parse_parameters(params)?
            } else {
                Vec::new()
            };
            
            Ok(ToolDef {
                name,
                description,
                parameters,
                handler,
            })
        }
        Err(e) => Err(format!("Failed to parse tool macro: {:?}", e)),
    }
}

// 辅助函数

fn extract_string(obj: &HashMap<String, Value>, key: &str) -> Result<String, String> {
    match obj.get(key) {
        Some(Value::String(s)) => Ok(s.clone()),
        Some(_) => Err(format!("Field '{}' must be a string", key)),
        None => Err(format!("Missing required field '{}'", key)),
    }
}

fn extract_expression(obj: &HashMap<String, Value>, key: &str) -> Result<String, String> {
    match obj.get(key) {
        Some(Value::Expression(e)) => Ok(e.clone()),
        Some(Value::Identifier(i)) => Ok(i.clone()),
        Some(_) => Err(format!("Field '{}' must be an expression or identifier", key)),
        None => Err(format!("Missing required field '{}'", key)),
    }
}

fn extract_tool_list(obj: &HashMap<String, Value>, key: &str) -> Result<Vec<String>, String> {
    match obj.get(key) {
        Some(Value::Array(arr)) => {
            let mut tools = Vec::new();
            for item in arr {
                match item {
                    Value::Identifier(name) => tools.push(name.clone()),
                    _ => return Err("Tool list must contain only identifiers".to_string()),
                }
            }
            Ok(tools)
        }
        Some(_) => Err(format!("Field '{}' must be an array", key)),
        None => Ok(Vec::new()), // 工具列表是可选的
    }
}

fn parse_parameters(params: &[Value]) -> Result<Vec<ParameterDef>, String> {
    let mut parameters = Vec::new();
    
    for param in params {
        match param {
            Value::Object(param_obj) => {
                let name = extract_string(param_obj, "name")?;
                let param_type = extract_string(param_obj, "type")?;
                let description = extract_string(param_obj, "description")?;
                let required = param_obj.get("required")
                    .map(|v| match v {
                        Value::Identifier(s) => s == "true",
                        _ => false,
                    })
                    .unwrap_or(true);
                
                parameters.push(ParameterDef {
                    name,
                    param_type,
                    description,
                    required,
                });
            }
            _ => return Err("Parameters must be objects".to_string()),
        }
    }
    
    Ok(parameters)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_agent() {
        let input = r#"{
            name: "test_agent",
            instructions: "Test instructions",
            provider: create_provider(),
            tools: [tool1, tool2]
        }"#;
        
        let result = parse_agent_macro(input);
        assert!(result.is_ok());
        
        let agent = result.unwrap();
        assert_eq!(agent.name, "test_agent");
        assert_eq!(agent.instructions, "Test instructions");
        assert_eq!(agent.provider, "create_provider()");
        assert_eq!(agent.tools, vec!["tool1", "tool2"]);
    }

    #[test]
    fn test_parse_simple_tool() {
        let input = r#"{
            name: "test_tool",
            description: "Test tool description",
            handler: handle_test
        }"#;
        
        let result = parse_tool_macro(input);
        assert!(result.is_ok());
        
        let tool = result.unwrap();
        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.description, "Test tool description");
        assert_eq!(tool.handler, "handle_test");
    }
}
