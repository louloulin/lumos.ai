//! Tests for lumos-vector-core

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use std::collections::HashMap;

    #[test]
    fn test_metadata_value_conversions() {
        let string_val: MetadataValue = "test".into();
        assert_eq!(string_val, MetadataValue::String("test".to_string()));

        let int_val: MetadataValue = 42i64.into();
        assert_eq!(int_val, MetadataValue::Integer(42));

        let float_val: MetadataValue = 3.14f64.into();
        assert_eq!(float_val, MetadataValue::Float(3.14));

        let bool_val: MetadataValue = true.into();
        assert_eq!(bool_val, MetadataValue::Boolean(true));
    }

    #[test]
    fn test_filter_condition_builders() {
        let eq_filter = FilterCondition::eq("field", "value");
        assert!(matches!(eq_filter, FilterCondition::Eq(_, _)));

        let gt_filter = FilterCondition::gt("score", 0.5);
        assert!(matches!(gt_filter, FilterCondition::Gt(_, _)));

        let and_filter = FilterCondition::and(vec![
            FilterCondition::eq("type", "document"),
            FilterCondition::gt("score", 0.8),
        ]);
        assert!(matches!(and_filter, FilterCondition::And(_)));

        let not_filter = FilterCondition::not(FilterCondition::eq("active", false));
        assert!(matches!(not_filter, FilterCondition::Not(_)));
    }

    #[test]
    fn test_index_config_builder() {
        let config = IndexConfig::new("test_index", 384)
            .with_metric(SimilarityMetric::Cosine)
            .with_option("approximate", true)
            .with_option("num_trees", 10);

        assert_eq!(config.name, "test_index");
        assert_eq!(config.dimension, 384);
        assert_eq!(config.metric, SimilarityMetric::Cosine);
        assert_eq!(config.options.len(), 2);
    }

    #[test]
    fn test_document_builder() {
        let doc = Document::new("doc1", "This is a test document")
            .with_embedding(vec![0.1, 0.2, 0.3])
            .with_metadata("type", "article")
            .with_metadata("score", 0.95);

        assert_eq!(doc.id, "doc1");
        assert_eq!(doc.content, "This is a test document");
        assert!(doc.embedding.is_some());
        assert_eq!(doc.metadata.len(), 2);
    }

    #[test]
    fn test_search_result_builder() {
        let result = SearchResult::new("doc1", 0.95)
            .with_vector(vec![0.1, 0.2, 0.3])
            .with_metadata(HashMap::from([
                ("type".to_string(), MetadataValue::String("article".to_string())),
            ]))
            .with_content("Test content");

        assert_eq!(result.id, "doc1");
        assert_eq!(result.score, 0.95);
        assert!(result.vector.is_some());
        assert!(result.metadata.is_some());
        assert!(result.content.is_some());
    }

    #[test]
    fn test_similarity_calculators() {
        use crate::traits::similarity::*;

        let vec_a = vec![1.0, 0.0, 0.0];
        let vec_b = vec![0.0, 1.0, 0.0];
        let vec_c = vec![1.0, 0.0, 0.0]; // Same as vec_a

        // Test cosine similarity
        let cosine_calc = CosineSimilarity;
        let cosine_ab = cosine_calc.calculate_similarity(&vec_a, &vec_b).unwrap();
        let cosine_ac = cosine_calc.calculate_similarity(&vec_a, &vec_c).unwrap();
        
        assert!((cosine_ab - 0.0).abs() < 1e-6); // Orthogonal vectors
        assert!((cosine_ac - 1.0).abs() < 1e-6); // Identical vectors

        // Test dot product similarity
        let dot_calc = DotProductSimilarity;
        let dot_ab = dot_calc.calculate_similarity(&vec_a, &vec_b).unwrap();
        let dot_ac = dot_calc.calculate_similarity(&vec_a, &vec_c).unwrap();
        
        assert!((dot_ab - 0.0).abs() < 1e-6);
        assert!((dot_ac - 1.0).abs() < 1e-6);

        // Test euclidean similarity
        let euclidean_calc = EuclideanSimilarity;
        let euclidean_ab = euclidean_calc.calculate_similarity(&vec_a, &vec_b).unwrap();
        let euclidean_ac = euclidean_calc.calculate_similarity(&vec_a, &vec_c).unwrap();
        
        assert!(euclidean_ab < euclidean_ac); // Different vectors should be less similar
        assert!((euclidean_ac - 1.0).abs() < 1e-6); // Identical vectors should have max similarity
    }

    #[test]
    fn test_filter_evaluator() {
        use crate::traits::filter::StandardFilterEvaluator;

        let evaluator = StandardFilterEvaluator;
        let metadata = HashMap::from([
            ("type".to_string(), MetadataValue::String("article".to_string())),
            ("score".to_string(), MetadataValue::Float(0.85)),
            ("active".to_string(), MetadataValue::Boolean(true)),
            ("tags".to_string(), MetadataValue::Array(vec![
                MetadataValue::String("tech".to_string()),
                MetadataValue::String("ai".to_string()),
            ])),
        ]);

        // Test equality filter
        let eq_filter = FilterCondition::eq("type", "article");
        assert!(evaluator.evaluate(&eq_filter, &metadata).unwrap());

        let ne_filter = FilterCondition::eq("type", "blog");
        assert!(!evaluator.evaluate(&ne_filter, &metadata).unwrap());

        // Test numeric comparison
        let gt_filter = FilterCondition::gt("score", 0.8);
        assert!(evaluator.evaluate(&gt_filter, &metadata).unwrap());

        let lt_filter = FilterCondition::lt("score", 0.8);
        assert!(!evaluator.evaluate(&lt_filter, &metadata).unwrap());

        // Test boolean filter
        let bool_filter = FilterCondition::eq("active", true);
        assert!(evaluator.evaluate(&bool_filter, &metadata).unwrap());

        // Test exists filter
        let exists_filter = FilterCondition::exists("type");
        assert!(evaluator.evaluate(&exists_filter, &metadata).unwrap());

        let not_exists_filter = FilterCondition::exists("nonexistent");
        assert!(!evaluator.evaluate(&not_exists_filter, &metadata).unwrap());

        // Test AND filter
        let and_filter = FilterCondition::and(vec![
            FilterCondition::eq("type", "article"),
            FilterCondition::gt("score", 0.8),
        ]);
        assert!(evaluator.evaluate(&and_filter, &metadata).unwrap());

        // Test OR filter
        let or_filter = FilterCondition::or(vec![
            FilterCondition::eq("type", "blog"),
            FilterCondition::gt("score", 0.8),
        ]);
        assert!(evaluator.evaluate(&or_filter, &metadata).unwrap());

        // Test NOT filter
        let not_filter = FilterCondition::not(FilterCondition::eq("active", false));
        assert!(evaluator.evaluate(&not_filter, &metadata).unwrap());
    }

    #[test]
    fn test_storage_config_builders() {
        // Test memory config
        let memory_config = StorageConfigBuilder::memory()
            .with_initial_capacity(1000)
            .build();
        
        if let StorageConfig::Memory { initial_capacity, .. } = memory_config {
            assert_eq!(initial_capacity, Some(1000));
        } else {
            panic!("Expected memory config");
        }

        // Test SQLite config
        let sqlite_config = StorageConfigBuilder::sqlite("test.db").build();
        
        if let StorageConfig::Sqlite { database_path, .. } = sqlite_config {
            assert_eq!(database_path, "test.db");
        } else {
            panic!("Expected SQLite config");
        }

        // Test Qdrant config
        let qdrant_config = StorageConfigBuilder::qdrant("http://localhost:6333", "test_collection")
            .with_api_key("secret")
            .with_tls(true)
            .build();
        
        if let StorageConfig::Qdrant { url, collection_name, api_key, tls, .. } = qdrant_config {
            assert_eq!(url, "http://localhost:6333");
            assert_eq!(collection_name, "test_collection");
            assert_eq!(api_key, Some("secret".to_string()));
            assert!(tls);
        } else {
            panic!("Expected Qdrant config");
        }
    }

    #[test]
    fn test_vector_system_config() {
        let config = VectorSystemConfig::memory()
            .with_embedding(EmbeddingConfig::OpenAI {
                api_key: "test_key".to_string(),
                model: "text-embedding-3-small".to_string(),
                base_url: None,
                organization: None,
                timeout: None,
                max_retries: 3,
            })
            .with_option("debug", true);

        assert!(matches!(config.storage, StorageConfig::Memory { .. }));
        assert!(config.embedding.is_some());
        assert_eq!(config.options.len(), 1);
    }

    #[test]
    fn test_error_classification() {
        let client_error = VectorError::index_not_found("test");
        assert!(client_error.is_client_error());
        assert!(!client_error.is_server_error());
        assert!(!client_error.is_retryable());

        let server_error = VectorError::internal("system failure");
        assert!(!server_error.is_client_error());
        assert!(server_error.is_server_error());
        assert!(!server_error.is_retryable());

        let retryable_error = VectorError::QueryTimeout { seconds: 30 };
        assert!(!retryable_error.is_client_error());
        assert!(!retryable_error.is_server_error());
        assert!(retryable_error.is_retryable());
    }

    #[test]
    fn test_backend_info() {
        let info = BackendInfo::new("memory", "1.0.0")
            .with_feature("approximate_search")
            .with_feature("filtering")
            .with_metadata("max_vectors", 1000000);

        assert_eq!(info.name, "memory");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.features.len(), 2);
        assert_eq!(info.metadata.len(), 1);
    }
}
