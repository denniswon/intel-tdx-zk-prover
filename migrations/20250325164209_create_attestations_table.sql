-- Add migration script here
CREATE TYPE attestation_type AS ENUM ('DcapV3', 'DcapV4');
CREATE TYPE verification_status AS ENUM ('Verified', 'Pending', 'Failed');

CREATE TABLE IF NOT EXISTS attestations (
    id SERIAL PRIMARY KEY,
    request_id INTEGER REFERENCES requests(id) ON DELETE CASCADE,
    attestation_type attestation_type NOT NULL DEFAULT 'DcapV3',
    verification_status verification_status NOT NULL DEFAULT 'Pending',
    attestation_data BYTEA NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
