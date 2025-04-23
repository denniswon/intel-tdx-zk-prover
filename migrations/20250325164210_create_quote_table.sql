-- Add migration script here
CREATE TYPE tdxquotestatus AS ENUM (
    'pending',
    'failure',
    'success'
);
ALTER TYPE tdxquotestatus OWNER TO postgres;

CREATE TABLE tdx_quote (
    id uuid NOT NULL,
    quote bytea NOT NULL,
    onchain_request_id uuid NOT NULL,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    status tdxquotestatus NOT NULL
);
ALTER TABLE tdx_quote OWNER TO postgres;
