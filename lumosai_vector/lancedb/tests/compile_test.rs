//! Simple compilation test for LanceDB integration

use lumosai_vector_lancedb::{LanceDbConfig, LanceDbConfigBuilder};

#[test]
fn test_config_creation() {
    let config = LanceDbConfig::local("./test_data");
    assert_eq!(config.uri, "file://./test_data");
    assert!(config.enable_wal);
}

#[test]
fn test_config_builder() {
    let config = LanceDbConfigBuilder::new("./test")
        .batch_size(500)
        .enable_compression(true)
        .build()
        .unwrap();
    
    assert_eq!(config.uri, "./test");
    assert_eq!(config.performance.batch_size, 500);
    assert!(config.performance.enable_compression);
}

#[test]
fn test_s3_config() {
    let config = LanceDbConfig::s3("my-bucket", "us-west-2");
    assert_eq!(config.uri, "s3://my-bucket");
    assert!(config.storage_options.is_some());
}

#[test]
fn test_azure_config() {
    let config = LanceDbConfig::azure("myaccount", "mycontainer");
    assert_eq!(config.uri, "azure://myaccount/mycontainer");
    assert!(config.storage_options.is_some());
}

#[test]
fn test_gcs_config() {
    let config = LanceDbConfig::gcs("my-project", "my-bucket");
    assert_eq!(config.uri, "gs://my-bucket");
    assert!(config.storage_options.is_some());
}
