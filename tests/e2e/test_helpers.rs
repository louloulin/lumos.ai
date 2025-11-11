//! E2E Test Helpers
//!
//! Utility functions for E2E testing.

use lumosai_core::prelude::*;
use std::time::Duration;

/// Assert that a response contains expected keywords
pub fn assert_response_contains(response: &str, keywords: &[&str]) {
    for keyword in keywords {
        assert!(
            response.contains(keyword),
            "Response '{}' does not contain keyword '{}'",
            response,
            keyword
        );
    }
}

/// Assert that a response does not contain forbidden keywords
pub fn assert_response_not_contains(response: &str, keywords: &[&str]) {
    for keyword in keywords {
        assert!(
            !response.contains(keyword),
            "Response '{}' contains forbidden keyword '{}'",
            response,
            keyword
        );
    }
}

/// Assert that execution time is within expected range
pub fn assert_execution_time(duration: Duration, max_duration: Duration) {
    assert!(
        duration <= max_duration,
        "Execution time {:?} exceeded maximum {:?}",
        duration,
        max_duration
    );
}

/// Assert that a result is successful
pub fn assert_success<T>(result: &Result<T>) {
    assert!(result.is_ok(), "Expected success but got error: {:?}", result);
}

/// Assert that a result is an error
pub fn assert_error<T>(result: &Result<T>) {
    assert!(result.is_err(), "Expected error but got success");
}

/// Assert that an error message contains expected text
pub fn assert_error_contains<T>(result: &Result<T>, expected: &str) {
    match result {
        Err(e) => {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains(expected),
                "Error message '{}' does not contain '{}'",
                error_msg,
                expected
            );
        }
        Ok(_) => panic!("Expected error but got success"),
    }
}

/// Wait for a condition to be true with timeout
pub async fn wait_for_condition<F>(mut condition: F, timeout: Duration) -> bool
where
    F: FnMut() -> bool,
{
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if condition() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    false
}

/// Retry an operation with exponential backoff
pub async fn retry_with_backoff<F, Fut, T>(
    mut operation: F,
    max_retries: usize,
    initial_delay: Duration,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut delay = initial_delay;
    let mut last_error = None;

    for attempt in 0..max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries - 1 {
                    tokio::time::sleep(delay).await;
                    delay *= 2; // Exponential backoff
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| Error::Other("Retry failed".to_string())))
}

/// Measure execution time of an async operation
pub async fn measure_time<F, Fut, T>(operation: F) -> (T, Duration)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let start = std::time::Instant::now();
    let result = operation().await;
    let duration = start.elapsed();
    (result, duration)
}

/// Create a test message
pub fn create_test_message(content: &str) -> Message {
    Message::user(content.to_string())
}

/// Create multiple test messages
pub fn create_test_messages(contents: &[&str]) -> Vec<Message> {
    contents.iter().map(|c| create_test_message(c)).collect()
}

/// Validate JSON structure
pub fn validate_json_structure(json: &serde_json::Value, expected_keys: &[&str]) -> bool {
    if let Some(obj) = json.as_object() {
        expected_keys.iter().all(|key| obj.contains_key(*key))
    } else {
        false
    }
}

/// Extract value from JSON by path
pub fn extract_json_value<'a>(
    json: &'a serde_json::Value,
    path: &[&str],
) -> Option<&'a serde_json::Value> {
    let mut current = json;
    for key in path {
        current = current.get(key)?;
    }
    Some(current)
}

/// Compare two JSON values with tolerance for floating point numbers
pub fn json_equals_with_tolerance(
    a: &serde_json::Value,
    b: &serde_json::Value,
    tolerance: f64,
) -> bool {
    match (a, b) {
        (serde_json::Value::Number(n1), serde_json::Value::Number(n2)) => {
            if let (Some(f1), Some(f2)) = (n1.as_f64(), n2.as_f64()) {
                (f1 - f2).abs() < tolerance
            } else {
                n1 == n2
            }
        }
        (serde_json::Value::Array(arr1), serde_json::Value::Array(arr2)) => {
            arr1.len() == arr2.len()
                && arr1
                    .iter()
                    .zip(arr2.iter())
                    .all(|(v1, v2)| json_equals_with_tolerance(v1, v2, tolerance))
        }
        (serde_json::Value::Object(obj1), serde_json::Value::Object(obj2)) => {
            obj1.len() == obj2.len()
                && obj1.iter().all(|(k, v1)| {
                    obj2.get(k)
                        .map(|v2| json_equals_with_tolerance(v1, v2, tolerance))
                        .unwrap_or(false)
                })
        }
        _ => a == b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_response_contains() {
        let response = "Hello, world!";
        assert_response_contains(response, &["Hello", "world"]);
    }

    #[test]
    #[should_panic]
    fn test_assert_response_contains_fail() {
        let response = "Hello, world!";
        assert_response_contains(response, &["goodbye"]);
    }

    #[test]
    fn test_assert_response_not_contains() {
        let response = "Hello, world!";
        assert_response_not_contains(response, &["goodbye", "error"]);
    }

    #[test]
    fn test_assert_execution_time() {
        let duration = Duration::from_millis(100);
        let max_duration = Duration::from_millis(200);
        assert_execution_time(duration, max_duration);
    }

    #[test]
    fn test_validate_json_structure() {
        let json = serde_json::json!({
            "name": "test",
            "value": 42
        });
        assert!(validate_json_structure(&json, &["name", "value"]));
        assert!(!validate_json_structure(&json, &["name", "missing"]));
    }

    #[test]
    fn test_extract_json_value() {
        let json = serde_json::json!({
            "user": {
                "name": "Alice",
                "age": 30
            }
        });
        let name = extract_json_value(&json, &["user", "name"]);
        assert_eq!(name, Some(&serde_json::json!("Alice")));
    }

    #[test]
    fn test_json_equals_with_tolerance() {
        let a = serde_json::json!(3.14159);
        let b = serde_json::json!(3.14160);
        assert!(json_equals_with_tolerance(&a, &b, 0.001));
        assert!(!json_equals_with_tolerance(&a, &b, 0.00001));
    }

    #[tokio::test]
    async fn test_measure_time() {
        let (result, duration) = measure_time(|| async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            42
        })
        .await;

        assert_eq!(result, 42);
        assert!(duration >= Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_retry_with_backoff_success() {
        let mut attempts = 0;
        let result = retry_with_backoff(
            || async {
                attempts += 1;
                if attempts < 3 {
                    Err(Error::Other("retry".to_string()))
                } else {
                    Ok(42)
                }
            },
            5,
            Duration::from_millis(10),
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts, 3);
    }

    #[tokio::test]
    async fn test_retry_with_backoff_failure() {
        let result = retry_with_backoff(
            || async { Err(Error::Other("always fail".to_string())) },
            3,
            Duration::from_millis(10),
        )
        .await;

        assert!(result.is_err());
    }
}

