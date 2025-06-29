# Multi-stage build for Rust application
FROM rust:1.75 as builder

WORKDIR /app

# Copy the workspace files
COPY Cargo.toml Cargo.lock ./
COPY libs/ libs/
COPY apps/zerg/api/ apps/zerg/api/
COPY apps/protos/api/ apps/protos/api/
COPY apps/terran/api/ apps/terran/api/

# Build the application
RUN cargo build --release -p zerg_api

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 appuser

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/zerg_api /app/zerg_api

# Change ownership to app user
RUN chown -R appuser:appuser /app
USER appuser

EXPOSE 8080

CMD ["./zerg_api"]