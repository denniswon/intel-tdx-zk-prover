-- Add migration script here
CREATE TYPE verification_status AS ENUM ('Verified', 'Pending', 'Failed');
CREATE TYPE ra_type AS ENUM ('DcapV3', 'DcapV4');

CREATE TABLE attestations (
    id SERIAL PRIMARY KEY,
    request_id INTEGER REFERENCES requests(id) ON DELETE CASCADE,
    attestation_type ra_type NOT NULL DEFAULT 'DcapV3',
    verification_status verification_status NOT NULL DEFAULT 'Pending',
    attestation_data BYTEA NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);
