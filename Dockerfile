# Build stage
FROM rustlang/rust:nightly-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Copy the source code
COPY . .

# Build the application
## Temporarily disabled, not sure about how sqlx and db connection will work like one for staging vs. prod
# RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder stage
## Temporarily disabled, not sure about how sqlx and db connection will work like one for staging vs. prod
# COPY --from=builder /usr/src/app/target/release/tdx-prover /app/tdx-prover

# Expose the port
EXPOSE 8002

# Run the application
CMD ["echo test"] 