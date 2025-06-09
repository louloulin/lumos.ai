//! Conversion utilities for LanceDB integration

use std::collections::HashMap;
use std::sync::Arc;

use arrow::array::{Float32Array, StringArray, ListBuilder, Float32Builder};
use arrow::record_batch::RecordBatch;
use arrow_schema::{DataType, Field, Schema};

use lumosai_vector_core::types::{Document, MetadataValue};
use crate::error::{LanceDbError, LanceDbResult};

/// Convert LumosAI documents to Arrow RecordBatch
pub fn documents_to_record_batch(
    documents: &[Document],
    schema: &Schema,
) -> LanceDbResult<RecordBatch> {
    if documents.is_empty() {
        return Ok(RecordBatch::new_empty(Arc::new(schema.clone())));
    }
    
    let mut ids = Vec::new();
    let mut contents = Vec::new();
    let mut vectors = Vec::new();
    let mut metadata_json = Vec::new();
    
    for doc in documents {
        ids.push(doc.id.clone());
        contents.push(doc.content.clone().unwrap_or_default());
        
        if let Some(embedding) = &doc.embedding {
            vectors.push(embedding.clone());
        } else {
            return Err(LanceDbError::InvalidData("Document missing embedding".to_string()));
        }
        
        let metadata_str = serde_json::to_string(&doc.metadata)
            .map_err(|e| LanceDbError::Serialization(e.to_string()))?;
        metadata_json.push(metadata_str);
    }
    
    // Create Arrow arrays
    let id_array = StringArray::from(ids);
    let content_array = StringArray::from(contents);
    let metadata_array = StringArray::from(metadata_json);
    
    // Create vector array (list of floats)
    let mut vector_builder = ListBuilder::new(Float32Builder::new());
    
    for vector in vectors {
        let float_array = Float32Array::from(vector);
        vector_builder.append_value(float_array.values());
    }
    
    let vector_array = vector_builder.finish();
    
    let batch = RecordBatch::try_new(
        Arc::new(schema.clone()),
        vec![
            Arc::new(id_array),
            Arc::new(content_array),
            Arc::new(vector_array),
            Arc::new(metadata_array),
        ],
    ).map_err(|e| LanceDbError::Arrow(e.to_string()))?;
    
    Ok(batch)
}

/// Convert Arrow RecordBatch to LumosAI documents
pub fn record_batch_to_documents(batch: &RecordBatch) -> LanceDbResult<Vec<Document>> {
    let mut documents = Vec::new();
    
    if batch.num_rows() == 0 {
        return Ok(documents);
    }
    
    let id_array = batch.column(0).as_any().downcast_ref::<StringArray>()
        .ok_or_else(|| LanceDbError::InvalidData("Invalid ID column".to_string()))?;
    
    let content_array = batch.column(1).as_any().downcast_ref::<StringArray>()
        .ok_or_else(|| LanceDbError::InvalidData("Invalid content column".to_string()))?;
    
    let vector_array = batch.column(2).as_any().downcast_ref::<arrow::array::ListArray>()
        .ok_or_else(|| LanceDbError::InvalidData("Invalid vector column".to_string()))?;
    
    let metadata_array = batch.column(3).as_any().downcast_ref::<StringArray>()
        .ok_or_else(|| LanceDbError::InvalidData("Invalid metadata column".to_string()))?;
    
    for i in 0..batch.num_rows() {
        let id = id_array.value(i).to_string();
        let content = if content_array.is_null(i) {
            None
        } else {
            Some(content_array.value(i).to_string())
        };
        
        // Extract vector
        let vector_list = vector_array.value(i);
        let float_array = vector_list.as_any().downcast_ref::<Float32Array>()
            .ok_or_else(|| LanceDbError::InvalidData("Invalid vector data".to_string()))?;
        let embedding = float_array.values().to_vec();
        
        // Extract metadata
        let metadata = if metadata_array.is_null(i) {
            HashMap::new()
        } else {
            let metadata_str = metadata_array.value(i);
            serde_json::from_str(metadata_str)
                .map_err(|e| LanceDbError::Serialization(e.to_string()))?
        };
        
        let mut document = Document::new(&id, content.as_deref().unwrap_or(""));
        document.embedding = Some(embedding);
        document.metadata = metadata;
        
        documents.push(document);
    }
    
    Ok(documents)
}

/// Create an Arrow schema for vector documents
pub fn create_document_schema(vector_dim: usize) -> Schema {
    let fields = vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("content", DataType::Utf8, true),
        Field::new("vector", DataType::List(
            Arc::new(Field::new("item", DataType::Float32, false))
        ), false),
        Field::new("metadata", DataType::Utf8, true), // JSON string
    ];
    
    Schema::new(fields)
}

/// Convert metadata value to JSON-compatible value
pub fn metadata_value_to_json(value: &MetadataValue) -> serde_json::Value {
    match value {
        MetadataValue::String(s) => serde_json::Value::String(s.clone()),
        MetadataValue::Integer(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
        MetadataValue::Float(f) => {
            serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
        MetadataValue::Boolean(b) => serde_json::Value::Bool(*b),
        MetadataValue::Array(arr) => {
            let json_arr: Vec<serde_json::Value> = arr
                .iter()
                .map(metadata_value_to_json)
                .collect();
            serde_json::Value::Array(json_arr)
        }
        MetadataValue::Object(obj) => {
            let json_obj: serde_json::Map<String, serde_json::Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), metadata_value_to_json(v)))
                .collect();
            serde_json::Value::Object(json_obj)
        }
    }
}

