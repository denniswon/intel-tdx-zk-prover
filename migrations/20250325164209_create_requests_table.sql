-- Add migration script here

CREATE TABLE requests (
    id SERIAL PRIMARY KEY,
    model_id INTEGER REFERENCES models(id) ON DELETE CASCADE,
    from_address VARCHAR(42) NOT NULL,
    prompt TEXT NOT NULL,
    request_data BYTEA NULL,
    fee_amount NUMERIC(78, 18) NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);
