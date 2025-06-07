//! Weaviate filter conversion utilities

use lumosai_vector_core::prelude::{FilterCondition, MetadataValue, VectorError, Result};
use serde_json::{json, Value};

/// Convert a filter condition to Weaviate GraphQL where clause
pub fn convert_filter_to_where(condition: FilterCondition) -> Result<String> {
    let where_obj = convert_condition_to_object(condition)?;
    Ok(format!("{}", where_obj))
}

/// Convert a filter condition to a JSON object
fn convert_condition_to_object(condition: FilterCondition) -> Result<Value> {
    match condition {
        FilterCondition::Eq(field, value) => {
            let weaviate_value = convert_metadata_value(value)?;
            Ok(json!({
                "path": [field],
                "operator": "Equal",
                "valueText": weaviate_value
            }))
        },
        FilterCondition::And(conditions) => {
            let mut operands = Vec::new();
            for cond in conditions {
                operands.push(convert_condition_to_object(cond)?);
            }
            Ok(json!({
                "operator": "And",
                "operands": operands
            }))
        },
        FilterCondition::Or(conditions) => {
            let mut operands = Vec::new();
            for cond in conditions {
                operands.push(convert_condition_to_object(cond)?);
            }
            Ok(json!({
                "operator": "Or",
                "operands": operands
            }))
        },
        FilterCondition::Not(condition) => {
            let operand = convert_condition_to_object(*condition)?;
            Ok(json!({
                "operator": "Not",
                "operands": [operand]
            }))
        },
        FilterCondition::Gt(field, value) => {
            let weaviate_value = convert_metadata_value(value)?;
            Ok(json!({
                "path": [field],
                "operator": "GreaterThan",
                "valueNumber": weaviate_value
            }))
        },
        FilterCondition::Lt(field, value) => {
            let weaviate_value = convert_metadata_value(value)?;
            Ok(json!({
                "path": [field],
                "operator": "LessThan",
                "valueNumber": weaviate_value
            }))
        },
        FilterCondition::In(field, values) => {
            // Weaviate doesn't have a direct "In" operator, so we use Or with multiple Equal conditions
            let mut operands = Vec::new();
            for value in values {
                let weaviate_value = convert_metadata_value(value)?;
                operands.push(json!({
                    "path": [field],
                    "operator": "Equal",
                    "valueText": weaviate_value
                }));
            }
            Ok(json!({
                "operator": "Or",
                "operands": operands
            }))
        },
        _ => {
            Err(VectorError::InvalidFilter("Unsupported filter condition for Weaviate".to_string()))
        }
    }
}

/// Convert metadata value to Weaviate-compatible value
fn convert_metadata_value(value: MetadataValue) -> Result<Value> {
    match value {
        MetadataValue::String(s) => Ok(json!(s)),
        MetadataValue::Integer(i) => Ok(json!(i)),
        MetadataValue::Float(f) => Ok(json!(f)),
        MetadataValue::Boolean(b) => Ok(json!(b)),
        _ => Err(VectorError::Serialization(
            "Unsupported metadata value type for Weaviate".to_string()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_eq_filter() {
        let condition = FilterCondition::Eq("category".to_string(), MetadataValue::String("test".to_string()));
        let result = convert_filter_to_where(condition).unwrap();
        assert!(result.contains("Equal"));
        assert!(result.contains("category"));
    }
    
    #[test]
    fn test_and_filter() {
        let conditions = vec![
            FilterCondition::Eq("category".to_string(), MetadataValue::String("test".to_string())),
            FilterCondition::Gt("score".to_string(), MetadataValue::Float(0.5)),
        ];
        let condition = FilterCondition::And(conditions);
        let result = convert_filter_to_where(condition).unwrap();
        assert!(result.contains("And"));
    }
}
