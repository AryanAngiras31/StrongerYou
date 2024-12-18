# Use the official Rust image as base
FROM rust:1.75-bullseye as builder

# Set working directory
WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    wget \
    pkg-config \
    libssl-dev

# Copy project files
COPY . .

# Install dependencies
RUN cargo build --release

# Create a lightweight runtime image
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/backend /usr/local/bin/backend

# Set the entrypoint
ENTRYPOINT ["/usr/local/bin/backend"]

# Expose the backend service port
EXPOSE 8080