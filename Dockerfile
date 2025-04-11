

FROM rust:1.86-slim-bullseye AS build

# install dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libpq-dev

RUN USER=root cargo new --bin tdx-prover
WORKDIR /tdx-prover

# Copy Cargo files
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# cache dependencies
RUN cargo build --release
RUN rm -f src/*.rs

# Copy source
COPY ./src ./src

# Copy migrations and sqlx
COPY migrations ./migrations
COPY .sqlx ./.sqlx

# SQLx Offline Mode
ENV SQLX_OFFLINE=true

# build for release
RUN rm -f ./target/release/dep/tdx-prover*
RUN cargo build --release

FROM debian:bullseye-slim

# Install only minimal dependencies
RUN apt-get update && apt-get install -y libpq5 ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /tdx-prover
COPY --from=build /tdx-prover/target/release/tdx-prover /usr/src/tdx-prover

CMD ["/usr/src/tdx-prover"]
