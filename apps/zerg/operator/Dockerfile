FROM rust:1.75 as builder

WORKDIR /workspace

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY libs ./libs
COPY apps/zerg/api ./apps/zerg/api
COPY apps/zerg/operator ./apps/zerg/operator

# Build the operator
RUN cargo build --release --bin zerg-operator

# Runtime image
FROM gcr.io/distroless/cc-debian12:nonroot

# Install kubectl, helm, and flux
COPY --from=bitnami/kubectl:latest /opt/bitnami/kubectl/bin/kubectl /usr/local/bin/kubectl
COPY --from=alpine/helm:latest /usr/bin/helm /usr/local/bin/helm

# Install flux CLI
RUN curl -s https://fluxcd.io/install.sh | bash && \
    mv flux /usr/local/bin/

# Copy the binary
COPY --from=builder /workspace/target/release/zerg-operator /zerg-operator

# Use non-root user
USER nonroot:nonroot

ENTRYPOINT ["/zerg-operator"]