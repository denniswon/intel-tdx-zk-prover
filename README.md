# TDX Prover

TDX Prover is a Rust Rest Api service built with Rust's Axum framework for providing Intel TDX DCAP attestation verification capabilities including DCAP verification zero knowledge proof (Groth16) using sp1 zkvm.

The service manages three main entities: agents, attestation requests, and attestations. It uses the Axum
web framework and SQLx for database access, with a PostgreSQL backend. The primary function is to verify TDX
DCAP attestations and generate cryptographic proofs of verification.

## Features

This project uses Axum framework and SQLx for DB access layer for storing agent, request, and attestation data. It includes three basic routes: agent, request, and attestation.

## Prerequisites

- [Rust toolchain](https://rustup.rs/)
- [Cargo Lambda](https://github.com/cargo-lambda/cargo-lambda)
- PostgreSQL
- [sqlx-cli](https://crates.io/crates/sqlx-cli)

## Deployment Prerequisites

- [cross-rs](https://github.com/cross-rs/cross)
- [Docker](https://docs.docker.com/engine/install/)
- [AWS CLI](https://aws.amazon.com/cli/)
- [SAM](https://aws.amazon.com/serverless/sam/)
- [Zig](https://ziglang.org/)

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

- POST `/attestation/register` - Register a new attestation
- GET `/attestation/{id}` - Get attestation by id
- GET `/attestation/verify_dcap_qvl/{id}` - Verify attestation with QVL
- GET `/attestation/verify_dcap/{id}` - Verify attestation with DCAP

- GET `/attestation/prove/{id}` - Generate zero knowledge proof of attestation
- POST `/attestation/verify` - Verify zero knowledge proof of attestation
- POST `/attestation/submit_proof` - Submit zero knowledge proof of attestation

## Development

1. Clone the project
2. Update `.env` file with the DB credentials
3. Install `sqlx-cli` or run `cargo sqlx database create` to create your DB
4. Run the migration file using `cargo sqlx migrate run`. This will run the migration file that exists in the migration folder in the root of the project.
5. Build the project and dependencies using `cargo build`
6. Run the project using `cargo run -- up`

## Database

- Create: `cargo sqlx database create`
- Migrate: `cargo sqlx migrate run`
- Offline: `cargo sqlx prepare -- --merged`

## Deploy

1. Install `cross-rs` for cross platform build
2. Build the project using `cross build`

## Lint

- Lint: `cargo clippy`

## Test

- Test: `cargo test [test_name]` (to run a specific test)

## Code Style Guidelines

- **Formatting**: Follow Rust standard style (rustfmt defaults)
- **Imports**: Group by external crates then internal modules
- **Naming**:
  - Use snake_case for files, modules, functions, variables
  - Use CamelCase for types, structs, enums
  - Always use descriptive variable names
- **Error Handling**:
  - Use thiserror for domain-specific errors
  - Implement IntoResponse for API errors
  - Use ? operator for error propagation
- **Types**: Prefer strong typing with explicit types
- **Documentation**: Document public API functions
