FROM messense/rust-musl-cross:x86_64-musl AS chef
WORKDIR /app
RUN cargo install cargo-chef --locked

FROM chef AS planner
ARG APP_NAME
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./apps ./apps/
COPY ./libs ./libs/
# Prepare dependency recipe for caching
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ARG APP_NAME
ENV RUST_BACKTRACE=1

# Build dependencies first (this layer will be cached)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl

# Copy source code and build application
COPY . .
RUN cargo build --release -p $APP_NAME --target x86_64-unknown-linux-musl

# Ultra-minimal runtime stage
FROM scratch AS rust
ARG APP_NAME

# Copy CA certificates for HTTPS requests (if needed)
#COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copy the statically linked binary
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/$APP_NAME /app

# Security: Run as non-root user
USER 65534

# Environment
ENV PORT=8080
EXPOSE ${PORT}

# Security and metadata labels
LABEL \
    org.opencontainers.image.title="${APP_NAME}" \
    org.opencontainers.image.description="Minimal Rust application" \
    security.non-root="true" \
    security.static-binary="true" \
    security.minimal-size="true" \
    security.no-shell="true" \
    security.distroless="false" \
    security.minimal="true" \
    security.base-image="scratch"

ENTRYPOINT ["/app"]
