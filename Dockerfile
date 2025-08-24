# Multi-stage build for OneSociety monolith server
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY apps/monolith-server/Cargo.toml ./apps/monolith-server/
COPY libs/common/Cargo.toml ./libs/common/
COPY libs/models/Cargo.toml ./libs/models/
COPY libs/proto/Cargo.toml ./libs/proto/

# Create dummy source files to build dependencies
RUN mkdir -p apps/monolith-server/src libs/common/src libs/models/src libs/proto/src
RUN echo "fn main() {}" > apps/monolith-server/src/main.rs
RUN echo "pub fn dummy() {}" > libs/common/src/lib.rs
RUN echo "pub fn dummy() {}" > libs/models/src/lib.rs
RUN echo "pub fn dummy() {}" > libs/proto/src/lib.rs

# Build dependencies
RUN cargo build --release --workspace

# Remove dummy files and copy real source code
RUN rm apps/monolith-server/src/main.rs
RUN rm libs/common/src/lib.rs
RUN rm libs/models/src/lib.rs
RUN rm libs/proto/src/lib.rs

# Copy source code
COPY . .

# Build the application
RUN cargo build --release --workspace

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false onesociety

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/monolith-server /app/monolith-server

# Copy database migrations
COPY --from=builder /app/db /app/db

# Copy configuration files
COPY --from=builder /app/docker-compose.yml /app/docker-compose.yml
COPY --from=builder /app/Makefile /app/Makefile

# Set ownership
RUN chown -R onesociety:onesociety /app

# Switch to non-root user
USER onesociety

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/healthz || exit 1

# Run the application
CMD ["/app/monolith-server"]
