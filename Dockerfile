

FROM rust:1.86.0-slim-bullseye AS build

RUN USER=root cargo new --bin tdx-prover
WORKDIR /tdx-prover

# install dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev
RUN apt-get update -y \
    && apt-get install -y psmisc \
    && apt-get install -y --no-install-recommends openssl \
    # Clean up
    && apt-get autoremove -y && apt-get clean -y && rm -rf /var/lib/apt/lists/*

# Copy Cargo files
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# cache dependencies
RUN cargo build --release
RUN rm -f src/*.rs

# Copy source code
COPY ./src ./src

# install sqlx-cli
RUN cargo install sqlx-cli

# Copy migrations and sqlx
COPY migrations ./migrations
COPY .sqlx ./.sqlx

# prepare sqlx
ENV SQLX_OFFLINE=true
RUN cargo sqlx prepare

# build for release
RUN rm -f ./target/release/dep/tdx-prover*
RUN cargo build --release

FROM rust:1.86.0-slim-bullseye
COPY --from=build /tdx-prover/target/release/tdx-prover .

CMD ["./tdx-prover"]
