-- Add migration script here
CREATE TYPE request_status AS ENUM ('Fulfilled', 'Pending', 'Failed');
ALTER TABLE requests
ADD COLUMN request_status request_status NOT NULL DEFAULT 'Pending';