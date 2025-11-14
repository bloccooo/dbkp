# Build stage
FROM rust:latest AS builder

# Install musl target for static linking
RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y \
	musl-tools \
	musl-dev \
	&& rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /build

# Copy workspace files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY core/Cargo.toml ./core/
COPY cli/Cargo.toml ./cli/

# Create dummy source files to cache dependencies
RUN mkdir -p core/src cli/src && \
	echo "fn main() {}" > cli/src/main.rs && \
	echo "" > core/src/lib.rs

# Build dependencies (this layer will be cached if Cargo files don't change)
RUN cargo build --release --target x86_64-unknown-linux-musl --features vendored-openssl --bin dbkp || true

# Copy actual source code
COPY core ./core
COPY cli ./cli

# Build the binary with vendored-openssl for static linking
RUN cargo build --release --target x86_64-unknown-linux-musl --features vendored-openssl --bin dbkp

# Runtime stage
FROM ubuntu:22.04

# Set environment variables
ENV DEBIAN_FRONTEND=noninteractive
ENV VPRS3BKP_VERSION=latest

# Install minimal runtime dependencies
# Note: PostgreSQL and MySQL clients are automatically installed by dbkp when needed
RUN apt-get update && apt-get install -y \
	ca-certificates \
	&& rm -rf /var/lib/apt/lists/*

# Copy the binary from builder stage
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/dbkp /usr/local/bin/dbkp
RUN chmod +x /usr/local/bin/dbkp

# Verify installation
RUN /usr/local/bin/dbkp --version && echo "✅ dbkp working"

# Create a non-root user for security
RUN useradd -r -u 1001 -g root backup-user -m -d /home/backup-user

# Create directories for backups and cache
RUN mkdir -p /backups /backups/.cache /home/backup-user/.cache && \
	chown -R backup-user:root /backups /home/backup-user && \
	chmod -R 755 /home/backup-user /backups

# Copy and make entrypoint script executable
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh && chown backup-user:root /usr/local/bin/entrypoint.sh

# Set working directory
WORKDIR /backups

# Switch to non-root user
USER backup-user

# Default environment variables (can be overridden)
ENV DATABASE_TYPE=postgresql
ENV DATABASE=""
ENV HOST=localhost
ENV PORT=5432
ENV USERNAME=""
ENV PASSWORD=""
ENV STORAGE_TYPE=s3
ENV LOCATION=""
ENV BUCKET=""
ENV REGION=us-east-1
ENV ENDPOINT=""
ENV ACCESS_KEY=""
ENV SECRET_KEY=""
ENV BACKUP_NAME=""
ENV FORMAT=custom
ENV COMPRESS=true
ENV VERBOSE=false
# Set cache directory to a writable location
ENV XDG_CACHE_HOME=/backups/.cache
ENV HOME=/home/backup-user

# Default command
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["backup"]
