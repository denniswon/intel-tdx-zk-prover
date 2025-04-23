-- Add migration script here
CREATE TYPE tdxquotestatus AS ENUM (
    'pending',
    'failure',
    'success'
);

CREATE TABLE tdx_quote (
    id uuid NOT NULL,
    quote bytea NOT NULL,
    onchain_request_id uuid NOT NULL,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    status tdxquotestatus NOT NULL
);

CREATE TABLE onchain_request (
    id uuid NOT NULL,
    creator_address character varying(42) NOT NULL,
    operator_address character varying(42) NOT NULL,
    model_id character varying(66) NOT NULL,
    fee_wei bigint NOT NULL,
    nonce bigint NOT NULL,
    request_id bytea NOT NULL,
    deadline timestamp with time zone NOT NULL,
    is_cancelled boolean NOT NULL,
    cancelled_at timestamp with time zone,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL
);

--
-- Name: onchain_request onchain_request_pkey; Type: CONSTRAINT; Owner: postgres
--

ALTER TABLE ONLY onchain_request
    ADD CONSTRAINT onchain_request_pkey PRIMARY KEY (id);


--
-- Name: tdx_quote tdx_quote_pkey; Type: CONSTRAINT; Owner: postgres
--

ALTER TABLE ONLY tdx_quote
    ADD CONSTRAINT tdx_quote_pkey PRIMARY KEY (id);


--
-- Name: tdx_quote tdx_quote_onchain_request_id_fkey; Type: FK CONSTRAINT; Owner: postgres
--

ALTER TABLE ONLY tdx_quote
    ADD CONSTRAINT tdx_quote_onchain_request_id_fkey FOREIGN KEY (onchain_request_id) REFERENCES onchain_request(id);
