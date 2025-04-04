# TDX Prover

TDX Prover is a Rust Rest Api service built with Rust's Axum framework for providing Intel TDX DCAP attestation verification capabilities including DCAP verification zero knowledge proof (Groth16) using sp1 zkvm.

## Features

This project uses Axum framework and SQLx for DB access layer for storing agent, request, and attestation data. It includes three basic routes: agent, request, and attestation.

## Routes

### Agent

- POST `/agent/register` - Register a new agent
- GET `/agent/{id}` - Get agent by id
- PUT `/agent/{id}` - Update a agent
- DELETE `/agent/{id}` - Delete a agent

### Request

- POST `/request/register` - Register a new request
- GET `/request/{id}` - Get request by id
- PUT `/request/{id}` - Update a request
- DELETE `/request/{id}` - Delete a request

### Attestation

- GET `/attestation/{id}` - Get attestation by id
- POST `/attestation/verify` - Verify a DCAP attestation
- GET `/attestation/prove/{id}` - Get attestation zero knowledge proof by id

## Development

1. Clone the project
2. Update `.env` file with the DB credentials
3. Install `sqlx-cli` or run `cargo sqlx database create` to create your DB
4. Run the migration file using `cargo sqlx migrate run`. This will run the migration file that exists in the migration folder in the root of the project.
5. Build the project and dependencies using `cargo build`
6. Run the project using `cargo run -- up`