/// Convert JSON value to metadata value
pub fn json_to_metadata_value(value: &serde_json::Value) -> LanceDbResult<MetadataValue> {
    match value {
        serde_json::Value::String(s) => Ok(MetadataValue::String(s.clone())),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(MetadataValue::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(MetadataValue::Float(f))
            } else {
                Err(LanceDbError::InvalidData("Invalid number format".to_string()))
            }
        }
        serde_json::Value::Bool(b) => Ok(MetadataValue::Boolean(*b)),
        serde_json::Value::Array(arr) => {
            let metadata_arr: Result<Vec<MetadataValue>, _> = arr
                .iter()
                .map(json_to_metadata_value)
                .collect();
            Ok(MetadataValue::Array(metadata_arr?))
        }
        serde_json::Value::Object(obj) => {
            let metadata_obj: Result<HashMap<String, MetadataValue>, _> = obj
                .iter()
                .map(|(k, v)| json_to_metadata_value(v).map(|mv| (k.clone(), mv)))
                .collect();
            Ok(MetadataValue::Object(metadata_obj?))
        }
        serde_json::Value::Null => {
            Err(LanceDbError::InvalidData("Null values not supported in metadata".to_string()))
        }
    }
}

/// Validate vector dimensions
pub fn validate_vector_dimension(vectors: &[Vec<f32>]) -> LanceDbResult<usize> {
    if vectors.is_empty() {
        return Err(LanceDbError::InvalidData("No vectors provided".to_string()));
    }
    
    let expected_dim = vectors[0].len();
    if expected_dim == 0 {
        return Err(LanceDbError::InvalidData("Vector dimension cannot be zero".to_string()));
    }
    
    for (i, vector) in vectors.iter().enumerate() {
        if vector.len() != expected_dim {
            return Err(LanceDbError::InvalidData(
                format!("Vector {} has dimension {} but expected {}", i, vector.len(), expected_dim)
            ));
        }
    }
    
    Ok(expected_dim)
}

/// Normalize vector for cosine similarity
pub fn normalize_vector(vector: &mut [f32]) -> LanceDbResult<()> {
    let magnitude: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if magnitude == 0.0 {
        return Err(LanceDbError::InvalidData("Cannot normalize zero vector".to_string()));
    }
    
    for x in vector.iter_mut() {
        *x /= magnitude;
    }
    
    Ok(())
}

/// Calculate vector magnitude
pub fn vector_magnitude(vector: &[f32]) -> f32 {
    vector.iter().map(|x| x * x).sum::<f32>().sqrt()
}

/// Calculate cosine similarity between two vectors
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> LanceDbResult<f32> {
    if a.len() != b.len() {
        return Err(LanceDbError::InvalidData(
            format!("Vector dimensions don't match: {} vs {}", a.len(), b.len())
        ));
    }
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a = vector_magnitude(a);
    let magnitude_b = vector_magnitude(b);
    
    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return Ok(0.0);
    }
    
    Ok(dot_product / (magnitude_a * magnitude_b))
}

/// Calculate Euclidean distance between two vectors
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> LanceDbResult<f32> {
    if a.len() != b.len() {
        return Err(LanceDbError::InvalidData(
            format!("Vector dimensions don't match: {} vs {}", a.len(), b.len())
        ));
    }
    
    let distance: f32 = a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y) * (x - y))
        .sum::<f32>()
        .sqrt();
    
    Ok(distance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumosai_vector_core::types::Document;
    
    #[test]
    fn test_create_document_schema() {
        let schema = create_document_schema(384);
        assert_eq!(schema.fields().len(), 4);
        assert_eq!(schema.field(0).name(), "id");
        assert_eq!(schema.field(1).name(), "content");
        assert_eq!(schema.field(2).name(), "vector");
        assert_eq!(schema.field(3).name(), "metadata");
    }
    
    #[test]
    fn test_validate_vector_dimension() {
        let vectors = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
            vec![7.0, 8.0, 9.0],
        ];
        
        let dim = validate_vector_dimension(&vectors).unwrap();
        assert_eq!(dim, 3);
        
        let invalid_vectors = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0], // Wrong dimension
        ];
        
        assert!(validate_vector_dimension(&invalid_vectors).is_err());
    }
    
    #[test]
    fn test_normalize_vector() {
        let mut vector = vec![3.0, 4.0];
        normalize_vector(&mut vector).unwrap();
        
        // Should be normalized to unit vector
        let magnitude = vector_magnitude(&vector);
        assert!((magnitude - 1.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let similarity = cosine_similarity(&a, &b).unwrap();
        assert!((similarity - 0.0).abs() < 1e-6);
        
        let c = vec![1.0, 1.0];
        let d = vec![1.0, 1.0];
        let similarity = cosine_similarity(&c, &d).unwrap();
        assert!((similarity - 1.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        let distance = euclidean_distance(&a, &b).unwrap();
        assert!((distance - 5.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_metadata_conversion() {
        let metadata_value = MetadataValue::String("test".to_string());
        let json_value = metadata_value_to_json(&metadata_value);
        let converted_back = json_to_metadata_value(&json_value).unwrap();
        
        match converted_back {
            MetadataValue::String(s) => assert_eq!(s, "test"),
            _ => panic!("Expected string value"),
        }
    }
}
