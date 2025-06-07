//! Filter conversion utilities for Qdrant

use qdrant_client::qdrant::{Filter, Condition, Match, Range, Value as QdrantValue};
use lumosai_vector_core::prelude::*;
use crate::{QdrantError, QdrantResult};

/// Qdrant filter converter
pub struct QdrantFilterConverter;

impl QdrantFilterConverter {
    /// Convert a filter condition to Qdrant filter
    pub fn convert_filter(condition: FilterCondition) -> QdrantResult<Filter> {
        match condition {
            FilterCondition::Eq(field, value) => {
                let qdrant_value = Self::convert_metadata_value(value)?;
                Ok(Filter {
                    must: Some(vec![Condition::field(field, Match::value(qdrant_value))]),
                    ..Default::default()
                })
            },
            FilterCondition::And(conditions) => {
                let mut must_conditions = Vec::new();
                for cond in conditions {
                    let filter = Self::convert_filter(cond)?;
                    if let Some(must) = filter.must {
                        must_conditions.extend(must);
                    }
                }
                Ok(Filter {
                    must: Some(must_conditions),
                    ..Default::default()
                })
            },
            FilterCondition::Or(conditions) => {
                let mut should_conditions = Vec::new();
                for cond in conditions {
                    let filter = Self::convert_filter(cond)?;
                    if let Some(must) = filter.must {
                        should_conditions.extend(must);
                    }
                }
                Ok(Filter {
                    should: Some(should_conditions),
                    ..Default::default()
                })
            },
            FilterCondition::Not(condition) => {
                let filter = Self::convert_filter(*condition)?;
                Ok(Filter {
                    must_not: filter.must,
                    ..Default::default()
                })
            },
            FilterCondition::Gt(field, value) => {
                let qdrant_value = Self::convert_metadata_value(value)?;
                Ok(Filter {
                    must: Some(vec![Condition::field(field, Range {
                        gt: Some(qdrant_value),
                        ..Default::default()
                    })]),
                    ..Default::default()
                })
            },
            FilterCondition::Gte(field, value) => {
                let qdrant_value = Self::convert_metadata_value(value)?;
                Ok(Filter {
                    must: Some(vec![Condition::field(field, Range {
                        gte: Some(qdrant_value),
                        ..Default::default()
                    })]),
                    ..Default::default()
                })
            },
            FilterCondition::Lt(field, value) => {
                let qdrant_value = Self::convert_metadata_value(value)?;
                Ok(Filter {
                    must: Some(vec![Condition::field(field, Range {
                        lt: Some(qdrant_value),
                        ..Default::default()
                    })]),
                    ..Default::default()
                })
            },
            FilterCondition::Lte(field, value) => {
                let qdrant_value = Self::convert_metadata_value(value)?;
                Ok(Filter {
                    must: Some(vec![Condition::field(field, Range {
                        lte: Some(qdrant_value),
                        ..Default::default()
                    })]),
                    ..Default::default()
                })
            },
            FilterCondition::In(field, values) => {
                let qdrant_values: QdrantResult<Vec<_>> = values.into_iter()
                    .map(Self::convert_metadata_value)
                    .collect();
                Ok(Filter {
                    must: Some(vec![Condition::field(field, Match::any(qdrant_values?))]),
                    ..Default::default()
                })
            },
            FilterCondition::NotIn(field, values) => {
                let qdrant_values: QdrantResult<Vec<_>> = values.into_iter()
                    .map(Self::convert_metadata_value)
                    .collect();
                Ok(Filter {
                    must_not: Some(vec![Condition::field(field, Match::any(qdrant_values?))]),
                    ..Default::default()
                })
            },
            FilterCondition::Contains(field, value) => {
                // For string contains, we can use text matching
                if let MetadataValue::String(text) = value {
                    Ok(Filter {
                        must: Some(vec![Condition::field(field, Match::text(text))]),
                        ..Default::default()
                    })
                } else {
                    Err(QdrantError::Search("Contains filter only supports string values".to_string()))
                }
            },
            FilterCondition::StartsWith(field, value) => {
                // Qdrant doesn't have native startsWith, so we use text matching
                if let MetadataValue::String(text) = value {
                    Ok(Filter {
                        must: Some(vec![Condition::field(field, Match::text(text))]),
                        ..Default::default()
                    })
                } else {
                    Err(QdrantError::Search("StartsWith filter only supports string values".to_string()))
                }
            },
            FilterCondition::EndsWith(field, value) => {
                // Qdrant doesn't have native endsWith, so we use text matching
                if let MetadataValue::String(text) = value {
                    Ok(Filter {
                        must: Some(vec![Condition::field(field, Match::text(text))]),
                        ..Default::default()
                    })
                } else {
                    Err(QdrantError::Search("EndsWith filter only supports string values".to_string()))
                }
            },
            FilterCondition::IsNull(field) => {
                Ok(Filter {
                    must: Some(vec![Condition::is_null(field)]),
                    ..Default::default()
                })
            },
            FilterCondition::IsNotNull(field) => {
                Ok(Filter {
                    must_not: Some(vec![Condition::is_null(field)]),
                    ..Default::default()
                })
            },
        }
    }
    
