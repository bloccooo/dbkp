# Multi-stage build for dbkp CLI
# Stage 1: Build
FROM rust:latest AS builder

# Install musl target for static linking
RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y \
    pkg-config \
    musl-tools \
    musl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY core ./core
COPY cli ./cli

# Build the release binary with musl for static linking
RUN cd cli && cargo build --release --target x86_64-unknown-linux-musl --features vendored-openssl

# Stage 2: Runtime
# Use Debian instead of Alpine because downloaded PostgreSQL/MySQL binaries are glibc-based
# The dbkp binary itself is statically linked with musl, so it will work on Debian
FROM debian:bookworm-slim

# Install runtime dependencies:
# - ca-certificates: for HTTPS connections (needed to download database client tools)
# - openssh-client: for SSH tunnel support (optional)
# - libpq5: PostgreSQL client library (needed by downloaded pg_dump binaries)
# Note: Database client tools (pg_dump, mysqldump) are automatically downloaded by dbkp
RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssh-client \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user for security
RUN useradd -m -u 1000 dbkp && \
    mkdir -p /backups /home/dbkp/.cache && \
    chown -R dbkp:dbkp /backups /home/dbkp/.cache

# Copy the binary from builder
# Note: musl builds go to target/x86_64-unknown-linux-musl/release/
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/dbkp /usr/local/bin/dbkp

# Set the binary as executable
RUN chmod +x /usr/local/bin/dbkp

# Switch to non-root user
USER dbkp

# Set working directory
WORKDIR /backups

# Default command
ENTRYPOINT ["/usr/local/bin/dbkp"]
