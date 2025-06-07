//! PostgreSQL configuration for vector storage

use std::time::Duration;
use serde::{Deserialize, Serialize};

/// PostgreSQL vector storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresConfig {
    /// Database connection URL
    pub database_url: String,
    
    /// Connection pool configuration
    pub pool: PoolConfig,
    
    /// Table configuration
    pub table: TableConfig,
    
    /// Performance settings
    pub performance: PerformanceConfig,
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    
    /// Minimum number of connections in the pool
    pub min_connections: u32,
    
    /// Connection timeout
    pub connect_timeout: Duration,
    
    /// Idle timeout for connections
    pub idle_timeout: Option<Duration>,
    
    /// Maximum lifetime of a connection
    pub max_lifetime: Option<Duration>,
}

/// Table configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableConfig {
    /// Schema name (default: "public")
    pub schema: String,
    
    /// Table prefix for vector tables
    pub table_prefix: Option<String>,
    
    /// Whether to create tables automatically
    pub auto_create_tables: bool,
    
    /// Whether to create indexes automatically
    pub auto_create_indexes: bool,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Batch size for bulk operations
    pub batch_size: usize,
    
    /// Vector index type
    pub index_type: VectorIndexType,
    
    /// Index parameters
    pub index_params: IndexParams,
    
    /// Whether to use prepared statements
    pub use_prepared_statements: bool,
}

/// Vector index types supported by pgvector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VectorIndexType {
    /// IVFFlat index - good for large datasets
    IvfFlat,
    /// HNSW index - good for high recall
    Hnsw,
    /// No index - for small datasets or testing
    None,
}

/// Index parameters for different index types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexParams {
    /// IVFFlat parameters
    pub ivf_flat: IvfFlatParams,
    
    /// HNSW parameters
    pub hnsw: HnswParams,
}

/// IVFFlat index parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IvfFlatParams {
    /// Number of lists (clusters)
    pub lists: u32,
    
    /// Number of probes during search
    pub probes: u32,
}

/// HNSW index parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswParams {
    /// Maximum number of connections per node
    pub m: u32,
    
    /// Size of the dynamic candidate list during construction
    pub ef_construction: u32,
    
    /// Size of the dynamic candidate list during search
    pub ef_search: u32,
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://localhost/lumos_vector".to_string(),
            pool: PoolConfig::default(),
            table: TableConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 1,
            connect_timeout: Duration::from_secs(30),
            idle_timeout: Some(Duration::from_secs(600)),
            max_lifetime: Some(Duration::from_secs(1800)),
        }
    }
}

impl Default for TableConfig {
    fn default() -> Self {
        Self {
            schema: "public".to_string(),
            table_prefix: Some("lumos_".to_string()),
            auto_create_tables: true,
            auto_create_indexes: true,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            index_type: VectorIndexType::Hnsw,
            index_params: IndexParams::default(),
            use_prepared_statements: true,
        }
    }
}

impl Default for IndexParams {
    fn default() -> Self {
        Self {
            ivf_flat: IvfFlatParams::default(),
            hnsw: HnswParams::default(),
        }
    }
}

impl Default for IvfFlatParams {
    fn default() -> Self {
        Self {
            lists: 100,
            probes: 10,
        }
    }
}

impl Default for HnswParams {
    fn default() -> Self {
        Self {
            m: 16,
            ef_construction: 64,
            ef_search: 40,
        }
    }
}

impl PostgresConfig {
    /// Create a new PostgreSQL configuration with database URL
    pub fn new(database_url: impl Into<String>) -> Self {
        Self {
            database_url: database_url.into(),
            ..Default::default()
        }
    }
    
    /// Set pool configuration
    pub fn with_pool(mut self, pool: PoolConfig) -> Self {
        self.pool = pool;
        self
    }
    
    /// Set table configuration
    pub fn with_table(mut self, table: TableConfig) -> Self {
        self.table = table;
        self
    }
    
    /// Set performance configuration
    pub fn with_performance(mut self, performance: PerformanceConfig) -> Self {
        self.performance = performance;
        self
    }
    
    /// Get the full table name with schema and prefix
    pub fn table_name(&self, name: &str) -> String {
        let prefix = self.table.table_prefix.as_deref().unwrap_or("");
        format!("{}.{}{}", self.table.schema, prefix, name)
    }
    
    /// Get the index name for a table
    pub fn index_name(&self, table_name: &str, index_type: &str) -> String {
        let prefix = self.table.table_prefix.as_deref().unwrap_or("");
        format!("{}{}_{}_idx", prefix, table_name, index_type)
    }
}

impl VectorIndexType {
    /// Get the SQL for creating this index type
    pub fn create_index_sql(&self, table_name: &str, index_name: &str, params: &IndexParams) -> String {
        match self {
            VectorIndexType::IvfFlat => {
                format!(
                    "CREATE INDEX {} ON {} USING ivfflat (embedding vector_cosine_ops) WITH (lists = {})",
                    index_name, table_name, params.ivf_flat.lists
                )
            },
            VectorIndexType::Hnsw => {
                format!(
                    "CREATE INDEX {} ON {} USING hnsw (embedding vector_cosine_ops) WITH (m = {}, ef_construction = {})",
                    index_name, table_name, params.hnsw.m, params.hnsw.ef_construction
                )
            },
            VectorIndexType::None => String::new(),
        }
    }
    
    /// Get the SQL for setting search parameters
    pub fn search_params_sql(&self, params: &IndexParams) -> Vec<String> {
        match self {
            VectorIndexType::IvfFlat => {
                vec![format!("SET ivfflat.probes = {}", params.ivf_flat.probes)]
            },
            VectorIndexType::Hnsw => {
                vec![format!("SET hnsw.ef_search = {}", params.hnsw.ef_search)]
            },
            VectorIndexType::None => vec![],
        }
    }
}
