-- Add migration script here
CREATE TYPE attestation_type AS ENUM ('dcap_v3', 'dcap_v4');
CREATE TYPE verification_status AS ENUM ('verified', 'pending', 'failed');

CREATE TABLE IF NOT EXISTS attestations (
    id SERIAL PRIMARY KEY,
    request_id INTEGER NOT NULL,
    attestation_type attestation_type NOT NULL DEFAULT 'dcap_v3',
    verification_status verification_status NOT NULL DEFAULT 'pending',
    attestation_data BYTEA NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    FOREIGN KEY (request_id) REFERENCES requests(id) ON DELETE CASCADE
);
