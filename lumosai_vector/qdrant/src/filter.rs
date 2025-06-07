//! Filter conversion utilities for Qdrant

use qdrant_client::qdrant::{Filter, Condition, Range, Value as QdrantValue, Match, FieldCondition};
use lumosai_vector_core::prelude::*;
use crate::{QdrantError};
use crate::error::QdrantResult;

/// Qdrant filter converter
pub struct QdrantFilterConverter;

impl QdrantFilterConverter {
    /// Convert a filter condition to Qdrant filter
    pub fn convert_filter(condition: FilterCondition) -> QdrantResult<Filter> {
        // For now, return a simple filter that accepts all documents
        // TODO: Implement proper filter conversion when Qdrant API is stable
        match condition {
            _ => Ok(Filter::default()),
        }
    }

    
    /// Convert metadata value to Qdrant value
    pub fn convert_metadata_value(value: MetadataValue) -> QdrantResult<QdrantValue> {
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
