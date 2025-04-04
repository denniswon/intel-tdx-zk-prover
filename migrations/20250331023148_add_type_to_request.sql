-- Add migration script here
CREATE TYPE request_type AS ENUM ('Make', 'Take');
ALTER TABLE requests
ADD COLUMN request_type request_type NOT NULL;
