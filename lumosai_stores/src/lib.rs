pub mod error;
pub mod vector;

#[cfg(feature = "qdrant")]
pub mod qdrant;

#[cfg(feature = "mongodb")]
pub mod mongodb;

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "vectorize")]
pub mod vectorize;

pub mod rag;

// Re-export common interfaces
pub use error::StoreError;
pub use vector::{VectorFilter, VectorFilterTranslator, VectorStore, VectorStoreParams};

// Re-export store implementations
#[cfg(feature = "qdrant")]
pub use qdrant::QdrantStore;

#[cfg(feature = "mongodb")]
pub use mongodb::MongoDBStore;

#[cfg(feature = "postgres")]
pub use postgres::PostgresVectorStore;