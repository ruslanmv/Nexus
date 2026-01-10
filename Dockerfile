# Dockerfile for Project Nexus
# Multi-stage build for optimal image size

############################
# Build stage
############################
FROM rust:1.78-slim AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests (Cargo.lock optional)
COPY Cargo.toml ./
COPY Cargo.lock* ./

# IMPORTANT: Cargo.toml references benches; copy them early so manifest parsing succeeds
# (If your repo has no benches directory, this will fail; remove this line in that case.)
COPY benches ./benches

# Create dummy source to cache dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn dummy() {}" > src/lib.rs

# Build dependencies (cached layer)
RUN cargo build --release --all-features && \
    rm -rf src

# Copy real source code
COPY src ./src
COPY bridges ./bridges
COPY examples ./examples

# Build the actual binary
RUN cargo build --release --all-features

# Normalize output name: ensure we always have /build/target/release/nexus-runtime
RUN set -eux; \
    if [ -f target/release/nexus-runtime ]; then \
      echo "nexus-runtime already exists"; \
    elif [ -f target/release/nexus-demo ]; then \
      cp target/release/nexus-demo target/release/nexus-runtime; \
    else \
      echo "ERROR: no expected binary found in target/release"; \
      ls -la target/release; \
      exit 1; \
    fi

############################
# Runtime stage
############################
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create nexus user
RUN useradd -m -u 1000 -s /bin/bash nexus

# Create necessary directories
RUN mkdir -p /etc/nexus /var/lib/nexus /var/log/nexus /usr/share/doc/nexus /usr/share/nexus && \
    chown -R nexus:nexus /etc/nexus /var/lib/nexus /var/log/nexus

# Copy binary from builder
COPY --from=builder /build/target/release/nexus-runtime /usr/local/bin/nexus-runtime

# Copy additional files (make sure these files exist in your repo)
# If LICENSE is missing, remove it from this COPY line.
COPY README.md LICENSE /usr/share/doc/nexus/
COPY bridges /usr/share/nexus/bridges/
COPY examples /usr/share/nexus/examples/

# Set up default configuration
RUN cat > /etc/nexus/config.toml << 'EOF'
[kernel]
mailbox_capacity = 1024
send_timeout_ms = 500
handle_timeout_ms = 30000

[logging]
level = "info"
format = "compact"

[runtime]
worker_threads = 0
max_blocking_threads = 512

[distributed]
enabled = true
cluster_name = "nexus-cluster"
node_name = "nexus-node-1"
bind_address = "0.0.0.0:7946"

[wasm]
enabled = true
max_memory_mb = 256
fuel_limit = 100000000
EOF

# Switch to nexus user
USER nexus
WORKDIR /home/nexus

# Expose cluster communication port
EXPOSE 7946

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD nexus-runtime --version || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV PATH="/usr/local/bin:$PATH"

# Default command
ENTRYPOINT ["nexus-runtime"]
CMD ["--config", "/etc/nexus/config.toml"]
