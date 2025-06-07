//! Weaviate schema management

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Weaviate class definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaviateClass {
    /// Class name (must start with uppercase)
    #[serde(rename = "class")]
    pub class: String,
    
    /// Optional description
    pub description: Option<String>,
    
    /// Vector index type (usually "hnsw")
    #[serde(rename = "vectorIndexType")]
    pub vector_index_type: String,
    
    /// Vector index configuration
    #[serde(rename = "vectorIndexConfig")]
    pub vector_index_config: Value,
    
    /// Vectorizer module
    pub vectorizer: String,
    
    /// Class properties
    pub properties: Vec<WeaviateProperty>,
}

/// Weaviate property definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaviateProperty {
    /// Property name
    pub name: String,
    
    /// Data type(s)
    #[serde(rename = "dataType")]
    pub data_type: Vec<String>,
    
    /// Optional description
    pub description: Option<String>,
    
    /// Whether to index this property
    pub index: Option<bool>,
}

impl WeaviateClass {
    /// Create a new class definition
    pub fn new(name: &str, dimension: usize) -> Self {
        Self {
            class: name.to_string(),
            description: Some(format!("Vector index for {}", name)),
            vector_index_type: "hnsw".to_string(),
            vector_index_config: serde_json::json!({
                "distance": "cosine",
                "efConstruction": 128,
                "maxConnections": 64
            }),
            vectorizer: "none".to_string(),
            properties: vec![
                WeaviateProperty {
                    name: "content".to_string(),
                    data_type: vec!["text".to_string()],
                    description: Some("Document content".to_string()),
                    index: Some(true),
                },
                WeaviateProperty {
                    name: "metadata".to_string(),
                    data_type: vec!["object".to_string()],
                    description: Some("Document metadata".to_string()),
                    index: Some(false),
                },
            ],
        }
    }
    
    /// Set the distance metric
    pub fn with_distance(mut self, distance: &str) -> Self {
        if let Some(config) = self.vector_index_config.as_object_mut() {
            config.insert("distance".to_string(), serde_json::Value::String(distance.to_string()));
        }
        self
    }
    
    /// Set the vectorizer
    pub fn with_vectorizer(mut self, vectorizer: String) -> Self {
        self.vectorizer = vectorizer;
        self
    }
    
    /// Add a property
    pub fn with_property(mut self, property: WeaviateProperty) -> Self {
        self.properties.push(property);
        self
    }
}

impl WeaviateProperty {
    /// Create a new property
    pub fn new(name: &str, data_type: &str) -> Self {
        Self {
            name: name.to_string(),
            data_type: vec![data_type.to_string()],
            description: None,
            index: None,
        }
    }
    
    /// Set description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
    
    /// Set indexing
    pub fn with_index(mut self, index: bool) -> Self {
        self.index = Some(index);
        self
    }
}
