-- ArcViz Database Initialization
-- This script runs automatically when the PostgreSQL container starts for the first time

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";  -- For fuzzy text search
CREATE EXTENSION IF NOT EXISTS "btree_gin"; -- For better indexing

-- Create enum types
CREATE TYPE user_role AS ENUM ('admin', 'architect', 'engineer', 'reviewer', 'viewer');
CREATE TYPE project_status AS ENUM ('active', 'archived', 'deleted');
CREATE TYPE safety_level AS ENUM ('ASIL_A', 'ASIL_B', 'ASIL_C', 'ASIL_D', 'DAL_A', 'DAL_B', 'DAL_C', 'DAL_D', 'QM');

-- Set timezone
SET timezone = 'UTC';

-- Create initial database info
CREATE TABLE IF NOT EXISTS db_info (
    version VARCHAR(50) PRIMARY KEY,
    applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    description TEXT
);

INSERT INTO db_info (version, description) VALUES 
    ('0.1.0', 'Initial database schema for ArcViz Web');

-- Log initialization
DO $$
BEGIN
    RAISE NOTICE 'ArcViz database initialized successfully';
    RAISE NOTICE 'Version: 0.1.0';
    RAISE NOTICE 'Timestamp: %', CURRENT_TIMESTAMP;
END $$;
