-- Add migration script here
CREATE TYPE request_status AS ENUM ('fulfilled', 'pending', 'failed');
CREATE TABLE IF NOT EXISTS requests (
    id SERIAL PRIMARY KEY,
    agent_id INTEGER NOT NULL,
    from_address VARCHAR(42) NOT NULL,
    request_data BYTEA NULL,
    request_status request_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
);
