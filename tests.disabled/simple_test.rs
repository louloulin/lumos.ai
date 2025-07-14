// Simple test to verify our test framework works
use std::time::Duration;

#[tokio::test]
async fn test_basic_functionality() {
    // Test that basic async functionality works
    let result = async_operation().await;
    assert_eq!(result, "success");
}

#[tokio::test]
async fn test_timeout_functionality() {
    // Test timeout handling
    let start = std::time::Instant::now();
    
    let result = tokio::time::timeout(
        Duration::from_secs(1),
        quick_operation()
    ).await;
    
    assert!(result.is_ok());
    assert!(start.elapsed() < Duration::from_secs(1));
}

#[tokio::test]
async fn test_error_handling() {
    // Test error handling
    let result = failing_operation().await;
    assert!(result.is_err());
}

#[test]
fn test_sync_functionality() {
    // Test synchronous functionality
    let result = sync_operation();
    assert_eq!(result, 42);
}

#[test]
fn test_performance_measurement() {
    // Test performance measurement utilities
    let start = std::time::Instant::now();
    
    // Simulate some work
    std::thread::sleep(Duration::from_millis(10));
    
    let duration = start.elapsed();
    assert!(duration >= Duration::from_millis(10));
    assert!(duration < Duration::from_millis(100));
}

// Helper functions for testing
async fn async_operation() -> &'static str {
    tokio::time::sleep(Duration::from_millis(1)).await;
    "success"
}

async fn quick_operation() -> &'static str {
    tokio::time::sleep(Duration::from_millis(10)).await;
    "quick"
}

async fn failing_operation() -> Result<(), &'static str> {
    Err("intentional failure")
}

fn sync_operation() -> i32 {
    42
}

#[cfg(test)]
mod test_utilities {
    use super::*;
    
    #[test]
    fn test_test_utilities() {
        // Test that our test utilities work
        assert_eq!(sync_operation(), 42);
    }
    
    #[tokio::test]
    async fn test_async_utilities() {
        let result = async_operation().await;
        assert_eq!(result, "success");
    }
}
