//! 向量存储过滤器模块
//! 提供条件过滤功能

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::Value;

/// 过滤条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterCondition {
    /// 等于
    Eq {
        /// 字段
        field_name: String,
        /// 值
        value: Value,
    },
    /// 不等于
    Ne {
        /// 字段
        field_name: String,
        /// 值
        value: Value,
    },
    /// 大于
    Gt {
        /// 字段
        field_name: String,
        /// 值
        value: Value,
    },
    /// 小于
    Lt {
        /// 字段
        field_name: String,
        /// 值
        value: Value,
    },
    /// 包含
    Contains {
        /// 字段
        field_name: String,
        /// 值
        value: Value,
    },
    /// 逻辑与
    And(Vec<FilterCondition>),
    /// 逻辑或
    Or(Vec<FilterCondition>),
}

/// 过滤器解释器
#[derive(Debug)]
pub struct FilterInterpreter;

impl Default for FilterInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterInterpreter {
    /// 创建新的过滤器解释器
    pub fn new() -> Self {
        Self
    }
    
    /// 评估过滤条件
    pub fn evaluate(&self, condition: &FilterCondition, metadata: &HashMap<String, Value>) -> bool {
        match condition {
            FilterCondition::Eq { field_name, value } => {
                if let Some(field_value) = metadata.get(field_name) {
                    field_value == value
                } else {
                    false
                }
            },
            FilterCondition::Ne { field_name, value } => {
                if let Some(field_value) = metadata.get(field_name) {
                    field_value != value
                } else {
                    true
                }
            },
            FilterCondition::Gt { field_name, value } => {
                if let Some(field_value) = metadata.get(field_name) {
                    self.compare_gt(field_value, value)
                } else {
                    false
                }
            },
            FilterCondition::Lt { field_name, value } => {
                if let Some(field_value) = metadata.get(field_name) {
                    self.compare_lt(field_value, value)
                } else {
                    false
                }
            },
            FilterCondition::Contains { field_name, value } => {
                if let Some(field_value) = metadata.get(field_name) {
                    self.check_contains(field_value, value)
                } else {
                    false
                }
            },
            FilterCondition::And(conditions) => {
                conditions.iter().all(|c| self.evaluate(c, metadata))
            },
            FilterCondition::Or(conditions) => {
                conditions.iter().any(|c| self.evaluate(c, metadata))
            },
        }
    }
    
    /// 比较大于
    fn compare_gt(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(a_num), Value::Number(b_num)) => {
                if let (Some(a_f64), Some(b_f64)) = (a_num.as_f64(), b_num.as_f64()) {
                    a_f64 > b_f64
                } else {
                    false
                }
            },
            (Value::String(a_str), Value::String(b_str)) => {
                a_str > b_str
            },
            _ => false,
        }
    }
    
    /// 比较小于
    fn compare_lt(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(a_num), Value::Number(b_num)) => {
                if let (Some(a_f64), Some(b_f64)) = (a_num.as_f64(), b_num.as_f64()) {
                    a_f64 < b_f64
                } else {
                    false
                }
            },
            (Value::String(a_str), Value::String(b_str)) => {
                a_str < b_str
            },
            _ => false,
        }
    }
    
    /// 检查包含
    fn check_contains(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::String(a_str), Value::String(b_str)) => {
                a_str.contains(b_str)
            },
            (Value::Array(a_arr), b_val) => {
                a_arr.contains(b_val)
            },
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_filter_eq() {
        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), json!("book"));
        metadata.insert("price".to_string(), json!(10.5));
        
        let interpreter = FilterInterpreter::new();
        
        let condition = FilterCondition::Eq {
            field_name: "category".to_string(),
            value: json!("book"),
        };
        assert!(interpreter.evaluate(&condition, &metadata));
        
        let condition = FilterCondition::Eq {
            field_name: "category".to_string(),
            value: json!("movie"),
        };
        assert!(!interpreter.evaluate(&condition, &metadata));
    }
    
    #[test]
    fn test_filter_complex() {
        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), json!("book"));
        metadata.insert("price".to_string(), json!(25.0));
        metadata.insert("tags".to_string(), json!(["fiction", "bestseller"]));
        
        let interpreter = FilterInterpreter::new();
        
        // (category = "book" AND price > 20.0) OR contains(tags, "bestseller")
        let condition = FilterCondition::Or(vec![
            FilterCondition::And(vec![
                FilterCondition::Eq {
                    field_name: "category".to_string(),
                    value: json!("book"),
                },
                FilterCondition::Gt {
                    field_name: "price".to_string(),
                    value: json!(20.0),
                },
            ]),
            FilterCondition::Contains {
                field_name: "tags".to_string(),
                value: json!("bestseller"),
            },
        ]);
        
        assert!(interpreter.evaluate(&condition, &metadata));
    }
} 