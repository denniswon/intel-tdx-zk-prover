# Build stage
FROM public.ecr.aws/docker/library/rust:slim-bookworm AS builder

# Install build dependencies and PostgreSQL
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    postgresql \
    postgresql-client \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Set PostgreSQL environment variables
ENV DATABASE_URL=postgres://magic:password@localhost:5432/tdx

WORKDIR /usr/src/app

# Install sqlx-cli
RUN cargo install sqlx-cli --no-default-features --features native-tls,postgres

# Copy the source code
COPY . .

# Initialize and start PostgreSQL (cargo build here because PSQL needs to be running)
RUN service postgresql start && \
    su postgres -c "psql -c \"CREATE DATABASE tdx;\"" && \
    su postgres -c "psql -c \"CREATE USER magic WITH PASSWORD 'password';\"" && \
    su postgres -c "psql -c \"GRANT ALL PRIVILEGES ON DATABASE tdx TO magic;\"" && \
    su postgres -c "psql -d tdx -c \"GRANT USAGE, CREATE ON SCHEMA public TO magic;\"" && \
    su postgres -c "psql -d tdx -c \"ALTER SCHEMA public OWNER TO magic;\"" && \
    cargo sqlx migrate run && \
    cargo build --release

# Runtime stage
FROM public.ecr.aws/docker/library/debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/tdx-prover /app/tdx-prover

# Expose the port
EXPOSE 8002

# Run the application
CMD ["./tdx-prover"]
