//! Real API tests for Memory module
//!
//! Tests memory functionality focusing on WorkingMemory

#[cfg(test)]
mod tests {
    use crate::memory::{BasicWorkingMemory, WorkingMemory, WorkingMemoryConfig};
    use serde_json::{json, Value};

    #[tokio::test]
    async fn test_working_memory_set_and_get() {
        let config = WorkingMemoryConfig {
            enabled: true,
            template: None,
            content_type: Some("application/json".to_string()),
            max_capacity: Some(1024),
        };

        let memory = BasicWorkingMemory::new(config);

        // Set a value
        memory
            .set_value("user_name", Value::String("Alice".to_string()))
            .await
            .unwrap();

        // Get the value
        let value = memory.get_value("user_name").await.unwrap();
        assert_eq!(value, Some(Value::String("Alice".to_string())));
    }

    #[tokio::test]
    async fn test_working_memory_delete_value() {
        let config = WorkingMemoryConfig {
            enabled: true,
            template: None,
            content_type: Some("application/json".to_string()),
            max_capacity: Some(1024),
        };

        let memory = BasicWorkingMemory::new(config);

        // Set a value
        memory
            .set_value("temp_key", Value::String("temp_value".to_string()))
            .await
            .unwrap();

        // Verify it exists
        let value = memory.get_value("temp_key").await.unwrap();
        assert!(value.is_some());

        // Delete the value
        memory.delete_value("temp_key").await.unwrap();

        // Verify it's gone
        let value = memory.get_value("temp_key").await.unwrap();
        assert!(value.is_none());
    }

    #[tokio::test]
    async fn test_working_memory_clear() {
        let config = WorkingMemoryConfig {
            enabled: true,
            template: None,
            content_type: Some("application/json".to_string()),
            max_capacity: Some(1024),
        };

        let memory = BasicWorkingMemory::new(config);

        // Set multiple values
        memory
            .set_value("key1", Value::String("value1".to_string()))
            .await
            .unwrap();
        memory
            .set_value("key2", Value::String("value2".to_string()))
            .await
            .unwrap();

        // Clear memory
        memory.clear().await.unwrap();

        // Verify all values are gone
        let value1 = memory.get_value("key1").await.unwrap();
        let value2 = memory.get_value("key2").await.unwrap();
        assert!(value1.is_none());
        assert!(value2.is_none());
    }

    #[tokio::test]
    async fn test_working_memory_update_content() {
        let config = WorkingMemoryConfig {
            enabled: true,
            template: None,
            content_type: Some("application/json".to_string()),
            max_capacity: Some(1024),
        };

        let memory = BasicWorkingMemory::new(config);

        // Get initial content
        let mut content = memory.get().await.unwrap();
        assert_eq!(content.content_type, "application/json");

        // Update content
        content.content = json!({
            "user": "Bob",
            "age": 30
        });

        memory.update(content.clone()).await.unwrap();

        // Verify update
        let updated = memory.get().await.unwrap();
        assert_eq!(updated.content["user"], "Bob");
        assert_eq!(updated.content["age"], 30);
    }

    #[tokio::test]
    async fn test_working_memory_metadata() {
        let config = WorkingMemoryConfig {
            enabled: true,
            template: None,
            content_type: Some("application/json".to_string()),
            max_capacity: Some(1024),
        };

        let memory = BasicWorkingMemory::new(config);

        // Get content and add metadata
        let mut content = memory.get().await.unwrap();
        content
            .metadata
            .insert("source".to_string(), Value::String("test".to_string()));

        memory.update(content).await.unwrap();

        // Verify metadata
        let updated = memory.get().await.unwrap();
        assert_eq!(
            updated.metadata.get("source"),
            Some(&Value::String("test".to_string()))
        );
    }

    #[tokio::test]
    async fn test_working_memory_multiple_operations() {
        let config = WorkingMemoryConfig {
            enabled: true,
            template: None,
            content_type: Some("application/json".to_string()),
            max_capacity: Some(1024),
        };

        let memory = BasicWorkingMemory::new(config);

        // Set multiple values
        memory
            .set_value("name", Value::String("Alice".to_string()))
            .await
            .unwrap();
        memory
            .set_value("age", Value::Number(30.into()))
            .await
            .unwrap();
        memory
            .set_value("city", Value::String("New York".to_string()))
            .await
            .unwrap();

        // Verify all values
        assert_eq!(
            memory.get_value("name").await.unwrap(),
            Some(Value::String("Alice".to_string()))
        );
        assert_eq!(
            memory.get_value("age").await.unwrap(),
            Some(Value::Number(30.into()))
        );
        assert_eq!(
            memory.get_value("city").await.unwrap(),
            Some(Value::String("New York".to_string()))
        );

        // Delete one value
        memory.delete_value("age").await.unwrap();

        // Verify deletion
        assert!(memory.get_value("age").await.unwrap().is_none());
        assert!(memory.get_value("name").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_working_memory_complex_values() {
        let config = WorkingMemoryConfig {
            enabled: true,
            template: None,
            content_type: Some("application/json".to_string()),
            max_capacity: Some(2048),
        };

        let memory = BasicWorkingMemory::new(config);

        // Set complex nested object
        let complex_value = json!({
            "user": {
                "name": "Bob",
                "profile": {
                    "age": 25,
                    "interests": ["coding", "music", "travel"]
                }
            },
            "settings": {
                "theme": "dark",
                "notifications": true
            }
        });

        memory
            .set_value("user_data", complex_value.clone())
            .await
            .unwrap();

        // Verify complex value
        let retrieved = memory.get_value("user_data").await.unwrap();
        assert_eq!(retrieved, Some(complex_value));
    }

    #[tokio::test]
    async fn test_working_memory_overwrite_value() {
        let config = WorkingMemoryConfig {
            enabled: true,
            template: None,
            content_type: Some("application/json".to_string()),
            max_capacity: Some(1024),
        };

        let memory = BasicWorkingMemory::new(config);

        // Set initial value
        memory
            .set_value("counter", Value::Number(1.into()))
            .await
            .unwrap();

        // Overwrite with new value
        memory
            .set_value("counter", Value::Number(2.into()))
            .await
            .unwrap();

        // Verify overwrite
        let value = memory.get_value("counter").await.unwrap();
        assert_eq!(value, Some(Value::Number(2.into())));
    }

    #[tokio::test]
    async fn test_working_memory_empty_key() {
        let config = WorkingMemoryConfig {
            enabled: true,
            template: None,
            content_type: Some("application/json".to_string()),
            max_capacity: Some(1024),
        };

        let memory = BasicWorkingMemory::new(config);

        // Get non-existent key
        let value = memory.get_value("non_existent").await.unwrap();
        assert!(value.is_none(), "Non-existent key should return None");
    }
}

