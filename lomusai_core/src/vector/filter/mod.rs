use serde::{Serialize, Deserialize};

/// Filter condition for vector queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterCondition {
    /// Equals condition
    Eq(String, serde_json::Value),
    /// Greater than condition
    Gt(String, serde_json::Value),
    /// Less than condition
    Lt(String, serde_json::Value),
    /// In array condition
    In(String, Vec<serde_json::Value>),
    /// And condition
    And(Vec<FilterCondition>),
    /// Or condition
    Or(Vec<FilterCondition>),
    /// Not condition
    Not(Box<FilterCondition>),
}

/// Filter interpreter to evaluate filter conditions against metadata
pub struct FilterInterpreter;

impl FilterInterpreter {
    /// Create a new filter interpreter
    pub fn new() -> Self {
        Self {}
    }

    /// Evaluate a filter condition against metadata
    pub fn evaluate(&self, filter: &FilterCondition, metadata: &std::collections::HashMap<String, serde_json::Value>) -> bool {
        match filter {
            FilterCondition::Eq(field, value) => {
                metadata.get(field).map_or(false, |v| v == value)
            }
            FilterCondition::Gt(field, value) => {
                metadata.get(field).map_or(false, |v| {
                    Self::compare_json_values(v, value)
                        .map_or(false, |ord| ord == std::cmp::Ordering::Greater)
                })
            }
            FilterCondition::Lt(field, value) => {
                metadata.get(field).map_or(false, |v| {
                    Self::compare_json_values(v, value)
                        .map_or(false, |ord| ord == std::cmp::Ordering::Less)
                })
            }
            FilterCondition::In(field, values) => {
                metadata.get(field).map_or(false, |v| values.contains(v))
            }
            FilterCondition::And(conditions) => {
                conditions.iter().all(|c| self.evaluate(c, metadata))
            }
            FilterCondition::Or(conditions) => {
                conditions.iter().any(|c| self.evaluate(c, metadata))
            }
            FilterCondition::Not(condition) => {
                !self.evaluate(condition, metadata)
            }
        }
    }

    /// Compare two JSON values
    fn compare_json_values(a: &serde_json::Value, b: &serde_json::Value) -> Option<std::cmp::Ordering> {
        match (a, b) {
            (serde_json::Value::Number(a), serde_json::Value::Number(b)) => {
                if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
                    a.partial_cmp(&b)
                } else {
                    None
                }
            }
            (serde_json::Value::String(a), serde_json::Value::String(b)) => Some(a.cmp(b)),
            (serde_json::Value::Bool(a), serde_json::Value::Bool(b)) => Some(a.cmp(b)),
            _ => None,
        }
    }
} 