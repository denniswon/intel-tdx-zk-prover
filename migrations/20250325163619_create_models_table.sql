-- Add migration script here

CREATE TABLE models (
    id SERIAL PRIMARY KEY,
    model_name VARCHAR(255) NOT NULL,
    model_type VARCHAR(255) NOT NULL,
    model_uri VARCHAR(255) NOT NULL,
    model_description TEXT DEFAULT NULL,
    model_creator_address VARCHAR(42) NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP NULL,
    is_active SMALLINT NOT NULL DEFAULT '0'
);
