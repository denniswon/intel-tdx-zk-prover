-- Add migration script here
CREATE TYPE request_status AS ENUM ('Fulfilled', 'Pending', 'Failed');
CREATE TABLE requests (
    id SERIAL PRIMARY KEY,
    agent_id INTEGER REFERENCES agents(id) ON DELETE CASCADE,
    from_address VARCHAR(42) NOT NULL,
    prompt TEXT NOT NULL,
    request_data BYTEA NULL,
    request_status request_status NOT NULL DEFAULT 'Pending',
    fee_amount NUMERIC(78, 18) NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);
