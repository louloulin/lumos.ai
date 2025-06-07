//! Filter conversion utilities for Qdrant

use qdrant_client::qdrant::{
    Filter, Condition, Range, Value as QdrantValue, Match, FieldCondition,
    condition::ConditionOneOf, r#match::MatchValue, RepeatedStrings, RepeatedIntegers,
};
use lumosai_vector_core::prelude::*;
use crate::{QdrantError};
use crate::error::QdrantResult;

/// Qdrant filter converter
pub struct QdrantFilterConverter;

impl QdrantFilterConverter {
    /// Convert a filter condition to Qdrant filter
    pub fn convert_filter(condition: FilterCondition) -> QdrantResult<Filter> {
        match condition {
            FilterCondition::Eq(field, value) => {
                let match_value = Self::convert_metadata_value(value)?;
                Ok(Filter {
                    must: vec![Condition {
                        condition_one_of: Some(ConditionOneOf::Field(FieldCondition {
                            key: field,
                            r#match: Some(Match {
                                match_value: Some(match_value),
                            }),
                            range: None,
                            geo_bounding_box: None,
                            geo_radius: None,
                            values_count: None,
                            geo_polygon: None,
                            datetime_range: None,
                            is_empty: None,
                            is_null: None,
                        })),
                    }],
                    ..Default::default()
                })
            },
            FilterCondition::Ne(field, value) => {
                let match_value = Self::convert_metadata_value(value)?;
                Ok(Filter {
                    must_not: vec![Condition {
                        condition_one_of: Some(ConditionOneOf::Field(FieldCondition {
                            key: field,
                            r#match: Some(Match {
                                match_value: Some(match_value),
                            }),
                            range: None,
                            geo_bounding_box: None,
                            geo_radius: None,
                            values_count: None,
                            geo_polygon: None,
                            datetime_range: None,
                            is_empty: None,
                            is_null: None,
                        })),
                    }],
                    ..Default::default()
                })
            },
            FilterCondition::Gt(field, value) => {
                let range_value = Self::convert_to_range_value(value)?;
                Ok(Filter {
                    must: vec![Condition {
                        condition_one_of: Some(ConditionOneOf::Field(FieldCondition {
                            key: field,
                            r#match: None,
                            range: Some(Range {
                                gt: Some(range_value),
                                ..Default::default()
                            }),
                            geo_bounding_box: None,
                            geo_radius: None,
                            values_count: None,
                            geo_polygon: None,
                            datetime_range: None,
                            is_empty: None,
                            is_null: None,
                        })),
                    }],
                    ..Default::default()
                })
            },
            FilterCondition::Gte(field, value) => {
                let range_value = Self::convert_to_range_value(value)?;
                Ok(Filter {
                    must: vec![Condition {
                        condition_one_of: Some(ConditionOneOf::Field(FieldCondition {
                            key: field,
                            r#match: None,
                            range: Some(Range {
                                gte: Some(range_value),
                                ..Default::default()
                            }),
                            geo_bounding_box: None,
                            geo_radius: None,
                            values_count: None,
                            geo_polygon: None,
                            datetime_range: None,
                            is_empty: None,
                            is_null: None,
                        })),
                    }],
                    ..Default::default()
                })
            },
            FilterCondition::Lt(field, value) => {
                let range_value = Self::convert_to_range_value(value)?;
                Ok(Filter {
                    must: vec![Condition {
                        condition_one_of: Some(ConditionOneOf::Field(FieldCondition {
                            key: field,
                            r#match: None,
                            range: Some(Range {
                                lt: Some(range_value),
                                ..Default::default()
                            }),
                            geo_bounding_box: None,
                            geo_radius: None,
                            values_count: None,
                            geo_polygon: None,
                            datetime_range: None,
                            is_empty: None,
                            is_null: None,
                        })),
                    }],
                    ..Default::default()
                })
            },
            FilterCondition::Lte(field, value) => {
                let range_value = Self::convert_to_range_value(value)?;
                Ok(Filter {
                    must: vec![Condition {
                        condition_one_of: Some(ConditionOneOf::Field(FieldCondition {
                            key: field,
                            r#match: None,
                            range: Some(Range {
                                lte: Some(range_value),
                                ..Default::default()
                            }),
                            geo_bounding_box: None,
                            geo_radius: None,
                            values_count: None,
                            geo_polygon: None,
                            datetime_range: None,
                            is_empty: None,
                            is_null: None,
                        })),
                    }],
                    ..Default::default()
                })
            },
            FilterCondition::In(field, values) => {
                // Convert values to appropriate MatchValue based on type
                let match_value = if values.is_empty() {
                    return Err(QdrantError::InvalidFilter("Empty values list not allowed".to_string()));
                } else {
                    // Check the type of the first value to determine the collection type
                    match &values[0] {
                    MetadataValue::String(_) => {
                        let strings: std::result::Result<Vec<String>, QdrantError> = values.into_iter()
                            .map(|v| match v {
                                MetadataValue::String(s) => Ok(s),
                                _ => Err(QdrantError::InvalidFilter("Mixed types in values list".to_string())),
                            })
                            .collect();
                        MatchValue::Keywords(RepeatedStrings { strings: strings? })
                    },
                    MetadataValue::Integer(_) => {
                        let integers: std::result::Result<Vec<i64>, QdrantError> = values.into_iter()
                            .map(|v| match v {
                                MetadataValue::Integer(i) => Ok(i),
                                _ => Err(QdrantError::InvalidFilter("Mixed types in values list".to_string())),
                            })
                            .collect();
                        MatchValue::Integers(RepeatedIntegers { integers: integers? })
                    },
                        _ => return Err(QdrantError::InvalidFilter("Unsupported value type for In filter".to_string())),
                    }
                };

                Ok(Filter {
                    must: vec![Condition {
                        condition_one_of: Some(ConditionOneOf::Field(FieldCondition {
                            key: field,
                            r#match: Some(Match {
                                match_value: Some(match_value),
                            }),
                            range: None,
                            geo_bounding_box: None,
                            geo_radius: None,
                            values_count: None,
                            geo_polygon: None,
                            datetime_range: None,
                            is_empty: None,
                            is_null: None,
                        })),
                    }],
                    ..Default::default()
                })
            },
            FilterCondition::NotIn(field, values) => {
                // Convert values to appropriate MatchValue based on type
                let match_value = if values.is_empty() {
                    return Err(QdrantError::InvalidFilter("Empty values list not allowed".to_string()));
                } else {
                    // Check the type of the first value to determine the collection type
                    match &values[0] {
                    MetadataValue::String(_) => {
                        let strings: std::result::Result<Vec<String>, QdrantError> = values.into_iter()
                            .map(|v| match v {
                                MetadataValue::String(s) => Ok(s),
                                _ => Err(QdrantError::InvalidFilter("Mixed types in values list".to_string())),
                            })
                            .collect();
                        MatchValue::ExceptKeywords(RepeatedStrings { strings: strings? })
                    },
                    MetadataValue::Integer(_) => {
                        let integers: std::result::Result<Vec<i64>, QdrantError> = values.into_iter()
                            .map(|v| match v {
                                MetadataValue::Integer(i) => Ok(i),
                                _ => Err(QdrantError::InvalidFilter("Mixed types in values list".to_string())),
                            })
                            .collect();
                        MatchValue::ExceptIntegers(RepeatedIntegers { integers: integers? })
                    },
                        _ => return Err(QdrantError::InvalidFilter("Unsupported value type for NotIn filter".to_string())),
                    }
                };

                Ok(Filter {
                    must: vec![Condition {
                        condition_one_of: Some(ConditionOneOf::Field(FieldCondition {
                            key: field,
                            r#match: Some(Match {
                                match_value: Some(match_value),
                            }),
                            range: None,
                            geo_bounding_box: None,
                            geo_radius: None,
                            values_count: None,
                            geo_polygon: None,
                            datetime_range: None,
                            is_empty: None,
                            is_null: None,
                        })),
                    }],
                    ..Default::default()
                })
            },
            FilterCondition::Contains(field, text) => {
                Ok(Filter {
                    must: vec![Condition {
                        condition_one_of: Some(ConditionOneOf::Field(FieldCondition {
                            key: field,
                            r#match: Some(Match {
                                match_value: Some(MatchValue::Text(text)),
                            }),
                            range: None,
                            geo_bounding_box: None,
                            geo_radius: None,
                            values_count: None,
                            geo_polygon: None,
                            datetime_range: None,
                            is_empty: None,
                            is_null: None,
                        })),
                    }],
                    ..Default::default()
                })
            },
            FilterCondition::StartsWith(field, text) => {
                // Qdrant doesn't have native startsWith, use text matching as approximation
                Ok(Filter {
                    must: vec![Condition {
                        condition_one_of: Some(ConditionOneOf::Field(FieldCondition {
                            key: field,
                            r#match: Some(Match {
                                match_value: Some(MatchValue::Text(text)),
                            }),
                            range: None,
                            geo_bounding_box: None,
                            geo_radius: None,
                            values_count: None,
                            geo_polygon: None,
                            datetime_range: None,
                            is_empty: None,
                            is_null: None,
                        })),
                    }],
                    ..Default::default()
                })
            },
            FilterCondition::EndsWith(field, text) => {
                // Qdrant doesn't have native endsWith, use text matching as approximation
                Ok(Filter {
                    must: vec![Condition {
                        condition_one_of: Some(ConditionOneOf::Field(FieldCondition {
                            key: field,
                            r#match: Some(Match {
                                match_value: Some(MatchValue::Text(text)),
                            }),
                            range: None,
                            geo_bounding_box: None,
                            geo_radius: None,
                            values_count: None,
                            geo_polygon: None,
                            datetime_range: None,
                            is_empty: None,
                            is_null: None,
                        })),
                    }],
                    ..Default::default()
                })
            },
            FilterCondition::Regex(field, pattern) => {
                // Qdrant doesn't have native regex, use text matching as approximation
                Ok(Filter {
                    must: vec![Condition {
                        condition_one_of: Some(ConditionOneOf::Field(FieldCondition {
                            key: field,
                            r#match: Some(Match {
                                match_value: Some(MatchValue::Text(pattern)),
                            }),
                            range: None,
                            geo_bounding_box: None,
                            geo_radius: None,
                            values_count: None,
                            geo_polygon: None,
                            datetime_range: None,
                            is_empty: None,
                            is_null: None,
                        })),
                    }],
                    ..Default::default()
                })
            },
            FilterCondition::Exists(field) => {
                Ok(Filter {
                    must: vec![Condition {
                        condition_one_of: Some(ConditionOneOf::Field(FieldCondition {
                            key: field,
                            r#match: None,
                            range: None,
                            geo_bounding_box: None,
                            geo_radius: None,
                            values_count: None,
                            geo_polygon: None,
                            datetime_range: None,
                            is_empty: None,
                            is_null: Some(false), // Field should not be null (i.e., should exist)
                        })),
                    }],
                    ..Default::default()
                })
            },
            FilterCondition::NotExists(field) => {
                Ok(Filter {
                    must: vec![Condition {
                        condition_one_of: Some(ConditionOneOf::Field(FieldCondition {
                            key: field,
                            r#match: None,
                            range: None,
                            geo_bounding_box: None,
                            geo_radius: None,
                            values_count: None,
                            geo_polygon: None,
                            datetime_range: None,
                            is_empty: None,
                            is_null: Some(true), // Field should be null (i.e., should not exist)
                        })),
                    }],
                    ..Default::default()
                })
            },
            FilterCondition::And(conditions) => {
                let mut must_conditions = Vec::new();
                for condition in conditions {
                    let filter = Self::convert_filter(condition)?;
                    must_conditions.extend(filter.must);
                    // Also handle must_not and should from sub-filters
                    if !filter.must_not.is_empty() {
                        // Convert must_not to nested filter
                        must_conditions.push(Condition {
                            condition_one_of: Some(ConditionOneOf::Filter(Filter {
                                must_not: filter.must_not,
                                ..Default::default()
                            })),
                        });
                    }
                    if !filter.should.is_empty() {
                        // Convert should to nested filter
                        must_conditions.push(Condition {
                            condition_one_of: Some(ConditionOneOf::Filter(Filter {
                                should: filter.should,
                                ..Default::default()
                            })),
                        });
                    }
                }
                Ok(Filter {
                    must: must_conditions,
                    ..Default::default()
                })
            },
            FilterCondition::Or(conditions) => {
                let mut should_conditions = Vec::new();
                for condition in conditions {
                    let filter = Self::convert_filter(condition)?;
                    // Convert each sub-filter to a condition
                    if !filter.must.is_empty() {
                        should_conditions.push(Condition {
                            condition_one_of: Some(ConditionOneOf::Filter(Filter {
                                must: filter.must,
                                ..Default::default()
                            })),
                        });
                    }
                    if !filter.must_not.is_empty() {
                        should_conditions.push(Condition {
                            condition_one_of: Some(ConditionOneOf::Filter(Filter {
                                must_not: filter.must_not,
                                ..Default::default()
                            })),
                        });
                    }
                    if !filter.should.is_empty() {
                        should_conditions.push(Condition {
                            condition_one_of: Some(ConditionOneOf::Filter(Filter {
                                should: filter.should,
                                ..Default::default()
                            })),
                        });
                    }
                }
                Ok(Filter {
                    should: should_conditions,
                    ..Default::default()
                })
            },
            FilterCondition::Not(condition) => {
                let filter = Self::convert_filter(*condition)?;
                Ok(Filter {
                    must_not: filter.must,
                    must: filter.must_not, // Invert must_not to must
                    should: if !filter.should.is_empty() {
                        // Convert should to must_not
                        vec![Condition {
                            condition_one_of: Some(ConditionOneOf::Filter(Filter {
                                should: filter.should,
                                ..Default::default()
                            })),
                        }]
                    } else {
                        vec![]
                    },
                    ..Default::default()
                })
            },
        }
    }

    /// Convert metadata value to Qdrant MatchValue
    fn convert_metadata_value(value: MetadataValue) -> QdrantResult<MatchValue> {
        match value {
            MetadataValue::String(s) => Ok(MatchValue::Keyword(s)),
            MetadataValue::Integer(i) => Ok(MatchValue::Integer(i)),
            MetadataValue::Float(f) => Ok(MatchValue::Integer(f as i64)), // Convert float to int for matching
            MetadataValue::Boolean(b) => Ok(MatchValue::Boolean(b)),
            MetadataValue::Array(_) => {
                Err(QdrantError::InvalidFilter("Array values not supported in filters".to_string()))
            },
            MetadataValue::Object(_) => {
                Err(QdrantError::InvalidFilter("Object values not supported in filters".to_string()))
            },
            MetadataValue::Null => {
                Err(QdrantError::InvalidFilter("Null values not supported in match filters".to_string()))
            },
        }
    }

    /// Convert metadata value to f64 for range operations
    fn convert_to_range_value(value: MetadataValue) -> QdrantResult<f64> {
        match value {
            MetadataValue::String(_) => {
                Err(QdrantError::InvalidFilter("String values not supported in range operations".to_string()))
            },
            MetadataValue::Integer(i) => Ok(i as f64),
            MetadataValue::Float(f) => Ok(f),
            MetadataValue::Boolean(_) => {
                Err(QdrantError::InvalidFilter("Boolean values not supported in range operations".to_string()))
            },
            MetadataValue::Array(_) => {
                Err(QdrantError::InvalidFilter("Array values not supported in range operations".to_string()))
            },
            MetadataValue::Object(_) => {
                Err(QdrantError::InvalidFilter("Object values not supported in range operations".to_string()))
            },
            MetadataValue::Null => {
                Err(QdrantError::InvalidFilter("Null values not supported in range operations".to_string()))
            },
        }
    }

    /// Convert metadata value to Qdrant Value for payload storage
    pub fn convert_metadata_to_value(value: MetadataValue) -> QdrantResult<QdrantValue> {
        match value {
            MetadataValue::String(s) => Ok(QdrantValue {
                kind: Some(qdrant_client::qdrant::value::Kind::StringValue(s)),
            }),
            MetadataValue::Integer(i) => Ok(QdrantValue {
                kind: Some(qdrant_client::qdrant::value::Kind::IntegerValue(i)),
            }),
            MetadataValue::Float(f) => Ok(QdrantValue {
                kind: Some(qdrant_client::qdrant::value::Kind::DoubleValue(f)),
            }),
            MetadataValue::Boolean(b) => Ok(QdrantValue {
                kind: Some(qdrant_client::qdrant::value::Kind::BoolValue(b)),
            }),
            MetadataValue::Null => Ok(QdrantValue {
                kind: Some(qdrant_client::qdrant::value::Kind::NullValue(0)),
            }),
            MetadataValue::Array(_) => {
                Err(QdrantError::Serialization(
                    "Array metadata values not supported in payload".to_string()
                ))
            },
            MetadataValue::Object(_) => {
                Err(QdrantError::Serialization(
                    "Object metadata values not supported in payload".to_string()
                ))
            },
        }
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