    /// Convert metadata value to Qdrant value
    fn convert_metadata_value(value: MetadataValue) -> QdrantResult<QdrantValue> {
        let qdrant_value = match value {
            MetadataValue::String(s) => QdrantValue {
                kind: Some(qdrant_client::qdrant::value::Kind::StringValue(s)),
            },
            MetadataValue::Integer(i) => QdrantValue {
                kind: Some(qdrant_client::qdrant::value::Kind::IntegerValue(i)),
            },
            MetadataValue::Float(f) => QdrantValue {
                kind: Some(qdrant_client::qdrant::value::Kind::DoubleValue(f)),
            },
            MetadataValue::Boolean(b) => QdrantValue {
                kind: Some(qdrant_client::qdrant::value::Kind::BoolValue(b)),
            },
            MetadataValue::Null => QdrantValue {
                kind: Some(qdrant_client::qdrant::value::Kind::NullValue(0)),
            },
            MetadataValue::Array(_) => {
                return Err(QdrantError::Serialization(
                    "Array metadata values not supported in filters".to_string()
                ));
            },
            MetadataValue::Object(_) => {
                return Err(QdrantError::Serialization(
                    "Object metadata values not supported in filters".to_string()
                ));
            },
        };
        Ok(qdrant_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_eq_filter() {
        let condition = FilterCondition::Eq("category".to_string(), MetadataValue::String("test".to_string()));
        let filter = QdrantFilterConverter::convert_filter(condition).unwrap();
        
        assert!(filter.must.is_some());
        assert_eq!(filter.must.unwrap().len(), 1);
    }
    
    #[test]
    fn test_and_filter() {
        let conditions = vec![
            FilterCondition::Eq("category".to_string(), MetadataValue::String("test".to_string())),
            FilterCondition::Gt("score".to_string(), MetadataValue::Float(0.5)),
        ];
        let condition = FilterCondition::And(conditions);
        let filter = QdrantFilterConverter::convert_filter(condition).unwrap();
        
        assert!(filter.must.is_some());
        assert_eq!(filter.must.unwrap().len(), 2);
    }
    
    #[test]
    fn test_or_filter() {
        let conditions = vec![
            FilterCondition::Eq("category".to_string(), MetadataValue::String("test1".to_string())),
            FilterCondition::Eq("category".to_string(), MetadataValue::String("test2".to_string())),
        ];
        let condition = FilterCondition::Or(conditions);
        let filter = QdrantFilterConverter::convert_filter(condition).unwrap();
        
        assert!(filter.should.is_some());
        assert_eq!(filter.should.unwrap().len(), 2);
    }
    
    #[test]
    fn test_range_filters() {
        let gt_condition = FilterCondition::Gt("score".to_string(), MetadataValue::Float(0.5));
        let gt_filter = QdrantFilterConverter::convert_filter(gt_condition).unwrap();
        assert!(gt_filter.must.is_some());
        
        let lt_condition = FilterCondition::Lt("score".to_string(), MetadataValue::Float(0.9));
        let lt_filter = QdrantFilterConverter::convert_filter(lt_condition).unwrap();
        assert!(lt_filter.must.is_some());
    }
    
    #[test]
    fn test_in_filter() {
        let values = vec![
            MetadataValue::String("cat1".to_string()),
            MetadataValue::String("cat2".to_string()),
        ];
        let condition = FilterCondition::In("category".to_string(), values);
        let filter = QdrantFilterConverter::convert_filter(condition).unwrap();
        
        assert!(filter.must.is_some());
        assert_eq!(filter.must.unwrap().len(), 1);
    }
}
