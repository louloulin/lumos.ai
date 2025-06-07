-- PostgreSQL initialization script for Lumos vector storage
-- This script sets up the database with pgvector extension and required tables

-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Create schema for vector storage
CREATE SCHEMA IF NOT EXISTS lumos_vector;

-- Set search path
SET search_path TO lumos_vector, public;

-- Create vector indexes table
CREATE TABLE IF NOT EXISTS vector_indexes (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL,
    dimension INTEGER NOT NULL,
    metric VARCHAR(50) NOT NULL DEFAULT 'cosine',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Create vector documents table
CREATE TABLE IF NOT EXISTS vector_documents (
    id VARCHAR(255) PRIMARY KEY,
    index_name VARCHAR(255) NOT NULL,
    content TEXT,
    embedding vector,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    FOREIGN KEY (index_name) REFERENCES vector_indexes(name) ON DELETE CASCADE
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_vector_documents_index_name ON vector_documents(index_name);
CREATE INDEX IF NOT EXISTS idx_vector_documents_metadata ON vector_documents USING GIN(metadata);
CREATE INDEX IF NOT EXISTS idx_vector_documents_created_at ON vector_documents(created_at);

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers for updated_at
DROP TRIGGER IF EXISTS update_vector_indexes_updated_at ON vector_indexes;
CREATE TRIGGER update_vector_indexes_updated_at
    BEFORE UPDATE ON vector_indexes
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_vector_documents_updated_at ON vector_documents;
CREATE TRIGGER update_vector_documents_updated_at
    BEFORE UPDATE ON vector_documents
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Create a sample index for testing
INSERT INTO vector_indexes (name, dimension, metric, metadata) 
VALUES ('test_index', 384, 'cosine', '{"description": "Test index for development"}')
ON CONFLICT (name) DO NOTHING;

-- Grant permissions
GRANT ALL PRIVILEGES ON SCHEMA lumos_vector TO postgres;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA lumos_vector TO postgres;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA lumos_vector TO postgres;

-- Create a read-only user for applications
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'lumos_app') THEN
        CREATE ROLE lumos_app WITH LOGIN PASSWORD 'lumos_app_password';
    END IF;
END
$$;

GRANT CONNECT ON DATABASE lumos TO lumos_app;
GRANT USAGE ON SCHEMA lumos_vector TO lumos_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA lumos_vector TO lumos_app;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA lumos_vector TO lumos_app;

-- Set default privileges for future tables
ALTER DEFAULT PRIVILEGES IN SCHEMA lumos_vector GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO lumos_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA lumos_vector GRANT USAGE, SELECT ON SEQUENCES TO lumos_app;

-- Display setup information
\echo 'Lumos PostgreSQL vector storage setup completed!'
\echo 'Schema: lumos_vector'
\echo 'Tables: vector_indexes, vector_documents'
\echo 'Extensions: vector (pgvector)'
\echo 'Users: postgres (admin), lumos_app (application)'
