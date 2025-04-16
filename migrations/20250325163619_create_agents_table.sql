-- Add migration script here
CREATE TYPE agent_status AS ENUM ('active', 'inactive');
CREATE TABLE IF NOT EXISTS agents (
    id SERIAL PRIMARY KEY,
    agent_name VARCHAR(255) NOT NULL,
    agent_type VARCHAR(255) NOT NULL,
    agent_uri VARCHAR(255) NOT NULL,
    agent_description TEXT DEFAULT NULL,
    agent_owner VARCHAR(42) NOT NULL,
    agent_status agent_status NOT NULL DEFAULT 'active',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NULL
);
